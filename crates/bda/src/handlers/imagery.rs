// BDA Imagery Handlers
// Purpose: HTTP request handlers for BDA imagery

use crate::features::bda::{
    domain::{CreateBdaImageryRequest, BdaImagery},
    repositories::ImageryRepository,
};
use axum::{
    extract::{Path, Multipart},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;

/// Upload imagery for BDA report (JSON metadata only)
pub async fn create_imagery(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    Json(payload): Json<CreateBdaImageryRequest>,
) -> impl IntoResponse {
    match repo.create(payload).await {
        Ok(imagery) => (StatusCode::CREATED, Json(imagery)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create imagery: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create imagery").into_response()
        }
    }
}

/// Upload imagery file with metadata (multipart/form-data)
pub async fn upload_imagery_file(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Create uploads directory if it doesn't exist
    let uploads_dir = PathBuf::from("backend/uploads/bda");
    if let Err(e) = tokio::fs::create_dir_all(&uploads_dir).await {
        tracing::error!("Failed to create uploads directory: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create uploads directory").into_response();
    }

    let mut bda_report_id: Option<String> = None;
    let mut collection_date: Option<String> = None;
    let mut collection_platform: Option<String> = None;
    let mut sensor_type: Option<String> = None;
    let mut is_pre_strike_baseline: Option<bool> = None;
    let mut classification_level: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;

    // Parse multipart form data
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        
        // Extract file metadata before reading bytes (which consumes field)
        let is_file_field = field_name == "file";
        let field_file_name = if is_file_field {
            field.file_name().map(|s| s.to_string())
        } else {
            None
        };
        let field_content_type = if is_file_field {
            field.content_type().map(|s| s.to_string())
        } else {
            None
        };
        
        let field_data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to read field data: {:?}", e);
                continue;
            }
        };

        match field_name.as_str() {
            "bda_report_id" => {
                bda_report_id = Some(String::from_utf8_lossy(&field_data).to_string());
            }
            "collection_date" => {
                collection_date = Some(String::from_utf8_lossy(&field_data).to_string());
            }
            "collection_platform" => {
                collection_platform = Some(String::from_utf8_lossy(&field_data).to_string());
            }
            "sensor_type" => {
                sensor_type = Some(String::from_utf8_lossy(&field_data).to_string());
            }
            "is_pre_strike_baseline" => {
                is_pre_strike_baseline = Some(String::from_utf8_lossy(&field_data) == "true");
            }
            "classification_level" => {
                classification_level = Some(String::from_utf8_lossy(&field_data).to_string());
            }
            "file" => {
                file_data = Some(field_data.to_vec());
                file_name = field_file_name;
                content_type = field_content_type;
            }
            _ => {}
        }
    }

    // Validate required fields
    let bda_report_id = match bda_report_id {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, "Missing bda_report_id").into_response();
        }
    };

    let collection_date = collection_date.unwrap_or_else(|| Utc::now().to_rfc3339());
    let classification_level = classification_level.unwrap_or_else(|| "SECRET".to_string());
    let is_pre_strike_baseline = is_pre_strike_baseline.unwrap_or(false);

    // Validate file was uploaded
    let file_data = match file_data {
        Some(data) if !data.is_empty() => data,
        _ => {
            return (StatusCode::BAD_REQUEST, "No file uploaded").into_response();
        }
    };

    let file_size = file_data.len() as i64;
    let file_name = file_name.unwrap_or_else(|| "image".to_string());
    let file_extension = PathBuf::from(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg")
        .to_string();

    // Generate unique filename
    let imagery_id = Uuid::new_v4().to_string();
    let saved_filename = format!("{}.{}", imagery_id, file_extension);
    let file_path = uploads_dir.join(&saved_filename);
    let image_url = format!("/api/bda/files/{}", saved_filename);

    // Save file to disk
    if let Err(e) = tokio::fs::write(&file_path, &file_data).await {
        tracing::error!("Failed to save file: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file").into_response();
    }

    // Create imagery record
    let create_request = CreateBdaImageryRequest {
        bda_report_id: bda_report_id.clone(),
        collection_date,
        collection_platform,
        sensor_type: sensor_type.and_then(|s| {
            match s.as_str() {
                "SAR" => Some(crate::features::bda::domain::SensorType::SAR),
                "EO" => Some(crate::features::bda::domain::SensorType::EO),
                "IR" => Some(crate::features::bda::domain::SensorType::IR),
                "FMV" => Some(crate::features::bda::domain::SensorType::FMV),
                "Commercial" => Some(crate::features::bda::domain::SensorType::Commercial),
                "Other" => Some(crate::features::bda::domain::SensorType::Other),
                _ => None,
            }
        }),
        ground_sample_distance_cm: None,
        cloud_cover_percentage: None,
        collection_angle_degrees: None,
        azimuth_degrees: None,
        quality_score: None,
        quality_notes: None,
        time_post_strike_hours: None,
        is_pre_strike_baseline,
        image_url: image_url.clone(),
        thumbnail_url: None,
        image_format: Some(content_type.unwrap_or_else(|| "image/jpeg".to_string())),
        file_size_bytes: Some(file_size),
        classification_level,
        handling_caveats: None,
        source_classification: None,
    };

    match repo.create(create_request).await {
        Ok(imagery) => (StatusCode::CREATED, Json(imagery)).into_response(),
        Err(e) => {
            // Clean up file if database insert fails
            let _ = tokio::fs::remove_file(&file_path).await;
            tracing::error!("Failed to create imagery record: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create imagery record").into_response()
        }
    }
}

/// Serve imagery file
pub async fn serve_imagery_file(
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let file_path = PathBuf::from("backend/uploads/bda").join(&filename);
    
    // Security: Prevent directory traversal
    if file_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return (StatusCode::BAD_REQUEST, "Invalid file path").into_response();
    }

    match tokio::fs::read(&file_path).await {
        Ok(file_data) => {
            // Determine content type from extension
            let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                Some("tif") | Some("tiff") => "image/tiff",
                Some("nitf") => "application/octet-stream",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                file_data,
            ).into_response()
        }
        Err(_) => {
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
    }
}

/// Get imagery by ID
pub async fn get_imagery(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_id(&id).await {
        Ok(Some(imagery)) => Json(imagery).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Imagery not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch imagery: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch imagery").into_response()
        }
    }
}

/// Update imagery metadata
pub async fn update_imagery(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Parse update request (partial update)
    match repo.update(&id, payload).await {
        Ok(Some(imagery)) => Json(imagery).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Imagery not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to update imagery: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update imagery").into_response()
        }
    }
}

/// Get all imagery for a BDA report
pub async fn get_report_imagery(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_report(&report_id).await {
        Ok(imagery_list) => Json(imagery_list).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report imagery: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report imagery").into_response()
        }
    }
}

/// Delete imagery
pub async fn delete_imagery(
    Extension(repo): Extension<Arc<ImageryRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete imagery: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete imagery").into_response()
        }
    }
}
