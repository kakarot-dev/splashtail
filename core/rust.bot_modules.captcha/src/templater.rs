/// A CaptchaContext is a context for captcha's
/// that can be accessed in captcha templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptchaContext {
    /// The user that triggered the captcha
    pub user: serenity::all::User,
    /// The guild ID that the user triggered the captcha in
    pub guild_id: serenity::all::GuildId,
    /// The channel ID that the user triggered the captcha in. May be None in some cases (captcha not in channel)
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[typetag::serde]
impl templating::Context for CaptchaContext {}
