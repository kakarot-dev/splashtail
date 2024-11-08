package get_bot_stats

import (
	"errors"
	"fmt"
	"net/http"

	"go.api/types"

	"time"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/state"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Bot Stats",
		Description: "This endpoint returns the current stats of the bot. Note that these results may be cached and may not be up to date.",
		Resp:        []types.GetStatusResponse{},
		Params:      []docs.Parameter{},
	}
}

var LastCacheUpdate int64
var SandwichResponse *types.GetStatusResponse

func getStatus(client http.Client) (*types.GetStatusResponse, error) {
	url := fmt.Sprintf("%s/api/status", state.Config.Meta.SandwichHttpApi)

	resp, err := client.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP error: %s", resp.Status)
	}

	var res types.Resp
	if err := jsonimpl.UnmarshalReader(resp.Body, &res); err != nil {
		return nil, err
	}

	if !res.Ok {
		return nil, errors.New("sandwich API returned not ok")
	}

	if res.Data == nil {
		return nil, errors.New("no data in response")
	}

	shards := make(map[int64]types.ShardConn)

	for _, manager := range res.Data.Managers {
		if manager.DisplayName != "Anti Raid" {
			continue
		}

		for _, v := range manager.ShardGroups {
			for _, shard := range v.Shards {
				shardID := shard[0]
				status := shard[1]
				latency := shard[2]
				guilds := shard[3]
				uptime := shard[4]
				totalUptime := shard[5]

				statusStr := "Unknown"
				switch status {
				case 0:
					statusStr = "Idle"
				case 1:
					statusStr = "Connecting"
				case 2:
					statusStr = "Connected"
				case 3:
					statusStr = "MarkedForClosure"
				case 4:
					statusStr = "Closing"
				case 5:
					statusStr = "Closed"
				case 6:
					statusStr = "Erroring"
				}

				shards[shardID] = types.ShardConn{
					Status:      statusStr,
					RealLatency: latency,
					Guilds:      guilds,
					Uptime:      uptime,
					TotalUptime: totalUptime,
				}
			}
		}
	}

	return &types.GetStatusResponse{
		Resp:       *res.Data,
		ShardConns: shards,
	}, nil
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	if SandwichResponse == nil || time.Now().Unix()-LastCacheUpdate > 60 {
		resp, err := getStatus(state.IpcClientHttp11)
		if err != nil {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json: types.ApiError{
					Message: fmt.Sprintf("failed to get status: %s", err),
				},
			}
		}

		SandwichResponse = resp
		LastCacheUpdate = time.Now().Unix()
	}

	return uapi.HttpResponse{
		Json: SandwichResponse,
	}
}
