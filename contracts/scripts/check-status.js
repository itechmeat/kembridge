const { ethers } = require('hardhat');
const fs = require('fs');

async function checkWalletAndContract() {
    try {
        console.log('=== Wallet & Balance Check ===');
        
        // Get the first signer from Hardhat
        const [signer] = await ethers.getSigners();
        console.log('Wallet Address:', signer.address);
        
        // Check balance
        const balance = await ethers.provider.getBalance(signer.address);
        console.log('Balance:', ethers.formatEther(balance), 'ETH');
        
        // Check network
        const network = await ethers.provider.getNetwork();
        console.log('Network:', network.name, 'Chain ID:', network.chainId.toString());
        
        // Get current gas price
        const feeData = await ethers.provider.getFeeData();
        console.log('Gas Price:', ethers.formatUnits(feeData.gasPrice, 'gwei'), 'gwei');
        
        // Estimate deployment cost (contract deployment uses ~600-800k gas)
        const deploymentGasEstimate = 800000n;
        const deploymentCost = deploymentGasEstimate * feeData.gasPrice;
        console.log('Estimated deployment cost:', ethers.formatEther(deploymentCost), 'ETH');
        
        // Check if we have enough for deployment
        const hasEnoughForDeployment = balance > deploymentCost;
        console.log('Has enough for deployment:', hasEnoughForDeployment);
        
        if (hasEnoughForDeployment) {
            const remaining = balance - deploymentCost;
            console.log('Remaining after deployment:', ethers.formatEther(remaining), 'ETH');
        } else {
            const needed = deploymentCost - balance;
            console.log('Additional ETH needed:', ethers.formatEther(needed), 'ETH');
        }
        
        // Check current block
        const blockNumber = await ethers.provider.getBlockNumber();
        console.log('Current Block:', blockNumber);
        
        // Check if contract is already deployed
        const contractAddress = process.env.BRIDGE_CONTRACT_ADDRESS;
        const deployedBlock = process.env.BRIDGE_CONTRACT_DEPLOYED_BLOCK;
        
        if (contractAddress && contractAddress !== 'YOUR_DEPLOYED_CONTRACT_ADDRESS') {
            console.log('\n=== Contract Testing ===');
            console.log('Contract Address:', contractAddress);
            console.log('Deployed Block:', deployedBlock);
            
            // Basic contract info
            const contractBalance = await ethers.provider.getBalance(contractAddress);
            console.log('Contract Balance:', ethers.formatEther(contractBalance), 'ETH');
            
            const code = await ethers.provider.getCode(contractAddress);
            if (code === '0x') {
                console.log('❌ No contract deployed at this address');
                return;
            } else {
                console.log('✅ Contract code deployed successfully');
                console.log('   Code size:', code.length / 2 - 1, 'bytes');
            }
            
            // Load contract ABI and test functions
            try {
                const contractArtifact = JSON.parse(fs.readFileSync('./artifacts/contracts/SimpleBridge.sol/SimpleBridge.json', 'utf8'));
                const contract = new ethers.Contract(contractAddress, contractArtifact.abi, ethers.provider);
                
                console.log('\n=== Contract Function Tests ===');
                
                // Test 1: Get contract owner
                try {
                    const owner = await contract.owner();
                    console.log('✅ Contract Owner:', owner);
                    console.log('   Is deployer owner:', owner.toLowerCase() === signer.address.toLowerCase());
                } catch (error) {
                    console.log('❌ Failed to get owner:', error.message);
                }
                
                // Test 2: Get contract balance via contract function
                try {
                    const contractBalanceFromContract = await contract.getBalance();
                    console.log('✅ Contract Balance (via function):', ethers.formatEther(contractBalanceFromContract), 'ETH');
                } catch (error) {
                    console.log('❌ Failed to get balance via contract:', error.message);
                }
                
                // Test 3: Get bridge statistics
                try {
                    const stats = await contract.getBridgeStats();
                    console.log('✅ Bridge Stats:', {
                        balance: ethers.formatEther(stats[0]),
                        locked: ethers.formatEther(stats[1]),
                        unlocked: ethers.formatEther(stats[2]),
                        activeBalance: ethers.formatEther(stats[3])
                    });
                } catch (error) {
                    console.log('❌ Failed to get bridge stats:', error.message);
                }
                
                // Test 4: Check constants
                try {
                    const minAmount = await contract.MIN_LOCK_AMOUNT();
                    const maxAmount = await contract.MAX_LOCK_AMOUNT();
                    console.log('✅ Lock Amount Range:', 
                        ethers.formatEther(minAmount), 'ETH to', 
                        ethers.formatEther(maxAmount), 'ETH');
                } catch (error) {
                    console.log('❌ Failed to get constants:', error.message);
                }
                
                // Test 5: Check blocks since deployment
                if (deployedBlock) {
                    console.log('✅ Blocks since deployment:', blockNumber - parseInt(deployedBlock));
                }
                
                console.log('\n🎉 Contract testing completed successfully!');
                
            } catch (error) {
                console.log('❌ Failed to load contract ABI or test functions:', error.message);
            }
        } else {
            console.log('\n⚠️  No contract address found in .env file');
            console.log('   Set BRIDGE_CONTRACT_ADDRESS after deployment');
        }
        
    } catch (error) {
        console.error('❌ Error:', error.message);
        throw error;
    }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
checkWalletAndContract()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });