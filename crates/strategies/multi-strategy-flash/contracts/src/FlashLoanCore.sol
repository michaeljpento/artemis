// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

// Interface for Aave V3 Flash Loan
interface IPoolAddressesProvider {
    function getPool() external view returns (address);
}

interface IPool {
    function flashLoanSimple(
        address receiverAddress,
        address asset,
        uint256 amount,
        bytes calldata params,
        uint16 referralCode
    ) external;

    function FLASHLOAN_PREMIUM_TOTAL() external view returns (uint128);
}

// Interface for Balancer Flash Loan
interface IBalancerVault {
    function flashLoan(
        address recipient,
        address[] memory tokens,
        uint256[] memory amounts,
        bytes memory userData
    ) external;
}

// Interface for Uniswap V3 Factory
interface IUniswapV3Factory {
    function getPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external view returns (address pool);
}

// Interface for Uniswap V3 Flash Loan
interface IUniswapV3Pool {
    function flash(
        address recipient,
        uint256 amount0,
        uint256 amount1,
        bytes calldata data
    ) external;

    function token0() external view returns (address);
    function token1() external view returns (address);
    function liquidity() external view returns (uint128);
}

/**
 * @title FlashLoanCore
 * @notice Core contract for executing flash loans across multiple providers
 * @dev Supports Aave, Balancer, and Uniswap V3 flash loans
 */
contract FlashLoanCore is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // Flash loan providers
    enum FlashLoanProvider {
        AAVE,
        BALANCER,
        UNISWAP_V3
    }

    // Provider addresses
    address public aaveAddressProvider;
    address public balancerVault;
    address public uniswapV3Factory;

    // Events
    event FlashLoanExecuted(address indexed token, uint256 amount, FlashLoanProvider provider);
    event ProviderUpdated(string indexed providerName, address indexed providerAddress);

    constructor(
        address _aaveAddressProvider,
        address _balancerVault,
        address _uniswapV3Factory
    ) {
        aaveAddressProvider = _aaveAddressProvider;
        balancerVault = _balancerVault;
        uniswapV3Factory = _uniswapV3Factory;
    }

    /**
     * @notice Set the Aave address provider
     * @param _aaveAddressProvider The address of the Aave address provider
     */
    function setAaveAddressProvider(address _aaveAddressProvider) external onlyOwner {
        aaveAddressProvider = _aaveAddressProvider;
        emit ProviderUpdated("AAVE", _aaveAddressProvider);
    }

    /**
     * @notice Set the Balancer vault address
     * @param _balancerVault The address of the Balancer vault
     */
    function setBalancerVault(address _balancerVault) external onlyOwner {
        balancerVault = _balancerVault;
        emit ProviderUpdated("BALANCER", _balancerVault);
    }

    /**
     * @notice Set the Uniswap V3 factory address
     * @param _uniswapV3Factory The address of the Uniswap V3 factory
     */
    function setUniswapV3Factory(address _uniswapV3Factory) external onlyOwner {
        uniswapV3Factory = _uniswapV3Factory;
        emit ProviderUpdated("UNISWAP_V3", _uniswapV3Factory);
    }

    /**
     * @notice Execute a flash loan from the specified provider
     * @param loanToken The token to borrow
     * @param loanAmount The amount to borrow
     * @param callbackData Additional data to pass to the callback
     * @param provider The flash loan provider to use
     */
    function executeFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes memory callbackData,
        FlashLoanProvider provider
    ) public nonReentrant {
        require(loanAmount > 0, "FlashLoanCore: Loan amount must be greater than 0");
        require(loanToken != address(0), "FlashLoanCore: Invalid loan token");

        if (provider == FlashLoanProvider.AAVE) {
            _executeAaveFlashLoan(loanToken, loanAmount, callbackData);
        } else if (provider == FlashLoanProvider.BALANCER) {
            _executeBalancerFlashLoan(loanToken, loanAmount, callbackData);
        } else if (provider == FlashLoanProvider.UNISWAP_V3) {
            _executeUniswapV3FlashLoan(loanToken, loanAmount, callbackData);
        } else {
            revert("FlashLoanCore: Unsupported flash loan provider");
        }

        emit FlashLoanExecuted(loanToken, loanAmount, provider);
    }

    /**
     * @notice Execute a flash loan from Aave
     * @param loanToken The token to borrow
     * @param loanAmount The amount to borrow
     * @param callbackData Additional data to pass to the callback
     */
    function _executeAaveFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes memory callbackData
    ) internal {
        require(aaveAddressProvider != address(0), "FlashLoanCore: Aave address provider not set");
        
        address pool = IPoolAddressesProvider(aaveAddressProvider).getPool();
        require(pool != address(0), "FlashLoanCore: Aave pool not found");

        IPool(pool).flashLoanSimple(
            address(this),
            loanToken,
            loanAmount,
            callbackData,
            0
        );
    }

    /**
     * @notice Execute a flash loan from Balancer
     * @param loanToken The token to borrow
     * @param loanAmount The amount to borrow
     * @param callbackData Additional data to pass to the callback
     */
    function _executeBalancerFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes memory callbackData
    ) internal {
        require(balancerVault != address(0), "FlashLoanCore: Balancer vault not set");
        
        address[] memory tokens = new address[](1);
        tokens[0] = loanToken;
        
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = loanAmount;

        IBalancerVault(balancerVault).flashLoan(
            address(this),
            tokens,
            amounts,
            callbackData
        );
    }

    /**
     * @notice Execute a flash loan from Uniswap V3
     * @param loanToken The token to borrow
     * @param loanAmount The amount to borrow
     * @param callbackData Additional data to pass to the callback
     */
    function _executeUniswapV3FlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes memory callbackData
    ) internal {
        require(uniswapV3Factory != address(0), "FlashLoanCore: Uniswap V3 factory not set");
        
        // Find the best Uniswap V3 pool for the flash loan
        // We look for the pool with the highest liquidity for the loan token
        
        // Standard Uniswap V3 fee tiers
        uint24[4] memory feeTiers = [uint24(100), uint24(500), uint24(3000), uint24(10000)]; // 0.01%, 0.05%, 0.3%, 1%
        
        // Common token pairs to search for on Polygon (WMATIC, USDC, USDT, DAI, WBTC)
        address[5] memory commonPairs = [
            0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270, // WMATIC
            0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174, // USDC
            0xc2132D05D31c914a87C6611C10748AEb04B58e8F, // USDT
            0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063, // DAI
            0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6  // WBTC
        ];
        
        address bestPool;
        uint256 highestLiquidity;
        
        // Try to find a pool with the requested token and sufficient liquidity
        for (uint i = 0; i < commonPairs.length; i++) {
            // Skip if the pair token is the same as the loan token
            if (commonPairs[i] == loanToken) continue;
            
            for (uint j = 0; j < feeTiers.length; j++) {
                // Get the pool address for the given token pair and fee
                address pool;
                try IUniswapV3Factory(uniswapV3Factory).getPool(
                    loanToken,
                    commonPairs[i],
                    feeTiers[j]
                ) returns (address poolAddress) {
                    pool = poolAddress;
                } catch {
                    // Skip if the call fails
                    continue;
                }
                
                // Skip if pool doesn't exist
                if (pool == address(0)) continue;
                
                // Try to get liquidity information (this is simplified)
                try IUniswapV3Pool(pool).liquidity() returns (uint128 liquidity) {
                    if (liquidity > highestLiquidity) {
                        highestLiquidity = liquidity;
                        bestPool = pool;
                    }
                } catch {
                    // If we can't get liquidity, just continue
                    continue;
                }
            }
        }
        
        // If we couldn't find a pool with sufficient liquidity,
        // try to extract pool from callback data as fallback
        if (bestPool == address(0)) {
            // Use the first 20 bytes of callbackData as a pool address if provided
            // This is just a fallback and should be improved in production
            if (callbackData.length >= 20) {
                // Copy callbackData to memory first
                bytes memory callbackDataCopy = callbackData;
                address poolFromCallbackData;
                
                assembly {
                    // Load the first 20 bytes after the length prefix (skip 32 bytes)
                    poolFromCallbackData := mload(add(callbackDataCopy, 32))
                }
                
                // No need to mask the address as we already have an address type
                
                if (poolFromCallbackData != address(0)) {
                    try IUniswapV3Pool(poolFromCallbackData).token0() returns (address) {
                        try IUniswapV3Pool(poolFromCallbackData).token1() returns (address) {
                            bestPool = poolFromCallbackData;
                        } catch {}
                    } catch {}
                }
            }
        }
        
        require(bestPool != address(0), "FlashLoanCore: No suitable Uniswap V3 pool found");
        
        // Determine if loanToken is token0 or token1
        address token0 = IUniswapV3Pool(bestPool).token0();
        address token1 = IUniswapV3Pool(bestPool).token1();
        
        require(loanToken == token0 || loanToken == token1, "FlashLoanCore: Token not in pool");
        
        uint256 amount0 = loanToken == token0 ? loanAmount : 0;
        uint256 amount1 = loanToken == token1 ? loanAmount : 0;
        
        IUniswapV3Pool(bestPool).flash(
            address(this),
            amount0,
            amount1,
            callbackData
        );
    }

    // Aave V3 flash loan callback for Polygon
    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        require(initiator == address(this), "FlashLoanCore: Unauthorized initiator");
        
        // Execute custom logic with the borrowed funds
        _executeFlashLoanLogic(asset, amount, params);
        
        // Repay the loan
        uint256 amountOwed = amount + premium;
        IERC20(asset).safeTransfer(msg.sender, amountOwed);
        
        return true;
    }

    // For compatibility with Aave V2 on Polygon (if needed)
    function executeOperation(
        address[] calldata assets,
        uint256[] calldata amounts,
        uint256[] calldata premiums,
        address initiator,
        bytes calldata params
    ) external returns (bool) {
        require(initiator == address(this), "FlashLoanCore: Unauthorized initiator");
        require(assets.length == 1 && amounts.length == 1 && premiums.length == 1, 
                "FlashLoanCore: Only single asset flash loans supported");
                
        // Execute custom logic with the borrowed funds
        _executeFlashLoanLogic(assets[0], amounts[0], params);
        
        // Repay the loan
        uint256 amountOwed = amounts[0] + premiums[0];
        IERC20(assets[0]).safeTransfer(msg.sender, amountOwed);
        
        return true;
    }

    // Balancer flash loan callback
    function receiveFlashLoan(
        address[] memory tokens,
        uint256[] memory amounts,
        uint256[] memory feeAmounts,
        bytes memory userData
    ) external {
        require(msg.sender == balancerVault, "FlashLoanCore: Unauthorized sender");
        require(tokens.length == 1 && amounts.length == 1, "FlashLoanCore: Invalid flash loan parameters");
        
        // Execute custom logic with the borrowed funds
        _executeFlashLoanLogic(tokens[0], amounts[0], userData);
        
        // Repay the loan
        uint256 amountOwed = amounts[0] + feeAmounts[0];
        IERC20(tokens[0]).safeTransfer(balancerVault, amountOwed);
    }

    // Uniswap V3 flash callback
    function uniswapV3FlashCallback(
        uint256 fee0,
        uint256 fee1,
        bytes calldata data
    ) external {
        address pool = msg.sender;
        address token0 = IUniswapV3Pool(pool).token0();
        address token1 = IUniswapV3Pool(pool).token1();
        
        // Extract the borrowed token and amount
        address token;
        uint256 amount;
        uint256 fee;
        
        if (fee0 > 0) {
            token = token0;
            amount = fee0;
            fee = fee0;
        } else {
            token = token1;
            amount = fee1;
            fee = fee1;
        }
        
        // Execute custom logic with the borrowed funds
        _executeFlashLoanLogic(token, amount, data);
        
        // Repay the loan
        uint256 amountOwed = amount + fee;
        IERC20(token).safeTransfer(pool, amountOwed);
    }

    /**
     * @notice Execute custom logic with the borrowed funds
     * @dev This function should be overridden by derived contracts
     * @param token The borrowed token
     * @param amount The borrowed amount
     * @param data Additional data for the flash loan
     */
    function _executeFlashLoanLogic(
        address token,
        uint256 amount,
        bytes memory data
    ) internal virtual {
        // This function should be overridden by derived contracts
        revert("FlashLoanCore: _executeFlashLoanLogic not implemented");
    }

    /**
     * @notice Rescue tokens that are sent to this contract by mistake
     * @param token The token to rescue
     * @param to The address to send the tokens to
     * @param amount The amount of tokens to rescue
     */
    function rescueTokens(
        address token,
        address to,
        uint256 amount
    ) external onlyOwner {
        IERC20(token).safeTransfer(to, amount);
    }
}