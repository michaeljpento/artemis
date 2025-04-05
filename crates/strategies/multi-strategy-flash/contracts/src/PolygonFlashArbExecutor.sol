// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./FlashLoanCore.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "./interfaces/IDexInterfaces.sol";
import "./interfaces/PolygonDexInterfaces.sol";

// Extend the IUniswapV3Pool interface from FlashLoanCore with additional functions for Polygon
interface IUniswapV3PoolExtended is IUniswapV3Pool {
    function swap(
        address recipient,
        bool zeroForOne,
        int256 amountSpecified,
        uint160 sqrtPriceLimitX96,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);
}

/**
 * @title PolygonFlashArbExecutor
 * @notice Executes arbitrage strategies using flash loans for capital on Polygon
 * @dev Extends FlashLoanCore to support multiple Polygon DEXes
 * @dev Optimized for Polygon's ~2 second block time (vs Ethereum's ~12 seconds)
 */
contract PolygonFlashArbExecutor is FlashLoanCore, IUniswapV3SwapCallback {
    using SafeERC20 for IERC20;

    // DEX types for Polygon
    enum DexType {
        QUICK_SWAP, // Uniswap V2 fork on Polygon
        SUSHI_SWAP, // Another Uniswap V2 fork on Polygon
        UNISWAP_V3, // Also available on Polygon
        CURVE       // Also available on Polygon
    }

    // Polygon DEX addresses
    address public quickSwapFactory;
    address public sushiSwapFactory;

    // Swap parameters
    struct SwapParams {
        address pool;
        DexType dexType;
        bool zeroForOne; // Direction of swap (for Uniswap V3)
        int128 i;        // Index of input token (for Curve)
        int128 j;        // Index of output token (for Curve)
        uint256 amountIn;
        uint256 minAmountOut;
        bool useUnderlying; // Whether to use underlying (for Curve)
        address token_in;   // Input token address (needed for Curve swaps)
    }

    // Arbitrage parameters
    struct ArbParams {
        address startToken;
        uint256 flashLoanAmount;
        SwapParams[] swaps;
    }

    // Events
    event ArbExecuted(
        address indexed startToken,
        uint256 flashLoanAmount,
        uint256 profit,
        uint256 executionTime
    );
    
    // Polygon-specific settings
    uint256 public constant POLYGON_MAX_EXECUTION_TIME = 1 seconds; // Max time for execution on Polygon's fast blocks
    uint256 public constant POLYGON_DEFAULT_GAS_PRICE_MULTIPLIER = 110; // 10% higher than base fee for faster inclusion
    uint256 public constant POLYGON_GAS_LIMIT_BUFFER = 200000; // Additional gas buffer for Polygon transactions
    
    // Gas price control for Polygon transactions
    uint256 public polygonMaxGasPrice = 100 gwei; // Max gas price willing to pay on Polygon (adjustable)
    uint256 public polygonPriorityFee = 35 gwei; // Priority fee for Polygon transactions (adjustable)

    constructor(
        address _aaveAddressProvider,
        address _balancerVault,
        address _uniswapV3Factory,
        address _quickSwapFactory,
        address _sushiSwapFactory
    ) FlashLoanCore(_aaveAddressProvider, _balancerVault, _uniswapV3Factory) {
        quickSwapFactory = _quickSwapFactory;
        sushiSwapFactory = _sushiSwapFactory;
    }

    /**
     * @notice Set the QuickSwap factory address
     * @param _quickSwapFactory The address of the QuickSwap factory
     */
    function setQuickSwapFactory(address _quickSwapFactory) external onlyOwner {
        quickSwapFactory = _quickSwapFactory;
    }

    /**
     * @notice Set the SushiSwap factory address
     * @param _sushiSwapFactory The address of the SushiSwap factory
     */
    function setSushiSwapFactory(address _sushiSwapFactory) external onlyOwner {
        sushiSwapFactory = _sushiSwapFactory;
    }
    
    /**
     * @notice Set the maximum gas price for Polygon transactions
     * @param _maxGasPrice The maximum gas price in wei
     */
    function setPolygonMaxGasPrice(uint256 _maxGasPrice) external onlyOwner {
        polygonMaxGasPrice = _maxGasPrice;
    }
    
    /**
     * @notice Set the priority fee for Polygon transactions
     * @param _priorityFee The priority fee in wei
     */
    function setPolygonPriorityFee(uint256 _priorityFee) external onlyOwner {
        polygonPriorityFee = _priorityFee;
    }

    /**
     * @notice Execute arbitrage using a flash loan on Polygon
     * @param params The arbitrage parameters
     * @param provider The flash loan provider to use
     */
    function executeArbitrage(
        ArbParams calldata params,
        FlashLoanProvider provider
    ) external onlyOwner {
        require(params.swaps.length > 0, "PolygonFlashArbExecutor: No swaps specified");
        
        // Directly pass the encoded data to the flash loan function
        super.executeFlashLoan(
            params.startToken,
            params.flashLoanAmount,
            abi.encode(params),
            provider
        );
    }

    /**
     * @notice Override the flash loan logic to execute arbitrage
     * @param token The borrowed token
     * @param amount The borrowed amount
     * @param data Encoded arbitrage parameters
     */
    function _executeFlashLoanLogic(
        address token,
        uint256 amount,
        bytes memory data
    ) internal override {
        // Optimization for Polygon's faster block time - track execution time
        uint256 startTime = block.timestamp;
        
        // Decode the arbitrage parameters
        ArbParams memory params = abi.decode(data, (ArbParams));
        
        require(token == params.startToken, "PolygonFlashArbExecutor: Token mismatch");
        require(amount >= params.flashLoanAmount, "PolygonFlashArbExecutor: Amount mismatch");
        
        // Track the initial balance
        uint256 initialBalance = IERC20(token).balanceOf(address(this));
        
        // Execute the arbitrage swaps
        _executeArbitrageSwaps(params.swaps);
        
        // Calculate the profit
        uint256 finalBalance = IERC20(token).balanceOf(address(this));
        require(finalBalance > initialBalance, "PolygonFlashArbExecutor: No profit");
        
        uint256 profit = finalBalance - initialBalance;
        uint256 executionTime = block.timestamp - startTime;
        
        // Ensure we didn't exceed Polygon's smaller block window
        require(executionTime <= POLYGON_MAX_EXECUTION_TIME, "PolygonFlashArbExecutor: Execution time too long for Polygon");
        
        emit ArbExecuted(token, amount, profit, executionTime);
    }

    /**
     * @notice Execute a series of swaps across different Polygon DEXes
     * @param swaps The swap parameters
     */
    function _executeArbitrageSwaps(SwapParams[] memory swaps) internal {
        for (uint256 i = 0; i < swaps.length; i++) {
            SwapParams memory swap = swaps[i];
            
            // Approve the pool to spend the token
            address inputToken = _getSwapInputToken(swap);
            uint256 balance = IERC20(inputToken).balanceOf(address(this));
            
            // Use the balance if it's less than the specified amount
            uint256 amountIn = swap.amountIn > balance ? balance : swap.amountIn;
            
            // Approve the pool to spend the token
            IERC20(inputToken).safeApprove(swap.pool, 0);
            IERC20(inputToken).safeApprove(swap.pool, amountIn);
            
            // Execute the swap based on the DEX type
            if (swap.dexType == DexType.QUICK_SWAP) {
                _executeQuickSwapSwap(swap, amountIn);
            } else if (swap.dexType == DexType.SUSHI_SWAP) {
                _executeSushiSwapSwap(swap, amountIn);
            } else if (swap.dexType == DexType.UNISWAP_V3) {
                _executeUniswapV3Swap(swap, amountIn);
            } else if (swap.dexType == DexType.CURVE) {
                _executeCurveSwap(swap, amountIn);
            }
        }
    }

    /**
     * @notice Execute a swap on QuickSwap (Uniswap V2 fork on Polygon)
     * @param swap The swap parameters
     * @param amountIn The input amount
     */
    function _executeQuickSwapSwap(SwapParams memory swap, uint256 amountIn) internal {
        IQuickSwapPair pair = IQuickSwapPair(swap.pool);
        
        // Determine the output token and amount
        (uint256 amount0Out, uint256 amount1Out) = swap.zeroForOne 
            ? (uint256(0), swap.minAmountOut) 
            : (swap.minAmountOut, uint256(0));
        
        // Execute the swap
        pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
    }

    /**
     * @notice Execute a swap on SushiSwap (Uniswap V2 fork on Polygon)
     * @param swap The swap parameters
     * @param amountIn The input amount
     */
    function _executeSushiSwapSwap(SwapParams memory swap, uint256 amountIn) internal {
        ISushiSwapPair pair = ISushiSwapPair(swap.pool);
        
        // Determine the output token and amount
        (uint256 amount0Out, uint256 amount1Out) = swap.zeroForOne 
            ? (uint256(0), swap.minAmountOut) 
            : (swap.minAmountOut, uint256(0));
        
        // Execute the swap
        pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
    }

    /**
     * @notice Execute a swap on Uniswap V3 (available on Polygon)
     * @param swap The swap parameters
     * @param amountIn The input amount
     */
    function _executeUniswapV3Swap(SwapParams memory swap, uint256 amountIn) internal {
        IUniswapV3PoolExtended pool = IUniswapV3PoolExtended(swap.pool);
        
        // Execute the swap
        pool.swap(
            address(this),
            swap.zeroForOne,
            int256(amountIn),
            type(uint160).max, // No price limit
            abi.encode(swap.pool, swap.minAmountOut)
        );
    }

    /**
     * @notice Execute a swap on Curve (available on Polygon)
     * @param swap The swap parameters
     * @param amountIn The input amount
     */
    function _executeCurveSwap(SwapParams memory swap, uint256 amountIn) internal {
        ICurvePool pool = ICurvePool(swap.pool);
        
        // Execute the swap
        if (swap.useUnderlying) {
            pool.exchange_underlying(swap.i, swap.j, amountIn, swap.minAmountOut);
        } else {
            pool.exchange(swap.i, swap.j, amountIn, swap.minAmountOut);
        }
    }

    /**
     * @notice Get the input token for a swap on Polygon
     * @param swap The swap parameters
     * @return The input token address
     */
    function _getSwapInputToken(SwapParams memory swap) internal view returns (address) {
        if (swap.dexType == DexType.QUICK_SWAP) {
            IQuickSwapPair pair = IQuickSwapPair(swap.pool);
            return swap.zeroForOne ? pair.token0() : pair.token1();
        } else if (swap.dexType == DexType.SUSHI_SWAP) {
            ISushiSwapPair pair = ISushiSwapPair(swap.pool);
            return swap.zeroForOne ? pair.token0() : pair.token1();
        } else if (swap.dexType == DexType.UNISWAP_V3) {
            IUniswapV3PoolExtended pool = IUniswapV3PoolExtended(swap.pool);
            return swap.zeroForOne ? pool.token0() : pool.token1();
        } else if (swap.dexType == DexType.CURVE) {
            // For Curve, we use the token_in from the swap params directly
            // This must be provided when creating the swap
            require(swap.token_in != address(0), "PolygonFlashArbExecutor: Invalid input token for Curve swap");
            return swap.token_in;
        } else {
            revert("PolygonFlashArbExecutor: Unsupported DEX type");
        }
    }

    /**
     * @notice Callback for Uniswap V3 swaps on Polygon
     * @param amount0Delta The change in token0 balance
     * @param amount1Delta The change in token1 balance
     * @param data Additional data (pool address and min output)
     */
    function uniswapV3SwapCallback(
        int256 amount0Delta,
        int256 amount1Delta,
        bytes calldata data
    ) external override {
        (address pool, uint256 minAmountOut) = abi.decode(data, (address, uint256));
        require(msg.sender == pool, "PolygonFlashArbExecutor: Invalid callback sender");
        
        // If amount0Delta > 0, we need to pay token0 to the pool
        if (amount0Delta > 0) {
            address token0 = IUniswapV3PoolExtended(pool).token0();
            IERC20(token0).safeTransfer(pool, uint256(amount0Delta));
        }
        
        // If amount1Delta > 0, we need to pay token1 to the pool
        if (amount1Delta > 0) {
            address token1 = IUniswapV3PoolExtended(pool).token1();
            IERC20(token1).safeTransfer(pool, uint256(amount1Delta));
        }
        
        // Verify that we received at least the minimum amount out
        int256 amountReceived;
        if (amount0Delta < 0) {
            amountReceived = -amount0Delta;
        } else if (amount1Delta < 0) {
            amountReceived = -amount1Delta;
        } else {
            amountReceived = 0;
        }
        require(uint256(amountReceived) >= minAmountOut, "PolygonFlashArbExecutor: Insufficient output amount");
    }
}