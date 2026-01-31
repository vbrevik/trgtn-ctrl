// BDA Distribution Handlers
// Purpose: HTTP request handlers for report distribution

use crate::features::bda::{
    domain::{
        CreateDistributionListRequest,
        AddDistributionMemberRequest,
        DistributeReportRequest,
    },
    repositories::{DistributionRepository, BdaRepository},
    services::ReportGenerator,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;

/// Create distribution list
pub async fn create_distribution_list(
    Extension(repo): Extension<Arc<DistributionRepository>>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreateDistributionListRequest>,
) -> impl IntoResponse {
    match repo.create_distribution_list(&user_id, payload).await {
        Ok(list) => (StatusCode::CREATED, Json(list)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create distribution list: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create distribution list").into_response()
        }
    }
}

/// Get all distribution lists
pub async fn get_distribution_lists(
    Extension(repo): Extension<Arc<DistributionRepository>>,
) -> impl IntoResponse {
    match repo.get_all_distribution_lists().await {
        Ok(lists) => Json(lists).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch distribution lists: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch distribution lists").into_response()
        }
    }
}

/// Add member to distribution list
pub async fn add_distribution_member(
    Extension(repo): Extension<Arc<DistributionRepository>>,
    Path(list_id): Path<String>,
    Json(payload): Json<AddDistributionMemberRequest>,
) -> impl IntoResponse {
    match repo.add_distribution_member(&list_id, payload).await {
        Ok(member) => (StatusCode::CREATED, Json(member)).into_response(),
        Err(e) => {
            tracing::error!("Failed to add distribution member: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add distribution member").into_response()
        }
    }
}

/// Get distribution list members
pub async fn get_distribution_list_members(
    Extension(repo): Extension<Arc<DistributionRepository>>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_list_members(&list_id).await {
        Ok(members) => Json(members).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch distribution members: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch distribution members").into_response()
        }
    }
}

/// Distribute report to recipients
pub async fn distribute_report(
    Extension(dist_repo): Extension<Arc<DistributionRepository>>,
    Extension(bda_repo): Extension<Arc<BdaRepository>>,
    Extension(user_id): Extension<String>,
    Path(report_id): Path<String>,
    Json(payload): Json<DistributeReportRequest>,
) -> impl IntoResponse {
    // Verify report exists
    let _report = match bda_repo.get_by_id(&report_id).await {

        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "BDA report not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch BDA report: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch BDA report").into_response();
        }
    };
    
    let mut distributions = Vec::new();
    
    // Distribute to distribution lists
    if let Some(list_ids) = &payload.distribution_list_ids {
        for list_id in list_ids {
            match dist_repo.get_list_members(list_id).await {
                Ok(members) => {
                    for member in members {
                        match dist_repo.create_distribution(
                            &user_id,
                            &report_id,
                            &member.recipient_name,
                            member.recipient_email.as_deref(),
                            Some(list_id),
                            &payload.report_format,
                            &payload.report_template_type,
                            &payload.classification_level,
                            payload.delivery_method.unwrap_or(member.delivery_method),
                        ).await {
                            Ok(dist) => distributions.push(dist),
                            Err(e) => {
                                tracing::error!("Failed to create distribution: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to fetch distribution list members: {:?}", e);
                }
            }
        }
    }
    
    // Distribute to individual recipients
    if let Some(recipients) = &payload.individual_recipients {
        for recipient in recipients {
            match dist_repo.create_distribution(
                &user_id,
                &report_id,
                &recipient.recipient_name,
                recipient.recipient_email.as_deref(),
                None,
                &payload.report_format,
                &payload.report_template_type,
                &payload.classification_level,
                payload.delivery_method.unwrap_or(crate::features::bda::domain::DeliveryMethod::System),
            ).await {
                Ok(dist) => distributions.push(dist),
                Err(e) => {
                    tracing::error!("Failed to create distribution: {:?}", e);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "distributions_created": distributions.len(),
        "distributions": distributions
    })).into_response()
}

/// Get report distribution summary
pub async fn get_report_distribution_summary(
    Extension(repo): Extension<Arc<DistributionRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_distribution_summary(&report_id).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch distribution summary: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch distribution summary").into_response()
        }
    }
}

/// Get all distributions for a report
pub async fn get_report_distributions(
    Extension(repo): Extension<Arc<DistributionRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_report_distributions(&report_id).await {
        Ok(distributions) => Json(distributions).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report distributions: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report distributions").into_response()
        }
    }
}
