// Binds onto eureka uapi
package api

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/constants"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"

	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5/pgtype"
	"golang.org/x/exp/slices"
)

const SESSION_EXPIRY = 60 * 30 // 30 minutes

type DefaultResponder struct{}

func (d DefaultResponder) New(err string, ctx map[string]string) any {
	return types.ApiError{
		Message: err,
		Context: ctx,
	}
}

// Authorizes a request
func Authorize(r uapi.Route, req *http.Request) (uapi.AuthData, uapi.HttpResponse, bool) {
	if len(r.ExtData) > 0 {
		// Immediate Oauth system (ioauth)
		if a, ok := r.ExtData["ioauth"]; ok {
			scopes, ok := a.([]string)

			if !ok {
				return uapi.AuthData{}, uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Internal server error: ioauth is not a string of scopes"},
				}, false
			}

			if len(scopes) == 0 {
				return uapi.AuthData{}, uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Internal server error: No scopes provided for ioauth"},
				}, false
			}

			if !slices.Contains(scopes, "identify") {
				return uapi.AuthData{}, uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Invalid scopes. Expected identify scope to be in " + strings.Join(scopes, ", ")},
				}, false
			}

			// Check if the user is authorized
			ioauthToken := req.URL.Query().Get("ioauth")

			if ioauthToken == "" {
				ioAuth := types.IOAuthRedirect{
					Dest:   req.URL.String(),
					Scopes: scopes,
				}

				bytes, err := json.Marshal(ioAuth)

				if err != nil {
					return uapi.AuthData{}, uapi.HttpResponse{
						Status: http.StatusInternalServerError,
						Json:   types.ApiError{Message: "Internal server error: Failed to marshal ioauth"},
					}, false
				}

				// Redirect to discord oauth
				return uapi.AuthData{}, uapi.HttpResponse{
					Status:   http.StatusTemporaryRedirect,
					Redirect: "/ioauth/login?rd=" + base64.RawURLEncoding.EncodeToString(bytes),
				}, false
			}
		}
	}

	authHeader := req.Header.Get("Authorization")

	if len(r.Auth) > 0 && authHeader == "" && !r.AuthOptional {
		return uapi.AuthData{}, uapi.DefaultResponse(http.StatusUnauthorized), false
	}

	authData := uapi.AuthData{}

	for _, auth := range r.Auth {
		// There are two cases, one with a URLVar (such as /bots/stats) and one without

		if authData.Authorized {
			break
		}

		if authHeader == "" {
			continue
		}

		var urlIds []string

		switch auth.Type {
		case types.TargetTypeUser:
			// Delete old/expiring auths first
			_, err := state.Pool.Exec(state.Context, "DELETE FROM web_api_tokens WHERE expiry < NOW()")

			if err != nil {
				state.Logger.Error("Failed to delete expired web API tokens [db delete]", zap.Error(err))
			}

			// Check if the user exists with said API token
			var id pgtype.Text
			var sessId string
			var permLimits []string

			err = state.Pool.QueryRow(state.Context, "SELECT id, user_id, perm_limits FROM web_api_tokens WHERE token = $1", strings.Replace(authHeader, "User ", "", 1)).Scan(&sessId, &id, &permLimits)

			if err != nil {
				state.Logger.Error("Failed to get user ID from web API token", zap.Error(err))
				continue
			}

			if !id.Valid {
				continue
			}

			// Check if the user is banned
			var userstate string

			err = state.Pool.QueryRow(state.Context, "SELECT state FROM users WHERE user_id = $1", id).Scan(&userstate)

			if err != nil {
				state.Logger.Error("Failed to get user state", zap.Error(err))
				continue
			}

			if !id.Valid {
				state.Logger.Error("User ID is not valid")
				continue
			}

			authData = uapi.AuthData{
				TargetType: types.TargetTypeUser,
				ID:         id.String,
				Authorized: true,
				Banned:     userstate == "banned" || userstate == "api_banned",
				Data: map[string]any{
					"session_id":  sessId,
					"perm_limits": permLimits,
				},
			}
			urlIds = []string{id.String}
		}

		// Now handle the URLVar
		if auth.URLVar != "" {
			state.Logger.Info("Checking URL variable against user ID from auth token", zap.String("URLVar", auth.URLVar))
			gotUserId := chi.URLParam(req, auth.URLVar)
			if !slices.Contains(urlIds, gotUserId) {
				authData = uapi.AuthData{} // Remove auth data
			}
		}

		// Banned users cannot use the API at all otherwise if not explicitly scoped to "ban_exempt"
		if authData.Banned && auth.AllowedScope != "ban_exempt" {
			return uapi.AuthData{}, uapi.HttpResponse{
				Status: http.StatusForbidden,
				Json:   types.ApiError{Message: "You are banned from Anti-Raid. If you think this is a mistake, please contact support."},
			}, false
		}
	}

	if len(r.Auth) > 0 && !authData.Authorized && !r.AuthOptional {
		return uapi.AuthData{}, uapi.DefaultResponse(http.StatusUnauthorized), false
	}

	return authData, uapi.HttpResponse{}, true
}

func Setup() {
	uapi.SetupState(uapi.UAPIState{
		Logger:    state.Logger,
		Authorize: Authorize,
		AuthTypeMap: map[string]string{
			types.TargetTypeUser:   types.TargetTypeUser,
			types.TargetTypeServer: types.TargetTypeServer,
		},
		Context: state.Context,
		Constants: &uapi.UAPIConstants{
			ResourceNotFound:    constants.ResourceNotFound,
			BadRequest:          constants.BadRequest,
			Forbidden:           constants.Forbidden,
			Unauthorized:        constants.Unauthorized,
			InternalServerError: constants.InternalServerError,
			MethodNotAllowed:    constants.MethodNotAllowed,
			BodyRequired:        constants.BodyRequired,
		},
		DefaultResponder: DefaultResponder{},
	})
}
