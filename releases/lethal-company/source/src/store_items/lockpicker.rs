// Sources: vault/store_items/lockpicker.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 20;
pub const WEIGHT: I32F32 = I32F32::lit("10.5");
// 30 seconds at 20 Hz sim rate
pub const OPEN_TICKS: u32 = 600;

#[derive(Event)]
pub struct LockpickerUsedEvent {
    pub on_security_door: bool,
}

#[derive(Event)]
pub struct LockpickerDoorOpenedEvent;

#[derive(Event)]
pub struct LockpickerDroppedEvent;

#[derive(Resource, Default)]
pub struct LockpickerState {
    pub in_use: bool,
    pub ticks_elapsed: u32,
}

pub struct LockpickerPlugin;

impl Plugin for LockpickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LockpickerUsedEvent>()
            .add_event::<LockpickerDoorOpenedEvent>()
            .add_event::<LockpickerDroppedEvent>()
            .init_resource::<LockpickerState>()
            .add_systems(
                FixedUpdate,
                (handle_use, tick_lockpicker, lockpicker_checksum).chain(),
            );
    }
}

fn handle_use(
    mut events: EventReader<LockpickerUsedEvent>,
    mut state: ResMut<LockpickerState>,
) {
    for ev in events.read() {
        if ev.on_security_door {
            continue;
        }
        if !state.in_use {
            state.in_use = true;
            state.ticks_elapsed = 0;
        }
    }
}

fn tick_lockpicker(
    mut state: ResMut<LockpickerState>,
    mut opened: EventWriter<LockpickerDoorOpenedEvent>,
    mut dropped: EventWriter<LockpickerDroppedEvent>,
) {
    if !state.in_use {
        return;
    }
    state.ticks_elapsed += 1;
    if state.ticks_elapsed >= OPEN_TICKS {
        state.in_use = false;
        state.ticks_elapsed = 0;
        opened.send(LockpickerDoorOpenedEvent);
        dropped.send(LockpickerDroppedEvent);
    }
}

fn lockpicker_checksum(state: Res<LockpickerState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.in_use as u64);
    cs.accumulate(state.ticks_elapsed as u64);
}