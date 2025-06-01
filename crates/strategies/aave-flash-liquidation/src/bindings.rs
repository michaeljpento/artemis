use alloy_sol_types::sol;

sol! {
    #[allow(missing_docs)]
    contract AaveV3FlashLiquidator {
        struct FlashLoanParams {
            address asset;
            uint256 amount;
            uint256 mode;
            address onBehalfOf;
            bytes params;
            uint16 referralCode;
        }

        struct LiquidationParams {
            address collateralAsset;
            address debtAsset;
            address user;
            uint256 debtToCover;
            bool receiveAToken;
        }

        struct SwapParams {
            address tokenIn;
            address tokenOut;
            uint256 amountIn;
            uint256 minAmountOut;
            address router;
            bytes swapData;
        }

        function flashLiquidate(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            bool receiveAToken
        ) external returns (bool);

        function calculateExpectedProfit(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover
        ) external view returns (uint256);

        function getUserHealthFactor(address user) external view returns (uint256);

        function isLiquidatable(address user) external view returns (bool);

        function getLiquidationBonus(address asset) external view returns (uint256);

        function calculateOptimalLiquidationAmount(
            address user,
            address debtAsset,
            uint256 maxAmount
        ) external view returns (uint256);

        function toggleFlashbotsProtection(bool enabled) external;

        function submitProtectedLiquidation(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            bool receiveAToken,
            bytes calldata flashbotsData
        ) external returns (bool);

        function isGasPriceAcceptable() external view returns (bool);

        function emergencyPause() external;

        function emergencyUnpause() external;

        function setMaxGasPrice(uint256 newMaxGasPrice) external;

        function setMinProfitThreshold(uint256 newThreshold) external;

        function withdrawProfits(address token, uint256 amount) external;

        function updateSwapRouter(address newRouter) external;

        function executeSwap(SwapParams calldata params) external returns (uint256);

        function getAssetPrice(address asset) external view returns (uint256);

        function getAssetDecimals(address asset) external view returns (uint8);

        function checkCircuitBreaker() external view returns (bool);

        function triggerCircuitBreaker(string calldata reason) external;

        function resetCircuitBreaker() external;

        event LiquidationExecuted(
            address indexed user,
            address indexed collateralAsset,
            address indexed debtAsset,
            uint256 debtToCover,
            uint256 liquidatedCollateralAmount,
            address liquidator,
            bool receiveAToken
        );

        event FlashbotsProtectionToggled(bool enabled);

        event CircuitBreakerTriggered(string reason);

        event ProfitWithdrawn(address indexed token, uint256 amount, address indexed to);

        event SwapExecuted(
            address indexed tokenIn,
            address indexed tokenOut,
            uint256 amountIn,
            uint256 amountOut,
            address indexed router
        );

        event MaxGasPriceUpdated(uint256 oldPrice, uint256 newPrice);

        event MinProfitThresholdUpdated(uint256 oldThreshold, uint256 newThreshold);

        error InsufficientProfit();
        error UserNotLiquidatable();
        error GasPriceTooHigh();
        error CircuitBreakerActive();
        error FlashLoanFailed();
        error SwapFailed();
        error UnauthorizedAccess();
        error InvalidParameters();
    }
}

sol! {
    #[allow(missing_docs)]
    contract IAavePool {
        function getUserAccountData(address user)
            external
            view
            returns (
                uint256 totalCollateralETH,
                uint256 totalDebtETH,
                uint256 availableBorrowsETH,
                uint256 currentLiquidationThreshold,
                uint256 ltv,
                uint256 healthFactor
            );

        function getReserveData(address asset)
            external
            view
            returns (
                uint256 configuration,
                uint128 liquidityIndex,
                uint128 variableBorrowIndex,
                uint128 currentLiquidityRate,
                uint128 currentVariableBorrowRate,
                uint128 currentStableBorrowRate,
                uint40 lastUpdateTimestamp,
                address aTokenAddress,
                address stableDebtTokenAddress,
                address variableDebtTokenAddress,
                address interestRateStrategyAddress,
                uint8 id
            );

        function liquidationCall(
            address collateralAsset,
            address debtAsset,
            address user,
            uint256 debtToCover,
            bool receiveAToken
        ) external;

        function flashLoan(
            address receiverAddress,
            address[] calldata assets,
            uint256[] calldata amounts,
            uint256[] calldata modes,
            address onBehalfOf,
            bytes calldata params,
            uint16 referralCode
        ) external;

        function getReservesList() external view returns (address[] memory);

        function getConfiguration(address asset) external view returns (uint256);
    }
}

sol! {
    #[allow(missing_docs)]
    contract IAaveOracle {
        function getAssetPrice(address asset) external view returns (uint256);
        
        function getAssetsPrices(address[] calldata assets) 
            external 
            view 
            returns (uint256[] memory);

        function getSourceOfAsset(address asset) external view returns (address);

        function getFallbackOracle() external view returns (address);
    }
}

sol! {
    #[allow(missing_docs)]
    contract IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
    }
}

sol! {
    #[allow(missing_docs)]
    contract IUniswapV2Router {
        function swapExactTokensForTokens(
            uint amountIn,
            uint amountOutMin,
            address[] calldata path,
            address to,
            uint deadline
        ) external returns (uint[] memory amounts);

        function getAmountsOut(uint amountIn, address[] calldata path)
            external view returns (uint[] memory amounts);

        function getAmountsIn(uint amountOut, address[] calldata path)
            external view returns (uint[] memory amounts);

        function factory() external pure returns (address);

        function WETH() external pure returns (address);
    }
}

sol! {
    #[allow(missing_docs)]
    contract IUniswapV3Router {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }

        struct ExactOutputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountOut;
            uint256 amountInMaximum;
            uint160 sqrtPriceLimitX96;
        }

        function exactInputSingle(ExactInputSingleParams calldata params)
            external payable returns (uint256 amountOut);

        function exactOutputSingle(ExactOutputSingleParams calldata params)
            external payable returns (uint256 amountIn);

        function factory() external view returns (address);

        function WETH9() external view returns (address);
    }
}

sol! {
    #[allow(missing_docs)]
    contract IBalancerVault {
        struct SingleSwap {
            bytes32 poolId;
            uint8 kind;
            address assetIn;
            address assetOut;
            uint256 amount;
            bytes userData;
        }

        struct FundManagement {
            address sender;
            bool fromInternalBalance;
            address recipient;
            bool toInternalBalance;
        }

        function swap(
            SingleSwap memory singleSwap,
            FundManagement memory funds,
            uint256 limit,
            uint256 deadline
        ) external payable returns (uint256);

        function querySwap(
            SingleSwap memory singleSwap,
            FundManagement memory funds
        ) external returns (uint256);
    }
}

sol! {
    #[allow(missing_docs)]
    contract ICurvePool {
        function exchange(
            int128 i,
            int128 j,
            uint256 dx,
            uint256 min_dy
        ) external returns (uint256);

        function get_dy(
            int128 i,
            int128 j,
            uint256 dx
        ) external view returns (uint256);

        function coins(uint256 i) external view returns (address);

        function balances(uint256 i) external view returns (uint256);
    }
}

pub use AaveV3FlashLiquidator::*;
pub use IAavePool::*;
pub use IAaveOracle::*;
pub use IERC20::*;
pub use IUniswapV2Router::*;
pub use IUniswapV3Router::*;
pub use IBalancerVault::*;
pub use ICurvePool::*;
