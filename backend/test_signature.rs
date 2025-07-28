// Test signature verification with real MetaMask data
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1};
use sha3::{Digest, Keccak256};

fn main() {
    // Real data from MetaMask from the logs
    let message = "Welcome to KEMBridge!\n\nThis request will not trigger a blockchain transaction or cost any gas fees.\n\nYour authentication status will reset after 24 hours.\n\nWallet address:\n0x7404555145Ce95D5d78e9d0Df748E1BBe5605EFA\n\nNonce:\ned52d88d9758644ccc105b232fbda8fcb3b124a2e1a614dbaf79da792175b8e5";
    let signature = "0x56ca2c3a7e192a3445af82b5a76ff76802812a96290614cf3bc442cb643a15ad0fe20bc12936b4f7a44887f7de32a671b";
    let expected_address = "0x7404555145Ce95D5d78e9d0Df748E1BBe5605EFA";
    
    println!("Testing signature verification...");
    println!("Message length: {}", message.len());
    println!("Signature length: {}", signature.len());
    println!("Expected address: {}", expected_address);
    
    // Test our verification logic
    match verify_signature(message, signature, expected_address) {
        Ok(valid) => {
            println!("✅ Verification result: {}", if valid { "VALID" } else { "INVALID" });
        }
        Err(e) => {
            println!("❌ Verification error: {:?}", e);
        }
    }
}

fn verify_signature(message: &str, signature: &str, expected_address: &str) -> Result<bool, String> {
    let secp = Secp256k1::new();
    
    // Ethereum signed message format
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    println!("Prefixed message: {:?}", prefixed_message);
    
    let mut hasher = sha2::Sha256::new();
    hasher.update(prefixed_message.as_bytes());
    let message_hash = hasher.finalize();
    let message = Message::from_digest(message_hash.into());

    // Parse signature (65 bytes: r + s + v)
    let hex_str = signature.strip_prefix("0x").unwrap_or(signature);
    let sig_bytes = hex::decode(hex_str).map_err(|e| format!("Hex decode error: {}", e))?;
    
    if sig_bytes.len() != 65 {
        return Err(format!("Invalid signature length: {} (expected 65)", sig_bytes.len()));
    }

    let recovery_id = sig_bytes[64];
    let recovery_id = secp256k1::ecdsa::RecoveryId::from_u8_masked(recovery_id);

    let signature = RecoverableSignature::from_compact(&sig_bytes[0..64], recovery_id)
        .map_err(|e| format!("Signature parse error: {}", e))?;

    let recovered_pubkey = secp.recover_ecdsa(message, &signature)
        .map_err(|e| format!("Key recovery error: {}", e))?;
    
    // Convert to address using Keccak-256
    let public_key_bytes = recovered_pubkey.serialize_uncompressed();
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]);
    let hash = hasher.finalize();
    let address = &hash[12..];
    let recovered_address = format!("0x{}", hex::encode(address));
    
    println!("Recovered address: {}", recovered_address);
    println!("Expected address:  {}", expected_address);
    
    Ok(recovered_address.to_lowercase() == expected_address.to_lowercase())
}