use crate::lang_lua::{multioption::MultiOption, state};
use futures_util::StreamExt;
use mlua::prelude::*;
use serenity::all::Mentionable;
use std::sync::Arc;

/// An action executor is used to execute actions such as kick/ban/timeout from Lua
/// templates
pub struct DiscordActionExecutor {
    template_data: Arc<state::TemplateData>,
    guild_id: serenity::all::GuildId,
    serenity_context: serenity::all::Context,
    shard_messenger: serenity::all::ShardMessenger,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    ratelimits: Arc<state::LuaRatelimits>,
}

impl DiscordActionExecutor {
    pub fn check_action(&self, action: String) -> Result<(), crate::Error> {
        if !self
            .template_data
            .pragma
            .allowed_caps
            .contains(&format!("discord:{}", action))
        {
            return Err("Discord action not allowed in this template context".into());
        }

        self.ratelimits.check(&action)?;

        Ok(())
    }

    pub async fn user_in_guild(&self, user_id: serenity::all::UserId) -> Result<(), crate::Error> {
        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err("User not found in guild".into());
        };

        if member.user.id != user_id {
            return Err("User not found in guild".into());
        }

        Ok(())
    }

    pub async fn check_permissions(
        &self,
        user_id: serenity::all::UserId,
        needed_permissions: serenity::all::Permissions,
    ) -> Result<(), crate::Error> {
        let guild =
            sandwich_driver::guild(&self.cache_http, &self.reqwest_client, self.guild_id).await?; // Get the guild

        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err("Bot user not found in guild".into());
        }; // Get the bot user

        if !guild
            .member_permissions(&member)
            .contains(needed_permissions)
        {
            return Err(format!(
                "Bot does not have the required permissions: {:?}",
                needed_permissions
            )
            .into());
        }

        Ok(())
    }

    pub async fn check_permissions_and_hierarchy(
        &self,
        user_id: serenity::all::UserId,
        target_id: serenity::all::UserId,
        needed_permissions: serenity::all::Permissions,
    ) -> Result<(), crate::Error> {
        let guild =
            sandwich_driver::guild(&self.cache_http, &self.reqwest_client, self.guild_id).await?; // Get the guild

        let Some(member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            user_id,
        )
        .await?
        else {
            return Err(format!("User not found in guild: {}", user_id.mention()).into());
        }; // Get the bot user

        if !guild
            .member_permissions(&member)
            .contains(needed_permissions)
        {
            return Err(format!(
                "User does not have the required permissions: {:?}: {}",
                needed_permissions, user_id
            )
            .into());
        }

        let Some(target_member) = sandwich_driver::member_in_guild(
            &self.cache_http,
            &self.reqwest_client,
            self.guild_id,
            target_id,
        )
        .await?
        else {
            return Err("Target user not found in guild".into());
        }; // Get the target user

        let higher_id = guild
            .greater_member_hierarchy(&member, &target_member)
            .ok_or_else(|| {
                format!(
                    "User does not have a higher role than the target user: {}",
                    user_id.mention()
                )
            })?;

        if higher_id != member.user.id {
            return Err(format!(
                "User does not have a higher role than the target user: {}",
                user_id.mention()
            )
            .into());
        }

        Ok(())
    }
}

impl LuaUserData for DiscordActionExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Audit logs
        methods.add_async_method("get_audit_logs", |lua, this, data: LuaValue| async move {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct GetAuditLogOptions {
                action_type: Option<serenity::all::audit_log::Action>,
                user_id: Option<serenity::all::UserId>,
                before: Option<serenity::all::AuditLogEntryId>,
                limit: Option<serenity::nonmax::NonMaxU8>,
            }

            let data = lua.from_value::<GetAuditLogOptions>(data)?;

            this.check_action("get_audit_logs".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions(bot_userid, serenity::all::Permissions::VIEW_AUDIT_LOG)
                .await
                .map_err(LuaError::external)?;

            let logs = this
                .serenity_context
                .http
                .get_audit_logs(
                    this.guild_id,
                    data.action_type,
                    data.user_id,
                    data.before,
                    data.limit,
                )
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&logs)?;

            Ok(v)
        });

        // Channel actions
        methods.add_async_method("get_channel", |lua, this, data: LuaValue| async move {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct GetChannelOptions {
                channel_id: serenity::all::ChannelId,
            }

            let data = lua.from_value::<GetChannelOptions>(data)?;

            this.check_action("get_channel".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.user_in_guild(bot_userid)
                .await
                .map_err(LuaError::external)?;

            let channel = this
                .serenity_context
                .http
                .get_channel(data.channel_id)
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&channel)?;

            Ok(v)
        });

        methods.add_async_method("edit_channel", |lua, this, data: LuaValue| async move {
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct EditChannelOptions {
                channel_id: serenity::all::ChannelId,
                reason: String,

                // Fields that can be edited
                name: Option<String>,                                     // done
                r#type: Option<serenity::all::ChannelType>,               // done
                position: Option<u16>,                                    // done
                topic: Option<String>,                                    // done
                nsfw: Option<bool>,                                       // done
                rate_limit_per_user: Option<serenity::nonmax::NonMaxU16>, // done
                bitrate: Option<u32>,                                     // done
                permission_overwrites: Option<Vec<serenity::all::PermissionOverwrite>>, // done
                parent_id: MultiOption<serenity::all::ChannelId>,         // done
                rtc_region: MultiOption<String>,                          // done
                video_quality_mode: Option<serenity::all::VideoQualityMode>, // done
                default_auto_archive_duration: Option<serenity::all::AutoArchiveDuration>, // done
                flags: Option<serenity::all::ChannelFlags>,               // done
                available_tags: Option<Vec<serenity::all::ForumTag>>,     // done
                default_reaction_emoji: MultiOption<serenity::all::ForumEmoji>, // done
                default_thread_rate_limit_per_user: Option<serenity::nonmax::NonMaxU16>, // done
                default_sort_order: Option<serenity::all::SortOrder>,     // done
                default_forum_layout: Option<serenity::all::ForumLayoutType>, // done
            }

            let data = lua.from_value::<EditChannelOptions>(data)?;

            this.check_action("edit_channel".to_string())
                .map_err(LuaError::external)?;

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions(bot_userid, serenity::all::Permissions::MANAGE_CHANNELS)
                .await
                .map_err(LuaError::external)?;

            let mut ec = serenity::all::EditChannel::default(); // Create a new EditChannel struct

            if let Some(name) = data.name {
                ec = ec.name(name);
            }

            if let Some(r#type) = data.r#type {
                ec = ec.kind(r#type);
            }

            if let Some(position) = data.position {
                ec = ec.position(position);
            }

            if let Some(topic) = data.topic {
                if topic.len() > 1024 {
                    return Err(LuaError::external(
                        "Topic must be less than 1024 characters",
                    ));
                }
                ec = ec.topic(topic);
            }

            if let Some(nsfw) = data.nsfw {
                ec = ec.nsfw(nsfw);
            }

            if let Some(rate_limit_per_user) = data.rate_limit_per_user {
                if rate_limit_per_user.get() > 21600 {
                    return Err(LuaError::external(
                        "Rate limit per user must be less than 21600 seconds",
                    ));
                }

                ec = ec.rate_limit_per_user(rate_limit_per_user);
            }

            if let Some(bitrate) = data.bitrate {
                ec = ec.bitrate(bitrate);
            }

            // TODO: Handle permission overwrites permissions
            if let Some(permission_overwrites) = data.permission_overwrites {
                ec = ec.permissions(permission_overwrites);
            }

            if let Some(parent_id) = data.parent_id.inner {
                ec = ec.category(parent_id);
            }

            if let Some(rtc_region) = data.rtc_region.inner {
                ec = ec.voice_region(rtc_region.map(|x| x.into()));
            }

            if let Some(video_quality_mode) = data.video_quality_mode {
                ec = ec.video_quality_mode(video_quality_mode);
            }

            if let Some(default_auto_archive_duration) = data.default_auto_archive_duration {
                ec = ec.default_auto_archive_duration(default_auto_archive_duration);
            }

            if let Some(flags) = data.flags {
                ec = ec.flags(flags);
            }

            if let Some(available_tags) = data.available_tags {
                let mut cft = Vec::new();

                for tag in available_tags {
                    if tag.name.len() > 20 {
                        return Err(LuaError::external(
                            "Tag name must be less than 20 characters",
                        ));
                    }

                    let cftt =
                        serenity::all::CreateForumTag::new(tag.name).moderated(tag.moderated);

                    // TODO: Emoji support

                    cft.push(cftt);
                }

                ec = ec.available_tags(cft);
            }

            if let Some(default_reaction_emoji) = data.default_reaction_emoji.inner {
                ec = ec.default_reaction_emoji(default_reaction_emoji);
            }

            if let Some(default_thread_rate_limit_per_user) =
                data.default_thread_rate_limit_per_user
            {
                ec = ec.default_thread_rate_limit_per_user(default_thread_rate_limit_per_user);
            }

            if let Some(default_sort_order) = data.default_sort_order {
                ec = ec.default_sort_order(default_sort_order);
            }

            if let Some(default_forum_layout) = data.default_forum_layout {
                ec = ec.default_forum_layout(default_forum_layout);
            }

            let channel = this
                .serenity_context
                .http
                .edit_channel(data.channel_id, &ec, Some(data.reason.as_str()))
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&channel)?;

            Ok(v)
        });

        // Members
        methods.add_async_method("ban", |lua, this, data: LuaValue| async move {
            /// A ban action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct BanAction {
                user_id: serenity::all::UserId,
                reason: String,
                delete_message_days: Option<u8>,
            }

            let data = lua.from_value::<BanAction>(data)?;

            this.check_action("ban".to_string())
                .map_err(LuaError::external)?;

            let delete_message_days = {
                if let Some(days) = data.delete_message_days {
                    if days > 7 {
                        return Err(LuaError::external(
                            "Delete message days must be between 0 and 7",
                        ));
                    }

                    days
                } else {
                    0
                }
            };

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::BAN_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            this.serenity_context
                .http
                .ban_user(
                    this.guild_id,
                    data.user_id,
                    delete_message_days,
                    Some(data.reason.as_str()),
                )
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        methods.add_async_method("kick", |lua, this, data: LuaValue| async move {
            /// A kick action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct KickAction {
                user_id: serenity::all::UserId,
                reason: String,
            }

            let data = lua.from_value::<KickAction>(data)?;

            this.check_action("kick".to_string())
                .map_err(LuaError::external)?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::KICK_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            this.serenity_context
                .http
                .kick_member(this.guild_id, data.user_id, Some(data.reason.as_str()))
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        methods.add_async_method("timeout", |lua, this, data: LuaValue| async move {
            /// A timeout action
            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct TimeoutAction {
                user_id: serenity::all::UserId,
                reason: String,
                duration_seconds: u64,
            }

            let data = lua.from_value::<TimeoutAction>(data)?;

            this.check_action("timeout".to_string())
                .map_err(LuaError::external)?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            if data.duration_seconds > 60 * 60 * 24 * 28 {
                return Err(LuaError::external(
                    "Timeout duration must be less than 28 days",
                ));
            }

            let bot_userid = this.serenity_context.cache.current_user().id;

            this.check_permissions_and_hierarchy(
                bot_userid,
                data.user_id,
                serenity::all::Permissions::MODERATE_MEMBERS,
            )
            .await
            .map_err(LuaError::external)?;

            let communication_disabled_until =
                chrono::Utc::now() + std::time::Duration::from_secs(data.duration_seconds);

            this.guild_id
                .edit_member(
                    &this.serenity_context.http,
                    data.user_id,
                    serenity::all::EditMember::new()
                        .audit_log_reason(data.reason.as_str())
                        .disable_communication_until(communication_disabled_until.into()),
                )
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });

        methods.add_async_method(
            "sendmessage_channel",
            |lua, this, data: LuaValue| async move {
                /// A kick action
                #[derive(serde::Serialize, serde::Deserialize)]
                pub struct SendMessageChannelAction {
                    channel_id: serenity::all::ChannelId, // Channel *must* be in the same guild
                    message: crate::core::messages::CreateMessage,
                }

                let data = lua.from_value::<SendMessageChannelAction>(data)?;

                this.check_action("sendmessage_channel".to_string())
                    .map_err(LuaError::external)?;

                let msg = crate::core::messages::to_discord_reply(data.message)
                    .map_err(LuaError::external)?;

                // Perform required checks
                let channel = sandwich_driver::channel(
                    &this.cache_http,
                    &this.reqwest_client,
                    Some(this.guild_id),
                    data.channel_id,
                )
                .await
                .map_err(LuaError::external)?;

                let Some(channel) = channel else {
                    return Err(LuaError::external("Channel not found"));
                };

                let Some(guild_channel) = channel.guild() else {
                    return Err(LuaError::external("Channel not in guild"));
                };

                if guild_channel.guild_id != this.guild_id {
                    return Err(LuaError::external("Channel not in guild"));
                }

                let bot_user_id = this.serenity_context.cache.current_user().id;

                let bot_user = sandwich_driver::member_in_guild(
                    &this.cache_http,
                    &this.reqwest_client,
                    this.guild_id,
                    bot_user_id,
                )
                .await
                .map_err(LuaError::external)?;

                let Some(bot_user) = bot_user else {
                    return Err(LuaError::external("Bot user not found"));
                };

                let guild =
                    sandwich_driver::guild(&this.cache_http, &this.reqwest_client, this.guild_id)
                        .await
                        .map_err(LuaError::external)?;

                // Check if the bot has permissions to send messages in the given channel
                if !guild
                    .user_permissions_in(&guild_channel, &bot_user)
                    .send_messages()
                {
                    return Err(LuaError::external(
                        "Bot does not have permission to send messages in the given channel",
                    ));
                }

                let cm = msg.to_create_message();

                let msg = guild_channel
                    .send_message(&this.serenity_context.http, cm)
                    .await
                    .map_err(LuaError::external)?;

                Ok(MessageHandle {
                    message: msg,
                    shard_messenger: this.shard_messenger.clone(),
                })
            },
        );
    }
}

pub struct MessageHandle {
    message: serenity::all::Message,
    shard_messenger: serenity::all::ShardMessenger,
}

impl LuaUserData for MessageHandle {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("data", |lua, this, _: ()| {
            let v = lua.to_value(&this.message)?;
            Ok(v)
        });

        methods.add_method("await_component_interaction", |_, this, _: ()| {
            let stream = crate::lang_lua::stream::LuaStream::new(
                this.message
                    .await_component_interaction(this.shard_messenger.clone())
                    .timeout(std::time::Duration::from_secs(60))
                    .stream()
                    .map(|interaction| MessageComponentHandle { interaction }),
            );

            Ok(stream)
        });
    }
}

pub struct MessageComponentHandle {
    pub interaction: serenity::all::ComponentInteraction,
}

impl LuaUserData for MessageComponentHandle {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("custom_id", |_, this, _: ()| {
            Ok(this.interaction.data.custom_id.to_string())
        });

        methods.add_method("data", |lua, this, _: ()| {
            let v = lua.to_value(&this.interaction)?;
            Ok(v)
        });
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (String,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let template_data = data
                .per_template
                .get(&token)
                .ok_or_else(|| LuaError::external("Template not found"))?;

            let executor = DiscordActionExecutor {
                template_data: template_data.clone(),
                guild_id: data.guild_id,
                cache_http: botox::cache::CacheHttpImpl::from_ctx(&data.serenity_context),
                serenity_context: data.serenity_context.clone(),
                shard_messenger: data.shard_messenger.clone(),
                reqwest_client: data.reqwest_client.clone(),
                ratelimits: data.actions_ratelimits.clone(),
            };

            Ok(executor)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
