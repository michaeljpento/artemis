// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./FlashLoanCore.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "./interfaces/IDexInterfaces.sol";

// Uniswap V3 interfaces
interface INonfungiblePositionManager {
    struct MintParams {
        address token0;
        address token1;
        uint24 fee;
        int24 tickLower;
        int24 tickUpper;
        uint256 amount0Desired;
        uint256 amount1Desired;
        uint256 amount0Min;
        uint256 amount1Min;
        address recipient;
        uint256 deadline;
    }

    function mint(MintParams calldata params) external returns (
        uint256 tokenId,
        uint128 liquidity,
        uint256 amount0,
        uint256 amount1
    );

    struct IncreaseLiquidityParams {
        uint256 tokenId;
        uint256 amount0Desired;
        uint256 amount1Desired;
        uint256 amount0Min;
        uint256 amount1Min;
        uint256 deadline;
    }

    function increaseLiquidity(IncreaseLiquidityParams calldata params) external returns (
        uint128 liquidity,
        uint256 amount0,
        uint256 amount1
    );

    struct DecreaseLiquidityParams {
        uint256 tokenId;
        uint128 liquidity;
        uint256 amount0Min;
        uint256 amount1Min;
        uint256 deadline;
    }

    function decreaseLiquidity(DecreaseLiquidityParams calldata params) external returns (
        uint256 amount0,
        uint256 amount1
    );

    struct CollectParams {
        uint256 tokenId;
        address recipient;
        uint128 amount0Max;
        uint128 amount1Max;
    }

    function collect(CollectParams calldata params) external returns (
        uint256 amount0,
        uint256 amount1
    );
}

/**
 * @title JITLiquidityProvider
 * @notice Provides Just-In-Time (JIT) liquidity to DEX pools to capture swap fees
 * @dev Uses flash loans to provide capital for liquidity provision
 */
contract JITLiquidityProvider is FlashLoanCore {
    using SafeERC20 for IERC20;

    // Pool types
    enum PoolType {
        UNISWAP_V2,
        UNISWAP_V3
    }

    // JIT liquidity parameters
    struct JITParams {
        address token0;
        address token1;
        uint256 amount0;
        uint256 amount1;
        address pool;
        PoolType poolType;
        uint256 minFeeExpected;
    }

    // Uniswap V3 position parameters
    struct V3PositionParams {
        uint24 fee;
        int24 tickLower;
        int24 tickUpper;
        uint256 tokenId; // Only used for existing positions
    }

    // Addresses
    address public uniswapV2Factory;
    address public nonfungiblePositionManager;

    // Token ID tracking for Uniswap V3 positions
    mapping(address => mapping(address => uint256[])) public positionTokenIds;

    // Events
    event JITLiquidityAdded(
        address indexed pool,
        PoolType poolType,
        address token0,
        address token1,
        uint256 amount0,
        uint256 amount1
    );

    event JITLiquidityRemoved(
        address indexed pool,
        PoolType poolType,
        address token0,
        address token1,
        uint256 amount0,
        uint256 amount1,
        uint256 fee0,
        uint256 fee1
    );

    constructor(
        address _aaveAddressProvider,
        address _balancerVault,
        address _uniswapV3Factory,
        address _uniswapV2Factory,
        address _nonfungiblePositionManager
    ) FlashLoanCore(_aaveAddressProvider, _balancerVault, _uniswapV3Factory) {
        uniswapV2Factory = _uniswapV2Factory;
        nonfungiblePositionManager = _nonfungiblePositionManager;
    }

    /**
     * @notice Set the Uniswap V2 factory address
     * @param _uniswapV2Factory The address of the Uniswap V2 factory
     */
    function setUniswapV2Factory(address _uniswapV2Factory) external onlyOwner {
        uniswapV2Factory = _uniswapV2Factory;
    }

    /**
     * @notice Set the Nonfungible Position Manager address for Uniswap V3
     * @param _nonfungiblePositionManager The address of the Nonfungible Position Manager
     */
    function setNonfungiblePositionManager(address _nonfungiblePositionManager) external onlyOwner {
        nonfungiblePositionManager = _nonfungiblePositionManager;
    }

    /**
     * @notice Execute JIT liquidity provision using a flash loan
     * @param jitParams The JIT liquidity parameters
     * @param v3Params The Uniswap V3 position parameters (only used for V3 pools)
     * @param provider The flash loan provider to use
     */
    function executeJITLiquidity(
        JITParams calldata jitParams,
        V3PositionParams calldata v3Params,
        FlashLoanProvider provider
    ) external onlyOwner {
        // Calculate total flash loan amount needed
        address flashLoanToken;
        uint256 flashLoanAmount;
        
        // Determine which token to flash loan based on availability
        if (jitParams.amount0 > 0 && jitParams.amount1 > 0) {
            // If both tokens are needed, choose the one with higher value
            // In a real implementation, you would need to consider price and liquidity
            flashLoanToken = jitParams.token0;
            flashLoanAmount = jitParams.amount0;
        } else if (jitParams.amount0 > 0) {
            flashLoanToken = jitParams.token0;
            flashLoanAmount = jitParams.amount0;
        } else {
            flashLoanToken = jitParams.token1;
            flashLoanAmount = jitParams.amount1;
        }
        
        // Execute the flash loan with directly encoded parameters
        super.executeFlashLoan(
            flashLoanToken,
            flashLoanAmount,
            abi.encode(jitParams, v3Params),
            provider
        );
    }

    /**
     * @notice Override the flash loan logic to execute JIT liquidity provision
     * @param token The borrowed token
     * @param amount The borrowed amount
     * @param data Encoded JIT parameters
     */
    function _executeFlashLoanLogic(
        address token,
        uint256 amount,
        bytes memory data
    ) internal override {
        // Decode the JIT parameters and V3 parameters
        (JITParams memory jitParams, V3PositionParams memory v3Params) = 
            abi.decode(data, (JITParams, V3PositionParams));
        
        // Track initial balances
        uint256 initialBalance0 = IERC20(jitParams.token0).balanceOf(address(this));
        uint256 initialBalance1 = IERC20(jitParams.token1).balanceOf(address(this));
        
        // If we borrowed one token, we might need to swap some to get the other token
        // This is simplified; in a real scenario, you'd need to implement swap logic
        
        // Add liquidity based on pool type
        if (jitParams.poolType == PoolType.UNISWAP_V2) {
            _addLiquidityV2(jitParams);
        } else {
            _addLiquidityV3(jitParams, v3Params);
        }

        emit JITLiquidityAdded(
            jitParams.pool,
            jitParams.poolType,
            jitParams.token0,
            jitParams.token1,
            jitParams.amount0,
            jitParams.amount1
        );
        
        // Wait for the profitable transaction to occur
        // This is a placeholder; in a real scenario, this would be more complex
        // and would likely involve monitoring mempool or being triggered externally
        
        // Remove liquidity
        (uint256 received0, uint256 received1, uint256 fee0, uint256 fee1) = 
            jitParams.poolType == PoolType.UNISWAP_V2 
                ? _removeLiquidityV2(jitParams) 
                : _removeLiquidityV3(jitParams, v3Params);
        
        emit JITLiquidityRemoved(
            jitParams.pool,
            jitParams.poolType,
            jitParams.token0,
            jitParams.token1,
            received0,
            received1,
            fee0,
            fee1
        );
        
        // Verify that we received at least the minimum expected fee
        uint256 totalFeeValue = fee0 + fee1; // This is simplified; in reality, you'd need to consider token prices
        require(totalFeeValue >= jitParams.minFeeExpected, "JITLiquidityProvider: Insufficient fee");
    }

    /**
     * @notice Add liquidity to a Uniswap V2 pool
     * @param jitParams The JIT liquidity parameters
     */
    function _addLiquidityV2(JITParams memory jitParams) internal {
        // Get the pair address if not provided
        address pair = jitParams.pool;
        if (pair == address(0)) {
            pair = IUniswapV2Factory(uniswapV2Factory).getPair(jitParams.token0, jitParams.token1);
            require(pair != address(0), "JITLiquidityProvider: Pair not found");
        }
        
        // Transfer tokens to the pair
        IERC20(jitParams.token0).safeTransfer(pair, jitParams.amount0);
        IERC20(jitParams.token1).safeTransfer(pair, jitParams.amount1);
        
        // Mint liquidity tokens
        IUniswapV2Pair(pair).mint(address(this));
    }

    /**
     * @notice Remove liquidity from a Uniswap V2 pool
     * @param jitParams The JIT liquidity parameters
     * @return amount0 The amount of token0 received
     * @return amount1 The amount of token1 received
     * @return fee0 The fee earned in token0
     * @return fee1 The fee earned in token1
     */
    function _removeLiquidityV2(JITParams memory jitParams) internal returns (
        uint256 amount0,
        uint256 amount1,
        uint256 fee0,
        uint256 fee1
    ) {
        address pair = jitParams.pool;
        
        // Get the liquidity token balance
        uint256 liquidity = IERC20(pair).balanceOf(address(this));
        
        // Track initial balances
        uint256 initialBalance0 = IERC20(jitParams.token0).balanceOf(address(this));
        uint256 initialBalance1 = IERC20(jitParams.token1).balanceOf(address(this));
        
        // Burn liquidity tokens
        IERC20(pair).safeTransfer(pair, liquidity);
        (amount0, amount1) = IUniswapV2Pair(pair).burn(address(this));
        
        // Calculate fees earned
        fee0 = amount0 > jitParams.amount0 ? amount0 - jitParams.amount0 : 0;
        fee1 = amount1 > jitParams.amount1 ? amount1 - jitParams.amount1 : 0;
        
        return (amount0, amount1, fee0, fee1);
    }

    /**
     * @notice Add liquidity to a Uniswap V3 pool
     * @param jitParams The JIT liquidity parameters
     * @param v3Params The Uniswap V3 position parameters
     */
    function _addLiquidityV3(JITParams memory jitParams, V3PositionParams memory v3Params) internal {
        // Approve tokens to the position manager
        IERC20(jitParams.token0).safeApprove(nonfungiblePositionManager, 0);
        IERC20(jitParams.token0).safeApprove(nonfungiblePositionManager, jitParams.amount0);
        IERC20(jitParams.token1).safeApprove(nonfungiblePositionManager, 0);
        IERC20(jitParams.token1).safeApprove(nonfungiblePositionManager, jitParams.amount1);
        
        // If tokenId is provided, increase liquidity on existing position
        if (v3Params.tokenId > 0) {
            INonfungiblePositionManager.IncreaseLiquidityParams memory params = 
                INonfungiblePositionManager.IncreaseLiquidityParams({
                    tokenId: v3Params.tokenId,
                    amount0Desired: jitParams.amount0,
                    amount1Desired: jitParams.amount1,
                    amount0Min: 0,
                    amount1Min: 0,
                    deadline: block.timestamp
                });
            
            INonfungiblePositionManager(nonfungiblePositionManager).increaseLiquidity(params);
        } else {
            // Create a new position
            INonfungiblePositionManager.MintParams memory params = 
                INonfungiblePositionManager.MintParams({
                    token0: jitParams.token0,
                    token1: jitParams.token1,
                    fee: v3Params.fee,
                    tickLower: v3Params.tickLower,
                    tickUpper: v3Params.tickUpper,
                    amount0Desired: jitParams.amount0,
                    amount1Desired: jitParams.amount1,
                    amount0Min: 0,
                    amount1Min: 0,
                    recipient: address(this),
                    deadline: block.timestamp
                });
            
            (uint256 tokenId, , , ) = INonfungiblePositionManager(nonfungiblePositionManager).mint(params);
            
            // Store the token ID
            positionTokenIds[jitParams.token0][jitParams.token1].push(tokenId);
        }
    }

    /**
     * @notice Remove liquidity from a Uniswap V3 pool
     * @param jitParams The JIT liquidity parameters
     * @param v3Params The Uniswap V3 position parameters
     * @return amount0 The amount of token0 received
     * @return amount1 The amount of token1 received
     * @return fee0 The fee earned in token0
     * @return fee1 The fee earned in token1
     */
    function _removeLiquidityV3(JITParams memory jitParams, V3PositionParams memory v3Params) internal returns (
        uint256 amount0,
        uint256 amount1,
        uint256 fee0,
        uint256 fee1
    ) {
        uint256 tokenId = v3Params.tokenId;
        
        // If tokenId is not provided, use the latest one
        if (tokenId == 0) {
            uint256[] memory tokenIds = positionTokenIds[jitParams.token0][jitParams.token1];
            require(tokenIds.length > 0, "JITLiquidityProvider: No position found");
            tokenId = tokenIds[tokenIds.length - 1];
        }
        
        // Track initial balances
        uint256 initialBalance0 = IERC20(jitParams.token0).balanceOf(address(this));
        uint256 initialBalance1 = IERC20(jitParams.token1).balanceOf(address(this));
        
        // First collect any accumulated fees
        INonfungiblePositionManager.CollectParams memory collectParams = 
            INonfungiblePositionManager.CollectParams({
                tokenId: tokenId,
                recipient: address(this),
                amount0Max: type(uint128).max,
                amount1Max: type(uint128).max
            });
        
        (fee0, fee1) = INonfungiblePositionManager(nonfungiblePositionManager).collect(collectParams);
        
        // Then decrease liquidity
        INonfungiblePositionManager.DecreaseLiquidityParams memory decreaseParams = 
            INonfungiblePositionManager.DecreaseLiquidityParams({
                tokenId: tokenId,
                liquidity: type(uint128).max, // Remove all liquidity
                amount0Min: 0,
                amount1Min: 0,
                deadline: block.timestamp
            });
        
        (uint256 removed0, uint256 removed1) = INonfungiblePositionManager(nonfungiblePositionManager).decreaseLiquidity(decreaseParams);
        
        // Collect the tokens from decreasing liquidity
        (uint256 collected0, uint256 collected1) = INonfungiblePositionManager(nonfungiblePositionManager).collect(collectParams);
        
        // Calculate total amounts received
        amount0 = collected0;
        amount1 = collected1;
        
        return (amount0, amount1, fee0, fee1);
    }
}