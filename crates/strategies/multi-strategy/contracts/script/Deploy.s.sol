// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/FlashArbExecutor.sol";
import "../src/JITLiquidityProvider.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);
        
        // Addresses for mainnet
        address WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
        address AAVE_LENDING_POOL = 0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9;
        address UNISWAP_V2_FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
        address NONFUNGIBLE_POSITION_MANAGER = 0xC36442b4a4522E871399CD717aBDD847Ab11FE88;
        
        // Deploy FlashArbExecutor
        FlashArbExecutor flashArbExecutor = new FlashArbExecutor(
            WETH,
            AAVE_LENDING_POOL
        );
        
        // Deploy JITLiquidityProvider
        JITLiquidityProvider jitLiquidityProvider = new JITLiquidityProvider(
            WETH,
            UNISWAP_V2_FACTORY,
            NONFUNGIBLE_POSITION_MANAGER
        );
        
        // Whitelist the caller (the bot operator)
        address botOperator = msg.sender;
        flashArbExecutor.addWhitelistedCaller(botOperator);
        jitLiquidityProvider.addWhitelistedCaller(botOperator);
        
        // Log the deployed contract addresses
        console.log("FlashArbExecutor deployed at:", address(flashArbExecutor));
        console.log("JITLiquidityProvider deployed at:", address(jitLiquidityProvider));
        
        vm.stopBroadcast();
        
        // Save deployment information to file
        string memory deploymentInfo = string(abi.encodePacked(
            "FLASH_ARB_EXECUTOR_ADDRESS=", vm.toString(address(flashArbExecutor)), "\n",
            "JIT_LIQUIDITY_PROVIDER_ADDRESS=", vm.toString(address(jitLiquidityProvider)), "\n"
        ));
        
        vm.writeFile("deployment.env", deploymentInfo);
    }
}