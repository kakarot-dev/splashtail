-- @pragma {"lang":"lua","allowed_caps":["discord:sendmessage_channel"]}
local args, token = ...
local discord = require "@antiraid/discord"
local interop = require "@antiraid/interop"
local formatter = require "@antiraid/formatters"

-- Make the embed
local embed = {
    title = args.event_titlename, 
    description = "", -- Start with empty description
}

-- Add the event data to the description
for key, value in pairs(args.event_data) do
    local should_set = false

    if value ~= nil and value.type ~= "None" then
        should_set = true
    end

    if should_set then
        local formatted_value = formatter.format_field(value)
        embed.description = embed.description .. "**" .. key:gsub("_", " "):upper() .. "**: " .. formatted_value .. "\n"
    end
end

local message = { embeds = {} }
setmetatable(message.embeds, interop.array_metatable)

table.insert(message.embeds, embed)

-- Send message using action executor
local discord_executor = discord.new(token);
discord_executor:sendmessage_channel({
    channel_id = args.sink,
    message = message
})
