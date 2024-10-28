# Permissions

Imagine. Imagine a discord bot that you could completely control. You could decide who can use any specific command, who can change the bot's settings, and who can even use the bot at all. 

*Thats AntiRaid...*

AntiRaid has a customizable permission system that uses both Discord permissions for simplicity and [kittycat](https://github.com/InfinityBotList/kittycat) permissions for more specific requirements. For more complex cases, AntiRaid provides support for Lua scripting, which can be used to extend Antiraid with arbitrarily complex permission systems, among other things.

The idea is simple: All roles have permissions attached to them and members can have special permission overrides. The members' permissions are then checked when a command is run.

## Modes

Anti-Raid has two different modes for permission checks depending on how custom your needs are:

- ``Simple``: In simple mode, just specify the exact permissions needed to run a command. This is the default mode.
- ``Template``: If you have more advanced needs, use custom templates to determine if a user has the required permissions. See [`Templating`](../templating-lua/1-intro.md) for more information on how templating works.

## Simple Permission Checks

Since not everyone knows how to code, AntiRaid provides a simple permission-checking system built in that should be enough for most:

1. Commands can have permissions that gate actions.
2. Commands can be either real or virtual. Real commands can be run. Virtual commands are placeholders for permission-gating actions.
3. Commands can be configured by setting their permissions or disabling them (some commands cannot be disabled to avoid breakage).
4. Server admins can set permissions on their server roles and then override them for specific users through permission overrides. 
5. Server admins can then set permissions on commands and default permissions on modules. These permissions are then checked when a command is run.

## Template Permission Checks

For more advanced users, AntiRaid provides a template system that allows you to create custom permission checks. This is done through custom Luau templating. 

See the [templating guide](../templating-lua/1-intro.md) for more information on how to use Lua templates. Then, just code away!

## TIP

For best results, consider limiting the server permissions of other users to the minimum required. Then, use AntiRaid for actual moderation. That's better than giving everyone admin permissions and then trying to restrict them with AntiRaid.