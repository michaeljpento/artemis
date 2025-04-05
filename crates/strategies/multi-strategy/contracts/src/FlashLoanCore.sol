// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title FlashLoanCore
 * @dev Core functionality for flash loan operations across all strategies
 * This abstract contract implements the core flash loan logic that can be used by all strategies
 */
abstract contract FlashLoanCore is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    
    // Events
    event FlashLoanExecuted(address indexed token, uint256 amount, address indexed executor);
    event WithdrawToken(address indexed token, uint256 amount);
    
    // Constants
    uint256 private constant MAX_INT = type(uint256).max;
    address public immutable WETH;
    
    // Whitelisted callers
    mapping(address => bool) public whitelistedCallers;
    
    // Flash loan providers
    enum FlashLoanProvider {
        AAVE,
        BALANCER,
        UNISWAP_V3
    }
    
    // Addresses of flash loan providers
    address public aaveLendingPool;
    address public balancerVault;
    address public uniswapV3Factory;
    
    constructor(
        address _weth,
        address _aaveLendingPool,
        address _balancerVault,
        address _uniswapV3Factory
    ) {
        WETH = _weth;
        aaveLendingPool = _aaveLendingPool;
        balancerVault = _balancerVault;
        uniswapV3Factory = _uniswapV3Factory;
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
     * @dev Update the address of a flash loan provider
     * @param provider The provider to update
     * @param newAddress The new address for the provider
     */
    function updateFlashLoanProvider(FlashLoanProvider provider, address newAddress) external onlyOwner {
        if (provider == FlashLoanProvider.AAVE) {
            aaveLendingPool = newAddress;
        } else if (provider == FlashLoanProvider.BALANCER) {
            balancerVault = newAddress;
        } else if (provider == FlashLoanProvider.UNISWAP_V3) {
            uniswapV3Factory = newAddress;
        }
    }
    
    /**
     * @dev Execute a flash loan using the most efficient provider
     * @param loanToken The token to borrow
     * @param loanAmount The amount to borrow
     * @param callbackData Data to pass to the callback function
     * @param provider The flash loan provider to use
     */
    function executeFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes calldata callbackData,
        FlashLoanProvider provider
    ) external nonReentrant {
        require(whitelistedCallers[msg.sender], "Caller not whitelisted");
        
        if (provider == FlashLoanProvider.AAVE) {
            executeAaveFlashLoan(loanToken, loanAmount, callbackData);
        } else if (provider == FlashLoanProvider.BALANCER) {
            executeBalancerFlashLoan(loanToken, loanAmount, callbackData);
        } else if (provider == FlashLoanProvider.UNISWAP_V3) {
            executeUniswapV3FlashLoan(loanToken, loanAmount, callbackData);
        } else {
            revert("Unsupported flash loan provider");
        }
        
        emit FlashLoanExecuted(loanToken, loanAmount, msg.sender);
    }
    
    /**
     * @dev Execute a flash loan using Aave
     */
    function executeAaveFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes calldata callbackData
    ) internal {
        address[] memory assets = new address[](1);
        assets[0] = loanToken;
        
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = loanAmount;
        
        uint256[] memory modes = new uint256[](1);
        modes[0] = 0; // no debt - we'll pay it back immediately
        
        // Call Aave's flash loan function
        bytes memory data = abi.encode(callbackData, FlashLoanProvider.AAVE);
        
        // Solidity interface for Aave's flash loan
        IAaveLendingPool(aaveLendingPool).flashLoan(
            address(this),
            assets,
            amounts,
            modes,
            address(this),
            data,
            0
        );
    }
    
    /**
     * @dev Execute a flash loan using Balancer
     */
    function executeBalancerFlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes calldata callbackData
    ) internal {
        // Encode the original callback data along with provider info
        bytes memory data = abi.encode(callbackData, FlashLoanProvider.BALANCER);
        
        // Call Balancer's flash loan function
        IERC20[] memory tokens = new IERC20[](1);
        tokens[0] = IERC20(loanToken);
        
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = loanAmount;
        
        // Solidity interface for Balancer's flash loan
        IBalancerVault(balancerVault).flashLoan(
            address(this),
            tokens,
            amounts,
            data
        );
    }
    
    /**
     * @dev Execute a flash loan using Uniswap V3
     */
    function executeUniswapV3FlashLoan(
        address loanToken,
        uint256 loanAmount,
        bytes calldata callbackData
    ) internal {
        // For Uniswap V3, we need to find a pool with enough liquidity
        // Typically, a WETH/token pool with 0.3% fee is a good choice
        address poolAddress = IUniswapV3Factory(uniswapV3Factory).getPool(
            loanToken,
            WETH,
            3000 // 0.3% fee tier
        );
        
        require(poolAddress != address(0), "No suitable Uniswap V3 pool found");
        
        // Encode the original callback data along with provider info
        bytes memory data = abi.encode(callbackData, FlashLoanProvider.UNISWAP_V3, loanToken);
        
        // Determine which token is token0 in the pool
        IUniswapV3Pool pool = IUniswapV3Pool(poolAddress);
        bool isToken0 = pool.token0() == loanToken;
        
        // Call the flash function on the pool
        pool.flash(
            address(this),
            isToken0 ? loanAmount : 0,
            isToken0 ? 0 : loanAmount,
            data
        );
    }
    
    /**
     * @dev Withdraw tokens from the contract
     * @param token Token to withdraw
     * @param amount Amount to withdraw (0 for all)
     */
    function withdrawToken(address token, uint256 amount) external onlyOwner {
        uint256 withdrawAmount = amount == 0 ? IERC20(token).balanceOf(address(this)) : amount;
        IERC20(token).safeTransfer(owner(), withdrawAmount);
        emit WithdrawToken(token, withdrawAmount);
    }
    
    /**
     * @dev Fallback function to receive ETH
     */
    receive() external payable {}
}

// Interface for Aave's flash loan
interface IAaveLendingPool {
    function flashLoan(
        address receiverAddress,
        address[] calldata assets,
        uint256[] calldata amounts,
        uint256[] calldata modes,
        address onBehalfOf,
        bytes calldata params,
        uint16 referralCode
    ) external;
}

// Interface for Balancer's flash loan
interface IBalancerVault {
    function flashLoan(
        address recipient,
        IERC20[] memory tokens,
        uint256[] memory amounts,
        bytes memory userData
    ) external;
}

// Interface for Uniswap V3 factory
interface IUniswapV3Factory {
    function getPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external view returns (address pool);
}

// Interface for Uniswap V3 pool
interface IUniswapV3Pool {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function flash(
        address recipient,
        uint256 amount0,
        uint256 amount1,
        bytes calldata data
    ) external;
}