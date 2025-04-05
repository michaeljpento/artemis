//! Bindings for the multi-strategy contracts

// These will be auto-generated once the contracts are compiled
// For now, we just have placeholder stub modules

pub mod flash_arb_executor {
    use ethers::prelude::*;

    abigen!(
        FlashArbExecutor,
        r#"[
            function executeArbitrage(address loanToken, uint256 loanAmount, bytes calldata arbData) external
            function executeV2Swap(address pair, bool zeroToOne, uint256 amountIn) external returns (uint256)
            function executeV3Swap(address pool, bool zeroForOne, int256 amountIn) external returns (uint256)
        ]"#
    );
}

pub mod jit_liquidity_provider {
    use ethers::prelude::*;

    abigen!(
        JITLiquidityProvider,
        r#"[
            function addLiquidityV2(address tokenA, address tokenB, uint256 amountA, uint256 amountB) external returns (uint256)
            function removeLiquidityV2(address pair, uint256 liquidity) external returns (uint256 amount0, uint256 amount1)
            function addLiquidityV3(address token0, address token1, uint24 fee, int24 tickLower, int24 tickUpper, uint256 amount0Desired, uint256 amount1Desired, uint256 amount0Min, uint256 amount1Min, uint256 deadline) external returns (uint256 tokenId, uint128 liquidity)
            function removeLiquidityV3(uint256 tokenId, uint128 liquidity, uint256 amount0Min, uint256 amount1Min, uint256 deadline) external returns (uint256 amount0, uint256 amount1)
        ]"#
    );
}