// General signature verification utilities

use crate::chains::{ChainType, MultiChainVerifier};
use crate::errors::AuthError;

#[derive(Clone)]
pub struct SignatureVerifier {
    multi_chain_verifier: MultiChainVerifier,
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self {
            multi_chain_verifier: MultiChainVerifier::new(),
        }
    }

    pub async fn verify_message_signature(
        &self,
        chain_type: ChainType,
        message: &str,
        signature: &str,
        address: &str,
    ) -> Result<bool, AuthError> {
        // Validate address format first
        if !self.multi_chain_verifier.validate_address(chain_type, address)? {
            return Err(AuthError::InvalidWalletAddress);
        }

        // Verify signature
        self.multi_chain_verifier
            .verify_signature(chain_type, message, signature, address)
            .await
    }

    pub fn validate_address_format(
        &self,
        chain_type: ChainType,
        address: &str,
    ) -> Result<bool, AuthError> {
        self.multi_chain_verifier.validate_address(chain_type, address)
    }
}