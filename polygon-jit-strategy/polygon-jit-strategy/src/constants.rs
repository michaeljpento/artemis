//! Constants for the Polygon JIT strategy
use std::str::FromStr;
use ethers::types::Address;

// Common token addresses on Polygon
pub const WMATIC_ADDRESS: &str = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270";
pub const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
pub const USDT_ADDRESS: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
pub const DAI_ADDRESS: &str = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063";
pub const WBTC_ADDRESS: &str = "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6";
pub const WETH_ADDRESS: &str = "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619";
pub const AAVE_ADDRESS: &str = "0xD6DF932A45C0f255f85145f286eA0b292B21C90B";
pub const QUICK_ADDRESS: &str = "0x831753DD7087CaC61aB5644b308642cc1c33Dc13";

// DEX router addresses
pub const QUICKSWAP_ROUTER: &str = "0xa5E0829CaCEd8fFDD4De3c43696c57F7D7A678ff";
pub const SUSHISWAP_ROUTER: &str = "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506";
pub const UNISWAP_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
pub const CURVE_ROUTER: &str = "0x8474DdbE98F5aA3179B3B3F5942D724aFcdec9f6";

// Factory addresses
pub const QUICKSWAP_FACTORY: &str = "0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32";
pub const SUSHISWAP_FACTORY: &str = "0xc35DADB65012eC5796536bD9864eD8773aBc74C4";
pub const UNISWAP_V3_FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

// Position manager for Uniswap V3
pub const POSITION_MANAGER: &str = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";

// Flash loan provider addresses
pub const AAVE_ADDRESS_PROVIDER: &str = "0xd05e3E715d945B59290df0ae8eF85c1BdB684744";
pub const BALANCER_VAULT: &str = "0xBA12222222228d8Ba445958a75a0704d566BF2C8";

// Common pool addresses (most liquid pools) - for quick lookup
lazy_static::lazy_static! {
    // WMATIC-USDC QuickSwap pool
    pub static ref WMATIC_USDC_QUICKSWAP: Address = 
        Address::from_str("0x6e7a5FAFcec6BB1e78bAE2A1F0B612012BF14827").unwrap();
    
    // WMATIC-USDC SushiSwap pool
    pub static ref WMATIC_USDC_SUSHISWAP: Address = 
        Address::from_str("0xcd353F79d9FADe311fC3119B841e1f456b54e858").unwrap();
    
    // WETH-USDC QuickSwap pool
    pub static ref WETH_USDC_QUICKSWAP: Address = 
        Address::from_str("0x853Ee4b2A13f8a742d64C8F088bE7bA2131f670d").unwrap();
    
    // WETH-USDC SushiSwap pool
    pub static ref WETH_USDC_SUSHISWAP: Address = 
        Address::from_str("0x34965ba0ac2451A34a0471F04CCa3F990b8dea27").unwrap();
    
    // WBTC-USDC QuickSwap pool
    pub static ref WBTC_USDC_QUICKSWAP: Address = 
        Address::from_str("0xF6a637525402643B0654a54bEAd2Cb9A83C8B498").unwrap();
    
    // Common tokens as addresses
    pub static ref WMATIC: Address = Address::from_str(WMATIC_ADDRESS).unwrap();
    pub static ref USDC: Address = Address::from_str(USDC_ADDRESS).unwrap();
    pub static ref USDT: Address = Address::from_str(USDT_ADDRESS).unwrap();
    pub static ref DAI: Address = Address::from_str(DAI_ADDRESS).unwrap();
    pub static ref WBTC: Address = Address::from_str(WBTC_ADDRESS).unwrap();
    pub static ref WETH: Address = Address::from_str(WETH_ADDRESS).unwrap();
}

// Dex type enum that matches the contract's enum
pub enum DexType {
    QuickSwap = 0,
    SushiSwap = 1,
    UniswapV3 = 2,
    Curve = 3,
}

// Pool type enum for JIT provider that matches contract's enum
pub enum PoolType {
    QuickSwap = 0,
    SushiSwap = 1,
    UniswapV3 = 2,
}

// Flash loan provider type that matches contract's enum
pub enum FlashLoanProvider {
    Aave = 0,
    Balancer = 1,
    UniswapV3 = 2,
}
