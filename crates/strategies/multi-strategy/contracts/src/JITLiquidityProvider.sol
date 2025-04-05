// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

// Interfaces for Uniswap V2
interface IUniswapV2Factory {
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}

interface IUniswapV2Pair {
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function mint(address to) external returns (uint256 liquidity);
    function burn(address to) external returns (uint256 amount0, uint256 amount1);
}

// Interfaces for Uniswap V3
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
    
    function positions(uint256 tokenId) external view returns (
        uint96 nonce,
        address operator,
        address token0,
        address token1,
        uint24 fee,
        int24 tickLower,
        int24 tickUpper,
        uint128 liquidity,
        uint256 feeGrowthInside0LastX128,
        uint256 feeGrowthInside1LastX128,
        uint128 tokensOwed0,
        uint128 tokensOwed1
    );
    
    function collect(
        CollectParams calldata params
    ) external returns (uint256 amount0, uint256 amount1);
    
    function decreaseLiquidity(
        DecreaseLiquidityParams calldata params
    ) external returns (uint256 amount0, uint256 amount1);
    
    struct CollectParams {
        uint256 tokenId;
        address recipient;
        uint128 amount0Max;
        uint128 amount1Max;
    }
    
    struct DecreaseLiquidityParams {
        uint256 tokenId;
        uint128 liquidity;
        uint256 amount0Min;
        uint256 amount1Min;
        uint256 deadline;
    }
}

/**
 * @title JITLiquidityProvider
 * @dev Provides Just-In-Time liquidity to DEX pools
 */
contract JITLiquidityProvider is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    
    // Events
    event LiquidityAdded(address pool, uint256 amount0, uint256 amount1);
    event LiquidityRemoved(address pool, uint256 amount0, uint256 amount1);
    event V3PositionCreated(uint256 tokenId, address token0, address token1, uint24 fee);
    event V3PositionClosed(uint256 tokenId, uint256 amount0, uint256 amount1);
    event TokenWithdrawn(address token, uint256 amount);
    
    // Constants
    uint256 private constant MAX_INT = type(uint256).max;
    address private immutable WETH;
    
    // Protocol addresses
    address public uniswapV2Factory;
    address public nonfungiblePositionManager;
    
    // Whitelisted callers
    mapping(address => bool) public whitelistedCallers;
    
    // Active V3 positions
    mapping(uint256 => bool) public activePositions;
    
    constructor(
        address _weth,
        address _uniswapV2Factory,
        address _nonfungiblePositionManager
    ) {
        WETH = _weth;
        uniswapV2Factory = _uniswapV2Factory;
        nonfungiblePositionManager = _nonfungiblePositionManager;
        whitelistedCallers[msg.sender] = true;
    }
    
    /**
     * @dev Add a whitelisted caller
     * @param _caller Address to whitelist
     */
    function addWhitelistedCaller(address _caller) external onlyOwner {
        whitelistedCallers[_caller] = true;
    }
    
    /**
     * @dev Remove a whitelisted caller
     * @param _caller Address to remove
     */
    function removeWhitelistedCaller(address _caller) external onlyOwner {
        whitelistedCallers[_caller] = false;
    }
    
    /**
     * @dev Add liquidity to a Uniswap V2 pool
     * @param tokenA First token in the pair
     * @param tokenB Second token in the pair
     * @param amountA Amount of tokenA to add
     * @param amountB Amount of tokenB to add
     * @return Liquidity tokens received
     */
    function addLiquidityV2(
        address tokenA,
        address tokenB,
        uint256 amountA,
        uint256 amountB
    ) external nonReentrant returns (uint256) {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        
        address pair = IUniswapV2Factory(uniswapV2Factory).getPair(tokenA, tokenB);
        require(pair != address(0), "Pair does not exist");
        
        // Transfer tokens to the pair
        IERC20(tokenA).safeTransfer(pair, amountA);
        IERC20(tokenB).safeTransfer(pair, amountB);
        
        // Mint liquidity tokens
        uint256 liquidity = IUniswapV2Pair(pair).mint(address(this));
        
        emit LiquidityAdded(pair, amountA, amountB);
        
        return liquidity;
    }
    
    /**
     * @dev Remove liquidity from a Uniswap V2 pool
     * @param pair The pair address
     * @param liquidity Amount of LP tokens to burn
     * @return amount0 Amount of token0 received
     * @return amount1 Amount of token1 received
     */
    function removeLiquidityV2(
        address pair,
        uint256 liquidity
    ) external nonReentrant returns (uint256 amount0, uint256 amount1) {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        
        // Burn LP tokens
        IERC20(pair).safeTransfer(pair, liquidity);
        (amount0, amount1) = IUniswapV2Pair(pair).burn(address(this));
        
        emit LiquidityRemoved(pair, amount0, amount1);
        
        return (amount0, amount1);
    }
    
    /**
     * @dev Add liquidity to a Uniswap V3 pool
     * @param token0 First token in the pair
     * @param token1 Second token in the pair
     * @param fee Fee tier
     * @param tickLower Lower tick boundary
     * @param tickUpper Upper tick boundary
     * @param amount0Desired Desired amount of token0
     * @param amount1Desired Desired amount of token1
     * @param amount0Min Minimum amount of token0
     * @param amount1Min Minimum amount of token1
     * @param deadline Deadline for the transaction
     * @return tokenId NFT position identifier
     * @return liquidity Amount of liquidity added
     */
    function addLiquidityV3(
        address token0,
        address token1,
        uint24 fee,
        int24 tickLower,
        int24 tickUpper,
        uint256 amount0Desired,
        uint256 amount1Desired,
        uint256 amount0Min,
        uint256 amount1Min,
        uint256 deadline
    ) external nonReentrant returns (uint256 tokenId, uint128 liquidity) {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        
        // Approve tokens
        IERC20(token0).safeApprove(nonfungiblePositionManager, amount0Desired);
        IERC20(token1).safeApprove(nonfungiblePositionManager, amount1Desired);
        
        // Create the position
        INonfungiblePositionManager.MintParams memory params = INonfungiblePositionManager.MintParams({
            token0: token0,
            token1: token1,
            fee: fee,
            tickLower: tickLower,
            tickUpper: tickUpper,
            amount0Desired: amount0Desired,
            amount1Desired: amount1Desired,
            amount0Min: amount0Min,
            amount1Min: amount1Min,
            recipient: address(this),
            deadline: deadline
        });
        
        (tokenId, liquidity, , ) = INonfungiblePositionManager(nonfungiblePositionManager).mint(params);
        
        // Track the position
        activePositions[tokenId] = true;
        
        emit V3PositionCreated(tokenId, token0, token1, fee);
        
        return (tokenId, liquidity);
    }
    
    /**
     * @dev Remove liquidity from a Uniswap V3 position
     * @param tokenId NFT position identifier
     * @param liquidity Amount of liquidity to remove
     * @param amount0Min Minimum amount of token0 expected
     * @param amount1Min Minimum amount of token1 expected
     * @param deadline Deadline for the transaction
     * @return amount0 Amount of token0 received
     * @return amount1 Amount of token1 received
     */
    function removeLiquidityV3(
        uint256 tokenId,
        uint128 liquidity,
        uint256 amount0Min,
        uint256 amount1Min,
        uint256 deadline
    ) external nonReentrant returns (uint256 amount0, uint256 amount1) {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        require(activePositions[tokenId], "Position not active or not owned");
        
        // Decrease liquidity
        INonfungiblePositionManager.DecreaseLiquidityParams memory params = INonfungiblePositionManager.DecreaseLiquidityParams({
            tokenId: tokenId,
            liquidity: liquidity,
            amount0Min: amount0Min,
            amount1Min: amount1Min,
            deadline: deadline
        });
        
        (amount0, amount1) = INonfungiblePositionManager(nonfungiblePositionManager).decreaseLiquidity(params);
        
        // Collect all fees and tokens
        INonfungiblePositionManager.CollectParams memory collectParams = INonfungiblePositionManager.CollectParams({
            tokenId: tokenId,
            recipient: address(this),
            amount0Max: type(uint128).max,
            amount1Max: type(uint128).max
        });
        
        (uint256 collected0, uint256 collected1) = INonfungiblePositionManager(nonfungiblePositionManager).collect(collectParams);
        
        // If we've removed all liquidity, mark position as inactive
        (,,,,,,,,uint128 positionLiquidity,,,) = INonfungiblePositionManager(nonfungiblePositionManager).positions(tokenId);
        if (positionLiquidity == 0) {
            activePositions[tokenId] = false;
        }
        
        emit V3PositionClosed(tokenId, collected0, collected1);
        
        return (collected0, collected1);
    }
    
    /**
     * @dev Withdraw tokens from the contract
     * @param token Token to withdraw
     * @param amount Amount to withdraw (0 for all)
     */
    function withdrawToken(address token, uint256 amount) external onlyOwner {
        uint256 withdrawAmount = amount == 0 ? IERC20(token).balanceOf(address(this)) : amount;
        IERC20(token).safeTransfer(owner(), withdrawAmount);
        emit TokenWithdrawn(token, withdrawAmount);
    }
    
    /**
     * @dev Fallback function to receive ETH
     */
    receive() external payable {}
}