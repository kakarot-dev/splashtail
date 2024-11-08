package types

type ShardConn struct {
	Status      string
	RealLatency int64
	Guilds      int64
	Uptime      int64
	TotalUptime int64
}

type GetStatusResponse struct {
	Resp        StatusEndpointResponse
	ShardConns  map[int64]ShardConn
	TotalGuilds int64
}

type StatusEndpointResponse struct {
	Uptime   int64                   `json:"uptime"`
	Managers []StatusEndpointManager `json:"managers"`
}

type StatusEndpointManager struct {
	DisplayName string       `json:"display_name"`
	ShardGroups []ShardGroup `json:"shard_groups"`
}

type ShardGroup struct {
	Shards [][]int64 `json:"shards"`
}

type Resp struct {
	Ok   bool                    `json:"ok"`
	Data *StatusEndpointResponse `json:"data"`
}
