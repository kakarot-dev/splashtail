use crate::cache::SilverpeltCache;
use crate::{
    module_config::{
        get_best_command_configuration, get_command_extended_data, get_module_configuration,
    },
    types::{GuildCommandConfiguration, GuildModuleConfiguration},
    utils::permute_command_names,
};
use botox::cache::CacheHttpImpl;
use kittycat::perms::Permission;
use log::info;
use permissions::types::{PermissionChecks, PermissionResult};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use serenity::small_fixed_array::FixedArray;
use sqlx::PgPool;

#[inline]
pub async fn get_user_discord_info(
    guild_id: GuildId,
    user_id: UserId,
    cache_http: &CacheHttpImpl,
    poise_ctx: &Option<crate::Context<'_>>,
) -> Result<
    (
        bool,                              // is_owner
        UserId,                            // owner_id
        serenity::all::Permissions,        // member_perms
        FixedArray<serenity::all::RoleId>, // roles
    ),
    PermissionResult,
> {
    #[cfg(test)]
    {
        // Check for env var CHECK_MODULES_TEST_ENABLED, if so, return dummy data
        if std::env::var("CHECK_MODULES_TEST_ENABLED").unwrap_or_default() == "true" {
            return Ok((
                true,
                UserId::new(1),
                serenity::all::Permissions::all(),
                FixedArray::new(),
            ));
        }
    }

    if let Some(cached_guild) = guild_id.to_guild_cached(&cache_http.cache) {
        // OPTIMIZATION: if owner, we dont need to continue further
        if user_id == cached_guild.owner_id {
            return Ok((
                true,                              // is_owner
                cached_guild.owner_id,             // owner_id
                serenity::all::Permissions::all(), // member_perms
                FixedArray::new(), // OPTIMIZATION: no role data is needed for perm checks for owners
            ));
        }

        // OPTIMIZATION: If we have a poise_ctx which is also a ApplicationContext, we can directly use it
        if let Some(poise::Context::Application(ref a)) = poise_ctx {
            if let Some(ref mem) = a.interaction.member {
                return Ok((
                    mem.user.id == cached_guild.owner_id,
                    cached_guild.owner_id,
                    cached_guild.member_permissions(mem),
                    mem.roles.clone(),
                ));
            }
        }

        // Now fetch the member, here calling member automatically tries to find in its cache first
        if let Some(member) = cached_guild.members.get(&user_id) {
            return Ok((
                member.user.id == cached_guild.owner_id,
                cached_guild.owner_id,
                cached_guild.member_permissions(member),
                member.roles.clone(),
            ));
        }
    }

    let guild = match guild_id.to_partial_guild(&cache_http).await {
        Ok(guild) => guild,
        Err(e) => {
            return Err(PermissionResult::DiscordError {
                error: e.to_string(),
            })
        }
    };

    // OPTIMIZATION: if owner, we dont need to continue further
    if user_id == guild.owner_id {
        return Ok((
            true,
            guild.owner_id,
            serenity::all::Permissions::all(),
            FixedArray::new(),
        ));
    }

    // OPTIMIZATION: If we have a poise_ctx which is also a ApplicationContext, we can directly use it
    if let Some(poise::Context::Application(ref a)) = poise_ctx {
        if let Some(ref mem) = a.interaction.member {
            return Ok((
                mem.user.id == guild.owner_id,
                guild.owner_id,
                guild.member_permissions(mem),
                mem.roles.clone(),
            ));
        }
    }

    let member = {
        let member = match proxy_support::member_in_guild(
            cache_http,
            &reqwest::Client::new(),
            guild_id,
            user_id,
        )
        .await
        {
            Ok(member) => member,
            Err(e) => {
                return Err(PermissionResult::DiscordError {
                    error: e.to_string(),
                });
            }
        };

        let Some(member) = member else {
            return Err(PermissionResult::DiscordError {
                error: "Member could not fetched".to_string(),
            });
        };

        member
    };

    Ok((
        member.user.id == guild.owner_id,
        guild.owner_id,
        guild.member_permissions(&member),
        member.roles.clone(),
    ))
}

pub async fn get_user_kittycat_perms(
    opts: &CheckCommandOptions,
    pool: &PgPool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    user_id: UserId,
    roles: &FixedArray<serenity::all::RoleId>,
) -> Result<Vec<kittycat::perms::Permission>, base_data::Error> {
    if let Some(ref custom_resolved_kittycat_perms) = opts.custom_resolved_kittycat_perms {
        if !opts.skip_custom_resolved_fit_checks {
            let kc_perms = crate::member_permission_calc::get_kittycat_perms(
                pool,
                guild_id,
                guild_owner_id,
                user_id,
                roles,
            )
            .await?;

            let mut resolved_perms = Vec::new();
            for perm in custom_resolved_kittycat_perms {
                if kittycat::perms::has_perm(&kc_perms, perm) {
                    resolved_perms.push(perm.clone());
                }
            }

            Ok(resolved_perms)
        } else {
            Ok(custom_resolved_kittycat_perms.to_vec())
        }
    } else {
        Ok(crate::member_permission_calc::get_kittycat_perms(
            pool,
            guild_id,
            guild_owner_id,
            user_id,
            roles,
        )
        .await?)
    }
}

/// Extra options for checking a command
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CheckCommandOptions {
    /// Whether or not to ignore the fact that the module is disabled in the guild
    #[serde(default)]
    pub ignore_module_disabled: bool,

    /// Whether or not to ignore the fact that the command is disabled in the guild
    #[serde(default)]
    pub ignore_command_disabled: bool,

    /// Skip custom resolved kittycat permission fit 'checks' (AKA does the user have the actual permissions ofthe custom resolved permissions)
    #[serde(default)]
    pub skip_custom_resolved_fit_checks: bool,

    /// What custom resolved permissions to use for the user. API needs this for limiting the permissions of a user
    #[serde(default)]
    pub custom_resolved_kittycat_perms: Option<Vec<Permission>>,

    /// Custom command configuration to use
    #[serde(default)]
    pub custom_command_configuration: Option<GuildCommandConfiguration>,

    /// Custom module configuration to use
    #[serde(default)]
    pub custom_module_configuration: Option<GuildModuleConfiguration>,

    /// The current channel id
    #[serde(default)]
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[allow(clippy::derivable_impls)]
impl Default for CheckCommandOptions {
    fn default() -> Self {
        Self {
            ignore_module_disabled: false,
            ignore_command_disabled: false,
            custom_resolved_kittycat_perms: None,
            skip_custom_resolved_fit_checks: false,
            custom_command_configuration: None,
            custom_module_configuration: None,
            channel_id: None,
        }
    }
}

/// Check command checks whether or not a user has permission to run a command
#[allow(clippy::too_many_arguments)]
pub async fn check_command(
    silverpelt_cache: &SilverpeltCache,
    command: &str,
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    // If a poise::Context is available and originates from a Application Command, we can fetch the guild+member from cache itself
    poise_ctx: &Option<crate::Context<'_>>,
    // Needed for settings and the website (potentially)
    opts: CheckCommandOptions,
) -> PermissionResult {
    let command_permutations = permute_command_names(command);

    let module = match silverpelt_cache
        .command_id_module_map
        .get(&command_permutations[0])
    {
        Some(module) => module,
        None => {
            return PermissionResult::ModuleNotFound {};
        }
    };

    info!("Checking if user {} can run command {}", user_id, command);

    if module == "root" {
        if !config::CONFIG.discord_auth.root_users.contains(&user_id) {
            return PermissionResult::SudoNotGranted {};
        }

        return PermissionResult::OkWithMessage {
            message: "root_cmd".to_string(),
        };
    }

    let module_config = {
        if let Some(ref custom_module_configuration) = opts.custom_module_configuration {
            custom_module_configuration.clone()
        } else {
            let gmc = match get_module_configuration(pool, &guild_id.to_string(), module.as_str())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    return e.into();
                }
            };

            gmc.unwrap_or(GuildModuleConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                module: module.clone(),
                disabled: None,
                default_perms: None,
            })
        }
    };

    let cmd_data = match get_command_extended_data(silverpelt_cache, &command_permutations) {
        Ok(v) => v,
        Err(e) => {
            return e.into();
        }
    };

    let command_config = {
        if let Some(ref custom_command_configuration) = opts.custom_command_configuration {
            custom_command_configuration.clone()
        } else {
            let gcc = match get_best_command_configuration(
                pool,
                &guild_id.to_string(),
                &command_permutations,
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    return e.into();
                }
            };

            gcc.unwrap_or(GuildCommandConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                command: command.to_string(),
                perms: None,
                disabled: None,
            })
        }
    };

    // Check if command is disabled if and only if ignore_command_disabled is false
    #[allow(clippy::collapsible_if)]
    if !opts.ignore_command_disabled {
        if command_config
            .disabled
            .unwrap_or(!cmd_data.is_default_enabled)
        {
            return PermissionResult::CommandDisabled {
                command: command.to_string(),
            };
        }
    }

    // Check if module is disabled if and only if ignore_module_disabled is false
    #[allow(clippy::collapsible_if)]
    if !opts.ignore_module_disabled {
        let module_default_enabled = {
            let Some(module) = silverpelt_cache.module_cache.get(module) else {
                return PermissionResult::UnknownModule {
                    module: module.to_string(),
                };
            };

            module.is_default_enabled
        };

        if module_config.disabled.unwrap_or(!module_default_enabled) {
            return PermissionResult::ModuleDisabled {
                module: module.to_string(),
            };
        }
    }

    // Try getting guild+member from cache to speed up response times first
    let (is_owner, guild_owner_id, member_perms, roles) =
        match get_user_discord_info(guild_id, user_id, cache_http, poise_ctx).await {
            Ok(v) => v,
            Err(e) => {
                return e;
            }
        };

    if is_owner {
        return PermissionResult::OkWithMessage {
            message: "owner".to_string(),
        };
    }

    let kittycat_perms =
        match get_user_kittycat_perms(&opts, pool, guild_id, guild_owner_id, user_id, &roles).await
        {
            Ok(v) => v,
            Err(e) => {
                return e.into();
            }
        };

    // Check for permission checks in this order:
    // - command_config.perms
    // - module_config.default_perms
    // - cmd_data.default_perms
    let perms = {
        if let Some(perms) = &command_config.perms {
            perms
        } else if let Some(perms) = &module_config.default_perms {
            perms
        } else {
            &cmd_data.default_perms
        }
    };

    match perms {
        PermissionChecks::Simple { checks } => {
            if checks.is_empty() {
                return PermissionResult::Ok {};
            }

            permissions::eval_checks(checks, member_perms, kittycat_perms)
        }
        PermissionChecks::Template { template } => {
            if template.is_empty() {
                return PermissionResult::Ok {};
            }

            templating::render_permissions_template(
                guild_id,
                template,
                pool.clone(),
                templating::core::PermissionTemplateContext {
                    member_native_permissions: member_perms,
                    member_kittycat_permissions: kittycat_perms,
                    user_id,
                    guild_id,
                    guild_owner_id,
                    channel_id: opts.channel_id,
                },
                templating::CompileTemplateOptions {
                    cache_result: true,
                    ignore_cache: false,
                },
            )
            .await
        }
    }
}

/*
TODO: Move to services/rust.bot

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn check_command_test_cache_bust() {
        // Set the env var CHECK_MODULES_TEST_ENABLED
        std::env::set_var("CHECK_MODULES_TEST_ENABLED", "true");

        // Set current directory to ../../
        let current_dir = std::env::current_dir().unwrap();

        if current_dir.ends_with("core/rust.bot_modules") {
            std::env::set_current_dir("../../").unwrap();
        }

        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection");

        let cache = serenity::all::Cache::new();
        let http = serenity::all::Http::new(&config::CONFIG.discord_auth.token);
        let cache_http = botox::cache::CacheHttpImpl {
            cache: cache.into(),
            http: http.into(),
        };

        let cmd = super::check_command(
            "afk create",
            serenity::all::GuildId::new(1),
            serenity::all::UserId::new(1),
            &pg_pool,
            &cache_http,
            &None,
            // Needed for settings and the website (potentially)
            super::CheckCommandOptions::default(),
        )
        .await;

        match cmd {
            super::PermissionResult::ModuleDisabled { module_config } => {
                assert_eq!(module_config.module, "afk".to_string());
            }
            _ => {
                panic!("Expected ModuleDisabled, got {:?}", cmd);
            }
        }

        let cmd = super::check_command(
            "afk list",
            serenity::all::GuildId::new(1064135068928454766),
            serenity::all::UserId::new(728871946456137770),
            &pg_pool,
            &cache_http,
            &None,
            // Needed for settings and the website (potentially)
            super::CheckCommandOptions::default(),
        )
        .await;

        if !matches!(cmd, super::PermissionResult::ModuleDisabled { .. }) {
            panic!("Expected ModuleDisabled, got {:?}", cmd);
        }
    }
}*/
