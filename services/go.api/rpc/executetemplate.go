package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the BaseGuildUserInfo method to get basic user + guild info (template-exec/:guild_id/:user_id)
func ExecuteTemplate(
	ctx context.Context,
	guildID string,
	userID string,
	req *rpc_messages.ExecuteTemplateRequest,
) (res *rpc_messages.ExecuteTemplateResponse, err error) {
	return RpcQuery[rpc_messages.ExecuteTemplateResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/template-exec/%s/%s", CalcBotAddr(), guildID, userID),
		req,
		true,
	)
}
