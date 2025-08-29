use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission, Override};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Set Role Permission
///
/// Sets permissions for the specified role in this channel.
///
/// Channel must be a `TextChannel` or `VoiceChannel`.
#[openapi(tag = "Channel Permissions")]
#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn set_role_permissions(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    role_id: String,
    data: Json<v0::DataSetRolePermissions>,
) -> Result<Json<v0::Channel>> {
    let mut channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel).hydrate().await;
    let permissions = calculate_channel_permissions(&mut query).await;

    permissions.throw_if_lacking_channel_permission(ChannelPermission::ManagePermissions)?;

    if let Some(server) = query.server_ref() {
        if let Some(role) = server.roles.get(&role_id) {
            // === NEW: allow a bot to edit its *own* role at equal rank ===
            let our_rank = query.get_member_rank().unwrap_or(i64::MAX); // fail-closed if unknown
            let is_bot = user.bot.is_some();
            let is_self_role = query
                .member_ref()
                .as_ref()
                .map(|m| m.roles.contains(&role_id))
                .unwrap_or(false);

            // Block if target role is strictly higher than us,
            // or equal rank but NOT our own role.
            if role.rank < our_rank || (role.rank == our_rank && !(is_bot && is_self_role)) {
                return Err(create_error!(NotElevated));
            }

            // Use the CHANNEL'S current override as the baseline (more correct than server role)
            let current_value: Override = match &channel {
                revolt_database::Channel::TextChannel { role_permissions, .. }
                | revolt_database::Channel::VoiceChannel { role_permissions, .. } => {
                    role_permissions.get(&role_id).cloned().unwrap_or_default().into()
                }
                _ => role.permissions.into(), // shouldn't happen due to route guard
            };
            permissions
                .throw_permission_override(current_value, &data.permissions)
                .await?;

            channel
                .set_role_permission(db, &role_id, data.permissions.clone().into())
                .await?;

            Ok(Json(channel.into()))
        } else {
            Err(create_error!(NotFound))
        }
    } else {
        Err(create_error!(InvalidOperation))
    }
}
