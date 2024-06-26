use super::{Channel, File};

use onechatsocial_permissions::OverrideField;
use std::collections::HashMap;

#[cfg(feature = "validator")]
use validator::Validate;

auto_derived_partial!(
    /// Server
    pub struct Server {
        /// Unique Id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,
        /// User id of the owner
        pub owner: String,

        /// Name of the server
        pub name: String,
        /// Description for the server
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub description: Option<String>,

        /// Channels within this server
        // TODO: investigate if this is redundant and can be removed
        pub channels: Vec<String>,
        /// Categories for this server
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub categories: Option<Vec<Category>>,
        /// Configuration for sending system event messages
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub system_messages: Option<SystemMessageChannels>,

        /// Roles for this server
        #[cfg_attr(
            feature = "serde",
            serde(
                default = "HashMap::<String, Role>::new",
                skip_serializing_if = "HashMap::<String, Role>::is_empty"
            )
        )]
        pub roles: HashMap<String, Role>,
        /// Default set of server and channel permissions
        pub default_permissions: i64,

        /// Icon attachment
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub icon: Option<File>,
        /// Banner attachment
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub banner: Option<File>,

        /// Bitfield of server flags
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_zero_u32", default)
        )]
        pub flags: u32,

        /// Whether this server is flagged as not safe for work
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub nsfw: bool,
        /// Whether to enable analytics
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub analytics: bool,
        /// Whether this server should be publicly discoverable
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub discoverable: bool,
    },
    "PartialServer"
);

auto_derived_partial!(
    /// Role
    pub struct Role {
        /// Role name
        pub name: String,
        /// Permissions available to this role
        pub permissions: OverrideField,
        /// Colour used for this role
        ///
        /// This can be any valid CSS colour
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub colour: Option<String>,
        /// Whether this role should be shown separately on the member sidebar
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub hoist: bool,
        /// Ranking of this role
        #[cfg_attr(feature = "serde", serde(default))]
        pub rank: i64,
    },
    "PartialRole"
);

auto_derived!(
    /// Optional fields on server object
    pub enum FieldsServer {
        Description,
        Categories,
        SystemMessages,
        Icon,
        Banner,
    }

    /// Optional fields on server object
    pub enum FieldsRole {
        Colour,
    }

    /// Channel category
    pub struct Category {
        /// Unique ID for this category
        pub id: String,
        /// Title for this category
        pub title: String,
        /// Channels in this category
        pub channels: Vec<String>,
    }

    /// System message channel assignments
    pub struct SystemMessageChannels {
        /// ID of channel to send user join messages in
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub user_joined: Option<String>,
        /// ID of channel to send user left messages in
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub user_left: Option<String>,
        /// ID of channel to send user kicked messages in
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub user_kicked: Option<String>,
        /// ID of channel to send user banned messages in
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub user_banned: Option<String>,
    }

    /// Information about new server to create
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataCreateServer {
        /// Server name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: String,
        /// Server description
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
        pub description: Option<String>,
        /// Whether this server is age-restricted
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub nsfw: Option<bool>,
    }

    /// Information returned when creating server
    pub struct CreateServerLegacyResponse {
        /// Server object
        pub server: Server,
        /// Default channels
        pub channels: Vec<Channel>,
    }
);
