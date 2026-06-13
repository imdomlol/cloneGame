// Sources: vault/store_items/romantic_table.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 120;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.005");

#[derive(Event)]
pub struct RomanticTablePurchasedEvent;

#[derive(Event)]
pub struct RomanticTableCandelabraInteractedEvent;

#[derive(Event)]
pub struct RomanticTableDimLightEmittedEvent;

#[derive(Event)]
pub struct RomanticTableQuietFireNoiseEvent;

#[derive(Event)]
pub struct RomanticTableCandlesStoodOnEvent;

#[derive(Resource, Default)]
pub struct RomanticTableState {
    pub owned: bool,
    pub candles_lit: bool,
    pub dim_light_emitting: bool,
    pub quiet_fire_noise_emitting: bool,
    pub outside_ghost_girl_kill_range: bool,
}

pub struct RomanticTablePlugin;

impl Plugin for RomanticTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RomanticTablePurchasedEvent>()
            .add_event::<RomanticTableCandelabraInteractedEvent>()
            .add_event::<RomanticTableDimLightEmittedEvent>()
            .add_event::<RomanticTableQuietFireNoiseEvent>()
            .add_event::<RomanticTableCandlesStoodOnEvent>()
            .init_resource::<RomanticTableState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_candelabra_interaction,
                    handle_candles_stood_on,
                    emit_candle_light_and_noise,
                    romantic_table_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<RomanticTablePurchasedEvent>,
    mut state: ResMut<RomanticTableState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_candelabra_interaction(
    mut events: EventReader<RomanticTableCandelabraInteractedEvent>,
    mut state: ResMut<RomanticTableState>,
) {
    for _ in events.read() {
        if !state.owned {
            continue;
        }

        state.candles_lit = !state.candles_lit;
        state.dim_light_emitting = state.candles_lit;
        state.quiet_fire_noise_emitting = state.candles_lit;

        if !state.candles_lit {
            state.outside_ghost_girl_kill_range = false;
        }
    }
}

fn handle_candles_stood_on(
    mut events: EventReader<RomanticTableCandlesStoodOnEvent>,
    mut state: ResMut<RomanticTableState>,
) {
    for _ in events.read() {
        if state.owned && state.candles_lit {
            state.outside_ghost_girl_kill_range = true;
        }
    }
}

fn emit_candle_light_and_noise(
    state: Res<RomanticTableState>,
    mut light_events: EventWriter<RomanticTableDimLightEmittedEvent>,
    mut noise_events: EventWriter<RomanticTableQuietFireNoiseEvent>,
) {
    if state.owned && state.candles_lit {
        light_events.send(RomanticTableDimLightEmittedEvent);
        noise_events.send(RomanticTableQuietFireNoiseEvent);
    }
}

fn romantic_table_checksum(state: Res<RomanticTableState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.candles_lit as u64);
    cs.accumulate(state.dim_light_emitting as u64);
    cs.accumulate(state.quiet_fire_noise_emitting as u64);
    cs.accumulate(state.outside_ghost_girl_kill_range as u64);
}