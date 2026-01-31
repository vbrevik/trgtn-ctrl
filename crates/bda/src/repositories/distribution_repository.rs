// BDA Distribution Repository
// Purpose: Database access layer for report distribution

use crate::features::bda::domain::distribution::{
    BdaDistributionList,
    BdaDistributionMember,
    BdaReportDistribution,
    CreateDistributionListRequest,
    AddDistributionMemberRequest,
    DistributeReportRequest,
    IndividualRecipient,
    DistributionSummary,
    DeliveryStatus,
    DeliveryMethod,
};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct DistributionRepository {
    pool: Pool<Sqlite>,
}

impl DistributionRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create distribution list
    pub async fn create_distribution_list(
        &self,
        created_by: &str,
        req: CreateDistributionListRequest,
    ) -> Result<BdaDistributionList, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        
        sqlx::query(
            "INSERT INTO bda_distribution_list (id, name, description, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(created_by)
        .bind(&created_at)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.get_distribution_list(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get distribution list by ID
    pub async fn get_distribution_list(&self, id: &str) -> Result<Option<BdaDistributionList>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_distribution_list WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_list(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all distribution lists
    pub async fn get_all_distribution_lists(&self) -> Result<Vec<BdaDistributionList>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM bda_distribution_list ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut lists = Vec::new();
        for row in rows {
            lists.push(self.row_to_list(row)?);
        }
        Ok(lists)
    }
    
    /// Add member to distribution list
    pub async fn add_distribution_member(
        &self,
        list_id: &str,
        req: AddDistributionMemberRequest,
    ) -> Result<BdaDistributionMember, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        
        let delivery_method_str = match req.delivery_method.unwrap_or(DeliveryMethod::System) {
            DeliveryMethod::Email => "email",
            DeliveryMethod::System => "system",
            DeliveryMethod::Manual => "manual",
            DeliveryMethod::Api => "api",
        };
        
        sqlx::query(
            r#"
            INSERT INTO bda_distribution_member (
                id, distribution_list_id, recipient_name, recipient_email,
                recipient_role, recipient_organization, delivery_method,
                delivery_preferences, active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(&id)
        .bind(list_id)
        .bind(&req.recipient_name)
        .bind(&req.recipient_email)
        .bind(&req.recipient_role)
        .bind(&req.recipient_organization)
        .bind(delivery_method_str)
        .bind(&req.delivery_preferences)
        .bind(true)
        .bind(&created_at)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.get_distribution_member(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get distribution member by ID
    pub async fn get_distribution_member(&self, id: &str) -> Result<Option<BdaDistributionMember>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_distribution_member WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_member(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all members of a distribution list
    pub async fn get_list_members(&self, list_id: &str) -> Result<Vec<BdaDistributionMember>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_distribution_member WHERE distribution_list_id = $1 AND active = TRUE ORDER BY recipient_name ASC"
        )
        .bind(list_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut members = Vec::new();
        for row in rows {
            members.push(self.row_to_member(row)?);
        }
        Ok(members)
    }
    
    /// Create report distribution record
    pub async fn create_distribution(
        &self,
        distributed_by: &str,
        bda_report_id: &str,
        recipient_name: &str,
        recipient_email: Option<&str>,
        distribution_list_id: Option<&str>,
        report_format: &str,
        report_template_type: &str,
        classification_level: &str,
        delivery_method: DeliveryMethod,
    ) -> Result<BdaReportDistribution, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let distributed_at = chrono::Utc::now();
        
        let delivery_method_str = match delivery_method {
            DeliveryMethod::Email => "email",
            DeliveryMethod::System => "system",
            DeliveryMethod::Manual => "manual",
            DeliveryMethod::Api => "api",
        };
        
        sqlx::query(
            r#"
            INSERT INTO bda_report_distribution (
                id, bda_report_id, distribution_list_id, recipient_name, recipient_email,
                report_format, report_template_type, classification_level,
                delivery_status, delivery_method, distributed_by, distributed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9, $10, $11)
            "#
        )
        .bind(&id)
        .bind(bda_report_id)
        .bind(distribution_list_id)
        .bind(recipient_name)
        .bind(recipient_email)
        .bind(report_format)
        .bind(report_template_type)
        .bind(classification_level)
        .bind(delivery_method_str)
        .bind(distributed_by)
        .bind(&distributed_at)
        .execute(&self.pool)
        .await?;
        
        self.get_distribution(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get distribution by ID
    pub async fn get_distribution(&self, id: &str) -> Result<Option<BdaReportDistribution>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_report_distribution WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_distribution(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all distributions for a report
    pub async fn get_report_distributions(&self, bda_report_id: &str) -> Result<Vec<BdaReportDistribution>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_report_distribution WHERE bda_report_id = $1 ORDER BY distributed_at DESC"
        )
        .bind(bda_report_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut distributions = Vec::new();
        for row in rows {
            distributions.push(self.row_to_distribution(row)?);
        }
        Ok(distributions)
    }
    
    /// Get distribution summary for a report
    pub async fn get_distribution_summary(&self, bda_report_id: &str) -> Result<DistributionSummary, sqlx::Error> {
        let row = sqlx::query(
            "SELECT * FROM v_bda_distribution_summary WHERE bda_report_id = $1"
        )
        .bind(bda_report_id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(DistributionSummary {
                bda_report_id: r.get("bda_report_id"),
                total_distributions: r.get("total_distributions"),
                delivered_count: r.get("delivered_count"),
                sent_count: r.get("sent_count"),
                pending_count: r.get("pending_count"),
                failed_count: r.get("failed_count"),
                confirmed_count: r.get("confirmed_count"),
                first_sent_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("first_sent_at")
                    .map(|d| d.to_rfc3339()),
                last_delivered_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_delivered_at")
                    .map(|d| d.to_rfc3339()),
            }),
            None => Ok(DistributionSummary {
                bda_report_id: bda_report_id.to_string(),
                total_distributions: 0,
                delivered_count: 0,
                sent_count: 0,
                pending_count: 0,
                failed_count: 0,
                confirmed_count: 0,
                first_sent_at: None,
                last_delivered_at: None,
            }),
        }
    }
    
    /// Update distribution status
    pub async fn update_distribution_status(
        &self,
        id: &str,
        status: DeliveryStatus,
        error: Option<&str>,
    ) -> Result<BdaReportDistribution, sqlx::Error> {
        let now = chrono::Utc::now();
        let status_str = match status {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Cancelled => "cancelled",
        };
        
        sqlx::query(
            r#"
            UPDATE bda_report_distribution 
            SET delivery_status = $1,
                sent_at = CASE WHEN $1 = 'sent' THEN COALESCE(sent_at, $2) ELSE sent_at END,
                delivered_at = CASE WHEN $1 = 'delivered' THEN COALESCE(delivered_at, $2) ELSE delivered_at END,
                delivery_attempts = delivery_attempts + 1,
                last_error = $3
            WHERE id = $4
            "#
        )
        .bind(status_str)
        .bind(&now)
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_distribution(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    // Helper methods for row conversion
    fn row_to_list(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaDistributionList, sqlx::Error> {
        Ok(BdaDistributionList {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_by: row.get("created_by"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn row_to_member(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaDistributionMember, sqlx::Error> {
        Ok(BdaDistributionMember {
            id: row.get("id"),
            distribution_list_id: row.get("distribution_list_id"),
            recipient_name: row.get("recipient_name"),
            recipient_email: row.get("recipient_email"),
            recipient_role: row.get("recipient_role"),
            recipient_organization: row.get("recipient_organization"),
            delivery_method: self.parse_delivery_method(&row.get::<String, _>("delivery_method")),
            delivery_preferences: row.get("delivery_preferences"),
            active: row.get("active"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn row_to_distribution(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaReportDistribution, sqlx::Error> {
        Ok(BdaReportDistribution {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            distribution_list_id: row.get("distribution_list_id"),
            recipient_id: row.get("recipient_id"),
            recipient_name: row.get("recipient_name"),
            recipient_email: row.get("recipient_email"),
            report_format: row.get("report_format"),
            report_template_type: row.get("report_template_type"),
            classification_level: row.get("classification_level"),
            delivery_status: self.parse_delivery_status(&row.get::<String, _>("delivery_status")),
            delivery_method: self.parse_delivery_method(&row.get::<String, _>("delivery_method")),
            sent_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("sent_at")
                .map(|d| d.to_rfc3339()),
            delivered_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("delivered_at")
                .map(|d| d.to_rfc3339()),
            delivery_attempts: row.get("delivery_attempts"),
            last_error: row.get("last_error"),
            distributed_by: row.get("distributed_by"),
            distributed_at: row.get::<chrono::DateTime<chrono::Utc>, _>("distributed_at").to_rfc3339(),
            delivery_confirmation_received: row.get("delivery_confirmation_received"),
            confirmation_received_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("confirmation_received_at")
                .map(|d| d.to_rfc3339()),
            file_url: row.get("file_url"),
            file_size_bytes: row.get("file_size_bytes"),
        })
    }
    
    fn parse_delivery_status(&self, s: &str) -> DeliveryStatus {
        match s {
            "sent" => DeliveryStatus::Sent,
            "delivered" => DeliveryStatus::Delivered,
            "failed" => DeliveryStatus::Failed,
            "cancelled" => DeliveryStatus::Cancelled,
            _ => DeliveryStatus::Pending,
        }
    }
    
    fn parse_delivery_method(&self, s: &str) -> DeliveryMethod {
        match s {
            "email" => DeliveryMethod::Email,
            "manual" => DeliveryMethod::Manual,
            "api" => DeliveryMethod::Api,
            _ => DeliveryMethod::System,
        }
    }
}
