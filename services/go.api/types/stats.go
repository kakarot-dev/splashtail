package types

type ShardConn struct {
	Status      string `json:"status"`
	RealLatency int64  `json:"real_latency"`
	Guilds      int64  `json:"guilds"`
	Uptime      int64  `json:"uptime"`
	TotalUptime int64  `json:"total_uptime"`
}

type GetStatusResponse struct {
	Resp        StatusEndpointResponse `json:"resp"`
	ShardConns  map[int64]ShardConn    `json:"shard_conns"`
	TotalGuilds int64                  `json:"total_guilds"`
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
