use onechatsocial_quark::models::User;
use onechatsocial_quark::{Database, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Unblock User
///
/// Unblock another user by their id.
#[openapi(tag = "Relationships")]
#[delete("/<target>/block")]
pub async fn req(db: &State<Database>, user: User, target: String) -> Result<Json<User>> {
    let mut target = db.fetch_user(&target).await?;
    user.unblock_user(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
