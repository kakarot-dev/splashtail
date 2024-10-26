-- @pragma {"lang":"lua","actions":["sendmessage_channel"]}
local args, token = ...
local message_plugin = require "@antiraid/message"
local actions_plugin = require "@antiraid/actions"

-- Make the embed
local embed = message_plugin.new_message_embed()
embed.title = args.event_titlename
embed.description = "" -- Start with an empty description

-- Add the event data to the description
for key, value in pairs(args.event_data) do
    local should_set = false

    if value ~= nil and value.type ~= "None" then
        should_set = true
    end

    if should_set then
        local formatted_value = message_plugin.format_gwevent_field(value)
        embed.description = embed.description .. "**" .. key:gsub("_", " "):upper() .. "**: " .. formatted_value .. "\n"
    end
end

local message = message_plugin.new_message()

table.insert(message.embeds, embed)

-- Send message using action executor
local actions_executor = actions_plugin.new(token);
actions_executor:sendmessage_channel({
    channel_id = args.sink,
    message = message
})
