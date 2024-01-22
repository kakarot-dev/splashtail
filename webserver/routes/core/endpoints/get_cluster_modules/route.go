package get_cluster_modules

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"strconv"
	"time"

	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/types/silverpelt"
	"github.com/anti-raid/splashtail/utils/rwmap"
	"github.com/go-chi/chi/v5"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

var reqBody = map[string]any{
	"Modules": new(map[string]string),
}

var reqBodyBytes []byte

var cmCache = rwmap.New[int, silverpelt.CanonicalModule]()

func Setup() {
	var err error
	reqBodyBytes, err = json.Marshal(reqBody)

	if err != nil {
		panic(err)
	}
}

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Cluster Modules",
		Description: "This endpoint returns the modules that are currently running on the cluster.",
		Resp:        silverpelt.CanonicalModule{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	if state.MewldInstanceList == nil {
		return uapi.HttpResponse{
			Status: http.StatusPreconditionFailed,
			Json: types.ApiError{
				Message: "Mewld instance list not exposed yet. Please try again in 5 seconds!",
			},
			Headers: map[string]string{
				"Retry-After": "5",
			},
		}
	}

	clusterIdStr := chi.URLParam(r, "clusterId")

	clusterId, err := strconv.Atoi(clusterIdStr)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid cluster ID",
			},
		}
	}

	if cm, ok := cmCache.Get(clusterId); ok {
		return uapi.HttpResponse{
			Json: cm,
		}
	}

	client := http.Client{
		Timeout: 10 * time.Second,
	}

	port := state.Config.Meta.BotIServerBasePort.Parse() + clusterId

	if port > 65535 {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid cluster ID [port > 65535]",
			},
		}
	}

	req, err := http.NewRequestWithContext(d.Context, "POST", "http://localhost:"+strconv.Itoa(port), bytes.NewReader(reqBodyBytes))

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to create request: " + err.Error(),
			},
		}
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to send request: " + err.Error(),
			},
		}
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to read response body: " + err.Error(),
			},
		}
	}

	if resp.StatusCode != http.StatusOK {
		return uapi.HttpResponse{
			Status: resp.StatusCode,
			Json: types.ApiError{
				Message: "Failed to get modules: " + string(body),
			},
		}
	}

	var cm silverpelt.CanonicalModule

	err = json.Unmarshal(body, &cm)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to parse response body: " + err.Error(),
			},
		}
	}

	cmCache.Set(clusterId, cm)

	return uapi.HttpResponse{
		Json: cm,
	}
}
