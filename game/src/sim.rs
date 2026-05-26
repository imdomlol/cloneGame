use bevy::prelude::*;
use fixed::types::I32F32;
use rand_chacha::ChaCha8Rng;
use rand_core::SeedableRng;

/// SplitMix64 finalizer: cheap, portable 64-bit avalanche mix.
const fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Canonical seeded RNG for the deterministic sim path.
///
/// Every random draw in `FixedUpdate` sim code MUST obtain its generator from
/// here — never `thread_rng()` or an ad-hoc `seed_from_u64`. The seed is a
/// deterministic mix of three values every client agrees on:
///
/// - `game_seed`: shared at match start.
/// - `tick`: the lockstep-synchronized tick number.
/// - `salt`: a stable per-call-site constant (and, for per-entity draws, a
///   replicated game entity id folded in). The salt decorrelates streams so
///   two draw sites on the same tick do not produce identical sequences;
///   without it, every system would roll the same numbers each tick.
///
/// Because the inputs and the mix are identical across clients, the stream is
/// bit-identical everywhere — the requirement for lockstep. Do NOT salt with
/// Bevy's raw `Entity` bits (index/generation are an engine allocation detail
/// that can differ across clients); use a game-assigned stable id.
pub fn tick_rng(game_seed: u64, tick: u64, salt: u64) -> ChaCha8Rng {
    let seed = mix64(game_seed ^ mix64(tick ^ mix64(salt)));
    ChaCha8Rng::seed_from_u64(seed)
}

/// Current simulation tick counter. Incremented each FixedUpdate step.
#[derive(Resource, Default, Clone, Copy)]
pub struct SimTick(pub u64);

/// Per-game seed for deterministic RNG.
#[derive(Resource, Clone, Copy)]
pub struct GameSeed(pub u64);

/// Fixed-update rate in Hz (20–30). Used to convert per-second stats to ticks.
#[derive(Resource, Clone, Copy)]
pub struct SimHz(pub I32F32);

impl Default for SimHz {
    fn default() -> Self {
        Self(I32F32::lit("25"))
    }
}

/// Running checksum of canonical sim state, for lockstep desync detection.
///
/// Each tick, every unit's `*_checksum` system folds its determinism-relevant
/// component bits in via [`SimChecksumState::accumulate`]. Compare this hash
/// across clients; the first divergence is a desync. `accumulate` is
/// commutative, so it does not matter which unit's checksum system runs first —
/// the result is identical across clients as long as the per-entity state is.
///
/// v1 is a running cumulative hash. Per-tick isolation (reset + per-tick
/// history) and the network broadcast/compare land with the netcode; units must
/// not implement any of that themselves.
#[derive(Resource, Default, Clone, Copy)]
pub struct SimChecksumState {
    pub hash: u64,
}

impl SimChecksumState {
    /// Fold one value into the checksum. Commutative (`wrapping_add` of a mixed
    /// value): system and entity iteration order do not affect the result, so a
    /// non-deterministic Bevy schedule order cannot cause a false desync.
    pub fn accumulate(&mut self, value: u64) {
        self.hash = self.hash.wrapping_add(mix64(value));
    }
}

/// Registers the shared sim-checksum resource. Add once at app construction;
/// unit plugins then fold their state into [`SimChecksumState`] each tick.
pub struct SimChecksumPlugin;

impl Plugin for SimChecksumPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimChecksumState>();
    }
}

/// World-space position in the fixed-tick sim. Fixed-point for cross-client determinism.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimPosition {
    pub x: I32F32,
    pub y: I32F32,
}

/// Canonical damage types resolved by per-unit armor systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DamageType {
    Standard,
    Fire,
    /// Venom damage, e.g. from infected_venom.
    Venom,
}

/// Incoming damage to any sim entity; each faction handles its own targets.
#[derive(Event, Debug, Clone, Copy)]
pub struct IncomingDamageEvent {
    pub target: Entity,
    pub raw_amount: I32F32,
    pub damage_type: DamageType,
    pub source: Entity,
}

/// Emitted by a damage-receive system when an entity's HP reaches zero.
#[derive(Event, Debug, Clone, Copy)]
pub struct EntityKilledEvent {
    pub entity: Entity,
    pub killer: Entity,
    /// Experience points the killed entity grants.
    pub exp_reward: I32F32,
    /// Difficulty tier of the killed entity (0 = basic … 3 = elite).
    pub difficulty_tier: u8,
}

/// Emitted when a unit fires, propagating noise to the tile noise system.
#[derive(Event, Debug, Clone, Copy)]
pub struct NoiseEmittedEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub amount: I32F32,
}

/// Shared hit-point state for any combatant. Reuse this instead of defining a
/// per-unit health component, so damage / heal / regen logic and the checksum
/// treat every unit's HP identically. Fixed-point for cross-client determinism.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub current: I32F32,
    pub max: I32F32,
}

impl Health {
    /// Construct at full health (`current == max`).
    pub fn full(max: I32F32) -> Self {
        Self { current: max, max }
    }
}

/// Shared baseline combat stats common to every unit. Translate the unit's
/// frontmatter stats into this; keep per-unit extras (veteran promotion,
/// carried barrels, burn DoT, campaign upgrades) as their own components on top.
/// Fixed-point for cross-client determinism.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitStats {
    pub move_speed: I32F32,
    pub attack_range: I32F32,
    pub attack_damage: I32F32,
    pub attack_speed: I32F32,
    pub watch_range: I32F32,
}
