use onechatsocial_quark::models::stats::Stats;
use onechatsocial_quark::{Db, Result};

use rocket::serde::json::Json;

/// # Query Stats
///
/// Fetch various technical statistics.
#[openapi(tag = "Admin")]
#[get("/stats")]
pub async fn stats(db: &Db) -> Result<Json<Stats>> {
    Ok(Json(db.generate_stats().await?))
}
