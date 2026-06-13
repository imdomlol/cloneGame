// Sources: vault/store_items/microwave.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 80;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.01");

#[derive(Event)]
pub struct MicrowavePurchasedEvent;

#[derive(Event)]
pub struct MicrowaveToggledEvent {
    pub on: bool,
}

#[derive(Event)]
pub struct MicrowaveItemPlacedEvent;

#[derive(Event)]
pub struct MicrowaveItemRemovedEvent;

#[derive(Resource, Default)]
pub struct MicrowaveState {
    pub owned: bool,
    pub powered_on: bool,
    pub item_inside: bool,
    pub item_spinning: bool,
    // item continues spinning outside the microwave after the spinning bug is triggered
    pub spinning_bug_active: bool,
}

pub struct MicrowavePlugin;

impl Plugin for MicrowavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MicrowavePurchasedEvent>()
            .add_event::<MicrowaveToggledEvent>()
            .add_event::<MicrowaveItemPlacedEvent>()
            .add_event::<MicrowaveItemRemovedEvent>()
            .init_resource::<MicrowaveState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_toggle,
                    handle_item_placed,
                    handle_item_removed,
                    microwave_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<MicrowavePurchasedEvent>,
    mut state: ResMut<MicrowaveState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_toggle(
    mut events: EventReader<MicrowaveToggledEvent>,
    mut state: ResMut<MicrowaveState>,
) {
    for ev in events.read() {
        if !state.owned {
            continue;
        }
        state.powered_on = ev.on;
        if state.powered_on && state.item_inside {
            state.item_spinning = true;
            state.spinning_bug_active = true;
        }
        if !state.powered_on {
            state.item_spinning = false;
        }
    }
}

fn handle_item_placed(
    mut events: EventReader<MicrowaveItemPlacedEvent>,
    mut state: ResMut<MicrowaveState>,
) {
    for _ in events.read() {
        if !state.owned {
            continue;
        }
        state.item_inside = true;
        if state.powered_on {
            state.item_spinning = true;
            state.spinning_bug_active = true;
        }
    }
}

fn handle_item_removed(
    mut events: EventReader<MicrowaveItemRemovedEvent>,
    mut state: ResMut<MicrowaveState>,
) {
    for _ in events.read() {
        state.item_inside = false;
        // item_spinning cleared; spinning_bug_active persists so item can spin outside
        state.item_spinning = false;
    }
}

fn microwave_checksum(state: Res<MicrowaveState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.powered_on as u64);
    cs.accumulate(state.item_inside as u64);
    cs.accumulate(state.item_spinning as u64);
    cs.accumulate(state.spinning_bug_active as u64);
}