// BDA Imagery Domain Model
// Purpose: Domain entity for post-strike imagery metadata

use serde::{Deserialize, Serialize};

// ============================================================================
// ENUMS
// ============================================================================

/// Sensor type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SensorType {
    /// Synthetic Aperture Radar
    SAR,
    /// Electro-Optical
    EO,
    /// Infrared
    IR,
    /// Full Motion Video
    FMV,
    /// Commercial satellite imagery
    Commercial,
    /// Other sensor types
    Other,
}

// ============================================================================
// STRUCTS
// ============================================================================

/// BDA Imagery metadata and storage reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaImagery {
    pub id: String,
    pub bda_report_id: String,
    
    // Imagery Metadata
    pub collection_date: String,  // ISO 8601 timestamp
    pub collection_platform: Option<String>,
    pub sensor_type: Option<SensorType>,
    
    // Image Quality
    pub ground_sample_distance_cm: Option<f32>,
    pub cloud_cover_percentage: Option<i32>,
    pub collection_angle_degrees: Option<f32>,
    pub azimuth_degrees: Option<f32>,
    pub quality_score: Option<f32>,
    pub quality_notes: Option<String>,
    
    // Image Timing
    pub time_post_strike_hours: Option<f32>,
    pub is_pre_strike_baseline: bool,
    
    // Storage
    pub image_url: String,
    pub thumbnail_url: Option<String>,
    pub image_format: Option<String>,
    pub file_size_bytes: Option<i64>,
    
    // Annotation
    pub annotations_json: Option<String>,
    pub annotated_by: Option<String>,
    pub annotated_at: Option<String>,
    
    // Classification
    pub classification_level: String,
    pub handling_caveats: Option<String>,
    pub source_classification: Option<String>,
    
    // Timestamps
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create new imagery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBdaImageryRequest {
    pub bda_report_id: String,
    pub collection_date: String,
    pub collection_platform: Option<String>,
    pub sensor_type: Option<SensorType>,
    pub ground_sample_distance_cm: Option<f32>,
    pub cloud_cover_percentage: Option<i32>,
    pub collection_angle_degrees: Option<f32>,
    pub azimuth_degrees: Option<f32>,
    pub quality_score: Option<f32>,
    pub quality_notes: Option<String>,
    pub time_post_strike_hours: Option<f32>,
    pub is_pre_strike_baseline: bool,
    pub image_url: String,
    pub thumbnail_url: Option<String>,
    pub image_format: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub classification_level: String,
    pub handling_caveats: Option<String>,
    pub source_classification: Option<String>,
}

// ============================================================================
// BUSINESS LOGIC
// ============================================================================

impl BdaImagery {
    /// Validate quality score is between 0.0 and 1.0
    pub fn validate_quality_score(&self) -> Result<(), String> {
        if let Some(score) = self.quality_score {
            if score < 0.0 || score > 1.0 {
                return Err(format!("Quality score must be between 0.0 and 1.0, got {}", score));
            }
        }
        Ok(())
    }
    
    /// Validate cloud cover percentage
    pub fn validate_cloud_cover(&self) -> Result<(), String> {
        if let Some(cc) = self.cloud_cover_percentage {
            if cc < 0 || cc > 100 {
                return Err(format!("Cloud cover percentage must be between 0 and 100, got {}", cc));
            }
        }
        Ok(())
    }
    
    /// Check if imagery is high quality (< 20% cloud cover, quality > 0.7)
    pub fn is_high_quality(&self) -> bool {
        let cloud_ok = self.cloud_cover_percentage.map_or(true, |cc| cc < 20);
        let quality_ok = self.quality_score.map_or(false, |q| q > 0.7);
        cloud_ok && quality_ok
    }
    
    /// Check if imagery is suitable for BDA (quality > 0.5)
    pub fn is_suitable_for_bda(&self) -> bool {
        self.quality_score.map_or(true, |q| q > 0.5)
    }
}

impl CreateBdaImageryRequest {
    /// Validate create request
    pub fn validate(&self) -> Result<(), String> {
        if self.bda_report_id.is_empty() {
            return Err("BDA report ID is required".to_string());
        }
        
        if self.image_url.is_empty() {
            return Err("Image URL is required".to_string());
        }
        
        if let Some(cc) = self.cloud_cover_percentage {
            if cc < 0 || cc > 100 {
                return Err("Cloud cover percentage must be between 0 and 100".to_string());
            }
        }
        
        if let Some(score) = self.quality_score {
            if score < 0.0 || score > 1.0 {
                return Err("Quality score must be between 0.0 and 1.0".to_string());
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_imagery() -> BdaImagery {
        BdaImagery {
            id: "img-123".to_string(),
            bda_report_id: "report-456".to_string(),
            collection_date: "2026-01-21T14:30:00Z".to_string(),
            collection_platform: Some("Sentinel-2".to_string()),
            sensor_type: Some(SensorType::EO),
            ground_sample_distance_cm: Some(10.0),
            cloud_cover_percentage: Some(15),
            collection_angle_degrees: Some(20.0),
            azimuth_degrees: Some(180.0),
            quality_score: Some(0.85),
            quality_notes: None,
            time_post_strike_hours: Some(2.5),
            is_pre_strike_baseline: false,
            image_url: "/imagery/post_strike_20260121.tif".to_string(),
            thumbnail_url: Some("/imagery/thumb_20260121.jpg".to_string()),
            image_format: Some("TIFF".to_string()),
            file_size_bytes: Some(52428800),  // 50MB
            annotations_json: None,
            annotated_by: None,
            annotated_at: None,
            classification_level: "SECRET".to_string(),
            handling_caveats: None,
            source_classification: Some("SECRET//NOFORN".to_string()),
            created_at: "2026-01-21T14:30:00Z".to_string(),
            updated_at: "2026-01-21T14:30:00Z".to_string(),
        }
    }
    
    #[test]
    fn test_quality_score_validation() {
        let mut imagery = create_test_imagery();
        assert!(imagery.validate_quality_score().is_ok());
        
        imagery.quality_score = Some(1.5);
        assert!(imagery.validate_quality_score().is_err());
    }
    
    #[test]
    fn test_cloud_cover_validation() {
        let mut imagery = create_test_imagery();
        assert!(imagery.validate_cloud_cover().is_ok());
        
        imagery.cloud_cover_percentage = Some(150);
        assert!(imagery.validate_cloud_cover().is_err());
    }
    
    #[test]
    fn test_high_quality_detection() {
        let mut imagery = create_test_imagery();
        
        // High quality: low cloud cover, high score
        imagery.cloud_cover_percentage = Some(10);
        imagery.quality_score = Some(0.85);
        assert!(imagery.is_high_quality());
        
        // Not high quality: high cloud cover
        imagery.cloud_cover_percentage = Some(50);
        assert!(!imagery.is_high_quality());
        
        // Not high quality: low score
        imagery.cloud_cover_percentage = Some(10);
        imagery.quality_score = Some(0.5);
        assert!(!imagery.is_high_quality());
    }
    
    #[test]
    fn test_suitability_for_bda() {
        let mut imagery = create_test_imagery();
        
        imagery.quality_score = Some(0.8);
        assert!(imagery.is_suitable_for_bda());
        
        imagery.quality_score = Some(0.3);
        assert!(!imagery.is_suitable_for_bda());
    }
}
