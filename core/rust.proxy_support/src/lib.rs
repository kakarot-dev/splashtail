use std::collections::HashMap;

pub mod sandwich;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Checks if anti-raid is in a server or not
/// Fetches a guild while handling all the pesky errors serenity normally has
/// with caching
pub async fn has_guild(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: serenity::all::GuildId,
) -> Result<bool, Error> {
    if ctx.cache.guilds().contains(&guild_id) {
        return Ok(true);
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Resp {
        ok: bool,
        data: Option<bool>,
        error: Option<String>,
    }

    // Check sandwich, it may be there
    if let Some(ref proxy_url) = config::CONFIG.meta.sandwich_http_api {
        let url = format!(
            "{}/antiraid/api/state?col=derived.has_guild_id&id={}",
            proxy_url, guild_id
        );

        let resp = reqwest_client.get(&url).send().await?.json::<Resp>().await;

        if let Ok(resp) = resp {
            if resp.ok {
                let Some(has_guild_id) = resp.data else {
                    return Err("Could not derive has_guild_id prop".into());
                };

                return Ok(has_guild_id);
            } else {
                log::warn!(
                    "Sandwich proxy returned error [has guild id]: {:?}",
                    resp.error
                );
            }
        } else {
            log::warn!(
                "Sandwich proxy returned invalid resp [has guild id]: {:?}",
                resp
            );
        }
    }

    let guild_id_immediately_preceding = serenity::all::GuildId::new(guild_id.get() - 1);

    // Last resort, fetch from http
    let gi = match ctx
        .http
        .get_guilds(
            Some(serenity::all::GuildPagination::After(
                guild_id_immediately_preceding,
            )),
            serenity::nonmax::NonMaxU8::new(3),
        )
        .await
    {
        Ok(gi) => gi,
        Err(e) => match e {
            serenity::Error::Http(e) => match e {
                serenity::all::HttpError::UnsuccessfulRequest(er) => {
                    return Err(format!("Failed to fetch guild info (http): {:?}", er).into());
                }
                _ => {
                    return Err(
                        format!("Failed to fetch guild info (non-http error): {:?}", e).into(),
                    );
                }
            },
            _ => {
                return Err(format!("Failed to fetch member: {:?}", e).into());
            }
        },
    };

    // Check if the guild is in the list
    let has_guild_id = gi.iter().any(|g| g.id == guild_id);

    Ok(has_guild_id)
}

/// Fetches a guild while handling all the pesky errors serenity normally has
/// with caching
pub async fn guild(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: serenity::model::id::GuildId,
) -> Result<serenity::all::PartialGuild, Error> {
    let res = ctx.cache.guild(guild_id);

    if let Some(res) = res {
        return Ok(res.clone().into());
    }

    drop(res);

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Resp {
        ok: bool,
        data: Option<serenity::all::PartialGuild>,
        error: Option<String>,
    }

    // Check sandwich, it may be there
    if let Some(ref proxy_url) = config::CONFIG.meta.sandwich_http_api {
        let url = format!(
            "{}/antiraid/api/state?col=guilds&id={}",
            proxy_url, guild_id
        );

        let resp = reqwest_client.get(&url).send().await?.json::<Resp>().await;

        if let Ok(resp) = resp {
            if resp.ok {
                let Some(guild) = resp.data else {
                    return Err("Guild not found".into());
                };

                return Ok(guild);
            } else {
                log::warn!(
                    "Sandwich proxy returned error [get guild]: {:?}",
                    resp.error
                );
            }
        } else {
            log::warn!(
                "Sandwich proxy returned invalid resp [get guild]: {:?}",
                resp
            );
        }
    }

    // Last resore: make the http call
    let res = ctx.http.get_guild(guild_id).await?;

    // Save to sandwich
    if let Some(ref proxy_url) = config::CONFIG.meta.sandwich_http_api {
        let url = format!(
            "{}/antiraid/api/state?col=guilds&id={}",
            proxy_url, guild_id
        );

        let resp = reqwest_client.post(&url).json(&res).send().await?;

        if !resp.status().is_success() {
            log::warn!(
                "Failed to update sandwich proxy with guild data: {:?}",
                resp.text().await
            );
        }
    }

    Ok(res)
}

/// Faster version of botox member in guild that also takes into account the sandwich proxy layer
pub async fn member_in_guild(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: serenity::model::id::GuildId,
    user_id: serenity::model::id::UserId,
) -> Result<Option<serenity::all::Member>, Error> {
    // No sandwich case
    if config::CONFIG.meta.sandwich_http_api.is_none() {
        let res = match botox::cache::member_on_guild(ctx, guild_id, user_id, true).await {
            Ok(res) => res,
            Err(e) => {
                return Err(format!("Failed to fetch member: {:?}", e).into());
            }
        };

        let Some(res) = res else {
            return Ok(None);
        };

        return Ok(Some(res));
    }

    // Check serenity cache
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if let Some(member) = guild.members.get(&user_id).cloned() {
            return Ok(Some(member));
        }
    }

    // Part 2, try sandwich state
    let Some(ref proxy_url) = config::CONFIG.meta.sandwich_http_api else {
        return Err("Sandwich proxy not configured, not proceeding".into());
    };

    let url = format!(
        "{}/antiraid/api/state?col=members&id={}&guild_id={}",
        proxy_url, user_id, guild_id
    );

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Resp {
        ok: bool,
        data: Option<serenity::all::Member>,
        error: Option<String>,
    }

    let resp = reqwest_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?
        .json::<Resp>()
        .await;

    match resp {
        Ok(resp) => {
            if resp.ok {
                let Some(member) = resp.data else {
                    return Ok(None);
                };

                return Ok(Some(member));
            } else {
                log::warn!(
                    "Sandwich proxy returned error [get member]: {:?}",
                    resp.error
                );
            }
        }
        Err(e) => {
            log::warn!("Failed to fetch member (http): {:?}", e);
        }
    }

    // Last resort, use botox to fetch from http and then update sandwich as well
    let member = match ctx.http.get_member(guild_id, user_id).await {
        Ok(mem) => mem,
        Err(e) => match e {
            serenity::Error::Http(e) => match e {
                serenity::all::HttpError::UnsuccessfulRequest(er) => {
                    if er.status_code == reqwest::StatusCode::NOT_FOUND {
                        return Ok(None);
                    } else {
                        return Err(
                            format!("Failed to fetch member (http, non-404): {:?}", er).into()
                        );
                    }
                }
                _ => {
                    return Err(format!("Failed to fetch member (http): {:?}", e).into());
                }
            },
            _ => {
                return Err(format!("Failed to fetch member: {:?}", e).into());
            }
        },
    };

    // Update sandwich with a POST
    let resp = reqwest_client.post(&url).json(&member).send().await?;

    if !resp.status().is_success() {
        log::warn!(
            "Failed to update sandwich proxy with member data: {:?}",
            resp.text().await
        );
    }

    Ok(Some(member))
}

/// Faster version of serenity guild_channels that also takes into account the sandwich proxy layer
pub async fn guild_channels(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: serenity::model::id::GuildId,
) -> Result<Vec<serenity::all::GuildChannel>, Error> {
    // Try serenity cache first
    {
        if let Some(guild) = ctx.cache.guild(guild_id) {
            let channels = guild.channels.clone();
            return Ok(channels.into_iter().collect());
        };
    }

    let Some(ref proxy_url) = config::CONFIG.meta.sandwich_http_api else {
        // Last resort, fetch from http and then update sandwich as well
        let channels = match ctx.http.get_channels(guild_id).await {
            Ok(mem) => mem,
            Err(e) => match e {
                serenity::Error::Http(e) => match e {
                    serenity::all::HttpError::UnsuccessfulRequest(er) => {
                        if er.status_code == reqwest::StatusCode::NOT_FOUND {
                            return Err("No channels found".into());
                        } else {
                            return Err(format!(
                                "Failed to fetch channels (http, non-404): {:?}",
                                er
                            )
                            .into());
                        }
                    }
                    _ => {
                        return Err(format!("Failed to fetch channels (http): {:?}", e).into());
                    }
                },
                _ => {
                    return Err(format!("Failed to fetch channels: {:?}", e).into());
                }
            },
        };

        let channels = channels.into_iter().collect();

        return Ok(channels);
    };

    let url = format!(
        "{}/antiraid/api/state?col=guild_channels&id={}",
        proxy_url, guild_id
    );

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Resp {
        ok: bool,
        data: Option<Vec<serenity::all::GuildChannel>>,
        error: Option<String>,
    }

    let resp = reqwest_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?
        .json::<Resp>()
        .await;

    match resp {
        Ok(resp) => {
            if resp.ok {
                let Some(channels) = resp.data else {
                    return Err("No channels found".into());
                };

                return Ok(channels);
            } else {
                log::warn!(
                    "Sandwich proxy returned error [get guild channels]: {:?}",
                    resp.error
                );
            }
        }
        Err(e) => {
            log::warn!("Failed to fetch member (http): {:?}", e);
        }
    }

    // Last resort, fetch from http and then update sandwich as well
    let channels = match ctx.http.get_channels(guild_id).await {
        Ok(mem) => mem,
        Err(e) => match e {
            serenity::Error::Http(e) => match e {
                serenity::all::HttpError::UnsuccessfulRequest(er) => {
                    if er.status_code == reqwest::StatusCode::NOT_FOUND {
                        return Err("No channels found".into());
                    } else {
                        return Err(
                            format!("Failed to fetch channels (http, non-404): {:?}", er).into(),
                        );
                    }
                }
                _ => {
                    return Err(format!("Failed to fetch channels (http): {:?}", e).into());
                }
            },
            _ => {
                return Err(format!("Failed to fetch channels: {:?}", e).into());
            }
        },
    };

    let channels = channels.into_iter().collect();

    // Update sandwich with a POST
    let resp = reqwest_client.post(&url).json(&channels).send().await?;

    if !resp.status().is_success() {
        log::warn!(
            "Failed to update sandwich proxy with channel data: {:?}",
            resp.text().await
        );
    }

    Ok(channels)
}

pub async fn channel(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: Option<serenity::model::id::GuildId>,
    channel_id: serenity::model::id::ChannelId,
) -> Result<Option<serenity::all::Channel>, Error> {
    // No sandwich case
    if config::CONFIG.meta.sandwich_http_api.is_none() {
        // Check for guild channel
        if let Some(guild_id) = guild_id {
            {
                let guild = ctx.cache.guild(guild_id);

                if let Some(guild) = guild {
                    if let Some(channel) = guild.channels.get(&channel_id) {
                        return Ok(Some(serenity::all::Channel::Guild(channel.clone())));
                    }
                }
            }
        }

        // Make http query
        let channel = match channel_id.to_channel(&ctx, guild_id).await {
            Ok(channel) => channel,
            Err(e) => match e {
                serenity::Error::Http(e) => match e {
                    serenity::all::HttpError::UnsuccessfulRequest(er) => {
                        if er.status_code == reqwest::StatusCode::NOT_FOUND {
                            return Ok(None);
                        } else {
                            return Err(format!(
                                "Failed to fetch channels (http, non-404): {:?}",
                                er
                            )
                            .into());
                        }
                    }
                    _ => {
                        return Err(format!("Failed to fetch channels (http): {:?}", e).into());
                    }
                },
                _ => {
                    return Err(format!("Failed to fetch channels: {:?}", e).into());
                }
            },
        };

        return Ok(Some(channel));
    }

    let proxy_url = config::CONFIG.meta.sandwich_http_api.as_ref().unwrap();

    let url = match guild_id {
        Some(guild_id) => format!(
            "{}/antiraid/api/state?col=channels&id={}&guild_id={}",
            proxy_url, channel_id, guild_id
        ),
        None => format!(
            "{}/antiraid/api/state?col=channels&id={}",
            proxy_url, channel_id
        ),
    };

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Resp {
        ok: bool,
        data: Option<serenity::all::Channel>,
        error: Option<String>,
    }

    let resp = reqwest_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    let status = resp.status();

    let json = resp.json::<Resp>().await?;

    if json.ok {
        return Ok(json.data);
    } else {
        log::warn!(
            "Sandwich proxy returned error [get channel]: {:?}, status: {:?}",
            json.error,
            status
        );
    }

    // Last resort, fetch from http and then update sandwich as well
    let channel = match channel_id.to_channel(&ctx, guild_id).await {
        Ok(channel) => channel,
        Err(e) => match e {
            serenity::Error::Http(e) => match e {
                serenity::all::HttpError::UnsuccessfulRequest(er) => {
                    if er.status_code == reqwest::StatusCode::NOT_FOUND {
                        return Ok(None);
                    } else {
                        return Err(
                            format!("Failed to fetch channels (http, non-404): {:?}", er).into(),
                        );
                    }
                }
                _ => {
                    return Err(format!("Failed to fetch channels (http): {:?}", e).into());
                }
            },
            _ => {
                return Err(format!("Failed to fetch channels: {:?}", e).into());
            }
        },
    };

    // Update sandwich with a POST
    let resp = reqwest_client
        .post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .json(&channel)
        .send()
        .await?;

    if !resp.status().is_success() {
        log::warn!(
            "Failed to update sandwich proxy with channel data: {:?}",
            resp.text().await
        );
    }

    Ok(Some(channel))
}

pub enum ProxyResponse {
    Sandwich(sandwich::StatusEndpointResponse),
}

impl ProxyResponse {
    pub fn to_support_data(&self) -> ProxySupportData {
        match self {
            ProxyResponse::Sandwich(data) => {
                let mut shards = HashMap::new();
                for manager in data.managers.iter() {
                    if manager.display_name != *"Anti Raid" {
                        continue; // Not for us
                    }

                    for v in manager.shard_groups.iter() {
                        for shard in v.shards.iter() {
                            let shard_id = shard[0];
                            let status = shard[1];
                            let latency = shard[2];
                            let guilds = shard[3];
                            let uptime = shard[4];
                            let total_uptime = shard[5];

                            shards.insert(
                                shard_id,
                                ShardConn {
                                    status: match status {
                                        0 => "Idle".to_string(),
                                        1 => "Connecting".to_string(),
                                        2 => "Connected".to_string(),
                                        3 => "MarkedForClosure".to_string(),
                                        4 => "Closing".to_string(),
                                        5 => "Closed".to_string(),
                                        6 => "Erroring".to_string(),
                                        _ => "Unknown".to_string(),
                                    },
                                    real_latency: latency,
                                    guilds,
                                    uptime,
                                    total_uptime,
                                },
                            );
                        }
                    }
                }

                ProxySupportData {
                    resp: ProxyResponse::Sandwich(data.clone()),
                    shard_conns: shards,
                }
            }
        }
    }
}

pub struct ShardConn {
    pub status: String,
    pub real_latency: i64,
    pub guilds: i64,
    pub uptime: i64,
    pub total_uptime: i64,
}

pub struct ProxySupportData {
    pub resp: ProxyResponse,
    pub shard_conns: HashMap<i64, ShardConn>,
}
