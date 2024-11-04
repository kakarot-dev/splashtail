use sandwich_driver::GetStatusResponse;
use tokio::sync::RwLock;

pub static SANDWICH_STATUS: std::sync::LazyLock<RwLock<Option<GetStatusResponse>>> =
    std::sync::LazyLock::new(|| RwLock::new(None));

pub async fn sandwich_status_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let mut sandwich_status_guard = SANDWICH_STATUS.write().await;

    let status = sandwich_driver::get_status(&data.reqwest).await?;

    if status.shard_conns.len() > data.props.shard_count().await?.into() {
        return Err("Sandwich API returned more shards than the bot has".into());
    }

    *sandwich_status_guard = Some(status);

    Ok(())
}

pub async fn punishment_expiry_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    let punishments = silverpelt::punishments::Punishment::get_expired(pool).await?;

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
                log::error!("Error in punishment_expiry_task: {:?}", e);
            }
            Err(e) => {
                log::error!("Error in punishment_expiry_task: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn stings_expiry_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    let stings = silverpelt::stings::Sting::get_expired(pool).await?;

    let mut set = tokio::task::JoinSet::new();

    let shard_count = data.props.shard_count().await?.try_into()?;
    let shards = data.props.shards().await?;

    for sting in stings {
        let guild_id = sting.guild_id;

        // Ensure shard id
        let shard_id = serenity::utils::shard_id(guild_id, shard_count);

        if !shards.contains(&shard_id) {
            continue;
        }

        // Dispatch event
        let event = silverpelt::ar_event::AntiraidEvent::StingExpire(sting);

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
                log::error!("Error in sting_expiry_task: {:?}", e);
            }
            Err(e) => {
                log::error!("Error in sting_expiry_task: {}", e);
            }
        }
    }

    Ok(())
}
