use rocket::serde::json::Json;
use serde::Deserialize;

use onechatsocial_quark::{
    models::{Channel, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

/// # Permission Value
#[derive(Deserialize, JsonSchema)]
pub struct Data {
    /// Allow / deny values to set for this role
    permissions: Override,
}

/// # Set Role Permission
///
/// Sets permissions for the specified role in this channel.
///
/// Channel must be a `TextChannel` or `VoiceChannel`.
#[openapi(tag = "Channel Permissions")]
#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
) -> Result<Json<Channel>> {
    // Code from default
    let mut channel = match target.as_channel(db).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error Code 1: {:?}", e);
            return Err(e);
        }
    };

    let mut perm = perms(&user).channel(&channel);

    match perm.throw_permission_and_view_channel(db, Permission::ManagePermissions).await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error Code 2: {:?}", e);
            return Err(e);
        }
    }

    match &channel {
        Channel::TextChannel { .. } | Channel::VoiceChannel { .. } => {
            let server = match perm.server.get().ok_or(Error::InvalidOperation) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error Code 3: {:?}", e);
                    return Err(e);
                }
            };

            let role = match server.roles.get(&role_id).ok_or(Error::NotFound) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error Code 4: {:?}", e);
                    return Err(e);
                }
            };

            //  We are not an elevated user
            if role.rank <= perm.get_member_rank().unwrap_or(i64::MIN) {
                return Err(Error::NotElevated);
            }

            let current_value: Override = role.permissions.into();

            match perm.throw_permission_override(db, current_value, data.permissions).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error Code 5: {:?}", e);
                    return Err(e);
                }
            }

            match channel.set_role_permission(db, &role_id, data.permissions.into()).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error Code 6: {:?}", e);
                    return Err(e);
                }
            }

            Ok(Json(channel))
        }
        _ => return Err(Error::InvalidOperation),
    }

    // original code
    /*
    let mut channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);

    permissions
        .throw_permission_and_view_channel(db, Permission::ManagePermissions)
        .await?;

    if let Some(server) = permissions.server.get() {
        if let Some(role) = server.roles.get(&role_id) {
            if role.rank <= permissions.get_member_rank().unwrap_or(i64::MIN) {
                return Err(Error::NotElevated);
            }

            let current_value: Override = role.permissions.into();
            permissions
                .throw_permission_override(db, current_value, data.permissions)
                .await?;

            channel
                .set_role_permission(db, &role_id, data.permissions.into())
                .await?;

            Ok(Json(channel))
        } else {
            Err(Error::NotFound)
        }
    } else {
        Err(Error::InvalidOperation)
    }
    */
}
