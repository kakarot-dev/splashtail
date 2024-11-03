-- @pragma {"lang":"lua","allowed_caps":["discord:create_message","kv:*"]}
local args, token = ...
local discord = require "@antiraid/discord"

if args.event_name ~= "MESSAGE" then
    return -- Early return if not a message event
end

-- Get the user ID from the event data
local user_id = args.event_data["new_message"]["user"]["id"]
local channel_id = args.event_data["new_message"]["channel_id"]

-- Send message using action executor
local discord_executor = discord.new(token);
discord_executor:create_message({
    channel_id = channel_id,
    message = user_id
})
