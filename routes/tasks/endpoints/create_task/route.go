package create_task

import (
	"encoding/json"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/ipcack"
	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/types"

	mredis "github.com/cheesycod/mewld/redis"
	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/crypto"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Task",
		Description: "Gets a task. Returns the task data if this is successful",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "User/Server ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "name",
				Description: "The name of the task",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "wait_for_execute_confirm",
				Description: "Whether or not to wait for the task to be confirmed by the job server",
				Required:    false,
				In:          "query",
				Schema:      docs.BoolSchema,
			},
		},
		Req:  "The tasks fields",
		Resp: types.TaskCreateResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      1 * time.Hour,
		MaxRequests: 50,
		Bucket:      "create_task",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "create_task"))
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

	taskName := chi.URLParam(r, "name")

	if taskName == "" {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Missing name",
			},
			Status: http.StatusBadRequest,
		}
	}

	baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Unknown task name",
			},
			Status: http.StatusBadRequest,
		}
	}

	task := baseTaskDef // Copy task

	err = json.NewDecoder(r.Body).Decode(&task)

	if err != nil {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Error decoding task: " + err.Error(),
			},
			Status: http.StatusBadRequest,
		}
	}

	tInfo := task.Info()

	// Access Control check
	if tInfo.TaskFor != nil {
		if tInfo.TaskFor.ID == "" || tInfo.TaskFor.TargetType == "" {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Invalid task.TaskFor. Missing ID or TargetType"},
			}
		}

		if tInfo.TaskFor.TargetType != d.Auth.TargetType {
			return uapi.HttpResponse{
				Status: http.StatusForbidden,
				Json:   types.ApiError{Message: "This task has a for of " + tInfo.TaskFor.TargetType + " but you are authenticated as a" + d.Auth.TargetType + "!"},
			}
		}

		if tInfo.TaskFor.ID != d.Auth.ID {
			return uapi.HttpResponse{
				Status: http.StatusForbidden,
				Json:   types.ApiError{Message: "You are not authorized to fetch this task!"},
			}
		}
	}

	tcr, err := tasks.CreateTask(state.Context, task)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Error creating task: " + err.Error()},
		}
	}

	var cmdId = crypto.RandString(32)
	var cmd = mredis.LauncherCmd{
		Scope:     "splashtail-web",
		Action:    state.Config.Meta.WebRedisChannel,
		CommandId: cmdId,
		Args: map[string]any{
			"task_id": tcr.TaskID,
			"name":    taskName,
		},
		Output: task,
	}

	bytes, err := json.Marshal(cmd)

	if err != nil {
		state.Logger.Error("Error marshalling IPC command", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Error marshalling IPC command: " + err.Error()},
		}
	}

	// Use execute_task IPC
	if r.URL.Query().Get("wait_for_execute_confirm") == "true" {
		acker := ipcack.Ack{
			Chan: make(chan *mredis.LauncherCmd),
		}

		var ackMsg *mredis.LauncherCmd
		var timeout bool

		go func() {
			timer := time.NewTimer(10 * time.Second)

			select {
			case <-timer.C:
				timeout = true
				timer.Stop()
				ipcack.AckQueue.Delete(cmdId)
				return
			case a := <-acker.Chan:
				ackMsg = a
				timer.Stop()
				ipcack.AckQueue.Delete(cmdId)
				return
			}
		}()

		ipcack.AckQueue.Store(cmdId, &acker)

		err = state.Redis.Publish(state.Context, state.Config.Meta.WebRedisChannel, bytes).Err()

		if err != nil {
			state.Logger.Error("Error publishing IPC command", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Error publishing IPC command: " + err.Error()},
			}
		}

		for {
			if timeout || ackMsg != nil {
				break
			}

			time.Sleep(10 * time.Millisecond)
		}

		if ackMsg == nil {
			if timeout {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Error waiting for IPC confirmation: Timeout"},
				}
			} else if err != nil {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Error waiting for IPC confirmation: " + err.Error()},
				}
			} else {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Error waiting for IPC confirmation: Unknown error"},
				}
			}
		}

		return uapi.HttpResponse{
			Json: types.TaskCreateResponseWithWait{
				TaskCreateResponse: tcr,
				Output:             ackMsg.Output,
			},
		}
	} else {
		err = state.Redis.Publish(state.Context, state.Config.Meta.WebRedisChannel, bytes).Err()

		if err != nil {
			state.Logger.Error("Error publishing IPC command", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Error publishing IPC command: " + err.Error()},
			}
		}
	}

	return uapi.HttpResponse{
		Json: tcr,
	}
}
