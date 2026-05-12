# Known Issues - TetraMem-XL v12.0

## Fixed in Latest

### ~~MCP / API Memory Store Isolation~~ (Fixed in current)
- **Was**: Standalone MCP kept a long-lived `TetraMemCore.memories` store that could diverge from HTTP `AppState.memory_store`.
- **Now**: `McpServer` owns a `SharedState` and each tool call reads/writes the same `AppState` memory store, semantic index, clustering index, Hebbian graph, universe, and crystal engine.
- **Note**: Running separate MCP and HTTP processes against the same persistence file still requires process-level coordination; use `mcp-proxy` when MCP should talk to a live HTTP server.

### ~~Restored Memories Missing Derived Indexes~~ (Fixed in current)
- **Was**: Persisted memories were restored into `MemoryStore`, but semantic and clustering indexes started empty until manual re-indexing or new writes.
- **Now**: HTTP startup and MCP persisted-state construction rebuild semantic and clustering indexes from decoded memory payloads.

### ~~7D Coordinate Support~~ (Fixed in 52fde1cd)
- **Was**: All endpoints only accepted `[i32; 3]` anchors, dark dimensions hardcoded to `[0,0,0,0]`.
- **Now**: Accept `[x,y,z]` (backward compatible) or `[x,y,z,d4,d5,d6,d7]` (full 7D).

### ~~Tag Search Missing from API~~ (Fixed in 52fde1cd)
- **Was**: `SemanticQuery` had tag filtering but no REST endpoint exposed it.
- **Now**: `/memory/recall` accepts `tags` and `tag_mode` (any/all) parameters.

### ~~Weight Adjustment API~~ (Fixed in 52fde1cd)
- **Was**: `HebbianMemory::boost_edge()` existed but no API endpoint.
- **Now**: `POST /memory/adjust_weight` with from/to anchors and boost (positive or negative).

## Monitoring

### AvgWeight Unbounded Growth Under Deferred Binding
- **Severity**: Low (cosmetic, no conservation violation)
- **Observed**: AvgWeight grows from 1.37 to 9.34 over 20h with `deferred_binding=true` and no new input. Hebbian reinforcement in dream/clustering cycles continuously strengthens existing edges without new competition.
- **Impact**: Weight growth alone does not indicate a bug; conservation is maintained. But extreme weight concentration may reduce the system's ability to incorporate new memories later.
- **Status**: Monitoring. May need a weight normalization or decay mechanism if growth continues unbounded.
