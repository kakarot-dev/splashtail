package get_apps_list

import (
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/core/go.std/structparser/db"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/state"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"

	"github.com/jackc/pgx/v5"
)

var (
	appColsArr = db.GetCols(types.AppResponse{})
	appCols    = strings.Join(appColsArr, ",")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Application List",
		Description: "Gets all applications of the user returning a list of apps.",
		Params:      []docs.Parameter{},
		Resp:        types.AppListResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	row, err := state.Pool.Query(d.Context, "SELECT "+appCols+" FROM apps WHERE user_id = $1", d.Auth.ID)

	if err != nil {
		state.Logger.Error("Failed to fetch application list [db fetch]", zap.String("userId", d.Auth.ID), zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	app, err := pgx.CollectRows(row, pgx.RowToStructByName[types.AppResponse])

	if err != nil {
		state.Logger.Error("Failed to fetch application list [collection]", zap.String("userId", d.Auth.ID), zap.Error(err))
		return uapi.DefaultResponse(http.StatusNotFound)
	}

	return uapi.HttpResponse{
		Json: types.AppListResponse{
			Apps: app,
		},
	}
}