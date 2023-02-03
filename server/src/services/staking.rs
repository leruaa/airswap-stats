use axum::{extract::Query, Json};
use serde::Deserialize;

use crate::utils::get_share;

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParams {
    points: f64,
}

pub async fn share(Query(params): Query<QueryParams>) -> Json<f64> {
    Json(get_share(params.points))
}
