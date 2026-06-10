// Sources: vault/gameplay_mechanics/orbit.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const ORBIT_ID: &str = "orbit";
pub const ORBIT_NAME: &str = "Orbit";
pub const ORBIT_TYPE: &str = "gameplay_mechanics";
pub const ORBIT_SUBTYPE: &str = "orbit";
pub const ORBIT_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Orbit";
pub const ORBIT_SOURCE_REVISION: u32 = 21483;
pub const ORBIT_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const ORBIT_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const ORBIT_OVERVIEW: &str =
    "The autopilot_ship is in orbit whenever it is not on the surface of a moon.";

pub const ORBIT_AUTOPILOT_SHIP_ID: &str = "autopilot_ship";
pub const ORBIT_MOON_ID: &str = "moon";
pub const ORBIT_ITEMS_ID: &str = "items";
pub const ORBIT_DELIVERY_SYSTEM_ID: &str = "delivery_system";
pub const ORBIT_TERMINAL_ID: &str = "terminal";

pub const ORBIT_DEPENDS_ON: [&str; 5] = [
    "autopilot_ship",
    "moon",
    "items",
    "delivery_system",
    "terminal",
];

pub const ORBIT_BEHAVIORAL_MECHANICS: [OrbitBehaviorRule; 6] = [
    OrbitBehaviorRule {
        condition: "a new season starts",
        outcome: "the autopilot_ship starts in orbit",
    },
    OrbitBehaviorRule {
        condition: "a work day ends",
        outcome: "the autopilot_ship returns to orbit",
    },
    OrbitBehaviorRule {
        condition: "the autopilot_ship is in orbit",
        outcome: "radar cams are disabled",
    },
    OrbitBehaviorRule {
        condition: "crew members select a new moon",
        outcome: "the autopilot can route the autopilot_ship to that moon",
    },
    OrbitBehaviorRule {
        condition: "items are ordered",
        outcome: "they are delivered by the delivery_system at the beginning of the next work day",
    },
    OrbitBehaviorRule {
        condition: "the SCAN command is used on the terminal",
        outcome: "it shows the amount and total value of every item inside the autopilot_ship",
    },
];

pub const ORBIT_MODIFIERS: [OrbitBehaviorRule; 1] = [OrbitBehaviorRule {
    condition: "the autopilot_ship is in orbit",
    outcome: "radar cams remain disabled",
}];

pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrbitState>()
            .add_event::<OrbitNewSeasonStartedEvent>()
            .add_event::<OrbitWorkDayEndedEvent>()
            .add_event::<OrbitWorkDayStartedEvent>()
            .add_event::<OrbitMoonSelectedEvent>()
            .add_event::<OrbitRouteAuthorizedEvent>()
            .add_event::<OrbitItemsOrderedEvent>()
            .add_event::<OrbitItemsDeliveredEvent>()
            .add_event::<OrbitTerminalScanCommandEvent>()
            .add_event::<OrbitScanResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    orbit_start_new_season_in_orbit,
                    orbit_return_after_work_day,
                    orbit_disable_radar_cams,
                    orbit_authorize_selected_moon_route,
                    orbit_queue_ordered_items,
                    orbit_deliver_ordered_items,
                    orbit_resolve_terminal_scan,
                    orbit_maintain_radar_cam_disabled_modifier,
                    orbit_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrbitBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct OrbitState {
    pub autopilot_ship_id: &'static str,
    pub in_orbit: bool,
    pub radar_cams_disabled: bool,
    pub selected_moon_id: Option<&'static str>,
    pub pending_delivery_item_count: u32,
    pub pending_delivery_total_value_credits: u32,
    pub ship_item_count: u32,
    pub ship_item_total_value_credits: u32,
    pub new_seasons_started: u64,
    pub work_days_ended: u64,
    pub work_days_started: u64,
    pub routes_authorized: u64,
    pub item_orders_queued: u64,
    pub item_deliveries_completed: u64,
    pub scan_commands_used: u64,
}

impl Default for OrbitState {
    fn default() -> Self {
        Self {
            autopilot_ship_id: ORBIT_AUTOPILOT_SHIP_ID,
            in_orbit: true,
            radar_cams_disabled: true,
            selected_moon_id: None,
            pending_delivery_item_count: 0,
            pending_delivery_total_value_credits: 0,
            ship_item_count: 0,
            ship_item_total_value_credits: 0,
            new_seasons_started: 0,
            work_days_ended: 0,
            work_days_started: 0,
            routes_authorized: 0,
            item_orders_queued: 0,
            item_deliveries_completed: 0,
            scan_commands_used: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitNewSeasonStartedEvent {
    pub season_index: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitWorkDayEndedEvent {
    pub day_index: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitWorkDayStartedEvent {
    pub day_index: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitMoonSelectedEvent {
    pub moon_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitRouteAuthorizedEvent {
    pub moon_id: &'static str,
    pub autopilot_ship_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitItemsOrderedEvent {
    pub item_count: u32,
    pub total_value_credits: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitItemsDeliveredEvent {
    pub item_count: u32,
    pub total_value_credits: u32,
    pub delivery_system_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitTerminalScanCommandEvent {
    pub context_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbitScanResolvedEvent {
    pub context_id: u64,
    pub item_count: u32,
    pub total_value_credits: u32,
    pub autopilot_ship_id: &'static str,
}

pub fn orbit_autopilot_ship_id() -> &'static str {
    ORBIT_AUTOPILOT_SHIP_ID
}

pub fn orbit_is_in_orbit(state: &OrbitState) -> bool {
    state.in_orbit
}

pub fn orbit_radar_cams_disabled(state: &OrbitState) -> bool {
    state.radar_cams_disabled
}

pub fn orbit_ship_scan_totals(state: &OrbitState) -> (u32, u32) {
    (state.ship_item_count, state.ship_item_total_value_credits)
}

fn orbit_start_new_season_in_orbit(
    mut events: EventReader<OrbitNewSeasonStartedEvent>,
    mut state: ResMut<OrbitState>,
) {
    for _event in events.read() {
        state.in_orbit = true;
        state.radar_cams_disabled = true;
        state.selected_moon_id = None;
        state.new_seasons_started = state.new_seasons_started.wrapping_add(1);
    }
}

fn orbit_return_after_work_day(
    mut events: EventReader<OrbitWorkDayEndedEvent>,
    mut state: ResMut<OrbitState>,
) {
    for _event in events.read() {
        state.in_orbit = true;
        state.radar_cams_disabled = true;
        state.work_days_ended = state.work_days_ended.wrapping_add(1);
    }
}

fn orbit_disable_radar_cams(mut state: ResMut<OrbitState>) {
    if state.in_orbit {
        state.radar_cams_disabled = true;
    }
}

fn orbit_authorize_selected_moon_route(
    mut selected_events: EventReader<OrbitMoonSelectedEvent>,
    mut route_events: EventWriter<OrbitRouteAuthorizedEvent>,
    mut state: ResMut<OrbitState>,
) {
    for event in selected_events.read() {
        state.selected_moon_id = Some(event.moon_id);
        state.routes_authorized = state.routes_authorized.wrapping_add(1);

        route_events.send(OrbitRouteAuthorizedEvent {
            moon_id: event.moon_id,
            autopilot_ship_id: state.autopilot_ship_id,
        });
    }
}

fn orbit_queue_ordered_items(
    mut ordered_events: EventReader<OrbitItemsOrderedEvent>,
    mut state: ResMut<OrbitState>,
) {
    for event in ordered_events.read() {
        state.pending_delivery_item_count = state
            .pending_delivery_item_count
            .wrapping_add(event.item_count);
        state.pending_delivery_total_value_credits = state
            .pending_delivery_total_value_credits
            .wrapping_add(event.total_value_credits);
        state.item_orders_queued = state.item_orders_queued.wrapping_add(1);
    }
}

fn orbit_deliver_ordered_items(
    mut day_started_events: EventReader<OrbitWorkDayStartedEvent>,
    mut delivered_events: EventWriter<OrbitItemsDeliveredEvent>,
    mut state: ResMut<OrbitState>,
) {
    for _event in day_started_events.read() {
        state.work_days_started = state.work_days_started.wrapping_add(1);

        if state.pending_delivery_item_count == 0 {
            continue;
        }

        let delivered_item_count = state.pending_delivery_item_count;
        let delivered_total_value = state.pending_delivery_total_value_credits;

        state.pending_delivery_item_count = 0;
        state.pending_delivery_total_value_credits = 0;
        state.ship_item_count = state.ship_item_count.wrapping_add(delivered_item_count);
        state.ship_item_total_value_credits = state
            .ship_item_total_value_credits
            .wrapping_add(delivered_total_value);
        state.item_deliveries_completed = state.item_deliveries_completed.wrapping_add(1);

        delivered_events.send(OrbitItemsDeliveredEvent {
            item_count: delivered_item_count,
            total_value_credits: delivered_total_value,
            delivery_system_id: ORBIT_DELIVERY_SYSTEM_ID,
        });
    }
}

fn orbit_resolve_terminal_scan(
    mut scan_events: EventReader<OrbitTerminalScanCommandEvent>,
    mut resolved_events: EventWriter<OrbitScanResolvedEvent>,
    mut state: ResMut<OrbitState>,
) {
    for event in scan_events.read() {
        state.scan_commands_used = state.scan_commands_used.wrapping_add(1);

        resolved_events.send(OrbitScanResolvedEvent {
            context_id: event.context_id,
            item_count: state.ship_item_count,
            total_value_credits: state.ship_item_total_value_credits,
            autopilot_ship_id: state.autopilot_ship_id,
        });
    }
}

fn orbit_maintain_radar_cam_disabled_modifier(mut state: ResMut<OrbitState>) {
    if state.in_orbit {
        state.radar_cams_disabled = true;
    }
}

fn orbit_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<OrbitState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ORBIT_SOURCE_REVISION as u64);
    checksum.accumulate(ORBIT_CONFIDENCE_BASIS_POINTS as u64);

    accumulate_str(&mut checksum, 0x1000, ORBIT_ID);
    accumulate_str(&mut checksum, 0x1001, ORBIT_NAME);
    accumulate_str(&mut checksum, 0x1002, ORBIT_TYPE);
    accumulate_str(&mut checksum, 0x1003, ORBIT_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ORBIT_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, ORBIT_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, ORBIT_OVERVIEW);

    for dependency in ORBIT_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in ORBIT_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for modifier in ORBIT_MODIFIERS {
        accumulate_str(&mut checksum, 0x4000, modifier.condition);
        accumulate_str(&mut checksum, 0x4001, modifier.outcome);
    }

    accumulate_str(&mut checksum, 0x5000, state.autopilot_ship_id);
    checksum.accumulate(state.in_orbit as u64);
    checksum.accumulate(state.radar_cams_disabled as u64);

    if let Some(selected_moon_id) = state.selected_moon_id {
        accumulate_str(&mut checksum, 0x5001, selected_moon_id);
    } else {
        checksum.accumulate(0x5001);
    }

    checksum.accumulate(state.pending_delivery_item_count as u64);
    checksum.accumulate(state.pending_delivery_total_value_credits as u64);
    checksum.accumulate(state.ship_item_count as u64);
    checksum.accumulate(state.ship_item_total_value_credits as u64);
    checksum.accumulate(state.new_seasons_started);
    checksum.accumulate(state.work_days_ended);
    checksum.accumulate(state.work_days_started);
    checksum.accumulate(state.routes_authorized);
    checksum.accumulate(state.item_orders_queued);
    checksum.accumulate(state.item_deliveries_completed);
    checksum.accumulate(state.scan_commands_used);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}