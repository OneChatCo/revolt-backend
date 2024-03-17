use onechatsocial_quark::models::User;
use onechatsocial_quark::Result;

use rocket::serde::json::Json;

/// # Fetch Self
///
/// Retrieve your user information.
#[openapi(tag = "User Information")]
#[get("/@me")]
pub async fn req(user: User) -> Result<Json<User>> {
    Ok(Json(user.foreign()))
}
