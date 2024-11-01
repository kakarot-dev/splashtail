use sandwich_driver::{guild, member_in_guild};
use serenity::all::{GuildId, UserId};

/// This struct stores a guild punishment autotrigger that can then be used to trigger punishments
/// on a user through the bot based on sting count
#[derive(Clone)]
pub struct GuildPunishmentAutoTrigger {
    pub id: String,
    pub guild_id: GuildId,
    pub created_by: UserId,
    pub stings: i32,
    pub action: String,
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
///
/// Note that the guild punishment list should not be modified directly
#[derive(Clone)]
pub struct GuildPunishmentAutoTriggerList {
    punishments: Vec<GuildPunishmentAutoTrigger>,
}

impl GuildPunishmentAutoTriggerList {
    /// Gets the punishment list of a specific guild
    pub async fn guild(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
    ) -> Result<Self, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();

        let rec = sqlx::query!(
                "SELECT id, guild_id, created_by, stings, action, modifiers, created_at, EXTRACT(seconds FROM duration)::integer AS duration FROM punishment_autotriggers__autotriggers WHERE guild_id = $1",
                guild_id.to_string(),
            )
            .fetch_all(&data.pool)
            .await?;

        let mut punishments = vec![];

        for row in rec {
            punishments.push(GuildPunishmentAutoTrigger {
                id: row.id.to_string(),
                guild_id: row.guild_id.parse::<GuildId>()?,
                created_by: row.created_by.parse::<UserId>()?,
                stings: row.stings,
                action: row.action,
                modifiers: row.modifiers,
                duration: row.duration,
                created_at: row.created_at,
            });
        }

        Ok(Self { punishments })
    }

    /// Returns the list of punishments
    ///
    /// This is a method to ensure that the returned list is not modified (is immutable)
    #[allow(dead_code)]
    pub fn punishments(&self) -> &Vec<GuildPunishmentAutoTrigger> {
        &self.punishments
    }

    /// Filter returns a new GuildPunishmentList with only the punishments that match the set of filters
    ///
    /// Note that this drops the existing punishment list
    pub fn filter(&self, stings: i32) -> Vec<GuildPunishmentAutoTrigger> {
        let mut punishments = vec![];

        for punishment in self.punishments.iter() {
            if punishment.stings <= stings {
                punishments.push(punishment.clone());
            }
        }

        punishments
    }
}

// TODO: Readd support for modifiers later
pub(crate) async fn autotrigger(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let (per_user_sting_counts, _system_stings) =
        silverpelt::stings::StingAggregate::total_stings_per_user(
            silverpelt::stings::get_aggregate_stings_for_guild(&data.pool, guild_id).await?,
        );

    let punishments = GuildPunishmentAutoTriggerList::guild(ctx, guild_id).await?;

    if punishments.punishments().is_empty() {
        return Ok(());
    }

    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);

    let bot_userid = ctx.cache.current_user().id;
    let Some(bot) = member_in_guild(&cache_http, &data.reqwest, guild_id, bot_userid).await? else {
        return Err("Bot not found".into());
    };

    let guild = guild(&cache_http, &data.reqwest, guild_id).await?;

    for (user_id, sting_count) in per_user_sting_counts {
        let Some(mut user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await?
        else {
            return Ok(());
        };

        if guild
            .greater_member_hierarchy(&bot, &user)
            .unwrap_or(user.user.id)
            == user.user.id
        {
            return Err(
                "Bot does not have the required permissions to carry out this action".into(),
            );
        }

        let punishments = punishments.filter(sting_count.try_into()?);

        for punishment in punishments.iter() {
            match punishment.action.as_str() {
                "kick" => {
                    user.kick(
                        &cache_http.http,
                        Some(&format!(
                            "[Auto-Triggered] {} at {} stings",
                            punishment.action, sting_count
                        )),
                    )
                    .await?
                }
                "ban" => {
                    user.ban(
                        &cache_http.http,
                        0,
                        Some(&format!(
                            "[Auto-Triggered] {} at {} stings",
                            punishment.action, sting_count
                        )),
                    )
                    .await?
                }
                "timeout" => {
                    let new_time = chrono::Utc::now()
                        + chrono::Duration::seconds(punishment.duration.unwrap_or(60 * 5).into());
                    user.edit(
                        &cache_http.http,
                        serenity::all::EditMember::new()
                            .disable_communication_until(serenity::all::Timestamp::from(new_time))
                            .audit_log_reason(&format!(
                                "[Auto-Triggered] {} at {} stings",
                                punishment.action, sting_count
                            )),
                    )
                    .await?
                }
                "removeallroles" => {
                    user.edit(
                        &cache_http.http,
                        serenity::all::EditMember::new()
                            .roles(Vec::new())
                            .audit_log_reason(&format!(
                                "[Auto-Triggered] {} at {} stings",
                                punishment.action, sting_count
                            )),
                    )
                    .await?
                }
                _ => {
                    return Ok(());
                }
            }

            // Add punishment
            silverpelt::punishments::PunishmentCreate {
                module: "punishments".to_string(),
                src: None,
                punishment: punishment.action.clone(),
                creator: silverpelt::punishments::PunishmentTarget::System,
                target: silverpelt::punishments::PunishmentTarget::User(user.user.id),
                handle_log: serde_json::json!({}),
                guild_id,
                duration: None, // TODO: Auto-triggered punishments do not support duration yet
                reason: format!(
                    "[Auto-Triggered] {} at {} stings",
                    punishment.action, sting_count
                ),
                data: None,
            }
            .create_and_dispatch(ctx.clone(), &data.pool)
            .await?;
        }
    }

    Ok(())
}
