// Sources: vault/store_items/pro_flashlight.md, vault/store_items/flashlight.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimHz};

pub const BUY_COST: u32 = 28;
pub const WEIGHT: u32 = 5;
pub const BATTERY_LIFE_SECS: I32F32 = I32F32::lit("300");

#[derive(Event)]
pub struct ProFlashlightPurchasedEvent;

#[derive(Event)]
pub struct ProFlashlightToggledEvent {
    pub on: bool,
}

#[derive(Event)]
pub struct ProFlashlightBatteryDepletedEvent;

#[derive(Resource)]
pub struct ProFlashlightState {
    pub owned: bool,
    pub active: bool,
    pub battery_secs: I32F32,
}

impl Default for ProFlashlightState {
    fn default() -> Self {
        Self {
            owned: false,
            active: false,
            battery_secs: BATTERY_LIFE_SECS,
        }
    }
}

pub struct ProFlashlightPlugin;

impl Plugin for ProFlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProFlashlightPurchasedEvent>()
            .add_event::<ProFlashlightToggledEvent>()
            .add_event::<ProFlashlightBatteryDepletedEvent>()
            .init_resource::<ProFlashlightState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_toggle,
                    drain_battery,
                    pro_flashlight_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<ProFlashlightPurchasedEvent>,
    mut state: ResMut<ProFlashlightState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_toggle(
    mut events: EventReader<ProFlashlightToggledEvent>,
    mut state: ResMut<ProFlashlightState>,
) {
    for ev in events.read() {
        if state.owned && state.battery_secs > I32F32::ZERO {
            state.active = ev.on;
        }
    }
}

fn drain_battery(
    mut state: ResMut<ProFlashlightState>,
    sim_hz: Res<SimHz>,
    mut depleted: EventWriter<ProFlashlightBatteryDepletedEvent>,
) {
    if !state.active {
        return;
    }
    let dt = I32F32::from_num(1_u32) / I32F32::from_num(sim_hz.0);
    state.battery_secs = (state.battery_secs - dt).max(I32F32::ZERO);
    if state.battery_secs == I32F32::ZERO {
        state.active = false;
        depleted.send(ProFlashlightBatteryDepletedEvent);
    }
}

fn pro_flashlight_checksum(state: Res<ProFlashlightState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
    cs.accumulate(state.battery_secs.to_bits() as u64);
}