//! Contract ABIs and bindings for interacting with our deployed contracts

use ethers::contract::abigen;

// Generate Rust bindings for JIT Liquidity Provider
abigen!(
    JitLiquidityProvider,
    r#"[
        {
            "inputs": [
                {
                    "components": [
                        {"name": "token0", "type": "address"},
                        {"name": "token1", "type": "address"},
                        {"name": "amount0", "type": "uint256"},
                        {"name": "amount1", "type": "uint256"},
                        {"name": "pool", "type": "address"},
                        {"name": "poolType", "type": "uint8"},
                        {"name": "minFeeExpected", "type": "uint256"}
                    ],
                    "name": "jitParams",
                    "type": "tuple"
                },
                {
                    "components": [
                        {"name": "fee", "type": "uint24"},
                        {"name": "tickLower", "type": "int24"},
                        {"name": "tickUpper", "type": "int24"},
                        {"name": "tokenId", "type": "uint256"}
                    ],
                    "name": "v3Params",
                    "type": "tuple"
                }
            ],
            "name": "executeBalancerJITLiquidity",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "components": [
                        {"name": "token0", "type": "address"},
                        {"name": "token1", "type": "address"},
                        {"name": "amount0", "type": "uint256"},
                        {"name": "amount1", "type": "uint256"},
                        {"name": "pool", "type": "address"},
                        {"name": "poolType", "type": "uint8"},
                        {"name": "minFeeExpected", "type": "uint256"}
                    ],
                    "name": "jitParams",
                    "type": "tuple"
                },
                {
                    "components": [
                        {"name": "fee", "type": "uint24"},
                        {"name": "tickLower", "type": "int24"},
                        {"name": "tickUpper", "type": "int24"},
                        {"name": "tokenId", "type": "uint256"}
                    ],
                    "name": "v3Params",
                    "type": "tuple"
                },
                {"name": "competitorTransaction", "type": "bytes32"},
                {"name": "maxPriorityFeeMultiplier", "type": "uint256"}
            ],
            "name": "executeUltraAggressiveJIT",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "components": [
                        {"name": "token0", "type": "address"},
                        {"name": "token1", "type": "address"},
                        {"name": "amount0", "type": "uint256"},
                        {"name": "amount1", "type": "uint256"},
                        {"name": "pool", "type": "address"},
                        {"name": "poolType", "type": "uint8"},
                        {"name": "minFeeExpected", "type": "uint256"}
                    ],
                    "name": "jitParams",
                    "type": "tuple[]"
                },
                {
                    "components": [
                        {"name": "fee", "type": "uint24"},
                        {"name": "tickLower", "type": "int24"},
                        {"name": "tickUpper", "type": "int24"},
                        {"name": "tokenId", "type": "uint256"}
                    ],
                    "name": "v3Params",
                    "type": "tuple[]"
                },
                {"name": "count", "type": "uint256"}
            ],
            "name": "executeBatchMicroJIT",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]"#
);

// Generate Rust bindings for Flash Arbitrage Executor
abigen!(
    FlashArbExecutor,
    r#"[
        {
            "inputs": [
                {
                    "components": [
                        {"name": "startToken", "type": "address"},
                        {"name": "flashLoanAmount", "type": "uint256"},
                        {
                            "components": [
                                {"name": "pool", "type": "address"},
                                {"name": "dexType", "type": "uint8"},
                                {"name": "zeroForOne", "type": "bool"},
                                {"name": "i", "type": "int128"},
                                {"name": "j", "type": "int128"},
                                {"name": "amountIn", "type": "uint256"},
                                {"name": "minAmountOut", "type": "uint256"},
                                {"name": "useUnderlying", "type": "bool"},
                                {"name": "token_in", "type": "address"}
                            ],
                            "name": "swaps",
                            "type": "tuple[]"
                        }
                    ],
                    "name": "params",
                    "type": "tuple"
                },
                {"name": "provider", "type": "uint8"}
            ],
            "name": "executeArbitrage",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]"#
);

// Export the raw ABIs for reference
pub const JIT_LIQUIDITY_PROVIDER_ABI: &str = r#"[
    {
        "inputs": [
            {
                "components": [
                    {"name": "token0", "type": "address"},
                    {"name": "token1", "type": "address"},
                    {"name": "amount0", "type": "uint256"},
                    {"name": "amount1", "type": "uint256"},
                    {"name": "pool", "type": "address"},
                    {"name": "poolType", "type": "uint8"},
                    {"name": "minFeeExpected", "type": "uint256"}
                ],
                "name": "jitParams",
                "type": "tuple"
            },
            {
                "components": [
                    {"name": "fee", "type": "uint24"},
                    {"name": "tickLower", "type": "int24"},
                    {"name": "tickUpper", "type": "int24"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "v3Params",
                "type": "tuple"
            }
        ],
        "name": "executeBalancerJITLiquidity",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "components": [
                    {"name": "token0", "type": "address"},
                    {"name": "token1", "type": "address"},
                    {"name": "amount0", "type": "uint256"},
                    {"name": "amount1", "type": "uint256"},
                    {"name": "pool", "type": "address"},
                    {"name": "poolType", "type": "uint8"},
                    {"name": "minFeeExpected", "type": "uint256"}
                ],
                "name": "jitParams",
                "type": "tuple"
            },
            {
                "components": [
                    {"name": "fee", "type": "uint24"},
                    {"name": "tickLower", "type": "int24"},
                    {"name": "tickUpper", "type": "int24"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "v3Params",
                "type": "tuple"
            },
            {"name": "competitorTransaction", "type": "bytes32"},
            {"name": "maxPriorityFeeMultiplier", "type": "uint256"}
        ],
        "name": "executeUltraAggressiveJIT",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
]"#;

pub const FLASH_ARB_EXECUTOR_ABI: &str = r#"[
    {
        "inputs": [
            {
                "components": [
                    {"name": "startToken", "type": "address"},
                    {"name": "flashLoanAmount", "type": "uint256"},
                    {
                        "components": [
                            {"name": "pool", "type": "address"},
                            {"name": "dexType", "type": "uint8"},
                            {"name": "zeroForOne", "type": "bool"},
                            {"name": "i", "type": "int128"},
                            {"name": "j", "type": "int128"},
                            {"name": "amountIn", "type": "uint256"},
                            {"name": "minAmountOut", "type": "uint256"},
                            {"name": "useUnderlying", "type": "bool"},
                            {"name": "token_in", "type": "address"}
                        ],
                        "name": "swaps",
                        "type": "tuple[]"
                    }
                ],
                "name": "params",
                "type": "tuple"
            },
            {"name": "provider", "type": "uint8"}
        ],
        "name": "executeArbitrage",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
]"#;
