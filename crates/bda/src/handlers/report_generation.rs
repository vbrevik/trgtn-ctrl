// BDA Report Generation Handlers
// Purpose: HTTP request handlers for report generation

use crate::features::bda::{
    domain::{GenerateReportRequest, ReportFormat, ReportGenerationResponse},
    repositories::BdaRepository,
    services::ReportGenerator,
};
use axum::{
    extract::Path,
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;

/// Generate BDA report
pub async fn generate_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Json(payload): Json<GenerateReportRequest>,
) -> impl IntoResponse {
    // Get the BDA report
    let report = match repo.get_by_id(&payload.bda_report_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "BDA report not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch BDA report: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch BDA report").into_response();
        }
    };
    
    // Generate report based on format
    match payload.format {
        ReportFormat::Json => {
            match ReportGenerator::generate_json(&report, &payload) {
                Ok(json_data) => {
                    let response = ReportGenerationResponse {
                        report_id: report.id.clone(),
                        template_type: payload.template_type,
                        format: payload.format,
                        file_url: None,
                        file_size_bytes: Some(serde_json::to_string(&json_data).unwrap().len() as u64),
                        generated_at: chrono::Utc::now().to_rfc3339(),
                        generated_by: user_id,
                        classification: payload.classification.unwrap_or(crate::features::bda::domain::ReportClassification::Secret),
                    };
                    Json(response).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to generate JSON report: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate report").into_response()
                }
            }
        }
        ReportFormat::Html => {
            match ReportGenerator::generate_html(&report, &payload) {
                Ok(html) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "text/html; charset=utf-8".parse().unwrap());
                    (headers, html).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to generate HTML report: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate report").into_response()
                }
            }
        }
        ReportFormat::Kml => {
            match ReportGenerator::generate_kml(&report, &payload) {
                Ok(kml) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "application/vnd.google-earth.kml+xml".parse().unwrap());
                    headers.insert(
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"bda_report_{}.kml\"", report.id).parse().unwrap()
                    );
                    (headers, kml).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to generate KML report: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate report").into_response()
                }
            }
        }
        ReportFormat::Pdf => {
            match ReportGenerator::generate_pdf(&report, &payload) {
                Ok(pdf_bytes) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
                    headers.insert(
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"bda_report_{}.pdf\"", report.id).parse().unwrap()
                    );
                    (headers, pdf_bytes).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to generate PDF report: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate PDF: {}", e)).into_response()
                }
            }
        }
        ReportFormat::Nitfs => {
            // NITFS generation requires specialized library
            (StatusCode::NOT_IMPLEMENTED, "NITFS format not yet implemented").into_response()
        }
    }
}

/// Get available report templates
pub async fn get_report_templates() -> impl IntoResponse {
    use crate::features::bda::domain::ReportTemplateType;
    
    let templates = vec![
        serde_json::json!({
            "type": "initial",
            "name": "Initial BDA Report",
            "description": "Initial assessment within 24h of strike",
        }),
        serde_json::json!({
            "type": "interim",
            "name": "Interim BDA Report",
            "description": "Interim assessment 24-72h post-strike",
        }),
        serde_json::json!({
            "type": "final",
            "name": "Final BDA Report",
            "description": "Final assessment after 72h",
        }),
        serde_json::json!({
            "type": "executive_summary",
            "name": "Executive Summary",
            "description": "High-level summary for leadership",
        }),
        serde_json::json!({
            "type": "technical",
            "name": "Technical Report",
            "description": "Detailed technical assessment",
        }),
    ];
    
    Json(templates).into_response()
}
