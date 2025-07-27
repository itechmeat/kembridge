// src/services/manual_review/notifications.rs - Notification system for manual review
use tracing::{info, warn, instrument};
use uuid::Uuid;

use crate::models::review::{
    ReviewQueueEntry, ReviewNotification, NotificationType, ReviewError
};

/// Notification service for manual review events
pub struct ReviewNotificationService {
    notifications_enabled: bool,
}

impl ReviewNotificationService {
    pub fn new(notifications_enabled: bool) -> Self {
        Self { notifications_enabled }
    }

    /// Send review notification
    #[instrument(skip(self, review_entry))]
    pub async fn send_review_notification(
        &self,
        review_entry: &ReviewQueueEntry,
        notification_type: NotificationType,
    ) -> Result<(), ReviewError> {
        if !self.notifications_enabled {
            return Ok(());
        }

        let notification = ReviewNotification {
            id: Uuid::new_v4(),
            review_id: review_entry.id,
            notification_type: notification_type.clone(),
            transaction_id: review_entry.transaction_id,
            priority: review_entry.priority.clone(),
            message: self.generate_notification_message(review_entry, &notification_type),
            created_at: chrono::Utc::now(),
            sent_to: vec![review_entry.assigned_to.unwrap_or(review_entry.user_id)],
            acknowledged_by: vec![],
        };

        // For demo purposes, just log the notification
        // In production, this would integrate with email/SMS/webhook services
        info!(
            notification_id = %notification.id,
            review_id = %review_entry.id,
            notification_type = ?notification_type,
            sent_to = ?notification.sent_to,
            "Sending review notification: {}",
            notification.message
        );

        Ok(())
    }

    /// Send escalation alert
    #[instrument(skip(self, review_entry))]
    pub async fn send_escalation_alert(&self, review_entry: &ReviewQueueEntry) -> Result<(), ReviewError> {
        if !self.notifications_enabled {
            return Ok(());
        }

        warn!(
            review_id = %review_entry.id,
            escalation_count = review_entry.escalation_count,
            "ESCALATION ALERT: Review {} has been escalated {} times",
            review_entry.id,
            review_entry.escalation_count
        );

        // Send high-priority notification for escalations
        self.send_review_notification(review_entry, NotificationType::Escalated).await
    }

    /// Generate notification message based on type
    fn generate_notification_message(
        &self,
        review_entry: &ReviewQueueEntry,
        notification_type: &NotificationType,
    ) -> String {
        match notification_type {
            NotificationType::NewReview => {
                format!(
                    "New {:?} priority review created for transaction {}",
                    review_entry.priority,
                    review_entry.transaction_id
                )
            }
            NotificationType::CriticalReview => {
                format!(
                    "CRITICAL: Review {} has been assigned to you for immediate review",
                    review_entry.id
                )
            }
            NotificationType::Escalated => {
                format!(
                    "URGENT: Review {} has been escalated (escalation #{}) - Immediate attention required",
                    review_entry.id,
                    review_entry.escalation_count
                )
            }
            NotificationType::ReviewExpired => {
                format!(
                    "Review {} has expired and requires immediate action",
                    review_entry.id
                )
            }
            NotificationType::QueueBacklog => {
                format!(
                    "Queue backlog warning: Review {} requires attention",
                    review_entry.id
                )
            }
        }
    }

    /// Check if notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.notifications_enabled
    }

    /// Enable or disable notifications
    pub fn set_enabled(&mut self, enabled: bool) {
        self.notifications_enabled = enabled;
        if enabled {
            info!("Review notifications enabled");
        } else {
            warn!("Review notifications disabled");
        }
    }
}