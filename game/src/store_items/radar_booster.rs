// Sources: vault/store_items/radar_booster.md

use bevy::prelude::*;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 60;
pub const WEIGHT_LBS: u32 = 19;
// 2.5 s cooldown_seconds at 30 Hz sim tick
pub const FLASH_COOLDOWN_TICKS: u32 = 75;

#[derive(Event)]
pub struct RadarBoosterPurchasedEvent;

#[derive(Event)]
pub struct RadarBoosterDeployedEvent {
    pub booster_id: u64,
}

#[derive(Event)]
pub struct RadarBoosterPingCommandEvent {
    pub booster_id: u64,
}

#[derive(Event)]
pub struct RadarBoosterFlashCommandEvent {
    pub booster_id: u64,
}

/// Fired after a PING command resolves; listeners (e.g. eyeless dog AI) emit noise reactions.
#[derive(Event)]
pub struct RadarBoosterPingedEvent {
    pub booster_id: u64,
}

/// Fired after a FLASH command resolves and the cooldown gate passes.
/// Per-entity stun systems (bracken, jester, coil_head, etc.) listen to this event.
#[derive(Event)]
pub struct RadarBoosterFlashedEvent {
    pub booster_id: u64,
}

#[derive(Resource, Default)]
pub struct RadarBoosterState {
    pub purchased: bool,
    pub deployed: bool,
    pub flash_cooldown_remaining: u32,
}

pub struct RadarBoosterPlugin;

impl Plugin for RadarBoosterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RadarBoosterPurchasedEvent>()
            .add_event::<RadarBoosterDeployedEvent>()
            .add_event::<RadarBoosterPingCommandEvent>()
            .add_event::<RadarBoosterFlashCommandEvent>()
            .add_event::<RadarBoosterPingedEvent>()
            .add_event::<RadarBoosterFlashedEvent>()
            .init_resource::<RadarBoosterState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_deploy,
                    tick_flash_cooldown,
                    handle_ping,
                    handle_flash,
                    radar_booster_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<RadarBoosterPurchasedEvent>,
    mut state: ResMut<RadarBoosterState>,
) {
    for _ in events.read() {
        if !state.purchased {
            state.purchased = true;
        }
    }
}

fn handle_deploy(
    mut events: EventReader<RadarBoosterDeployedEvent>,
    mut state: ResMut<RadarBoosterState>,
) {
    for _ in events.read() {
        state.deployed = true;
    }
}

fn tick_flash_cooldown(mut state: ResMut<RadarBoosterState>) {
    if state.flash_cooldown_remaining > 0 {
        state.flash_cooldown_remaining -= 1;
    }
}

fn handle_ping(
    mut commands_in: EventReader<RadarBoosterPingCommandEvent>,
    state: Res<RadarBoosterState>,
    mut pinged: EventWriter<RadarBoosterPingedEvent>,
) {
    for ev in commands_in.read() {
        if state.deployed {
            pinged.send(RadarBoosterPingedEvent {
                booster_id: ev.booster_id,
            });
        }
    }
}

fn handle_flash(
    mut commands_in: EventReader<RadarBoosterFlashCommandEvent>,
    mut state: ResMut<RadarBoosterState>,
    mut flashed: EventWriter<RadarBoosterFlashedEvent>,
) {
    for ev in commands_in.read() {
        if state.deployed && state.flash_cooldown_remaining == 0 {
            state.flash_cooldown_remaining = FLASH_COOLDOWN_TICKS;
            flashed.send(RadarBoosterFlashedEvent {
                booster_id: ev.booster_id,
            });
        }
    }
}

fn radar_booster_checksum(state: Res<RadarBoosterState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.purchased as u64);
    cs.accumulate(state.deployed as u64);
    cs.accumulate(state.flash_cooldown_remaining as u64);
}