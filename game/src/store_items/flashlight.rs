// Sources: vault/store_items/flashlight.md

use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimHz};

pub const BUY_COST: u32 = 15;
pub const WEIGHT_LBS: u32 = 0;
pub const BATTERY_LIFE_SECONDS: u32 = 140;
pub const CONDUCTIVE: bool = false;

#[derive(Event)]
pub struct FlashlightPurchasedEvent;

#[derive(Event)]
pub struct FlashlightToggledEvent {
    pub active: bool,
}

#[derive(Event)]
pub struct FlashlightBatteryDepletedEvent;

#[derive(Event)]
pub struct FlashlightRechargedEvent;

#[derive(Resource, Default)]
pub struct FlashlightState {
    pub owned: bool,
    pub active: bool,
    pub battery_ticks_remaining: u32,
    pub battery_ticks_max: u32,
}

pub struct FlashlightPlugin;

impl Plugin for FlashlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FlashlightPurchasedEvent>()
            .add_event::<FlashlightToggledEvent>()
            .add_event::<FlashlightBatteryDepletedEvent>()
            .add_event::<FlashlightRechargedEvent>()
            .init_resource::<FlashlightState>()
            .add_systems(Startup, init_battery)
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_toggle,
                    handle_recharge,
                    drain_battery,
                    flashlight_checksum,
                )
                    .chain(),
            );
    }
}

fn init_battery(sim_hz: Res<SimHz>, mut state: ResMut<FlashlightState>) {
    let ticks_max = BATTERY_LIFE_SECONDS * sim_hz.0.to_num::<u32>();
    state.battery_ticks_max = ticks_max;
    state.battery_ticks_remaining = ticks_max;
}

fn handle_purchase(
    mut events: EventReader<FlashlightPurchasedEvent>,
    mut state: ResMut<FlashlightState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_toggle(
    mut events: EventReader<FlashlightToggledEvent>,
    mut state: ResMut<FlashlightState>,
) {
    for ev in events.read() {
        if state.owned && state.battery_ticks_remaining > 0 {
            state.active = ev.active;
        }
    }
}

fn handle_recharge(
    mut events: EventReader<FlashlightRechargedEvent>,
    mut state: ResMut<FlashlightState>,
) {
    for _ in events.read() {
        state.battery_ticks_remaining = state.battery_ticks_max;
    }
}

fn drain_battery(
    mut state: ResMut<FlashlightState>,
    mut depleted: EventWriter<FlashlightBatteryDepletedEvent>,
) {
    if state.active && state.battery_ticks_remaining > 0 {
        state.battery_ticks_remaining -= 1;
        if state.battery_ticks_remaining == 0 {
            state.active = false;
            depleted.send(FlashlightBatteryDepletedEvent);
        }
    }
}

fn flashlight_checksum(state: Res<FlashlightState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
    cs.accumulate(state.battery_ticks_remaining as u64);
}