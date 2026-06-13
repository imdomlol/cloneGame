// Sources: vault/store_items/television.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 130;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.02");
pub const SOURCE_ASPECT_WIDTH: u32 = 16;
pub const SOURCE_ASPECT_HEIGHT: u32 = 9;
pub const SCREEN_ASPECT_WIDTH: u32 = 8;
pub const SCREEN_ASPECT_HEIGHT: u32 = 7;

#[derive(Event)]
pub struct TelevisionPurchasedEvent;

#[derive(Event)]
pub struct TelevisionButtonUsedEvent;

#[derive(Event)]
pub struct TelevisionToggledEvent {
    pub on: bool,
}

#[derive(Event)]
pub struct TelevisionChannelCycleEvent {
    pub cycle_tick: u32,
}

#[derive(Resource, Default)]
pub struct TelevisionState {
    pub owned: bool,
    pub on: bool,
    pub channel_cycle_ticks: u32,
}

pub struct TelevisionPlugin;

impl Plugin for TelevisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TelevisionPurchasedEvent>()
            .add_event::<TelevisionButtonUsedEvent>()
            .add_event::<TelevisionToggledEvent>()
            .add_event::<TelevisionChannelCycleEvent>()
            .init_resource::<TelevisionState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_button_toggle,
                    cycle_channels_while_on,
                    television_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<TelevisionPurchasedEvent>,
    mut state: ResMut<TelevisionState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_button_toggle(
    mut events: EventReader<TelevisionButtonUsedEvent>,
    mut toggled: EventWriter<TelevisionToggledEvent>,
    mut state: ResMut<TelevisionState>,
) {
    for _ in events.read() {
        if state.owned {
            state.on = !state.on;
            toggled.send(TelevisionToggledEvent { on: state.on });
        }
    }
}

fn cycle_channels_while_on(
    mut events: EventWriter<TelevisionChannelCycleEvent>,
    mut state: ResMut<TelevisionState>,
) {
    if state.on {
        state.channel_cycle_ticks = state.channel_cycle_ticks.wrapping_add(1);
        events.send(TelevisionChannelCycleEvent {
            cycle_tick: state.channel_cycle_ticks,
        });
    }
}

fn television_checksum(state: Res<TelevisionState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.on as u64);
    cs.accumulate(state.channel_cycle_ticks as u64);
}