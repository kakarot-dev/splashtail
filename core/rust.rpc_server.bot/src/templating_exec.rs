use crate::types::{ExecuteTemplateResponse, RpcExecuteTemplateContext};
use crate::AppData;
use axum::{
    extract::{Path, State},
    Json,
};

/// Executes a template on a Lua VM
pub(crate) async fn execute_template(
    State(AppData {
        data,
        serenity_context,
        ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
    Json(req): Json<crate::types::ExecuteTemplateRequest>,
) -> Json<ExecuteTemplateResponse> {
    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        "templating.exec_template",
        guild_id,
        user_id,
        &data.pool,
        &serenity_context,
        &data.reqwest,
        &None,
        permission_checks::CheckCommandOptions::default(),
    )
    .await;

    if !perm_res.is_ok() {
        return Json(ExecuteTemplateResponse::PermissionError { res: perm_res });
    }

    let resp = templating::execute::<_, Option<serde_json::Value>>(
        guild_id,
        templating::Template::Raw(req.template),
        data.pool.clone(),
        serenity_context.clone(),
        data.reqwest.clone(),
        RpcExecuteTemplateContext {
            args: req.args,
            guild_id,
            user_id,
        },
    )
    .await;

    match resp {
        Ok(reply) => Json(ExecuteTemplateResponse::Ok { result: reply }),
        Err(e) => Json(ExecuteTemplateResponse::ExecErr {
            error: e.to_string(),
        }),
    }
}
