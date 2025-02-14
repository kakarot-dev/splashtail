use crate::impls::cache::CacheHttpImpl;
/// Bot animus contains the request and response for a bot
///
/// To edit/add responses, add them both to bot.rs and to splashcore/animusmagic/types.go
use crate::silverpelt::{
    canonical_module::CanonicalModule,
    permissions::PermissionResult,
    silverpelt_cache::SILVERPELT_CACHE,
};
use splashcore_rs::animusmagic_protocol::AnimusErrorResponse;
use crate::silverpelt;

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Role, RoleId, UserId};

#[derive(Serialize, Deserialize, Clone)]
pub enum BotAnimusResponse {
    Ok {
        message: String,
    },
    /// Modules event contains module related data
    Modules { modules: Vec<CanonicalModule> },
    /// GuildsExist event contains a list of u8s, where 1 means the guild exists and 0 means it doesn't
    GuildsExist { guilds_exist: Vec<u8> },
    /// BaseGuildUserInfo event is described in AnimusMessage
    BaseGuildUserInfo {
        owner_id: String,
        name: String,
        icon: Option<String>,
        /// List of all roles in the server
        roles: std::collections::HashMap<RoleId, Role>,
        /// List of roles the user has
        user_roles: Vec<RoleId>,
        /// List of roles the bot has
        bot_roles: Vec<RoleId>,
    },
    /// Returns the response of a command permission check
    CheckCommandPermission { 
        perm_res: PermissionResult,
        is_ok: bool,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum BotAnimusMessage {
    /// Ask the bot for module data
    Modules {},
    /// Given a list of guild IDs, return whether or not they exist on the bot
    GuildsExist { guilds: Vec<GuildId> },
    /// Given a guild ID and a user ID, check:
    /// - The server owner
    /// - The server name
    /// - The server icon
    /// - The users roles
    /// - The bots highest role
    BaseGuildUserInfo { guild_id: GuildId, user_id: UserId },
    /// Given a guild id, a user id and a command name, check if the user has permission to run the command
    CheckCommandPermission { guild_id: GuildId, user_id: UserId, command: String, custom_resolved_kittycat_perms: Option<Vec<String>> },
    /// Toggles a module within the bot clearing any cache in the process
    ToggleModule { guild_id: Option<GuildId>, module: String, enabled: bool },
}

impl BotAnimusMessage {
    pub async fn response(self, pool: &PgPool, cache_http: &CacheHttpImpl) -> Result<BotAnimusResponse, AnimusErrorResponse> {
        match self {
            Self::Modules {} => {
                let mut modules = Vec::new();

                for idm in SILVERPELT_CACHE.canonical_module_cache.iter() {
                    let module = idm.value();
                    modules.push(module.clone());
                }

                Ok(BotAnimusResponse::Modules { modules })
            }
            Self::GuildsExist { guilds } => {
                let mut guilds_exist = Vec::with_capacity(guilds.len());

                for guild in guilds {
                    guilds_exist.push({
                        if cache_http.cache.guild(guild).is_some() {
                            1
                        } else {
                            0
                        }
                    });
                }

                Ok(BotAnimusResponse::GuildsExist { guilds_exist })
            }
            Self::BaseGuildUserInfo { guild_id, user_id } => {
                let (name, icon, owner, roles, user_roles, bot_roles) = {
                    let guild = match cache_http.cache.guild(guild_id) {
                        Some(guild) => guild,
                        None => return Err("Guild not found".into()),
                    }
                    .clone();

                    let user_roles = {
                        let mem = match guild.member(cache_http, user_id).await {
                            Ok(member) => member,
                            Err(e) => return Err(format!("Failed to get member: {}", e).into()),
                        };

                        mem.roles.to_vec()
                    };

                    let bot_user_id = cache_http.cache.current_user().id;
                    let bot_roles = guild.member(&cache_http, bot_user_id).await?.roles.to_vec();

                    (
                        guild.name.to_string(),
                        guild.icon_url(),
                        guild.owner_id,
                        guild.roles,
                        user_roles,
                        bot_roles,
                    )
                };

                Ok(BotAnimusResponse::BaseGuildUserInfo {
                    name,
                    icon,
                    owner_id: owner.to_string(),
                    roles,
                    user_roles,
                    bot_roles,
                })
            },
            Self::CheckCommandPermission { guild_id, user_id, command, custom_resolved_kittycat_perms } => {
                // Check COMMAND_ID_MODULE_MAP
                let base_command = command.split_whitespace().next().unwrap();
                
                let perm_res = silverpelt::cmd::check_command(
                    base_command,
                    &command,
                    guild_id,
                    user_id,
                    pool,
                    cache_http,
                    &None,
                    custom_resolved_kittycat_perms
                )
                .await;

                let is_ok = perm_res.is_ok();

                Ok(BotAnimusResponse::CheckCommandPermission {
                    perm_res,
                    is_ok,
                })
            },
            Self::ToggleModule { guild_id, module, enabled } => {
                if let Some(guild_id) = guild_id {
                    if enabled {
                        SILVERPELT_CACHE
                            .module_enabled_cache
                            .insert((guild_id, module.clone()), true)
                            .await;
                    } else {
                        SILVERPELT_CACHE
                            .module_enabled_cache
                            .insert((guild_id, module.clone()), false)
                            .await;
                    }

                    tokio::spawn(async move {
                        if let Err(err) = SILVERPELT_CACHE.command_permission_cache.invalidate_entries_if(move |k, _| {
                            k.0 == guild_id
                        }) {
                            log::error!("Failed to invalidate command permission cache for guild {}: {}", guild_id, err);
                        } else {
                            log::info!("Invalidated cache for guild {}", guild_id);
                        }
                    });
                } else {
                    // Global enable/disable the module by iterating the entire cache
                    for (k, v) in SILVERPELT_CACHE.module_enabled_cache.iter() {
                        if k.1 == *module && enabled != v {
                            SILVERPELT_CACHE
                                .module_enabled_cache
                                .insert((k.0, module.clone()), enabled)
                                .await;

                            // Invalidate command permission cache entries here too
                            let gid = k.0;
                            tokio::spawn(async move {
                                if let Err(err) = SILVERPELT_CACHE.command_permission_cache.invalidate_entries_if(move |g, _| {
                                    g.0 == gid
                                }) {
                                    log::error!("Failed to invalidate command permission cache for guild {}: {}", k.0, err);
                                } else {
                                    log::info!("Invalidated cache for guild {}", k.0);
                                }
                            });
                        }
                    }
                }

                Ok(BotAnimusResponse::Ok {
                    message: "".to_string()
                })
            }
        }
    }
}
