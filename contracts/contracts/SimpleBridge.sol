// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/**
 * @title SimpleBridge
 * @dev A simple bridge contract for KEMBridge demo purposes
 * @notice This contract handles ETH lock/unlock operations for cross-chain transfers
 */
contract SimpleBridge {
    // Events
    event TokensLocked(
        address indexed user,
        uint256 amount,
        string recipientChain,
        string quantumHash,
        uint256 timestamp
    );
    
    event TokensUnlocked(
        address indexed recipient,
        uint256 amount,
        string sourceChain,
        string quantumHash,
        uint256 timestamp
    );
    
    event BridgeDeposit(
        address indexed user,
        uint256 amount,
        string destinationChain,
        uint256 timestamp
    );
    
    event BridgeWithdrawal(
        address indexed recipient,
        uint256 amount,
        string sourceChain,
        uint256 timestamp
    );
    
    // State variables
    address public owner;
    uint256 public totalLocked;
    uint256 public totalUnlocked;
    mapping(bytes32 => bool) public processedHashes;
    mapping(address => uint256) public userBalances;
    
    // Constants
    uint256 public constant MIN_LOCK_AMOUNT = 0.001 ether;
    uint256 public constant MAX_LOCK_AMOUNT = 10 ether;
    
    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }
    
    modifier validAmount(uint256 amount) {
        require(amount >= MIN_LOCK_AMOUNT, "Amount too small");
        require(amount <= MAX_LOCK_AMOUNT, "Amount too large");
        _;
    }
    
    modifier notProcessed(bytes32 hash) {
        require(!processedHashes[hash], "Already processed");
        _;
    }
    
    constructor() {
        owner = msg.sender;
        totalLocked = 0;
        totalUnlocked = 0;
    }
    
    /**
     * @dev Lock ETH tokens for cross-chain transfer
     * @param recipientChain Target blockchain (e.g., "near")
     * @param quantumHash Quantum-protected hash for verification
     */
    function lockTokens(
        string memory recipientChain,
        string memory quantumHash
    ) external payable validAmount(msg.value) {
        require(bytes(recipientChain).length > 0, "Invalid recipient chain");
        require(bytes(quantumHash).length > 0, "Invalid quantum hash");
        
        // Update balances
        userBalances[msg.sender] += msg.value;
        totalLocked += msg.value;
        
        // Create unique hash for this transaction
        bytes32 txHash = keccak256(abi.encodePacked(
            msg.sender,
            msg.value,
            recipientChain,
            quantumHash,
            block.timestamp
        ));
        
        // Mark as processed
        processedHashes[txHash] = true;
        
        // Emit events
        emit TokensLocked(
            msg.sender,
            msg.value,
            recipientChain,
            quantumHash,
            block.timestamp
        );
        
        emit BridgeDeposit(
            msg.sender,
            msg.value,
            recipientChain,
            block.timestamp
        );
    }
    
    /**
     * @dev Unlock ETH tokens from cross-chain transfer
     * @param recipient Address to receive unlocked tokens
     * @param amount Amount to unlock
     * @param sourceChain Source blockchain (e.g., "near")
     * @param quantumHash Quantum-protected hash for verification
     */
    function unlockTokens(
        address payable recipient,
        uint256 amount,
        string memory sourceChain,
        string memory quantumHash
    ) external onlyOwner validAmount(amount) {
        require(recipient != address(0), "Invalid recipient");
        require(bytes(sourceChain).length > 0, "Invalid source chain");
        require(bytes(quantumHash).length > 0, "Invalid quantum hash");
        require(address(this).balance >= amount, "Insufficient contract balance");
        
        // Create unique hash for this transaction
        bytes32 txHash = keccak256(abi.encodePacked(
            recipient,
            amount,
            sourceChain,
            quantumHash,
            block.timestamp
        ));
        
        // Check if already processed
        require(!processedHashes[txHash], "Already processed");
        
        // Mark as processed
        processedHashes[txHash] = true;
        
        // Update balances
        totalUnlocked += amount;
        
        // Transfer ETH
        (bool success, ) = recipient.call{value: amount}("");
        require(success, "Transfer failed");
        
        // Emit events
        emit TokensUnlocked(
            recipient,
            amount,
            sourceChain,
            quantumHash,
            block.timestamp
        );
        
        emit BridgeWithdrawal(
            recipient,
            amount,
            sourceChain,
            block.timestamp
        );
    }
    
    /**
     * @dev Emergency withdrawal for owner
     * @param amount Amount to withdraw
     */
    function emergencyWithdraw(uint256 amount) external onlyOwner {
        require(address(this).balance >= amount, "Insufficient balance");
        require(amount > 0, "Invalid amount");
        
        (bool success, ) = owner.call{value: amount}("");
        require(success, "Emergency withdrawal failed");
    }
    
    /**
     * @dev Get contract balance
     */
    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev Get bridge statistics
     */
    function getBridgeStats() external view returns (
        uint256 balance,
        uint256 locked,
        uint256 unlocked,
        uint256 activeBalance
    ) {
        return (
            address(this).balance,
            totalLocked,
            totalUnlocked,
            totalLocked - totalUnlocked
        );
    }
    
    /**
     * @dev Check if a hash has been processed
     */
    function isProcessed(bytes32 hash) external view returns (bool) {
        return processedHashes[hash];
    }
    
    /**
     * @dev Get user's locked balance
     */
    function getUserBalance(address user) external view returns (uint256) {
        return userBalances[user];
    }
    
    /**
     * @dev Transfer ownership
     */
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Invalid new owner");
        owner = newOwner;
    }
    
    /**
     * @dev Receive function to accept ETH
     */
    receive() external payable {
        // Allow contract to receive ETH
        // This is useful for initial funding
    }
    
    /**
     * @dev Fallback function
     */
    fallback() external payable {
        revert("Function not found");
    }
}