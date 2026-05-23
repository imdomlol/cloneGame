1. **Fixed-point math for sim state.** Positions, velocities, damage, RNG seeds (anything that affects the next tick) must use the `fixed` / `sfixed` crates, not `f32` / `f64`. Floats are fine for render-only state.
2. **No transcendentals in the sim path.** `sin`, `cos`, `sqrt`, `atan2` on floats vary across CPUs and compiler versions. Use fixed-point approximations or precomputed lookup tables.
3. **Seeded RNG everywhere.** Replace `rand::thread_rng()` with a per-tick `ChaCha8Rng` seeded from `(game_seed, tick_number)`. Every random draw in the sim path uses it.
4. **Deterministic system order.** Bevy's scheduler parallelizes by default. Force order on sim systems with `.chain()` or explicit `before()` / `after()` constraints.
5. **No HashMap iteration in sim code.** Rust's default `HashMap` randomizes its hash seed per-process. Use `BTreeMap`, sorted `Vec`, or `IndexMap` with a fixed `BuildHasher`.
6. **Sim/render split.** Sim runs in `FixedUpdate` at 20-30Hz under the above rules. Render runs in `Update` at vsync rate and interpolates between the last two sim states.
7. **Desync detection.** Every sim tick computes a checksum of canonical state (entity transforms + key game state). Broadcast the hash periodically; the first cross-client mismatch logs the offending tick and the diverging entity set.
