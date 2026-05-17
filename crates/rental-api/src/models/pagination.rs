use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    /// Maximum number of results to return
    pub limit: Option<i64>,
    /// Number of results to skip
    pub offset: Option<i64>,
}
