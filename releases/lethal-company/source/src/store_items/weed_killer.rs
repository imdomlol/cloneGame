// Sources: vault/store_items/weed_killer.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 20;
pub const SELL_VALUE: u32 = 0;
pub const WEIGHT: u32 = 0;
pub const BATTERY_LIFE_SECONDS: u32 = 30;
pub const BATTERY_LIFE_TICKS: u32 = 600;
pub const CONDUCTIVE: bool = false;
pub const INFECTED_FACE_DAMAGE: i32 = 8;
pub const CADAVER_INFECTION_REMOVED_PER_TICK: I32F32 = I32F32::lit("0.10");
pub const VAIN_SHROUD_ID: &str = "vain_shroud";
pub const COMPANY_CRUISER_ID: &str = "company_cruiser";
pub const CADAVER_ID: &str = "cadaver";
pub const CADAVER_BLOOM_ID: &str = "cadaver_bloom";
pub const KIDNAPPER_FOX_ID: &str = "kidnapper_fox";

#[derive(Event)]
pub struct WeedKillerPurchasedEvent;

#[derive(Event)]
pub struct WeedKillerSprayStartedEvent {
    pub item_id: u64,
}

#[derive(Event)]
pub struct WeedKillerSprayStoppedEvent {
    pub item_id: u64,
}

#[derive(Event)]
pub struct WeedKillerSprayedVainShroudEvent {
    pub item_id: u64,
    pub vain_shroud_id: u64,
}

#[derive(Event)]
pub struct WeedKillerAllVainShroudsEliminatedEvent;

#[derive(Event)]
pub struct WeedKillerSprayedCompanyCruiserEvent {
    pub item_id: u64,
    pub cruiser_id: u64,
}

#[derive(Event)]
pub struct WeedKillerSprayedInfectedFaceEvent {
    pub item_id: u64,
    pub target_id: u64,
    pub burst_phase: bool,
}

#[derive(Event)]
pub struct WeedKillerCadaverBloomConversionEvent {
    pub target_id: u64,
}

#[derive(Event)]
pub struct WeedKillerDepletedEvent {
    pub item_id: u64,
}

#[derive(Event)]
pub struct WeedKillerDroppedEvent {
    pub item_id: u64,
}

#[derive(Resource, Default)]
pub struct WeedKillerState {
    pub owned: bool,
    pub active_item_id: Option<u64>,
    pub spraying: bool,
    pub battery_remaining_ticks: u32,
    pub depleted: bool,
    pub vain_shrouds_remaining: u32,
    pub kidnapper_fox_spawn_suppressed: bool,
    pub cruiser_repair_ticks: u32,
    pub cruiser_turbo_boost_ticks: u32,
    pub infected_face_damage_events: u32,
    pub cadaver_cure_ticks: u32,
    pub cadaver_bloom_conversions: u32,
    pub drop_sound_events: u32,
}

pub struct WeedKillerPlugin;

impl Plugin for WeedKillerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WeedKillerPurchasedEvent>()
            .add_event::<WeedKillerSprayStartedEvent>()
            .add_event::<WeedKillerSprayStoppedEvent>()
            .add_event::<WeedKillerSprayedVainShroudEvent>()
            .add_event::<WeedKillerAllVainShroudsEliminatedEvent>()
            .add_event::<WeedKillerSprayedCompanyCruiserEvent>()
            .add_event::<WeedKillerSprayedInfectedFaceEvent>()
            .add_event::<WeedKillerCadaverBloomConversionEvent>()
            .add_event::<WeedKillerDepletedEvent>()
            .add_event::<WeedKillerDroppedEvent>()
            .init_resource::<WeedKillerState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_spray_start,
                    handle_spray_stop,
                    drain_active_spray,
                    handle_vain_shroud_spray,
                    suppress_kidnapper_fox_when_shrouds_gone,
                    handle_company_cruiser_spray,
                    handle_infected_face_spray,
                    handle_drop_sound,
                    weed_killer_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<WeedKillerPurchasedEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
            state.battery_remaining_ticks = BATTERY_LIFE_TICKS;
            state.depleted = false;
        }
    }
}

fn handle_spray_start(
    mut events: EventReader<WeedKillerSprayStartedEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for event in events.read() {
        if state.owned && !state.depleted && state.battery_remaining_ticks > 0 {
            state.active_item_id = Some(event.item_id);
            state.spraying = true;
        }
    }
}

fn handle_spray_stop(
    mut events: EventReader<WeedKillerSprayStoppedEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for event in events.read() {
        if state.active_item_id == Some(event.item_id) {
            state.spraying = false;
        }
    }
}

fn drain_active_spray(
    mut state: ResMut<WeedKillerState>,
    mut depleted_events: EventWriter<WeedKillerDepletedEvent>,
) {
    if !state.spraying || state.depleted {
        return;
    }

    if state.battery_remaining_ticks > 0 {
        state.battery_remaining_ticks -= 1;
    }

    if state.battery_remaining_ticks == 0 {
        state.depleted = true;
        state.spraying = false;
        if let Some(item_id) = state.active_item_id {
            depleted_events.send(WeedKillerDepletedEvent { item_id });
        }
    }
}

fn handle_vain_shroud_spray(
    mut events: EventReader<WeedKillerSprayedVainShroudEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for _ in events.read() {
        if state.spraying && !state.depleted && state.vain_shrouds_remaining > 0 {
            state.vain_shrouds_remaining -= 1;
        }
    }
}

fn suppress_kidnapper_fox_when_shrouds_gone(
    mut state: ResMut<WeedKillerState>,
    mut events: EventWriter<WeedKillerAllVainShroudsEliminatedEvent>,
) {
    if state.vain_shrouds_remaining == 0 && !state.kidnapper_fox_spawn_suppressed {
        state.kidnapper_fox_spawn_suppressed = true;
        events.send(WeedKillerAllVainShroudsEliminatedEvent);
    }
}

fn handle_company_cruiser_spray(
    mut events: EventReader<WeedKillerSprayedCompanyCruiserEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for _ in events.read() {
        if state.spraying && !state.depleted {
            state.cruiser_repair_ticks = state.cruiser_repair_ticks.saturating_add(1);
            state.cruiser_turbo_boost_ticks = state.cruiser_turbo_boost_ticks.saturating_add(1);
        }
    }
}

fn handle_infected_face_spray(
    mut events: EventReader<WeedKillerSprayedInfectedFaceEvent>,
    mut state: ResMut<WeedKillerState>,
    mut bloom_events: EventWriter<WeedKillerCadaverBloomConversionEvent>,
) {
    for event in events.read() {
        if state.spraying && !state.depleted {
            state.infected_face_damage_events = state.infected_face_damage_events.saturating_add(1);

            if event.burst_phase {
                state.cadaver_bloom_conversions = state.cadaver_bloom_conversions.saturating_add(1);
                bloom_events.send(WeedKillerCadaverBloomConversionEvent {
                    target_id: event.target_id,
                });
            } else {
                state.cadaver_cure_ticks = state.cadaver_cure_ticks.saturating_add(1);
            }
        }
    }
}

fn handle_drop_sound(
    mut events: EventReader<WeedKillerDroppedEvent>,
    mut state: ResMut<WeedKillerState>,
) {
    for _ in events.read() {
        state.drop_sound_events = state.drop_sound_events.saturating_add(1);
    }
}

fn weed_killer_checksum(state: Res<WeedKillerState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active_item_id.unwrap_or(0));
    cs.accumulate(state.spraying as u64);
    cs.accumulate(state.battery_remaining_ticks as u64);
    cs.accumulate(state.depleted as u64);
    cs.accumulate(state.vain_shrouds_remaining as u64);
    cs.accumulate(state.kidnapper_fox_spawn_suppressed as u64);
    cs.accumulate(state.cruiser_repair_ticks as u64);
    cs.accumulate(state.cruiser_turbo_boost_ticks as u64);
    cs.accumulate(state.infected_face_damage_events as u64);
    cs.accumulate(state.cadaver_cure_ticks as u64);
    cs.accumulate(state.cadaver_bloom_conversions as u64);
    cs.accumulate(state.drop_sound_events as u64);
    cs.accumulate(CADAVER_INFECTION_REMOVED_PER_TICK.to_bits() as u64);
}