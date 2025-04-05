// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/PolygonJITLiquidityProvider.sol";

contract DeployPolygonBalancerScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        // Polygon-specific gas settings for faster inclusion
        uint256 priorityFee = 35 gwei; // Competitive priority fee for Polygon
        
        // Start broadcast with Polygon-optimized gas settings
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

        // Deploy Balancer-optimized JIT Liquidity Provider for Polygon
        PolygonJITLiquidityProvider balancerJitProvider = new PolygonJITLiquidityProvider(
            aaveAddressProvider,
            balancerVault,
            uniswapV3Factory,
            quickswapFactory,
            sushiswapFactory,
            uniswapV3PositionManager
        );

        // Configure gas settings for Polygon's network characteristics
        balancerJitProvider.setPolygonMaxGasPrice(100 gwei);
        balancerJitProvider.setPolygonPriorityFee(priorityFee);

        // Log deployed address
        console.log("Balancer-Optimized JIT Liquidity Provider deployed at:", address(balancerJitProvider));

        vm.stopBroadcast();
    }
}