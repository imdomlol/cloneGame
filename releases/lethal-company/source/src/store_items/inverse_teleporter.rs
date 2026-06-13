// Sources: vault/store_items/inverse_teleporter.md

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{GameSeed, SimChecksumState, SimHz, SimTick, tick_rng};

pub const BUY_COST: u32 = 425;
pub const COOLDOWN_SECONDS: u32 = 210;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.004");

const SALT_DEST: u64 = 0x494e_5645_5253_4544;

/// Fired by the purchase layer when the inverse teleporter is bought.
#[derive(Event)]
pub struct InverseTeleporterPurchasedEvent;

/// Fired by the ship/input layer when the button is pressed.
/// `employee_ids` are replicated game-entity ids of every employee on the pad.
#[derive(Event)]
pub struct InverseTeleporterActivateEvent {
    pub employee_ids: Vec<u64>,
}

/// Fired once per teleported employee; `destination_roll` is the deterministic
/// facility-destination selector consumed by the facility system.
#[derive(Event)]
pub struct InverseTeleporterTeleportedEvent {
    pub employee_id: u64,
    pub destination_roll: u64,
}

#[derive(Resource, Default)]
pub struct InverseTeleporterState {
    pub owned: bool,
    pub cooldown_ticks_remaining: u32,
}

pub struct InverseTeleporterPlugin;

impl Plugin for InverseTeleporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InverseTeleporterPurchasedEvent>()
            .add_event::<InverseTeleporterActivateEvent>()
            .add_event::<InverseTeleporterTeleportedEvent>()
            .init_resource::<InverseTeleporterState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    tick_cooldown,
                    handle_activate,
                    inverse_teleporter_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<InverseTeleporterPurchasedEvent>,
    mut state: ResMut<InverseTeleporterState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn tick_cooldown(mut state: ResMut<InverseTeleporterState>) {
    if state.cooldown_ticks_remaining > 0 {
        state.cooldown_ticks_remaining -= 1;
    }
}

fn handle_activate(
    mut activations: EventReader<InverseTeleporterActivateEvent>,
    mut state: ResMut<InverseTeleporterState>,
    mut teleported: EventWriter<InverseTeleporterTeleportedEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
) {
    for event in activations.read() {
        if !state.owned || state.cooldown_ticks_remaining > 0 {
            continue;
        }
        for &employee_id in &event.employee_ids {
            let mut rng = tick_rng(seed.0, tick.0, SALT_DEST ^ employee_id);
            let destination_roll = rng.next_u64();
            teleported.send(InverseTeleporterTeleportedEvent {
                employee_id,
                destination_roll,
            });
        }
        state.cooldown_ticks_remaining = COOLDOWN_SECONDS * sim_hz.0.to_num::<u32>();
    }
}

fn inverse_teleporter_checksum(
    state: Res<InverseTeleporterState>,
    mut cs: ResMut<SimChecksumState>,
) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.cooldown_ticks_remaining as u64);
}