use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};
use iso8601_timestamp::Timestamp;
use onechatsocial_config::config;
use onechatsocial_models::v0::{
    self, DataMessageSend, Embed, MessageAuthor, MessageSort, MessageWebhook, PushNotification,
    ReplyIntent, SendableEmbed, RE_MENTION,
};
use onechatsocial_permissions::{ChannelPermission, PermissionValue};
use onechatsocial_result::Result;
use ulid::Ulid;

use crate::{
    events::client::EventV1,
    tasks::{self, ack::AckEvent},
    util::idempotency::IdempotencyKey,
    Channel, Database, Emoji, File,
};

auto_derived_partial!(
    /// Message
    pub struct Message {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Unique value generated by client sending this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nonce: Option<String>,
        /// Id of the channel this message was sent in
        pub channel: String,
        /// Id of the user or webhook that sent this message
        pub author: String,
        /// The webhook that sent this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub webhook: Option<MessageWebhook>,
        /// Message content
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        /// System message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system: Option<SystemMessage>,
        /// Array of attachments
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<File>>,
        /// Time at which this message was last edited
        #[serde(skip_serializing_if = "Option::is_none")]
        pub edited: Option<Timestamp>,
        /// Attached embeds to this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
        /// Array of user ids mentioned in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mentions: Option<Vec<String>>,
        /// Array of message ids this message is replying to
        #[serde(skip_serializing_if = "Option::is_none")]
        pub replies: Option<Vec<String>>,
        /// Hashmap of emoji IDs to array of user IDs
        #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
        pub reactions: IndexMap<String, IndexSet<String>>,
        /// Information about how this message should be interacted with
        #[serde(skip_serializing_if = "Interactions::is_default", default)]
        pub interactions: Interactions,
        /// Name and / or avatar overrides for this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub masquerade: Option<Masquerade>,
    },
    "PartialMessage"
);

auto_derived!(
    /// System Event
    #[serde(tag = "type")]
    pub enum SystemMessage {
        #[serde(rename = "text")]
        Text { content: String },
        #[serde(rename = "user_added")]
        UserAdded { id: String, by: String },
        #[serde(rename = "user_remove")]
        UserRemove { id: String, by: String },
        #[serde(rename = "user_joined")]
        UserJoined { id: String },
        #[serde(rename = "user_left")]
        UserLeft { id: String },
        #[serde(rename = "user_kicked")]
        UserKicked { id: String },
        #[serde(rename = "user_banned")]
        UserBanned { id: String },
        #[serde(rename = "channel_renamed")]
        ChannelRenamed { name: String, by: String },
        #[serde(rename = "channel_description_changed")]
        ChannelDescriptionChanged { by: String },
        #[serde(rename = "channel_icon_changed")]
        ChannelIconChanged { by: String },
        #[serde(rename = "channel_ownership_changed")]
        ChannelOwnershipChanged { from: String, to: String },
    }

    /// Name and / or avatar override information
    pub struct Masquerade {
        /// Replace the display name shown on this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// Replace the avatar shown on this message (URL to image file)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar: Option<String>,
        /// Replace the display role colour shown on this message
        ///
        /// Must have `ManageRole` permission to use
        #[serde(skip_serializing_if = "Option::is_none")]
        pub colour: Option<String>,
    }

    /// Information to guide interactions on this message
    #[derive(Default)]
    pub struct Interactions {
        /// Reactions which should always appear and be distinct
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub reactions: Option<IndexSet<String>>,
        /// Whether reactions should be restricted to the given list
        ///
        /// Can only be set to true if reactions list is of at least length 1
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub restrict_reactions: bool,
    }

    /// Appended Information
    pub struct AppendMessage {
        /// Additional embeds to include in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
    }

    /// Message Time Period
    ///
    /// Filter and sort messages by time
    #[serde(untagged)]
    pub enum MessageTimePeriod {
        Relative {
            /// Message id to search around
            ///
            /// Specifying 'nearby' ignores 'before', 'after' and 'sort'.
            /// It will also take half of limit rounded as the limits to each side.
            /// It also fetches the message ID specified.
            nearby: String,
        },
        Absolute {
            /// Message id before which messages should be fetched
            before: Option<String>,
            /// Message id after which messages should be fetched
            after: Option<String>,
            /// Message sort direction
            sort: Option<MessageSort>,
        },
    }

    /// Message Filter
    pub struct MessageFilter {
        /// Parent channel ID
        pub channel: Option<String>,
        /// Message author ID
        pub author: Option<String>,
        /// Search query
        pub query: Option<String>,
    }

    /// Message Query
    pub struct MessageQuery {
        /// Maximum number of messages to fetch
        ///
        /// For fetching nearby messages, this is \`(limit + 1)\`.
        pub limit: Option<i64>,
        /// Filter to apply
        #[serde(flatten)]
        pub filter: MessageFilter,
        /// Time period to fetch
        #[serde(flatten)]
        pub time_period: MessageTimePeriod,
    }
);

#[allow(clippy::derivable_impls)]
impl Default for Message {
    fn default() -> Self {
        Self {
            id: Default::default(),
            nonce: None,
            channel: Default::default(),
            author: Default::default(),
            webhook: None,
            content: None,
            system: None,
            attachments: None,
            edited: None,
            embeds: None,
            mentions: None,
            replies: None,
            reactions: Default::default(),
            interactions: Default::default(),
            masquerade: None,
        }
    }
}

#[allow(clippy::disallowed_methods)]
impl Message {
    /// Create message from API data
    pub async fn create_from_api(
        db: &Database,
        channel: Channel,
        data: DataMessageSend,
        author: MessageAuthor<'_>,
        mut idempotency: IdempotencyKey,
        generate_embeds: bool,
        allow_mentions: bool,
    ) -> Result<Message> {
        let config = config().await;

        Message::validate_sum(
            &data.content,
            data.embeds.as_deref().unwrap_or_default(),
            config.features.limits.default.message_length,
        )?;

        idempotency
            .consume_nonce(data.nonce)
            .await
            .map_err(|_| create_error!(InvalidOperation))?;

        // Check the message is not empty
        if (data.content.as_ref().map_or(true, |v| v.is_empty()))
            && (data.attachments.as_ref().map_or(true, |v| v.is_empty()))
            && (data.embeds.as_ref().map_or(true, |v| v.is_empty()))
        {
            return Err(create_error!(EmptyMessage));
        }

        // Ensure restrict_reactions is not specified without reactions list
        if let Some(interactions) = &data.interactions {
            if interactions.restrict_reactions {
                let disallowed = if let Some(list) = &interactions.reactions {
                    list.is_empty()
                } else {
                    true
                };

                if disallowed {
                    return Err(create_error!(InvalidProperty));
                }
            }
        }

        let (author_id, webhook) = match &author {
            MessageAuthor::User(user) => (user.id.clone(), None),
            MessageAuthor::Webhook(webhook) => (webhook.id.clone(), Some((*webhook).clone())),
            MessageAuthor::System { .. } => ("00000000000000000000000000".to_string(), None),
        };

        // Start constructing the message
        let message_id = Ulid::new().to_string();
        let mut message = Message {
            id: message_id.clone(),
            channel: channel.id(),
            masquerade: data.masquerade.map(|masquerade| masquerade.into()),
            interactions: data
                .interactions
                .map(|interactions| interactions.into())
                .unwrap_or_default(),
            author: author_id,
            webhook: webhook.map(|w| w.into()),
            ..Default::default()
        };

        // Parse mentions in message.
        let mut mentions = HashSet::new();
        if allow_mentions {
            if let Some(content) = &data.content {
                for capture in RE_MENTION.captures_iter(content) {
                    if let Some(mention) = capture.get(1) {
                        mentions.insert(mention.as_str().to_string());
                    }
                }
            }
        }

        // Verify replies are valid.
        let mut replies = HashSet::new();
        if let Some(entries) = data.replies {
            if entries.len() > config.features.limits.default.message_replies {
                return Err(create_error!(TooManyReplies {
                    max: config.features.limits.default.message_replies,
                }));
            }

            for ReplyIntent { id, mention } in entries {
                let message = db.fetch_message(&id).await?;

                if mention && allow_mentions {
                    mentions.insert(message.author.to_owned());
                }

                replies.insert(message.id);
            }
        }

        if !mentions.is_empty() {
            message.mentions.replace(mentions.into_iter().collect());
        }

        if !replies.is_empty() {
            message
                .replies
                .replace(replies.into_iter().collect::<Vec<String>>());
        }

        // Add attachments to message.
        let mut attachments = vec![];
        if data
            .attachments
            .as_ref()
            .is_some_and(|v| v.len() > config.features.limits.default.message_attachments)
        {
            return Err(create_error!(TooManyAttachments {
                max: config.features.limits.default.message_attachments,
            }));
        }

        if data
            .embeds
            .as_ref()
            .is_some_and(|v| v.len() > config.features.limits.default.message_embeds)
        {
            return Err(create_error!(TooManyEmbeds {
                max: config.features.limits.default.message_embeds,
            }));
        }

        for attachment_id in data.attachments.as_deref().unwrap_or_default() {
            attachments.push(
                db.find_and_use_attachment(attachment_id, "attachments", "message", &message_id)
                    .await?,
            );
        }

        if !attachments.is_empty() {
            message.attachments.replace(attachments);
        }

        // Process included embeds.
        for sendable_embed in data.embeds.unwrap_or_default() {
            message.attach_sendable_embed(db, sendable_embed).await?;
        }

        // Set content
        message.content = data.content;

        // Pass-through nonce value for clients
        message.nonce = Some(idempotency.into_key());

        // Send the message
        message.send(db, author, &channel, generate_embeds).await?;

        Ok(message)
    }

    /// Send a message without any notifications
    pub async fn send_without_notifications(
        &mut self,
        db: &Database,
        is_dm: bool,
        generate_embeds: bool,
    ) -> Result<()> {
        db.insert_message(self).await?;

        // Fan out events
        EventV1::Message(self.clone().into())
            .p(self.channel.to_string())
            .await;

        // Update last_message_id
        tasks::last_message_id::queue(self.channel.to_string(), self.id.to_string(), is_dm).await;

        // Add mentions for affected users
        if let Some(mentions) = &self.mentions {
            for user in mentions {
                tasks::ack::queue(
                    self.channel.to_string(),
                    user.to_string(),
                    AckEvent::AddMention {
                        ids: vec![self.id.to_string()],
                    },
                )
                .await;
            }
        }

        // Generate embeds
        if generate_embeds {
            if let Some(content) = &self.content {
                tasks::process_embeds::queue(
                    self.channel.to_string(),
                    self.id.to_string(),
                    content.clone(),
                )
                .await;
            }
        }

        Ok(())
    }

    /// Send a message
    pub async fn send(
        &mut self,
        db: &Database,
        author: MessageAuthor<'_>,
        channel: &Channel,
        generate_embeds: bool,
    ) -> Result<()> {
        self.send_without_notifications(
            db,
            matches!(channel, Channel::DirectMessage { .. }),
            generate_embeds,
        )
        .await?;

        // Push out Web Push notifications
        crate::tasks::web_push::queue(
            {
                match channel {
                    Channel::DirectMessage { recipients, .. }
                    | Channel::Group { recipients, .. } => recipients.clone(),
                    Channel::TextChannel { .. } => self.mentions.clone().unwrap_or_default(),
                    _ => vec![],
                }
            },
            PushNotification::from(self.clone().into(), Some(author), &channel.id()).await,
        )
        .await;

        Ok(())
    }

    /// Append content to message
    pub async fn append(
        db: &Database,
        id: String,
        channel: String,
        append: AppendMessage,
    ) -> Result<()> {
        db.append_message(&id, &append).await?;

        EventV1::MessageAppend {
            id,
            channel: channel.to_string(),
            append: append.into(),
        }
        .p(channel)
        .await;

        Ok(())
    }

    /// Convert sendable embed to text embed and attach to message
    pub async fn attach_sendable_embed(
        &mut self,
        db: &Database,
        embed: v0::SendableEmbed,
    ) -> Result<()> {
        let media: Option<v0::File> = if let Some(id) = embed.media {
            Some(
                db.find_and_use_attachment(&id, "attachments", "message", &self.id)
                    .await?
                    .into(),
            )
        } else {
            None
        };

        let embed = v0::Embed::Text(v0::Text {
            icon_url: embed.icon_url,
            url: embed.url,
            title: embed.title,
            description: embed.description,
            media,
            colour: embed.colour,
        });

        if let Some(embeds) = &mut self.embeds {
            embeds.push(embed);
        } else {
            self.embeds = Some(vec![embed]);
        }

        Ok(())
    }

    /// Validate the sum of content of a message is under threshold
    pub fn validate_sum(
        content: &Option<String>,
        embeds: &[SendableEmbed],
        max_length: usize,
    ) -> Result<()> {
        let mut running_total = 0;
        if let Some(content) = content {
            running_total += content.len();
        }

        for embed in embeds {
            if let Some(desc) = &embed.description {
                running_total += desc.len();
            }
        }

        if running_total <= max_length {
            Ok(())
        } else {
            Err(create_error!(PayloadTooLarge))
        }
    }
}

impl SystemMessage {
    pub fn into_message(self, channel: String) -> Message {
        Message {
            id: Ulid::new().to_string(),
            channel,
            author: "00000000000000000000000000".to_string(),
            system: Some(self),

            ..Default::default()
        }
    }
}

impl Interactions {
    /// Validate interactions info is correct
    pub async fn validate(&self, db: &Database, permissions: &PermissionValue) -> Result<()> {
        let config = config().await;

        if let Some(reactions) = &self.reactions {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::React)?;

            if reactions.len() > config.features.limits.default.message_reactions {
                return Err(create_error!(InvalidOperation));
            }

            for reaction in reactions {
                if !Emoji::can_use(db, reaction).await? {
                    return Err(create_error!(InvalidOperation));
                }
            }
        }

        Ok(())
    }

    /// Check if we can use a given emoji to react
    pub fn can_use(&self, emoji: &str) -> bool {
        if self.restrict_reactions {
            if let Some(reactions) = &self.reactions {
                reactions.contains(emoji)
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Check if default initialisation of fields
    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}
