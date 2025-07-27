use crate::{BridgeError, SwapOperation, SwapStatus};
use std::collections::HashMap;
use chrono::Utc;

pub struct StateMachine {
    transitions: HashMap<SwapStatus, Vec<SwapStatus>>,
}

impl StateMachine {
    pub fn new() -> Self {
        let mut transitions = HashMap::new();
        
        // Define valid state transitions
        transitions.insert(
            SwapStatus::Initialized,
            vec![SwapStatus::EthLocking, SwapStatus::Failed, SwapStatus::Cancelled]
        );
        
        transitions.insert(
            SwapStatus::EthLocking,
            vec![SwapStatus::EthLocked, SwapStatus::Failed, SwapStatus::Timeout]
        );
        
        transitions.insert(
            SwapStatus::EthLocked,
            vec![SwapStatus::NearMinting, SwapStatus::Failed, SwapStatus::Timeout]
        );
        
        transitions.insert(
            SwapStatus::NearMinting,
            vec![SwapStatus::NearMinted, SwapStatus::Failed, SwapStatus::Timeout]
        );
        
        transitions.insert(
            SwapStatus::NearMinted,
            vec![SwapStatus::Completed, SwapStatus::Failed]
        );

        transitions.insert(
            SwapStatus::Failed,
            vec![SwapStatus::RolledBack]
        );

        transitions.insert(
            SwapStatus::Timeout,
            vec![SwapStatus::RolledBack]
        );

        transitions.insert(
            SwapStatus::Cancelled,
            vec![SwapStatus::RolledBack]
        );

        // Terminal states
        transitions.insert(SwapStatus::Completed, vec![]);
        transitions.insert(SwapStatus::RolledBack, vec![]);

        Self { transitions }
    }

    pub fn can_transition(&self, from: SwapStatus, to: SwapStatus) -> bool {
        self.transitions
            .get(&from)
            .map(|allowed| allowed.contains(&to))
            .unwrap_or(false)
    }

    pub fn transition_state(
        &self,
        swap_operation: &mut SwapOperation,
        new_status: SwapStatus,
    ) -> Result<(), BridgeError> {
        if !self.can_transition(swap_operation.status.clone(), new_status.clone()) {
            return Err(BridgeError::InvalidStateTransition {
                from: swap_operation.status.clone(),
                to: new_status,
            });
        }

        let old_status = swap_operation.status.clone();
        swap_operation.status = new_status.clone();
        swap_operation.updated_at = Utc::now();

        tracing::info!(
            "State transition for swap {}: {} -> {}",
            swap_operation.swap_id,
            old_status,
            new_status
        );

        Ok(())
    }

    pub fn get_valid_transitions(&self, from: SwapStatus) -> Vec<SwapStatus> {
        self.transitions
            .get(&from)
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_terminal_state(&self, status: &SwapStatus) -> bool {
        matches!(status, SwapStatus::Completed | SwapStatus::RolledBack)
    }

    pub fn is_error_state(&self, status: &SwapStatus) -> bool {
        matches!(status, SwapStatus::Failed | SwapStatus::Timeout | SwapStatus::Cancelled)
    }

    pub fn requires_rollback(&self, status: &SwapStatus) -> bool {
        self.is_error_state(status) && !self.is_terminal_state(status)
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

// Clone implementation for Arc usage
impl Clone for StateMachine {
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_swap_operation() -> SwapOperation {
        SwapOperation {
            swap_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: 1000000000000000000, // 1 ETH
            recipient: "test.near".to_string(),
            status: SwapStatus::Initialized,
            quantum_key_id: None,
            eth_tx_hash: None,
            near_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
        }
    }

    #[test]
    fn test_valid_state_transitions() {
        let state_machine = StateMachine::new();
        
        // Test valid transitions
        assert!(state_machine.can_transition(SwapStatus::Initialized, SwapStatus::EthLocking));
        assert!(state_machine.can_transition(SwapStatus::EthLocking, SwapStatus::EthLocked));
        assert!(state_machine.can_transition(SwapStatus::EthLocked, SwapStatus::NearMinting));
        assert!(state_machine.can_transition(SwapStatus::NearMinting, SwapStatus::NearMinted));
        assert!(state_machine.can_transition(SwapStatus::NearMinted, SwapStatus::Completed));
        
        // Test error transitions
        assert!(state_machine.can_transition(SwapStatus::Initialized, SwapStatus::Failed));
        assert!(state_machine.can_transition(SwapStatus::EthLocking, SwapStatus::Timeout));
        assert!(state_machine.can_transition(SwapStatus::Failed, SwapStatus::RolledBack));
    }

    #[test]
    fn test_invalid_state_transitions() {
        let state_machine = StateMachine::new();
        
        // Test invalid transitions
        assert!(!state_machine.can_transition(SwapStatus::Initialized, SwapStatus::NearMinting));
        assert!(!state_machine.can_transition(SwapStatus::Completed, SwapStatus::EthLocking));
        assert!(!state_machine.can_transition(SwapStatus::RolledBack, SwapStatus::Initialized));
    }

    #[test]
    fn test_transition_state_success() {
        let state_machine = StateMachine::new();
        let mut swap_operation = create_test_swap_operation();
        
        let result = state_machine.transition_state(&mut swap_operation, SwapStatus::EthLocking);
        
        assert!(result.is_ok());
        assert_eq!(swap_operation.status, SwapStatus::EthLocking);
    }

    #[test]
    fn test_transition_state_invalid() {
        let state_machine = StateMachine::new();
        let mut swap_operation = create_test_swap_operation();
        
        let result = state_machine.transition_state(&mut swap_operation, SwapStatus::NearMinting);
        
        assert!(result.is_err());
        assert_eq!(swap_operation.status, SwapStatus::Initialized); // Should remain unchanged
    }

    #[test]
    fn test_terminal_states() {
        let state_machine = StateMachine::new();
        
        assert!(state_machine.is_terminal_state(&SwapStatus::Completed));
        assert!(state_machine.is_terminal_state(&SwapStatus::RolledBack));
        assert!(!state_machine.is_terminal_state(&SwapStatus::Initialized));
        assert!(!state_machine.is_terminal_state(&SwapStatus::Failed));
    }

    #[test]
    fn test_error_states() {
        let state_machine = StateMachine::new();
        
        assert!(state_machine.is_error_state(&SwapStatus::Failed));
        assert!(state_machine.is_error_state(&SwapStatus::Timeout));
        assert!(state_machine.is_error_state(&SwapStatus::Cancelled));
        assert!(!state_machine.is_error_state(&SwapStatus::Completed));
        assert!(!state_machine.is_error_state(&SwapStatus::Initialized));
    }

    #[test]
    fn test_requires_rollback() {
        let state_machine = StateMachine::new();
        
        assert!(state_machine.requires_rollback(&SwapStatus::Failed));
        assert!(state_machine.requires_rollback(&SwapStatus::Timeout));
        assert!(state_machine.requires_rollback(&SwapStatus::Cancelled));
        assert!(!state_machine.requires_rollback(&SwapStatus::Completed));
        assert!(!state_machine.requires_rollback(&SwapStatus::RolledBack));
    }
}