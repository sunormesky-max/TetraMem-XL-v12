# Known Issues — TetraMem-XL v12.0

## Fixed in Latest

### ~~7D Coordinate Support~~ (Fixed in 52fde1cd)
- **Was**: All endpoints only accepted `[i32; 3]` anchors, dark dimensions hardcoded to `[0,0,0,0]`
- **Now**: Accept `[x,y,z]` (backward compatible) or `[x,y,z,d4,d5,d6,d7]` (full 7D)

### ~~Tag Search Missing from API~~ (Fixed in 52fde1cd)
- **Was**: `SemanticQuery` had tag filtering but no REST endpoint exposed it
- **Now**: `/memory/recall` accepts `tags` and `tag_mode` (any/all) parameters

### ~~Weight Adjustment API~~ (Fixed in 52fde1cd)
- **Was**: `HebbianMemory::boost_edge()` existed but no API endpoint
- **Now**: `POST /memory/adjust_weight` with from/to anchors and boost (positive or negative)

## Architecture

### MCP / API Memory Store Isolation
- **Severity**: Medium (no impact on HTTP-only deployments)
- **Discovered**: User report (OpenClaw integration testing)
- **Symptom**: Memories written via MCP mode (`TetraMemCore.memories`) are not visible via HTTP API (`AppState.memory_store`), and vice versa.
- **Root Cause**: `TetraMemCore` (used by MCP) maintains its own independent `Vec<MemoryAtom>`, while the HTTP server uses `AppState.memory_store`. These two stores are never synchronized.
- **Workaround**: Use only `serve` mode (HTTP API). All OpenClaw and Crystal Agent integrations work correctly via HTTP because they read/write the same `AppState`.
- **Proposed Fix**: Unify storage at the `main.rs` layer — have `TetraMemCore` hold `Arc<SharedState>` instead of maintaining a separate `Vec<MemoryAtom>`.

## Monitoring

### AvgWeight Unbounded Growth Under Deferred Binding
- **Severity**: Low (cosmetic, no conservation violation)
- **Observed**: AvgWeight grows from 1.37 to 9.34 over 20h with `deferred_binding=true` and no new input. Hebbian reinforcement in dream/clustering cycles continuously strengthens existing edges without new competition.
- **Impact**: Weight growth alone does not indicate a bug — conservation is maintained. But extreme weight concentration may reduce the system's ability to incorporate new memories later.
- **Status**: Monitoring. May need a weight normalization or decay mechanism if growth continues unbounded.
