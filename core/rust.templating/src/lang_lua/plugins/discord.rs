use crate::lang_lua::state;
use futures_util::StreamExt;
use mlua::prelude::*;
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

            this.check_permissions(bot_userid, serenity::all::Permissions::BAN_MEMBERS)
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

            this.check_permissions(bot_userid, serenity::all::Permissions::KICK_MEMBERS)
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

            this.check_permissions(bot_userid, serenity::all::Permissions::KICK_MEMBERS)
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
