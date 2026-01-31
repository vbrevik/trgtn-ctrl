// BDA Report Distribution Domain Model
// Purpose: Track report distribution and delivery

use serde::{Deserialize, Serialize};

/// Delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Cancelled,
}

/// Delivery method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMethod {
    Email,
    System,
    Manual,
    Api,
}

/// Distribution List
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaDistributionList {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Distribution List Member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaDistributionMember {
    pub id: String,
    pub distribution_list_id: String,
    pub recipient_name: String,
    pub recipient_email: Option<String>,
    pub recipient_role: Option<String>,
    pub recipient_organization: Option<String>,
    pub delivery_method: DeliveryMethod,
    pub delivery_preferences: Option<String>,  // JSON
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Report Distribution Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaReportDistribution {
    pub id: String,
    pub bda_report_id: String,
    pub distribution_list_id: Option<String>,
    pub recipient_id: Option<String>,
    pub recipient_name: String,
    pub recipient_email: Option<String>,
    
    // Distribution details
    pub report_format: String,
    pub report_template_type: String,
    pub classification_level: String,
    
    // Delivery status
    pub delivery_status: DeliveryStatus,
    pub delivery_method: DeliveryMethod,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub delivery_attempts: i32,
    pub last_error: Option<String>,
    
    // Distribution metadata
    pub distributed_by: String,
    pub distributed_at: String,
    pub delivery_confirmation_received: bool,
    pub confirmation_received_at: Option<String>,
    
    // File reference
    pub file_url: Option<String>,
    pub file_size_bytes: Option<i64>,
}

/// Request to create distribution list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDistributionListRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to add member to distribution list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDistributionMemberRequest {
    pub recipient_name: String,
    pub recipient_email: Option<String>,
    pub recipient_role: Option<String>,
    pub recipient_organization: Option<String>,
    pub delivery_method: Option<DeliveryMethod>,
    pub delivery_preferences: Option<String>,
}

/// Request to distribute report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributeReportRequest {
    pub distribution_list_ids: Option<Vec<String>>,
    pub individual_recipients: Option<Vec<IndividualRecipient>>,
    pub report_format: String,
    pub report_template_type: String,
    pub classification_level: String,
    pub delivery_method: Option<DeliveryMethod>,
}

/// Individual recipient (not from list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualRecipient {
    pub recipient_name: String,
    pub recipient_email: Option<String>,
    pub recipient_role: Option<String>,
    pub recipient_organization: Option<String>,
}

/// Distribution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSummary {
    pub bda_report_id: String,
    pub total_distributions: i32,
    pub delivered_count: i32,
    pub sent_count: i32,
    pub pending_count: i32,
    pub failed_count: i32,
    pub confirmed_count: i32,
    pub first_sent_at: Option<String>,
    pub last_delivered_at: Option<String>,
}

impl CreateDistributionListRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Distribution list name is required".to_string());
        }
        Ok(())
    }
}

impl AddDistributionMemberRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.recipient_name.is_empty() {
            return Err("Recipient name is required".to_string());
        }
        Ok(())
    }
}

impl BdaReportDistribution {
    /// Check if distribution is successful
    pub fn is_successful(&self) -> bool {
        matches!(self.delivery_status, DeliveryStatus::Delivered)
    }
    
    /// Check if distribution needs retry
    pub fn needs_retry(&self) -> bool {
        matches!(self.delivery_status, DeliveryStatus::Failed) && self.delivery_attempts < 3
    }
    
    /// Get delivery status display
    pub fn status_display(&self) -> &'static str {
        match self.delivery_status {
            DeliveryStatus::Pending => "Pending",
            DeliveryStatus::Sent => "Sent",
            DeliveryStatus::Delivered => "Delivered",
            DeliveryStatus::Failed => "Failed",
            DeliveryStatus::Cancelled => "Cancelled",
        }
    }
}
