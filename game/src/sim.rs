use bevy::prelude::*;
use fixed::types::I32F32;

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
