// Sources: vault/store_items/boombox.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimHz};

pub const BUY_COST: u32 = 60;
pub const SELL_VALUE: u32 = 0;
pub const WEIGHT_LB: u32 = 16;
pub const BATTERY_LIFE_SECS: I32F32 = I32F32::lit("340");
pub const PITCH_DOWN_THRESHOLD: I32F32 = I32F32::lit("0.05");
pub const SONG_COUNT: u8 = 5;

#[derive(Event)]
pub struct BoomboxPurchasedEvent;

#[derive(Event)]
pub struct BoomboxActivatedEvent {
    pub active: bool,
}

#[derive(Resource)]
pub struct BoomboxState {
    pub owned: bool,
    pub active: bool,
    /// Normalized battery charge, fixed-point 0.0..=1.0.
    pub battery: I32F32,
    pub song_index: u8,
    /// True once battery drops to PITCH_DOWN_THRESHOLD; persists after recharge.
    pub pitched_down: bool,
    pub ticks_on_current_song: u32,
}

impl Default for BoomboxState {
    fn default() -> Self {
        Self {
            owned: false,
            active: false,
            battery: I32F32::ONE,
            song_index: 0,
            pitched_down: false,
            ticks_on_current_song: 0,
        }
    }
}

pub struct BoomboxPlugin;

impl Plugin for BoomboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BoomboxPurchasedEvent>()
            .add_event::<BoomboxActivatedEvent>()
            .init_resource::<BoomboxState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_activation,
                    tick_battery,
                    tick_songs,
                    boombox_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<BoomboxPurchasedEvent>,
    mut state: ResMut<BoomboxState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_activation(
    mut events: EventReader<BoomboxActivatedEvent>,
    mut state: ResMut<BoomboxState>,
) {
    for ev in events.read() {
        if state.owned && state.battery > I32F32::ZERO {
            state.active = ev.active;
        }
    }
}

fn tick_battery(mut state: ResMut<BoomboxState>, sim_hz: Res<SimHz>) {
    if !state.active {
        return;
    }
    let hz = I32F32::from_num(sim_hz.0);
    let drain_per_tick = I32F32::ONE / (BATTERY_LIFE_SECS * hz);
    state.battery = (state.battery - drain_per_tick).max(I32F32::ZERO);
    if state.battery <= PITCH_DOWN_THRESHOLD {
        // Pitched-down state persists even after an electric_coil recharge.
        // TODO: listen for ElectricCoilChargedEvent to restore battery without
        //       clearing pitched_down (import path unverified).
        state.pitched_down = true;
    }
    if state.battery == I32F32::ZERO {
        state.active = false;
    }
}

fn tick_songs(mut state: ResMut<BoomboxState>, sim_hz: Res<SimHz>) {
    if !state.active {
        return;
    }
    // Each of the 5 songs fills an equal share of total battery life.
    let hz = I32F32::from_num(sim_hz.0);
    let secs_per_song = BATTERY_LIFE_SECS / I32F32::from_num(SONG_COUNT);
    let ticks_per_song = (secs_per_song * hz).to_num::<u32>().max(1);
    state.ticks_on_current_song += 1;
    if state.ticks_on_current_song >= ticks_per_song {
        state.ticks_on_current_song = 0;
        state.song_index = (state.song_index + 1) % SONG_COUNT;
    }
    // TODO: emit NoiseEmittedEvent and HygrodereAgitatedEvent each active tick.
    // Both types are in external modules whose import paths are unverified;
    // the events are already registered so EventWriters can be added once
    // paths are confirmed.
}

fn boombox_checksum(state: Res<BoomboxState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
    cs.accumulate(state.battery.to_bits() as u64);
    cs.accumulate(state.song_index as u64);
    cs.accumulate(state.pitched_down as u64);
    cs.accumulate(state.ticks_on_current_song as u64);
}