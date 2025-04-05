// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/PolygonFlashArbExecutor.sol";
import "../src/PolygonJITLiquidityProvider.sol";

contract DeployPolygonScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // Deploy contracts with Polygon addresses
        
        // Aave V3 Address Provider on Polygon
        address aaveAddressProvider = 0xd05e3E715d945B59290df0ae8eF85c1BdB684744;
        
        // Balancer Vault on Polygon
        address balancerVault = 0xBA12222222228d8Ba445958a75a0704d566BF2C8;
        
        // Uniswap V3 Factory on Polygon
        address uniswapV3Factory = 0x1F98431c8aD98523631AE4a59f267346ea31F984;
        
        // QuickSwap Factory (Uniswap V2 Fork) on Polygon
        address quickswapFactory = 0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32;
        
        // SushiSwap Factory on Polygon
        address sushiswapFactory = 0xc35DADB65012eC5796536bD9864eD8773aBc74C4;
        
        // Uniswap V3 NFT Position Manager on Polygon
        address uniswapV3PositionManager = 0xC36442b4a4522E871399CD717aBDD847Ab11FE88;

        // Deploy PolygonFlashArbExecutor
        PolygonFlashArbExecutor polygonFlashArbExecutor = new PolygonFlashArbExecutor(
            aaveAddressProvider,
            balancerVault,
            uniswapV3Factory,
            quickswapFactory,
            sushiswapFactory
        );

        // Deploy PolygonJITLiquidityProvider
        PolygonJITLiquidityProvider polygonJitLiquidityProvider = new PolygonJITLiquidityProvider(
            aaveAddressProvider,
            balancerVault,
            uniswapV3Factory,
            quickswapFactory,
            sushiswapFactory,
            uniswapV3PositionManager
        );

        // Log deployed addresses
        console.log("PolygonFlashArbExecutor deployed at:", address(polygonFlashArbExecutor));
        console.log("PolygonJITLiquidityProvider deployed at:", address(polygonJitLiquidityProvider));

        vm.stopBroadcast();
    }
}