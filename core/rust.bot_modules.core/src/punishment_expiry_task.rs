pub async fn punishment_expiry_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    let punishments = silverpelt::punishments::GuildPunishment::get_expired(pool).await?;

    let mut set = tokio::task::JoinSet::new();

    let shard_count = data.props.shard_count().await?.try_into()?;
    let shards = data.props.shards().await?;

    for punishment in punishments {
        let guild_id = punishment.guild_id;

        // Ensure shard id
        let shard_id = serenity::utils::shard_id(guild_id, shard_count);

        if !shards.contains(&shard_id) {
            continue;
        }

        // Dispatch event
        let event = silverpelt::ar_event::AntiraidEvent::PunishmentExpire(punishment);

        let event_handler_context =
            std::sync::Arc::new(silverpelt::ar_event::EventHandlerContext {
                event,
                guild_id,
                data: data.clone(),
                serenity_context: ctx.clone(),
            });

        // Spawn task to dispatch event
        set.spawn(silverpelt::ar_event::dispatch_event_to_modules(
            event_handler_context,
        ));
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                log::error!("Error in temporary_punishment_task: {:?}", e);
            }
            Err(e) => {
                log::error!("Error in temporary_punishment_task: {}", e);
            }
        }
    }

    Ok(())
}
