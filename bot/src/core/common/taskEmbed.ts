import { ActionRowBuilder, ButtonBuilder, ButtonStyle, EmbedBuilder } from "discord.js";
import { CommandContext, ContextEdit, Component } from "../context";
import { Task } from "../../generatedTypes/types";

export const createTaskEmbed = (ctx: CommandContext, task: Task): ContextEdit => {
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