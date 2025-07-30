# KEMBridge Crypto

Post-quantum cryptography module for KEMBridge using ML-KEM-1024.

## Overview

This crate implements FIPS 203 ML-KEM-1024 (Module-Lattice-Based Key-Encapsulation Mechanism) for quantum-safe cryptographic key exchange. ML-KEM provides 256-bit security level and is designed to resist attacks from both classical and quantum computers.

## Features

- **FIPS 203 Compliance**: Uses the standardized ML-KEM-1024 algorithm
- **Pure Rust**: No C dependencies, works on all Rust-supported platforms
- **Memory Safe**: Automatic secure memory cleanup with zeroization
- **High Performance**: Faster than X25519 for key exchange operations
- **Easy to Use**: High-level API with comprehensive error handling

## Quick Start

```rust
use kembridge_crypto::{QuantumKeyManager, MlKemCrypto};
use rand::thread_rng;

// High-level API (recommended)
let manager = QuantumKeyManager::new();
let keypair = manager.generate_ml_kem_keypair()?;

// Encapsulate a shared secret
let result = manager.secure_encapsulate(&keypair)?;

// Decapsulate the shared secret
let shared_secret = manager.secure_decapsulate(&keypair, &result.ciphertext)?;

assert_eq!(result.shared_secret, shared_secret);
```

## Low-Level API

```rust
use kembridge_crypto::MlKemCrypto;
use rand::thread_rng;

let mut rng = thread_rng();

// Generate key pair
let (private_key, public_key) = MlKemCrypto::generate_keypair(&mut rng)?;

// Encapsulate
let (ciphertext, shared_secret_sender) = MlKemCrypto::encapsulate(&public_key, &mut rng)?;

// Decapsulate
let shared_secret_receiver = MlKemCrypto::decapsulate(&private_key, &ciphertext)?;

assert_eq!(shared_secret_sender, shared_secret_receiver);
```

## Algorithm Parameters

| Parameter | Value |
|-----------|-------|
| Algorithm | ML-KEM-1024 |
| Standard | FIPS 203 |
| Security Level | 256-bit |
| Public Key Size | 1,568 bytes |
| Private Key Size | 3,168 bytes |
| Ciphertext Size | 1,568 bytes |
| Shared Secret Size | 32 bytes |

## Performance

On modern hardware, typical performance is:

- **Key Generation**: < 1ms per key pair
- **Encapsulation**: < 100μs per operation
- **Decapsulation**: < 200μs per operation

ML-KEM-1024 is typically faster than X25519, but requires approximately 1.6KB additional data transfer.

## Security Considerations

⚠️ **Important Security Notes:**

1. **Audit Status**: The underlying `ml-kem` crate has not been independently audited
2. **Hybrid Approach**: Consider using ML-KEM in combination with classical cryptography
3. **Key Rotation**: Implement regular key rotation for long-term security
4. **Memory Safety**: Private keys are automatically zeroized when dropped

## Examples

### Key Serialization

```rust
let manager = QuantumKeyManager::new();
let keypair = manager.generate_ml_kem_keypair()?;

// Export for storage
let exported = manager.export_keypair(&keypair);

// Import from storage
let imported_keypair = manager.import_keypair(exported)?;

// Verify the imported key pair works
manager.verify_keypair(&imported_keypair)?;
```

### Error Handling

```rust
use kembridge_crypto::QuantumCryptoError;

match manager.generate_ml_kem_keypair() {
    Ok(keypair) => {
        // Use the key pair
    },
    Err(QuantumCryptoError::KeyGenerationFailed(msg)) => {
        eprintln!("Failed to generate keys: {}", msg);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run performance benchmarks:

```bash
cargo test test_performance_characteristics -- --nocapture
```

## Integration with KEMBridge

This crate is designed to integrate with the KEMBridge cross-chain bridge:

1. **Phase 3.1**: Core ML-KEM-1024 implementation (current)
2. **Phase 3.2**: Database integration and HTTP API endpoints
3. **Phase 3.3**: Hybrid cryptography with AES-256-GCM
4. **Phase 4.3**: Integration with cross-chain bridge operations

## Dependencies

- `ml-kem 0.2.1`: FIPS 203 ML-KEM implementation
- `rand 0.9.1`: Cryptographically secure random number generation
- `zeroize 1.8.1`: Secure memory cleanup
- `serde 1.0.217`: Serialization support
- `chrono 0.4.38`: Date/time handling
- `uuid 1.11.0`: Unique identifiers
- `thiserror 2.0.11`: Error handling

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.

## Contributing

This crate is part of the KEMBridge project. Contributions should follow the project's security guidelines and coding standards.

## Roadmap

- **Phase 3.2**: Database integration and key management API
- **Phase 3.3**: Hybrid cryptography implementation
- **Future**: Integration with ML-DSA (Dilithium) and SLH-DSA (SPHINCS+)