// Deploy script for multi-strategy contracts
// Run with: npx hardhat run scripts/deploy.js --network <network>

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);
  
  // Get contract factories
  const FlashArbExecutor = await ethers.getContractFactory("FlashArbExecutor");
  const JITLiquidityProvider = await ethers.getContractFactory("JITLiquidityProvider");
  
  // Deploy contracts
  
  // First, define the constructor parameters
  
  // WETH address (Mainnet)
  const WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
  
  // Aave Lending Pool address (Mainnet V2)
  const AAVE_LENDING_POOL = "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9";
  
  // Uniswap V2 Factory address
  const UNISWAP_V2_FACTORY = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
  
  // Uniswap V3 Position Manager
  const NONFUNGIBLE_POSITION_MANAGER = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";
  
  // Deploy FlashArbExecutor
  console.log("Deploying FlashArbExecutor...");
  const flashArbExecutor = await FlashArbExecutor.deploy(WETH, AAVE_LENDING_POOL);
  await flashArbExecutor.deployed();
  console.log("FlashArbExecutor deployed to:", flashArbExecutor.address);
  
  // Deploy JITLiquidityProvider
  console.log("Deploying JITLiquidityProvider...");
  const jitLiquidityProvider = await JITLiquidityProvider.deploy(
    WETH,
    UNISWAP_V2_FACTORY,
    NONFUNGIBLE_POSITION_MANAGER
  );
  await jitLiquidityProvider.deployed();
  console.log("JITLiquidityProvider deployed to:", jitLiquidityProvider.address);
  
  // Set up permissions
  
  // Add the bot's address as a whitelisted caller for both contracts
  const BOT_ADDRESS = process.env.BOT_ADDRESS || deployer.address;
  
  console.log("Whitelisting caller:", BOT_ADDRESS);
  
  await flashArbExecutor.addWhitelistedCaller(BOT_ADDRESS);
  await jitLiquidityProvider.addWhitelistedCaller(BOT_ADDRESS);
  
  console.log("Deployment complete!");
  
  // Write the addresses to a deployment file
  const fs = require("fs");
  const deploymentInfo = {
    network: network.name,
    flashArbExecutor: flashArbExecutor.address,
    jitLiquidityProvider: jitLiquidityProvider.address,
    timestamp: new Date().toISOString(),
  };
  
  fs.writeFileSync(
    `deployment-${network.name}.json`,
    JSON.stringify(deploymentInfo, null, 2)
  );
  
  console.log("Deployment info saved to deployment.json");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });