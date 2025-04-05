// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// QuickSwap interfaces (Uniswap V2 fork on Polygon)
interface IQuickSwapPair {
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function mint(address to) external returns (uint liquidity);
    function burn(address to) external returns (uint amount0, uint amount1);
}

interface IQuickSwapFactory {
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}

// SushiSwap interfaces (another Uniswap V2 fork on Polygon)
interface ISushiSwapPair {
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function mint(address to) external returns (uint liquidity);
    function burn(address to) external returns (uint amount0, uint amount1);
}

interface ISushiSwapFactory {
    function getPair(address tokenA, address tokenB) external view returns (address pair);
}

// Curve interface for Polygon (already defined in IDexInterfaces.sol)
// Using the same interface as in IDexInterfaces.sol

// Uniswap V3 Position Manager interface
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

// Common Polygon token addresses
contract PolygonTokens {
    address public constant WMATIC = 0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270;
    address public constant USDC = 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174;
    address public constant USDT = 0xc2132D05D31c914a87C6611C10748AEb04B58e8F;
    address public constant DAI = 0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063;
    address public constant WBTC = 0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6;
    address public constant WETH = 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619;
    address public constant AAVE = 0xD6DF932A45C0f255f85145f286eA0b292B21C90B;
    address public constant QUICK = 0x831753DD7087CaC61aB5644b308642cc1c33Dc13;
}