// Sources: vault/store_items/jetpack.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimHz};

pub const BUY_COST: u32 = 900;
pub const WEIGHT_LBS: I32F32 = I32F32::lit("52.5");
pub const BATTERY_LIFE_SECS: I32F32 = I32F32::lit("50");
pub const CONDUCTIVE: bool = true;

#[derive(Event)]
pub struct JetpackPurchasedEvent;

#[derive(Event)]
pub struct JetpackActivatedEvent;

#[derive(Event)]
pub struct JetpackDeactivatedEvent;

#[derive(Event)]
pub struct JetpackBatteryDepletedEvent;

#[derive(Event)]
pub struct JetpackExplosionRollEvent {
    pub cause: JetpackExplosionCause,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JetpackExplosionCause {
    HighSpeedCollision,
    ExtendedUse,
    FlownTooHigh,
}

#[derive(Event)]
pub struct JetpackSuppressedEvent;

#[derive(Event)]
pub struct JetpackSuppressionLiftedEvent;

#[derive(Resource)]
pub struct JetpackState {
    pub owned: bool,
    pub equipped: bool,
    pub active: bool,
    pub suppressed: bool,
    pub battery_remaining: I32F32,
    pub active_ticks: u32,
}

impl Default for JetpackState {
    fn default() -> Self {
        Self {
            owned: false,
            equipped: false,
            active: false,
            suppressed: false,
            battery_remaining: BATTERY_LIFE_SECS,
            active_ticks: 0,
        }
    }
}

pub struct JetpackPlugin;

impl Plugin for JetpackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JetpackPurchasedEvent>()
            .add_event::<JetpackActivatedEvent>()
            .add_event::<JetpackDeactivatedEvent>()
            .add_event::<JetpackBatteryDepletedEvent>()
            .add_event::<JetpackExplosionRollEvent>()
            .add_event::<JetpackSuppressedEvent>()
            .add_event::<JetpackSuppressionLiftedEvent>()
            .init_resource::<JetpackState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_activation,
                    handle_suppression,
                    tick_battery,
                    jetpack_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<JetpackPurchasedEvent>,
    mut state: ResMut<JetpackState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_activation(
    mut activated: EventReader<JetpackActivatedEvent>,
    mut deactivated: EventReader<JetpackDeactivatedEvent>,
    mut state: ResMut<JetpackState>,
) {
    for _ in activated.read() {
        if state.owned
            && state.equipped
            && !state.suppressed
            && state.battery_remaining > I32F32::ZERO
        {
            state.active = true;
        }
    }
    for _ in deactivated.read() {
        if state.active {
            state.active = false;
            state.active_ticks = 0;
        }
    }
}

fn handle_suppression(
    mut suppressed_events: EventReader<JetpackSuppressedEvent>,
    mut lifted_events: EventReader<JetpackSuppressionLiftedEvent>,
    mut state: ResMut<JetpackState>,
) {
    for _ in suppressed_events.read() {
        state.suppressed = true;
        state.active = false;
        state.active_ticks = 0;
    }
    for _ in lifted_events.read() {
        state.suppressed = false;
    }
}

fn tick_battery(
    mut state: ResMut<JetpackState>,
    sim_hz: Res<SimHz>,
    mut depleted: EventWriter<JetpackBatteryDepletedEvent>,
    mut explosion_roll: EventWriter<JetpackExplosionRollEvent>,
) {
    if !state.active {
        return;
    }

    let drain = I32F32::ONE / I32F32::from_num(sim_hz.0);
    state.battery_remaining -= drain;
    state.active_ticks = state.active_ticks.saturating_add(1);

    if state.battery_remaining <= I32F32::ZERO {
        state.battery_remaining = I32F32::ZERO;
        state.active = false;
        state.active_ticks = 0;
        depleted.send(JetpackBatteryDepletedEvent);
        return;
    }

    // Emit once when continuous flight exceeds half the total battery life
    let hz_u32 = sim_hz.0.to_num::<u32>();
    let extended_threshold = hz_u32.saturating_mul(BATTERY_LIFE_SECS.to_num::<u32>() / 2);
    if state.active_ticks == extended_threshold {
        explosion_roll.send(JetpackExplosionRollEvent {
            cause: JetpackExplosionCause::ExtendedUse,
        });
    }
}

fn jetpack_checksum(state: Res<JetpackState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.equipped as u64);
    cs.accumulate(state.active as u64);
    cs.accumulate(state.suppressed as u64);
    cs.accumulate(state.battery_remaining.to_bits() as u64);
    cs.accumulate(state.active_ticks as u64);
}