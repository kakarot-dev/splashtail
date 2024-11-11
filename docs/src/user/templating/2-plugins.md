# @antiraid/async

Utilities for asynchronous operations and timing

## Methods

**sleep**

```lua
function sleep(duration: f64) -> slept_time: f64
```

Sleep for a given duration.

### Parameters

- `duration` ([f64](#type.f64)): The duration to sleep for.


### Returns

- `slept_time` ([f64](#type.f64)): The actual duration slept for.



---

# @antiraid/interop

This plugin allows interoperability with AntiRaid and controlled interaction with the low-levels of AntiRaid templating subsystem.

## Methods

**memusage**

```lua
function memusage() -> memory_usage: f64
```

Returns the current memory usage of the Lua VM.

### Returns

- `memory_usage` ([f64](#type.f64)): The current memory usage, in bytes, of the Lua VM.

**guild_id**

```lua
function guild_id() -> guild_id: string
```

Returns the current guild ID of the Lua VM.

### Returns

- `guild_id` ([string](#type.string)): The current guild ID.

**gettemplatedata**

```lua
function gettemplatedata(token: string) -> data: TemplateData?
```

Returns the data associated with a template token.

### Parameters

- `token` ([string](#type.string)): The token of the template to retrieve data for.


### Returns

- `data` ([TemplateData?](#type.TemplateData)): The data associated with the template token, or `null` if no data is found.

**current_user**

```lua
function current_user() -> user: serenity::model::user::User
```

Returns the current user of the Lua VM.

### Returns

- `user` ([serenity::model::user::User](https://docs.rs/serenity/latest/model/user/struct.User.html)): Returns AntiRaid's discord user object.



---

# @antiraid/img_captcha

This plugin allows for the creation of text/image CAPTCHA's with customizable filters which can be useful in protecting against bots.

## Methods

**new**

```lua
function new(config: CaptchaConfig) -> captcha: {u8}
```

Creates a new CAPTCHA with the given configuration.

### Parameters

- `config` ([CaptchaConfig](#type.CaptchaConfig)): The configuration to use for the CAPTCHA.


### Returns

- `captcha` ([{u8}](#type.u8)): The created CAPTCHA object.



---

# @antiraid/kv

Utilities for key-value operations.

## Methods

**new**

```lua
function new(token: string)
```

### Parameters

- `token` ([string](#type.string)): The token of the template to use.




---

# @antiraid/permissions

Utilities for handling permission checks.

## Methods

**permission_from_string**

```lua
function permission_from_string(perm_string: string) -> permission: Permission
```

Returns a Permission object from a string.

### Parameters

- `perm_string` ([string](#type.string)): The string to parse into a Permission object.


### Returns

- `permission` ([Permission](#type.Permission)): The parsed Permission object.

**permission_to_string**

```lua
function permission_to_string(permission: Permission) -> perm_string: string
```

Returns a string from a Permission object.

### Parameters

- `permission` ([Permission](#type.Permission)): The Permission object to parse into a string.


### Returns

- `perm_string` ([string](#type.string)): The parsed string.

**has_perm**

```lua
function has_perm(permissions: {Permission}, permission: Permission) -> has_perm: boolean
```

Checks if a list of permissions in Permission object form contains a specific permission.

### Parameters

- `permissions` ([{Permission}](#type.Permission)): The list of permissions
- `permission` ([Permission](#type.Permission)): The permission to check for.


### Returns

- `has_perm` ([boolean](#type.boolean)): Whether the permission is present in the list of permissions as per kittycat rules.

**has_perm_str**

```lua
function has_perm_str(permissions: {string}, permission: string) -> has_perm: boolean
```

Checks if a list of permissions in canonical string form contains a specific permission.

### Parameters

- `permissions` ([{string}](#type.string)): The list of permissions
- `permission` ([string](#type.string)): The permission to check for.


### Returns

- `has_perm` ([boolean](#type.boolean)): Whether the permission is present in the list of permissions as per kittycat rules.

**check_perms_single**

```lua
function check_perms_single(check: PermissionCheck, member_native_perms: Permissions, member_kittycat_perms: {Permission}) -> result: LuaPermissionResult
```

Checks if a single permission check passes.

### Parameters

- `check` ([PermissionCheck](#type.PermissionCheck)): The permission check to evaluate.
- `member_native_perms` ([Permissions](#type.Permissions)): The native permissions of the member.
- `member_kittycat_perms` ([{Permission}](#type.Permission)): The kittycat permissions of the member.


### Returns

- `result` ([LuaPermissionResult](#type.LuaPermissionResult)): The result of the permission check.

**eval_checks**

```lua
function eval_checks(checks: {PermissionCheck}, member_native_perms: Permissions, member_kittycat_perms: {Permission}) -> result: LuaPermissionResult
```

Evaluates a list of permission checks.

### Parameters

- `checks` ([{PermissionCheck}](#type.PermissionCheck)): The list of permission checks to evaluate.
- `member_native_perms` ([Permissions](#type.Permissions)): The native permissions of the member.
- `member_kittycat_perms` ([{Permission}](#type.Permission)): The kittycat permissions of the member.


### Returns

- `result` ([LuaPermissionResult](#type.LuaPermissionResult)): The result of the permission check.



---


