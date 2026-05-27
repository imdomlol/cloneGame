"""Phase 2: vault → vector index → retrieval → codegen → game/ source.

All modules are implemented (see DEPLOYMENT_GUIDE.md §3):

- `indexer.py`    -- Repomix XML → Chroma collections + `build/graph.json`.
- `retrieval.py`  -- vector seeds + 1-hop graph expansion, 2k-token cap (Layer 3).
- `baseline.py`   -- renders the per-engine `build/engine_baseline.md` (Layer 1).
- `system_map.py` -- rolling `build/system_map.yaml` ledger + 1k-token projection (Layer 4).
- `codegen.py`    -- assembles the 4-layer prompt and calls a Claude backend.
- `driver.py`     -- single-shot: one task → one validated file + system_map update.
- `loop_driver.py`-- runs N goals, merges a multi-file tree into `game/`, gates each
                     turn on `cargo build` with revert-on-failure.
"""
