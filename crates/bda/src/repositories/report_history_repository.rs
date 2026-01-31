// BDA Report History Repository
// Purpose: Database access layer for BDA report history

use crate::features::bda::domain::report_history::{BdaReportHistory, ReportHistoryResponse};
use sqlx::{Pool, Sqlite, Row};

pub struct ReportHistoryRepository {
    pool: Pool<Sqlite>,
}

impl ReportHistoryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Get history for a BDA report
    pub async fn get_by_report_id(
        &self,
        bda_report_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<ReportHistoryResponse, sqlx::Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        // Get total count
        let total_row = sqlx::query("SELECT COUNT(*) as total FROM bda_report_history WHERE bda_report_id = $1")
            .bind(bda_report_id)
            .fetch_one(&self.pool)
            .await?;
        let total: i32 = total_row.get("total");
        
        // Get latest version
        let latest_row = sqlx::query(
            "SELECT MAX(version_number) as latest FROM bda_report_history WHERE bda_report_id = $1"
        )
        .bind(bda_report_id)
        .fetch_optional(&self.pool)
        .await?;
        let latest_version: i32 = latest_row
            .and_then(|r| r.try_get("latest").ok())
            .unwrap_or(0);
        
        // Get history entries
        let rows = sqlx::query(
            "SELECT * FROM bda_report_history WHERE bda_report_id = $1 ORDER BY version_number DESC LIMIT $2 OFFSET $3"
        )
        .bind(bda_report_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let mut history = Vec::new();
        for row in rows {
            history.push(self.row_to_history(row)?);
        }
        
        Ok(ReportHistoryResponse {
            history,
            total,
            latest_version,
        })
    }
    
    /// Get specific version of a report
    pub async fn get_by_version(
        &self,
        bda_report_id: &str,
        version_number: i32,
    ) -> Result<Option<BdaReportHistory>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT * FROM bda_report_history WHERE bda_report_id = $1 AND version_number = $2"
        )
        .bind(bda_report_id)
        .bind(version_number)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_history(r)?)),
            None => Ok(None),
        }
    }
    
    /// Helper: Convert SQLite row to BdaReportHistory
    fn row_to_history(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaReportHistory, sqlx::Error> {
        Ok(BdaReportHistory {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            version_number: row.get("version_number"),
            report_data_json: row.get("report_data_json"),
            changed_by: row.get("changed_by"),
            changed_at: row.get::<chrono::DateTime<chrono::Utc>, _>("changed_at").to_rfc3339(),
            change_type: self.parse_change_type(&row.get::<String, _>("change_type")),
            change_description: row.get("change_description"),
            status: row.get("status"),
        })
    }
    
    fn parse_change_type(&self, s: &str) -> crate::features::bda::domain::report_history::ChangeType {
        match s {
            "created" => crate::features::bda::domain::report_history::ChangeType::Created,
            "updated" => crate::features::bda::domain::report_history::ChangeType::Updated,
            "submitted" => crate::features::bda::domain::report_history::ChangeType::Submitted,
            "reviewed" => crate::features::bda::domain::report_history::ChangeType::Reviewed,
            "approved" => crate::features::bda::domain::report_history::ChangeType::Approved,
            "rejected" => crate::features::bda::domain::report_history::ChangeType::Rejected,
            _ => crate::features::bda::domain::report_history::ChangeType::Updated,
        }
    }
}
