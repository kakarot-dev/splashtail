use crate::{impls::cache::CacheHttpImpl, jobserver::Task};
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use serde_json::Value;

pub struct PollTaskOptions {
    /// The interval at which to update/poll at in seconds
    pub interval: Option<u64>,
}

/**
 * export const createTaskEmbed = (ctx: CommandContext, task: Task): ContextEdit => {
    let taskStatuses: string[] = []
    let taskStatusesLength = 0

    let taskState = task?.state

    for(let status of task.statuses) {
        if(taskStatusesLength > 2500) {
            // Keep removing elements from start of array until we are under 2500 characters
            while(taskStatusesLength > 2500) {
                let removed = taskStatuses.shift()
                taskStatusesLength -= removed.length
            }
        }

        let add = `\`${status?.level}\` ${status?.msg}`

        let vs: string[] = []
        for(let [k, v] of Object.entries(status || {})) {
            if(k == "level" || k == "msg" || k == "ts" || k == "botDisplayIgnore") continue
            if(status["botDisplayIgnore"]?.includes(k)) continue

            vs.push(`${k}=${typeof v == "object" ? JSON.stringify(v) : v}`)
        }

        if(vs.length > 0) add += ` ${vs.join(", ")}`

        add = add.slice(0, 500) + (add.length > 500 ? "..." : "")

        add += ` | \`[${new Date(status?.ts * 1000)}]\``

        taskStatuses.push(add)
        taskStatusesLength += (add.length > 500 ? 500 : add.length)
    }

    let emoji = ":white_check_mark:"

    switch (taskState) {
        case "pending":
            emoji = ":hourglass:"
            break;
        case "running":
            emoji = ":hourglass_flowing_sand:"
            break;
        case "completed":
            emoji = ":white_check_mark:"
            break;
        case "failed":
            emoji = ":x:"
            break;
    }

    let description = `${emoji} Task state: ${taskState}\nTask ID: ${task?.task_id}\n\n${taskStatuses.join("\n")}`
    let components: Component[] = []

    if(taskState == "completed") {
        if(task?.output?.filename) {
            description += `\n\n:link: [Download](${ctx.client.apiUrl}/tasks/${task?.task_id}/ioauth/download-link)`

            components.push(
                new ActionRowBuilder()
                .addComponents(
                    new ButtonBuilder()
                    .setLabel("Download")
                    .setStyle(ButtonStyle.Link)
                    .setURL(`${ctx.client.apiUrl}/tasks/${task?.task_id}/ioauth/download-link`)
                    .setEmoji("📥")
                )
                .toJSON()
            )    
        }
    }

    let embed = new EmbedBuilder()
    .setTitle("Creating backup")
    .setDescription(description)
    .setColor("Green")

    if(taskState == "completed") {
        embed.setFooter({
            text: "Backup created successfully"
        })
    }

    return {
        embeds: [embed],
        components
    }
}
 */

fn _to_string(v: &Option<&Value>) -> String {
    let v = match v {
        Some(v) => v,
        None => return "null".to_string(),
    };

    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(a) => a.iter().map(|v| _to_string(&Some(v))).collect::<Vec<_>>().join(", "),
        Value::Object(o) => o.iter().map(|(k, v)| format!("{}={}", k, _to_string(&Some(v)))).collect::<Vec<_>>().join(", "),
    }
}

pub async fn embed<'a>(task: &Task) -> Result<poise::CreateReply<'a>, crate::Error> {
    let mut task_statuses: Vec<String> = Vec::new();
    let mut task_statuses_length = 0;
    let mut components = Vec::new();

    let task_state = &task.state;

    for status in &task.statuses {
        if task_statuses_length > 2500 {
            // Keep removing elements from start of array until we are under 2500 characters
            while task_statuses_length > 2500 {
                let removed = task_statuses.remove(0);
                task_statuses_length -= removed.len();
            }
        }

        let mut add = format!("`{}` {}", status.level, status.msg);

        let mut vs = Vec::new();

        let bdi = status.bot_display_ignore.clone().unwrap_or_default();

        for (k, v) in status.extra_info.iter() {
            if bdi.contains(k) {
                continue;
            }

            vs.push(format!("{}={}", k, serde_json::to_string(v)?));
        }

        if !vs.is_empty() {
            add += &format!(" {}", vs.join(", "));
        }

        add = add.chars().take(500).collect::<String>() + if add.len() > 500 { "..." } else { "" };

        add += &format!(" | <t:{}:R>`", status.ts);

        task_statuses_length += if add.len() > 500 { 500 } else { add.len() };
        task_statuses.push(add);
    }

    let mut description = format!(
        "{} Task state: {}\nTask ID: {}\n\n{}",
        match task_state.as_str() {
            "pending" => ":hourglass:",
            "running" => ":hourglass_flowing_sand:",
            "completed" => ":white_check_mark:",
            "failed" => ":x:",
            _ => ":question:",
        },
        task_state,
        task.task_id,
        task_statuses.join("\n")
    );

    if task.state == "completed" {
        if let Some(ref output) = task.output {
            let furl = format!("{}/tasks/{}/ioauth/download-link", crate::config::CONFIG.sites.api.get(), task.task_id);
            description += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);

            components.push(
                poise::serenity_prelude::CreateActionRow::Buttons(
                    vec![
                        poise::serenity_prelude::CreateButton::new_link(
                            furl,
                        )
                        .label("Download")
                        .emoji('📥'),
                    ]
                ),
            );
        }
    }

    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title("Task Status")
        .description(description)
        .color(poise::serenity_prelude::Colour::DARK_GREEN);

    let msg = poise::CreateReply::default().embed(embed).components(components);

    Ok(msg)
}

pub async fn reactive(
    cache_http: &CacheHttpImpl,
    pool: &sqlx::PgPool,
    task_id: &str,
    func: impl Fn(&CacheHttpImpl, &Task) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>>>>,
    to: PollTaskOptions,
) -> Result<(), crate::Error> {
    let interval = to.interval.unwrap_or(1000);
    let duration = std::time::Duration::from_secs(interval);
    let mut interval = tokio::time::interval(duration);
    let task_id = sqlx::types::uuid::Uuid::parse_str(task_id)?;
    let mut prev_task: Option<Arc<Task>> = None;
    loop {
        interval.tick().await;

        let rec = sqlx::query!(
            "SELECT task_id, task_name, output, task_info, statuses, task_for, expiry, state, created_at FROM tasks WHERE task_id = $1",
            task_id,
        )
        .fetch_one(pool)
        .await?;

        let mut statuses = Vec::new();

        for status in &rec.statuses {
            let status = serde_json::from_value::<crate::jobserver::TaskStatuses>(status.clone())?;
            statuses.push(status);
        }

        let task = Arc::new(Task {
            task_id: rec.task_id,
            task_name: rec.task_name,
            output: rec.output.map(serde_json::from_value::<crate::jobserver::TaskOutput>).transpose()?,
            task_info: serde_json::from_value::<crate::jobserver::TaskInfo>(rec.task_info)?,
            statuses,
            task_for: rec.task_for.map(|task_for| task_for.into()),
            expiry: {
                if let Some(expiry) = rec.expiry {
                    let t = expiry.microseconds + 60 * 1_000_000 + (expiry.days as i64) * 24 * 60 * 60 * 1_000_000 + (expiry.months as i64) * 30 * 24 * 60 * 60 * 1_000_000;
                    Some(
                        chrono::Duration::microseconds(t)
                    )
                } else {
                    None
                }
            },
            state: rec.state,
            created_at: rec.created_at,
        });

        if let Some(ref prev_task) = prev_task {
            if prev_task.state == task.state && task.statuses == prev_task.statuses {
                continue;
            }
        }

        prev_task = Some(task.clone());

        if task.state != "pending" && task.state != "running" {
            break;
        }

        func(cache_http, &task).await?;
    }

    drop(prev_task); // Drop prev_task

    Ok(())
}