# @antiraid/async

Utilities for asynchronous operations and timing

## Methods

### sleep

```lua
function sleep(duration: f64) -> f64
```

Sleep for a given duration.

#### Parameters

- `duration` ([f64](#type.f64)): The duration to sleep for.


#### Returns

- `slept_time` ([f64](#type.f64)): The actual duration slept for.



---

# @antiraid/interop

This plugin allows interoperability with AntiRaid and controlled interaction with the low-levels of AntiRaid templating subsystem.

## Types

<div id="type.null" />

### null

`null` is a special value that represents nothing. It is often used in AntiRaid instead of `nil` due to issues regarding existence etc. `null` is not equal to `nil` but is also an opaque type.



<div id="type.array_metatable" />

### array_metatable

`array_metatable` is a special metatable that is used to represent arrays across the Lua-AntiRaid templating subsystem boundary. This metatable must be set on all arrays over this boundary and is required to ensure AntiRaid knows the value you're sending it is actually an array and not an arbitrary Luau table.



<div id="type.TemplatePragma" />

### TemplatePragma

`TemplatePragma` contains the pragma of the template. Note that the list of fields below in non-exhaustive as templates can define extra fields on the pragma as well

```json
{
  "lang": "lua",
  "allowed_caps": []
}
```

#### Fields

- `lang` ([string](#type.string)): The language of the template.
- `allowed_caps` ([{string}](#type.string)): The allowed capabilities provided to the template.


<div id="type.TemplateData" />

### TemplateData

`TemplateData` is a struct that represents the data associated with a template token. It is used to store the path and pragma of a template token.

```json
{
  "path": "",
  "pragma": {
    "lang": "lua",
    "allowed_caps": []
  }
}
```

#### Fields

- `path` ([string](#type.string)): The path of the template token.
- `pragma` ([TemplatePragma](#type.TemplatePragma)): The pragma of the template.


## Methods

### array_metatable

```lua
function array_metatable() -> table
```

Returns the array metatable.

#### Returns

- `array_metatable` ([table](#type.table)): The array metatable.

### null

```lua
function null() -> null
```

Returns the null value.

#### Returns

- `null` ([null](#type.null)): The null value.

### memusage

```lua
function memusage() -> f64
```

Returns the current memory usage of the Lua VM.

#### Returns

- `memory_usage` ([f64](#type.f64)): The current memory usage, in bytes, of the Lua VM.

### guild_id

```lua
function guild_id() -> string
```

Returns the current guild ID of the Lua VM.

#### Returns

- `guild_id` ([string](#type.string)): The current guild ID.

### gettemplatedata

```lua
function gettemplatedata(token: string) -> TemplateData?
```

Returns the data associated with a template token.

#### Parameters

- `token` ([string](#type.string)): The token of the template to retrieve data for.


#### Returns

- `data` ([TemplateData?](#type.TemplateData)): The data associated with the template token, or `null` if no data is found.

### current_user

```lua
function current_user() -> serenity::model::user::User
```

Returns the current user of the Lua VM.

#### Returns

- `user` ([serenity::model::user::User](https://docs.rs/serenity/latest/serenity/model/user/struct.User.html)): Returns AntiRaid's discord user object.



---

# @antiraid/img_captcha

This plugin allows for the creation of text/image CAPTCHA's with customizable filters which can be useful in protecting against bots.

## Types

<div id="type.CaptchaConfig" />

### CaptchaConfig

Captcha configuration. See examples for the arguments

```json
{
  "char_count": 5,
  "filters": [
    {
      "filter": "Noise",
      "prob": 0.1
    },
    {
      "filter": "Wave",
      "f": 4.0,
      "amp": 2.0,
      "d": "horizontal"
    },
    {
      "filter": "Line",
      "p1": [
        1.0,
        0.0
      ],
      "p2": [
        20.0,
        20.0
      ],
      "thickness": 2.0,
      "color": {
        "r": 0,
        "g": 30,
        "b": 100
      }
    },
    {
      "filter": "RandomLine"
    },
    {
      "filter": "Grid",
      "y_gap": 30,
      "x_gap": 10
    },
    {
      "filter": "ColorInvert"
    }
  ],
  "viewbox_size": [
    512,
    512
  ],
  "set_viewbox_at_idx": null
}
```

#### Fields

- `filter` ([string](#type.string)): The name of the filter to use. See example for the parameters to pass for the filter as well as https://github.com/Anti-Raid/captcha.


## Methods

### new

```lua
function new(config: CaptchaConfig) -> {u8}
```

Creates a new CAPTCHA with the given configuration.

#### Parameters

- `config` ([CaptchaConfig](#type.CaptchaConfig)): The configuration to use for the CAPTCHA.


#### Returns

- `captcha` ([{u8}](#type.u8)): The created CAPTCHA object.



---

# @antiraid/kv

Utilities for key-value operations.

## Types

<div id="type.KvRecord" />

### KvRecord

KvRecord represents a key-value record with metadata.

```json
{
  "key": "",
  "value": null,
  "exists": false,
  "created_at": null,
  "last_updated_at": null
}
```

#### Fields

- `key` ([string](#type.string)): The key of the record.
- `value` ([any](#type.any)): The value of the record.
- `exists` ([boolean](#type.boolean)): Whether the record exists.
- `created_at` ([datetime](#type.datetime)): The time the record was created.
- `last_updated_at` ([datetime](#type.datetime)): The time the record was last updated.


<div id="type.KvExecutor" />

### KvExecutor

KvExecutor allows templates to get, store and find persistent data within a server.



#### Methods

### KvExecutor:find

```lua
function KvExecutor:find(key: string)
```

#### Parameters

- `key` ([string](#type.string)): The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%

### KvExecutor:get

```lua
function KvExecutor:get(key: string)
```

#### Parameters

- `key` ([string](#type.string)): The key to get.


#### Returns

- `value` ([any](#type.any)): The value of the key.- `exists` ([boolean](#type.boolean)): Whether the key exists.
### KvExecutor:getrecord

```lua
function KvExecutor:getrecord(key: string) -> KvRecord
```

#### Parameters

- `key` ([string](#type.string)): The key to get.


#### Returns

- `record` ([KvRecord](#type.KvRecord)): The record of the key.
### KvExecutor:set

```lua
function KvExecutor:set(key: string, value: any)
```

#### Parameters

- `key` ([string](#type.string)): The key to set.
- `value` ([any](#type.any)): The value to set.

### KvExecutor:delete

```lua
function KvExecutor:delete(key: string)
```

#### Parameters

- `key` ([string](#type.string)): The key to delete.



## Methods

### new

```lua
function new(token: string) -> KvExecutor
```

#### Parameters

- `token` ([string](#type.string)): The token of the template to use.


#### Returns

- `executor` ([KvExecutor](#type.KvExecutor)): A key-value executor.



---

# @antiraid/permissions

Utilities for handling permission checks.

## Types

<div id="type.PermissionResult" />

### PermissionResult

PermissionResult is an internal type containing the status of a permission check in AntiRaid. The exact contents are undocumented as of now



<div id="type.LuaPermissionResult" />

### LuaPermissionResult

LuaPermissionResult is a type containing the status of a permission check in AntiRaid with prior parsing done for Lua.

```json
{
  "result": {
    "var": "Ok"
  },
  "is_ok": true,
  "code": "Ok",
  "markdown": ""
}
```

#### Fields

- `result` ([PermissionResult](#type.PermissionResult)): The raw/underlying result of the permission check.
- `is_ok` ([boolean](#type.boolean)): Whether the permission check was successful.
- `code` ([string](#type.string)): The code of the permission check.
- `markdown` ([string](#type.string)): The markdown representation of the permission check.


<div id="type.PermissionCheck" />

### PermissionCheck

PermissionCheck is a type containing the permissions to check for a user.

```json
{
  "kittycat_perms": [],
  "native_perms": [],
  "outer_and": false,
  "inner_and": false
}
```

#### Fields

- `kittycat_perms` ([{Permission}](#type.Permission)): The kittycat permissions needed to run the command.
- `native_perms` ([{string}](#type.string)): The native permissions needed to run the command.
- `outer_and` ([boolean](#type.boolean)): Whether the next permission check should be ANDed (all needed) or OR'd (at least one) to the current
- `inner_and` ([boolean](#type.boolean)): Whether or not the perms are ANDed (all needed) or OR'd (at least one)


<div id="type.Permission" />

### Permission

Permission is the primitive permission type used by AntiRaid. See https://github.com/InfinityBotList/kittycat for more information

```json
{
  "namespace": "moderation",
  "perm": "ban",
  "negator": false
}
```

#### Fields

- `namespace` ([string](#type.string)): The namespace of the permission.
- `perm` ([string](#type.string)): The permission bit on the namespace.
- `negator` ([boolean](#type.boolean)): Whether the permission is a negator permission or not


## Methods

### permission_from_string

```lua
function permission_from_string(perm_string: string) -> Permission
```

Returns a Permission object from a string.

#### Parameters

- `perm_string` ([string](#type.string)): The string to parse into a Permission object.


#### Returns

- `permission` ([Permission](#type.Permission)): The parsed Permission object.

### permission_to_string

```lua
function permission_to_string(permission: Permission) -> string
```

Returns a string from a Permission object.

#### Parameters

- `permission` ([Permission](#type.Permission)): The Permission object to parse into a string.


#### Returns

- `perm_string` ([string](#type.string)): The parsed string.

### has_perm

```lua
function has_perm(permissions: {Permission}, permission: Permission) -> boolean
```

Checks if a list of permissions in Permission object form contains a specific permission.

#### Parameters

- `permissions` ([{Permission}](#type.Permission)): The list of permissions
- `permission` ([Permission](#type.Permission)): The permission to check for.


#### Returns

- `has_perm` ([boolean](#type.boolean)): Whether the permission is present in the list of permissions as per kittycat rules.

### has_perm_str

```lua
function has_perm_str(permissions: {string}, permission: string) -> boolean
```

Checks if a list of permissions in canonical string form contains a specific permission.

#### Parameters

- `permissions` ([{string}](#type.string)): The list of permissions
- `permission` ([string](#type.string)): The permission to check for.


#### Returns

- `has_perm` ([boolean](#type.boolean)): Whether the permission is present in the list of permissions as per kittycat rules.

### check_perms_single

```lua
function check_perms_single(check: PermissionCheck, member_native_perms: Permissions, member_kittycat_perms: {Permission}) -> LuaPermissionResult
```

Checks if a single permission check passes.

#### Parameters

- `check` ([PermissionCheck](#type.PermissionCheck)): The permission check to evaluate.
- `member_native_perms` ([Permissions](#type.Permissions)): The native permissions of the member.
- `member_kittycat_perms` ([{Permission}](#type.Permission)): The kittycat permissions of the member.


#### Returns

- `result` ([LuaPermissionResult](#type.LuaPermissionResult)): The result of the permission check.

### eval_checks

```lua
function eval_checks(checks: {PermissionCheck}, member_native_perms: Permissions, member_kittycat_perms: {Permission}) -> LuaPermissionResult
```

Evaluates a list of permission checks.

#### Parameters

- `checks` ([{PermissionCheck}](#type.PermissionCheck)): The list of permission checks to evaluate.
- `member_native_perms` ([Permissions](#type.Permissions)): The native permissions of the member.
- `member_kittycat_perms` ([{Permission}](#type.Permission)): The kittycat permissions of the member.


#### Returns

- `result` ([LuaPermissionResult](#type.LuaPermissionResult)): The result of the permission check.



---


