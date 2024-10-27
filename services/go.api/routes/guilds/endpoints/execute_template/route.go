package execute_template

import (
	"net/http"
	"time"

	"go.api/rpc"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	"go.uber.org/zap"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Eexecute Template",
		Description: "This endpoint will execute a Lua template and return the result.",
		Resp:        types.ExecuteTemplateResponse{},
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Whether to refresh the user's guilds from discord",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      6 * time.Minute,
		MaxRequests: 5,
		Bucket:      "execute_template",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "execute_template"))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if limit.Exceeded {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "You are being ratelimited. Please try again in " + limit.TimeToReset.String(),
			},
			Headers: limit.Headers(),
			Status:  http.StatusTooManyRequests,
		}
	}

	guildId := chi.URLParam(r, "guild_id")

	if guildId == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	// Read body
	var body types.ExecuteTemplateRequest

	hresp, ok := uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	// Execute the template
	resp, err := rpc.ExecuteTemplate(d.Context, guildId, d.Auth.ID, &rpc_messages.ExecuteTemplateRequest{
		Args:     body.Args,
		Template: body.Template,
	})

	if err != nil {
		state.Logger.Error("Failed to execute template", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	return uapi.HttpResponse{
		Json: types.ExecuteTemplateResponse{
			Ok:              resp.Ok,
			ExecErr:         resp.ExecErr,
			PermissionError: resp.PermissionError,
		},
	}
}
