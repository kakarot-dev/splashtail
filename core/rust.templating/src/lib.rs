mod atomicinstant;
pub mod core;

mod lang_lua;
pub use lang_lua::samples;
pub use lang_lua::state::LuaKVConstraints;

type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted

use std::str::FromStr;

const MAX_CAPS: usize = 50;
const MAX_PRAGMA_SIZE: usize = 2048;

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TemplatePragma {
    pub lang: TemplateLanguage,

    #[serde(default)]
    pub allowed_caps: Vec<String>,

    #[serde(flatten)]
    pub extra_info: indexmap::IndexMap<String, serde_json::Value>,
}

impl TemplatePragma {
    pub fn parse(template: &str) -> Result<(&str, Self), Error> {
        let (first_line, rest) = match template.find('\n') {
            Some(i) => template.split_at(i),
            None => return Ok((template, Self::default())),
        };

        // Unravel any comments before the @pragma
        let first_line = first_line.trim_start_matches("--").trim();

        if !first_line.contains("@pragma ") {
            return Ok((template, Self::default()));
        }

        // Remove out the @pragma and serde parse it
        let first_line = first_line.replace("@pragma ", "");

        if first_line.as_bytes().len() > MAX_PRAGMA_SIZE {
            return Err("Pragma too large".into());
        }

        let pragma: TemplatePragma = serde_json::from_str(&first_line)?;

        if pragma.allowed_caps.len() > MAX_CAPS {
            return Err("Too many allowed capabilities specified".into());
        }

        Ok((rest, pragma))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum TemplateLanguage {
    #[cfg(feature = "lua")]
    #[serde(rename = "lua")]
    #[default]
    Lua,
}

impl FromStr for TemplateLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "lua")]
            "lang_lua" => Ok(Self::Lua),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for TemplateLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lua")]
            Self::Lua => write!(f, "lang_lua"),
        }
    }
}

/// Parses a shop template of form template_name#version
pub fn parse_shop_template(s: &str) -> Result<(String, String), Error> {
    let s = s.trim_start_matches("$shop/");
    let (template, version) = match s.split_once('#') {
        Some((template, version)) => (template, version),
        None => return Err("Invalid shop template".into()),
    };

    Ok((template.to_string(), version.to_string()))
}

async fn get_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    pool: &sqlx::PgPool,
) -> Result<String, Error> {
    let rec = sqlx::query!(
        "SELECT content FROM guild_templates WHERE guild_id = $1 AND name = $2",
        guild_id.to_string(),
        template
    )
    .fetch_optional(pool)
    .await?;

    let content = match rec {
        Some(rec) => rec.content,
        None => return Err("Template not found".into()),
    };

    if template.starts_with("$shop/") {
        let (shop_tname, shop_tversion) = parse_shop_template(template)?;

        let shop_template = sqlx::query!(
            "SELECT content FROM template_shop WHERE name = $1 AND version = $2",
            shop_tname,
            shop_tversion
        )
        .fetch_optional(pool)
        .await?;

        match shop_template {
            Some(shop_template) => Ok(format!("{}{}", content, shop_template.content)),
            None => Err("Shop template not found".into()),
        }
    } else {
        Ok(content)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Template {
    Raw(String),
    Named(String),
}

#[allow(unused_variables)]
pub async fn parse(
    guild_id: serenity::all::GuildId,
    template: Template,
    pool: sqlx::PgPool,
) -> Result<(), Error> {
    let template_content = match template {
        Template::Raw(ref template) => template.clone(),
        Template::Named(ref template) => get_template(guild_id, template, &pool).await?,
    };

    let (template_content, pragma) = TemplatePragma::parse(&template_content)?;

    Ok(())
}

#[typetag::serde(tag = "type")]
pub trait Context: Send + Sync {}

/// Executes a template
pub async fn execute<C: Context + serde::Serialize, RenderResult: serde::de::DeserializeOwned>(
    guild_id: serenity::all::GuildId,
    template: Template,
    pool: sqlx::PgPool,
    serenity_context: serenity::all::Context,
    reqwest_client: reqwest::Client,
    ctx: C,
) -> Result<RenderResult, Error> {
    let template_content = match template {
        Template::Raw(ref template) => template.clone(),
        Template::Named(ref template) => get_template(guild_id, template, &pool).await?,
    };

    let (template_content, pragma) = TemplatePragma::parse(&template_content)?;

    match pragma.lang {
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => lang_lua::render_template(
            ctx,
            lang_lua::ParseCompileState {
                serenity_context,
                reqwest_client,
                guild_id,
                template,
                pragma,
                template_content: template_content.to_string(),
                pool,
            },
        )
        .await
        .map_err(|e| e.into()),
    }
}
