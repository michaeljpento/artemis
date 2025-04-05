// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./FlashLoanCore.sol";

// Interfaces
interface IUniswapV2Pair {
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function token0() external view returns (address);
    function token1() external view returns (address);
}

interface IUniswapV3Pool {
    function swap(
        address recipient,
        bool zeroForOne,
        int256 amountSpecified,
        uint160 sqrtPriceLimitX96,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);
}

interface ICurvePool {
    function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external returns (uint256);
    function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);
}

/**
 * @title FlashArbExecutor
 * @dev Executes flash loan-based arbitrage across multiple DEXes
 */
contract FlashArbExecutor is FlashLoanCore {
    // Events
    event ArbExecuted(uint256 profit, address token);
    
    // Structs
    struct SwapParams {
        address pool;
        uint8 poolType; // 0 = UniswapV2, 1 = UniswapV3, 2 = Curve
        bool zeroForOne;
        uint256 amountIn;
        uint256 minAmountOut;
    }
    
    struct ArbParams {
        address startToken;
        uint256 borrowAmount;
        SwapParams[] swaps;
    }
    
    constructor(
        address _weth,
        address _aaveLendingPool,
        address _balancerVault,
        address _uniswapV3Factory
    ) FlashLoanCore(
        _weth,
        _aaveLendingPool,
        _balancerVault,
        _uniswapV3Factory
    ) {}
    
    /**
     * @dev Execute arbitrage using a flash loan
     * @param loanToken Address of token to borrow
     * @param loanAmount Amount to borrow
     * @param arbData Encoded arbitrage path data
     */
    function executeArbitrage(
        address loanToken,
        uint256 loanAmount,
        bytes calldata arbData
    ) external {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        
        // Choose the most suitable flash loan provider based on the token and amount
        FlashLoanProvider provider = selectBestProvider(loanToken, loanAmount);
        
        // Execute the flash loan
        executeFlashLoan(loanToken, loanAmount, arbData, provider);
    }
    
    /**
     * @dev Select the best flash loan provider based on token and amount
     * This can be refined based on gas costs, fees, and available liquidity
     */
    function selectBestProvider(address token, uint256 amount) internal view returns (FlashLoanProvider) {
        // For WETH and major tokens, Aave often has better liquidity
        if (token == WETH ||
            token == 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 || // USDC
            token == 0x6B175474E89094C44Da98b954EedeAC495271d0F) { // DAI
            return FlashLoanProvider.AAVE;
        }
        
        // For smaller amounts, Balancer might be more gas efficient
        if (amount < 1000 * 10**18) {
            return FlashLoanProvider.BALANCER;
        }
        
        // Default to Aave for most cases
        return FlashLoanProvider.AAVE;
    }
    
    /**
     * @dev AAVE flash loan callback function
     * @param assets Array of asset addresses
     * @param amounts Array of amounts
     * @param premiums Array of premiums
     * @param initiator Initiator of the flash loan
     * @param params Additional parameters
     * @return Return value indicating success
     */
    function executeOperation(
        address[] calldata assets,
        uint256[] calldata amounts,
        uint256[] calldata premiums,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        require(msg.sender == aaveLendingPool, "Caller must be lending pool");
        require(initiator == address(this), "Initiator must be this contract");
        
        // Decode the original callback data
        (bytes memory arbData, FlashLoanProvider provider) = abi.decode(params, (bytes, FlashLoanProvider));
        
        // Execute the arbitrage steps
        _executeArbitrage(assets[0], amounts[0], arbData);
        
        // Repay the loan with premium
        uint256 amountOwed = amounts[0] + premiums[0];
        IERC20(assets[0]).approve(aaveLendingPool, amountOwed);
        
        return true;
    }
    
    /**
     * @dev Balancer flash loan callback function
     * @param tokens Array of tokens
     * @param amounts Array of amounts
     * @param feeAmounts Array of fee amounts
     * @param userData User data passed in the flash loan
     */
    function receiveFlashLoan(
        IERC20[] memory tokens,
        uint256[] memory amounts,
        uint256[] memory feeAmounts,
        bytes memory userData
    ) external {
        require(msg.sender == balancerVault, "Caller must be Balancer vault");
        
        // Decode the original callback data
        (bytes memory arbData, FlashLoanProvider provider) = abi.decode(userData, (bytes, FlashLoanProvider));
        
        // Execute the arbitrage steps
        _executeArbitrage(address(tokens[0]), amounts[0], arbData);
        
        // Repay the loan with fee
        uint256 amountOwed = amounts[0] + feeAmounts[0];
        tokens[0].transfer(balancerVault, amountOwed);
    }
    
    /**
     * @dev Uniswap V3 flash callback function
     * @param fee0 Fee for token0
     * @param fee1 Fee for token1
     * @param data Callback data
     */
    function uniswapV3FlashCallback(
        uint256 fee0,
        uint256 fee1,
        bytes calldata data
    ) external {
        // Decode the callback data
        (bytes memory arbData, FlashLoanProvider provider, address token) = abi.decode(data, (bytes, FlashLoanProvider, address));
        
        // Get the pool that called this function
        address pool = msg.sender;
        bool isToken0 = IUniswapV3Pool(pool).token0() == token;
        
        // Execute the arbitrage steps
        uint256 amount = isToken0 ? fee0 : fee1;
        _executeArbitrage(token, amount, arbData);
        
        // Repay the loan with fee
        uint256 amountOwed = amount + (isToken0 ? fee0 : fee1);
        IERC20(token).transfer(pool, amountOwed);
    }
    
    /**
     * @dev Internal function to execute the arbitrage
     * @param token Token borrowed
     * @param amount Amount borrowed
     * @param arbData Encoded arbitrage data
     */
    function _executeArbitrage(
        address token,
        uint256 amount,
        bytes memory arbData
    ) internal {
        // Decode the arbitrage parameters
        ArbParams memory params = abi.decode(arbData, (ArbParams));
        
        // Verify the token matches
        require(token == params.startToken, "Token mismatch");
        
        // Current token and amount we're working with
        address currentToken = token;
        uint256 currentAmount = amount;
        
        // Execute each swap in the path
        for (uint i = 0; i < params.swaps.length; i++) {
            SwapParams memory swap = params.swaps[i];
            
            // If this is the first swap, use the borrowed amount
            // For subsequent swaps, use all available tokens
            if (i > 0) {
                swap.amountIn = IERC20(currentToken).balanceOf(address(this));
            }
            
            // Execute the swap based on pool type
            if (swap.poolType == 0) {
                // Uniswap V2 / SushiSwap
                (currentToken, currentAmount) = _executeV2Swap(swap, currentToken);
            } else if (swap.poolType == 1) {
                // Uniswap V3
                (currentToken, currentAmount) = _executeV3Swap(swap, currentToken);
            } else if (swap.poolType == 2) {
                // Curve
                (currentToken, currentAmount) = _executeCurveSwap(swap, currentToken);
            }
        }
        
        // Verify we ended up with the starting token
        require(currentToken == token, "Arbitrage path must return to starting token");
        
        // Calculate profit
        uint256 profit = currentAmount > amount ? currentAmount - amount : 0;
        
        emit ArbExecuted(profit, token);
    }
    
    /**
     * @dev Execute swap on Uniswap V2 / SushiSwap
     * @param swap Swap parameters
     * @param tokenIn Input token
     * @return Output token and amount
     */
    function _executeV2Swap(
        SwapParams memory swap,
        address tokenIn
    ) internal returns (address tokenOut, uint256 amountOut) {
        IUniswapV2Pair pair = IUniswapV2Pair(swap.pool);
        
        // Get the pool tokens
        address token0 = pair.token0();
        address token1 = pair.token1();
        
        // Determine which token is the output
        tokenOut = swap.zeroForOne ? token1 : token0;
        
        // Approve tokens to the pair
        IERC20(tokenIn).approve(swap.pool, swap.amountIn);
        
        // Calculate the expected output amount
        uint amount0Out = swap.zeroForOne ? 0 : swap.minAmountOut;
        uint amount1Out = swap.zeroForOne ? swap.minAmountOut : 0;
        
        // Execute the swap
        pair.swap(amount0Out, amount1Out, address(this), new bytes(0));
        
        // Get the actual output amount
        amountOut = IERC20(tokenOut).balanceOf(address(this));
        
        return (tokenOut, amountOut);
    }
    
    /**
     * @dev Execute swap on Uniswap V3
     * @param swap Swap parameters
     * @param tokenIn Input token
     * @return Output token and amount
     */
    function _executeV3Swap(
        SwapParams memory swap,
        address tokenIn
    ) internal returns (address tokenOut, uint256 amountOut) {
        IUniswapV3Pool pool = IUniswapV3Pool(swap.pool);
        
        // Get the pool tokens
        address token0 = pool.token0();
        address token1 = pool.token1();
        
        // Determine which token is the output
        tokenOut = swap.zeroForOne ? token1 : token0;
        
        // Approve tokens to the pool
        IERC20(tokenIn).approve(swap.pool, swap.amountIn);
        
        // Calculate price limits (essentially unlimited for arbitrage)
        uint160 sqrtPriceLimitX96 = swap.zeroForOne ? 4295128740 : 1461446703485210103287273052203988822378723970341;
        
        // Execute the swap
        pool.swap(
            address(this),
            swap.zeroForOne,
            int256(swap.amountIn),
            sqrtPriceLimitX96,
            new bytes(0)
        );
        
        // Get the actual output amount
        amountOut = IERC20(tokenOut).balanceOf(address(this));
        
        return (tokenOut, amountOut);
    }
    
    /**
     * @dev Execute swap on Curve
     * @param swap Swap parameters
     * @param tokenIn Input token
     * @return Output token and amount
     */
    function _executeCurveSwap(
        SwapParams memory swap,
        address tokenIn
    ) internal returns (address tokenOut, uint256 amountOut) {
        ICurvePool curvePool = ICurvePool(swap.pool);
        
        // For Curve, we need to know the token indices
        // This would typically be determined off-chain and passed in the swap params
        int128 i = swap.zeroForOne ? 0 : 1;
        int128 j = swap.zeroForOne ? 1 : 0;
        
        // Determine output token based on Curve pool type
        // In a real implementation, this would need to be more sophisticated
        tokenOut = swap.zeroForOne ? address(0x6B175474E89094C44Da98b954EedeAC495271d0F) : address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);
        
        // Approve tokens to the pool
        IERC20(tokenIn).approve(swap.pool, swap.amountIn);
        
        // Execute the swap
        uint256 received = curvePool.exchange(i, j, swap.amountIn, swap.minAmountOut);
        
        // Get the actual output amount
        amountOut = received;
        
        return (tokenOut, amountOut);
    }
}