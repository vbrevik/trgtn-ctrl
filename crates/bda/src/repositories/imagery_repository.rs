// BDA Imagery Repository
// Purpose: Database access layer for BDA imagery

use crate::features::bda::domain::{BdaImagery, CreateBdaImageryRequest, SensorType};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct ImageryRepository {
    pool: Pool<Sqlite>,
}

impl ImageryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create new imagery record
    pub async fn create(&self, req: CreateBdaImageryRequest) -> Result<BdaImagery, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        
        let sensor_type_str = req.sensor_type.as_ref().map(|s| match s {
            SensorType::SAR => "SAR",
            SensorType::EO => "EO",
            SensorType::IR => "IR",
            SensorType::FMV => "FMV",
            SensorType::Commercial => "Commercial",
            SensorType::Other => "Other",
        });
        
        sqlx::query(
            r#"
            INSERT INTO bda_imagery (
                id, bda_report_id, collection_date, collection_platform, sensor_type,
                ground_sample_distance_cm, cloud_cover_percentage, collection_angle_degrees,
                azimuth_degrees, quality_score, quality_notes,
                time_post_strike_hours, is_pre_strike_baseline,
                image_url, thumbnail_url, image_format, file_size_bytes,
                classification_level, handling_caveats, source_classification,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, $11,
                $12, $13,
                $14, $15, $16, $17,
                $18, $19, $20,
                $21, $21
            )
            "#
        )
        .bind(&id)
        .bind(&req.bda_report_id)
        .bind(&req.collection_date)
        .bind(&req.collection_platform)
        .bind(sensor_type_str)
        .bind(req.ground_sample_distance_cm)
        .bind(req.cloud_cover_percentage)
        .bind(req.collection_angle_degrees)
        .bind(req.azimuth_degrees)
        .bind(req.quality_score)
        .bind(&req.quality_notes)
        .bind(req.time_post_strike_hours)
        .bind(req.is_pre_strike_baseline)
        .bind(&req.image_url)
        .bind(&req.thumbnail_url)
        .bind(&req.image_format)
        .bind(req.file_size_bytes)
        .bind(&req.classification_level)
        .bind(&req.handling_caveats)
        .bind(&req.source_classification)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get imagery by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<BdaImagery>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_imagery WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_imagery(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all imagery for a BDA report
    pub async fn get_by_report(&self, bda_report_id: &str) -> Result<Vec<BdaImagery>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_imagery WHERE bda_report_id = $1 ORDER BY collection_date DESC"
        )
        .bind(bda_report_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut imagery_list = Vec::new();
        for row in rows {
            imagery_list.push(self.row_to_imagery(row)?);
        }
        
        Ok(imagery_list)
    }
    
    /// Update imagery (partial update)
    pub async fn update(&self, id: &str, updates: serde_json::Value) -> Result<Option<BdaImagery>, sqlx::Error> {
        let updated_at = chrono::Utc::now();
        
        let annotations_json = updates.get("annotations_json").and_then(|v| v.as_str());
        let annotated_by = updates.get("annotated_by").and_then(|v| v.as_str());
        let annotated_at_str = updates.get("annotated_at").and_then(|v| v.as_str());

        sqlx::query(
            r#"
            UPDATE bda_imagery 
            SET annotations_json = COALESCE($1, annotations_json),
                annotated_by = COALESCE($2, annotated_by),
                annotated_at = COALESCE($3, annotated_at),
                updated_at = $4
            WHERE id = $5
            "#
        )
        .bind(annotations_json)
        .bind(annotated_by)
        .bind(annotated_at_str)
        .bind(&updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // Return updated record
        self.get_by_id(id).await
    }

    /// Delete imagery
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bda_imagery WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Helper: Convert SQLite row to BdaImagery
    fn row_to_imagery(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaImagery, sqlx::Error> {
        Ok(BdaImagery {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            collection_date: row.get::<chrono::DateTime<chrono::Utc>, _>("collection_date").to_rfc3339(),
            collection_platform: row.get("collection_platform"),
            sensor_type: row.get::<Option<String>, _>("sensor_type").map(|s| self.parse_sensor_type(&s)),
            ground_sample_distance_cm: row.get::<Option<f64>, _>("ground_sample_distance_cm").map(|v| v as f32),
            cloud_cover_percentage: row.get("cloud_cover_percentage"),
            collection_angle_degrees: row.get::<Option<f64>, _>("collection_angle_degrees").map(|v| v as f32),
            azimuth_degrees: row.get::<Option<f64>, _>("azimuth_degrees").map(|v| v as f32),
            quality_score: row.get::<Option<f64>, _>("quality_score").map(|v| v as f32),
            quality_notes: row.get("quality_notes"),
            time_post_strike_hours: row.get::<Option<f64>, _>("time_post_strike_hours").map(|v| v as f32),
            is_pre_strike_baseline: row.get("is_pre_strike_baseline"),
            image_url: row.get("image_url"),
            thumbnail_url: row.get("thumbnail_url"),
            image_format: row.get("image_format"),
            file_size_bytes: row.get("file_size_bytes"),
            annotations_json: row.get("annotations_json"),
            annotated_by: row.get("annotated_by"),
            annotated_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("annotated_at").map(|dt| dt.to_rfc3339()),
            classification_level: row.get("classification_level"),
            handling_caveats: row.get("handling_caveats"),
            source_classification: row.get("source_classification"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn parse_sensor_type(&self, s: &str) -> SensorType {
        match s {
            "SAR" => SensorType::SAR,
            "EO" => SensorType::EO,
            "IR" => SensorType::IR,
            "FMV" => SensorType::FMV,
            "Commercial" => SensorType::Commercial,
            _ => SensorType::Other,
        }
    }
}
