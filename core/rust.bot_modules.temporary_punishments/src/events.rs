use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

/// Temporary Punishments event listener
pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    match ectx.event {
        AntiraidEvent::PunishmentExpire(ref punishment) => {
            let target_user_id = match punishment.target {
                silverpelt::punishments::PunishmentTarget::User(user_id) => user_id,
                _ => return Ok(()),
            };

            let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);

            let bot_id = ectx.serenity_context.cache.current_user().id;

            let current_user = match sandwich_driver::member_in_guild(
                &cache_http,
                &ectx.data.reqwest,
                punishment.guild_id,
                bot_id,
            )
            .await?
            {
                Some(user) => user,
                None => {
                    sqlx::query!(
                        "UPDATE punishments SET duration = NULL, handle_log = $1 WHERE id = $2",
                        serde_json::json!({
                            "error": "Bot not in guild",
                        }),
                        punishment.id
                    )
                    .execute(&ectx.data.pool)
                    .await?;

                    return Ok(());
                }
            };

            let permissions = current_user.permissions(&ectx.serenity_context.cache)?;

            // Bot doesn't have permissions to unban
            if !permissions.ban_members() {
                sqlx::query!(
                    "UPDATE punishments SET duration = NULL, handle_log = $1 WHERE id = $2",
                    serde_json::json!({
                        "error": "Bot doesn't have permissions to unban",
                    }),
                    punishment.id
                )
                .execute(&ectx.data.pool)
                .await?;
            }

            let reason = format!(
                "Revert expired ban with reason={}, duration={:#?}",
                punishment.reason, punishment.duration
            );

            match punishment.punishment.as_str() {
                "ban" => {
                    punishment
                        .guild_id
                        .unban(&ectx.serenity_context.http, target_user_id, Some(&reason))
                        .await?;
                }
                "timeout" => {
                    punishment
                        .guild_id
                        .edit_member(
                            &ectx.serenity_context.http,
                            target_user_id,
                            serenity::all::EditMember::new()
                                .enable_communication()
                                .audit_log_reason(&reason),
                        )
                        .await?;
                }
                "removeallroles" => {
                    punishment
                        .guild_id
                        .edit_member(
                            &ectx.serenity_context.http,
                            target_user_id,
                            serenity::all::EditMember::new()
                                .roles(Vec::new())
                                .audit_log_reason(&reason),
                        )
                        .await?;
                }
                _ => {
                    return Ok(());
                }
            }

            Ok(())
        }
        _ => {
            Ok(()) // Ignore non-discord events
        }
    }
}
