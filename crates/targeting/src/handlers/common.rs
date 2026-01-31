use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TargetQueryParams {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformQueryParams {
    pub status: Option<String>,
}
