// Tests for Historical Data Endpoints

#[cfg(test)]
mod tests {
    use crate::features::targeting::handlers::historical::{get_historical_status, get_historical_f3ead, get_historical_bda, HistoricalQueryParams};
    use sqlx::sqlite::SqlitePoolOptions;
    use chrono::Utc;
    use axum::extract::{State, Query};

    async fn setup_test_db() -> sqlx::Pool<sqlx::Sqlite> {
        let pool = SqlitePoolOptions::new()
            .connect(":memory:")
            .await
            .unwrap();
        
        // Create test tables
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS targets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                target_status TEXT NOT NULL,
                f3ead_stage TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await
        .unwrap();
        
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS bda_reports (
                id TEXT PRIMARY KEY,
                target_id TEXT NOT NULL,
                recommendation TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await
        .unwrap();
        
        pool
    }

    #[tokio::test]
    async fn test_historical_status_query() {
        let pool = setup_test_db().await;
        
        // Insert test data
        let now = Utc::now();
        let yesterday = now - chrono::Duration::days(1);
        
        sqlx::query("INSERT INTO targets (id, name, target_status, f3ead_stage, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("target-1")
            .bind("Test Target 1")
            .bind("NOMINATED")
            .bind("FIND")
            .bind(yesterday.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
        
        sqlx::query("INSERT INTO targets (id, name, target_status, f3ead_stage, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("target-2")
            .bind("Test Target 2")
            .bind("VALIDATED")
            .bind("FIX")
            .bind(now.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
        
        // Test query
        let from = (yesterday - chrono::Duration::days(1)).to_rfc3339();
        let to = (now + chrono::Duration::days(1)).to_rfc3339();
        
        let params = HistoricalQueryParams {
            from: Some(from),
            to: Some(to),
            limit: Some(30),
        };
        
        let result = get_historical_status(axum::extract::State(pool), axum::extract::Query(params)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_historical_f3ead_query() {
        let pool = setup_test_db().await;
        
        let now = Utc::now();
        
        sqlx::query("INSERT INTO targets (id, name, target_status, f3ead_stage, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("target-1")
            .bind("Test Target")
            .bind("NOMINATED")
            .bind("FIND")
            .bind(now.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
        
        let from = (now - chrono::Duration::days(1)).to_rfc3339();
        let to = (now + chrono::Duration::days(1)).to_rfc3339();
        
        let params = HistoricalQueryParams {
            from: Some(from),
            to: Some(to),
            limit: Some(30),
        };
        
        let result = get_historical_f3ead(axum::extract::State(pool), axum::extract::Query(params)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_historical_bda_query() {
        let pool = setup_test_db().await;
        
        let now = Utc::now();
        
        sqlx::query("INSERT INTO bda_reports (id, target_id, recommendation, created_at) VALUES (?, ?, ?, ?)")
            .bind("bda-1")
            .bind("target-1")
            .bind("EFFECTIVE")
            .bind(now.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
        
        let from = (now - chrono::Duration::days(1)).to_rfc3339();
        let to = (now + chrono::Duration::days(1)).to_rfc3339();
        
        let params = HistoricalQueryParams {
            from: Some(from),
            to: Some(to),
            limit: Some(30),
        };
        
        let result = get_historical_bda(axum::extract::State(pool), axum::extract::Query(params)).await;
        assert!(result.is_ok());
    }
}
