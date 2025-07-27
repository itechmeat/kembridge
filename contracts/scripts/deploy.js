const { ethers } = require('hardhat');

async function main() {
  console.log('Deploying SimpleBridge contract to Sepolia...');
  
  // Get the account that will deploy the contract
  const [deployer] = await ethers.getSigners();
  console.log('Deploying contracts with the account:', deployer.address);
  
  // Check the deployer balance
  const balance = await ethers.provider.getBalance(deployer.address);
  console.log('Account balance:', ethers.formatEther(balance), 'ETH');
  
  // Deploy the contract
  const SimpleBridge = await ethers.getContractFactory('SimpleBridge');
  const simpleBridge = await SimpleBridge.deploy();
  
  // Wait for deployment to complete
  await simpleBridge.waitForDeployment();
  const contractAddress = await simpleBridge.getAddress();
  
  console.log('SimpleBridge deployed to:', contractAddress);
  console.log('Transaction hash:', simpleBridge.deploymentTransaction().hash);
  
  // Fund the contract with some ETH for testing
  console.log('Funding contract with 0.1 ETH...');
  const fundTx = await deployer.sendTransaction({
    to: contractAddress,
    value: ethers.parseEther('0.1')
  });
  
  await fundTx.wait();
  console.log('Contract funded! Transaction hash:', fundTx.hash);
  
  // Get contract balance
  const contractBalance = await ethers.provider.getBalance(contractAddress);
  console.log('Contract balance:', ethers.formatEther(contractBalance), 'ETH');
  
  // Get current block number
  const currentBlock = await ethers.provider.getBlockNumber();
  
  // Display deployment summary
  console.log('\n=== Deployment Summary ===');
  console.log('Contract Address:', contractAddress);
  console.log('Deployer Address:', deployer.address);
  console.log('Network:', (await ethers.provider.getNetwork()).name);
  console.log('Chain ID:', (await ethers.provider.getNetwork()).chainId);
  console.log('Block Number:', currentBlock);
  console.log('Gas Price:', ethers.formatUnits(await ethers.provider.getFeeData().then(f => f.gasPrice), 'gwei'), 'gwei');
  console.log('Contract Balance:', ethers.formatEther(contractBalance), 'ETH');
  
  // Create constants for Rust integration
  console.log('\n=== For Rust Integration ===');
  console.log('Add to constants.rs:');
  console.log(`pub const BRIDGE_CONTRACT_ADDRESS: &str = "${contractAddress}";`);
  console.log(`pub const BRIDGE_CONTRACT_DEPLOYED_BLOCK: u64 = ${currentBlock};`);
  
  // Test basic functionality
  console.log('\nTesting basic functionality...');
  try {
    const stats = await simpleBridge.getBridgeStats();
    console.log('Bridge Stats:', {
      balance: ethers.formatEther(stats[0]),
      locked: ethers.formatEther(stats[1]),
      unlocked: ethers.formatEther(stats[2]),
      activeBalance: ethers.formatEther(stats[3])
    });
    
    // Test contract ownership
    const owner = await simpleBridge.owner();
    console.log('Contract Owner:', owner);
    console.log('Is Deployer Owner:', owner.toLowerCase() === deployer.address.toLowerCase());
    
  } catch (error) {
    console.error('Error testing contract functionality:', error.message);
  }
  
  console.log('\nDeployment completed successfully!');
  
  // Return deployment info for further use
  return {
    contractAddress,
    deployerAddress: deployer.address,
    blockNumber: currentBlock,
    transactionHash: simpleBridge.deploymentTransaction().hash
  };
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });