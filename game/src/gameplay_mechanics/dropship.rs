// Sources: vault/gameplay_mechanics/dropship.md, vault/item_index_pages/company_store.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const DROPSHIP_ID: &str = "dropship";
pub const DROPSHIP_NAME: &str = "Dropship";
pub const DROPSHIP_TYPE: &str = "gameplay_mechanics";
pub const DROPSHIP_SUBTYPE: &str = "transport_vehicle";
pub const DROPSHIP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Dropship";
pub const DROPSHIP_SOURCE_REVISION: u32 = 20542;
pub const DROPSHIP_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const DROPSHIP_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const DROPSHIP_MAX_DELIVERY_CAPACITY_ITEMS: u8 = 12;
pub const DROPSHIP_LANDING_MELODY_SECONDS: u32 = 30;
pub const DROPSHIP_DEPARTURE_AFTER_LANDING_SECONDS: u32 = 30;
pub const DROPSHIP_SEQUENCE_START_MINUTE_OF_DAY: u16 = 510;
pub const DROPSHIP_LANDING_MINUTE_OF_DAY: u16 = 525;
pub const DROPSHIP_RECOVERY_MINUTE_OF_DAY: u16 = 540;
pub const DROPSHIP_RECOVERY_DELAY_HOURS_NUMERATOR: u16 = 10;
pub const DROPSHIP_RECOVERY_DELAY_HOURS_DENOMINATOR: u16 = 10;

pub const DROPSHIP_DEPENDS_ON: [&str; 3] = ["terminal", "company_cruiser", "eyeless_dog"];

pub const DROPSHIP_BEHAVIORAL_MECHANICS: [DropshipBehaviorRule; 12] = [
    DropshipBehaviorRule {
        condition: "an employee buys items while the ship is landed on a moon",
        outcome: "the dropship delivers those items to that moon",
    },
    DropshipBehaviorRule {
        condition: "an employee buys items while the ship is in orbit",
        outcome: "the dropship delivers those items to the next moon it lands on",
    },
    DropshipBehaviorRule {
        condition: "the dropship is landed",
        outcome: "it plays a locating melody for 30 seconds",
    },
    DropshipBehaviorRule {
        condition: "the crew opens the exterior compartments before departure",
        outcome: "they can retrieve the delivered items",
    },
    DropshipBehaviorRule {
        condition: "30 seconds pass after landing",
        outcome: "the dropship leaves and any items that were not retrieved are lost",
    },
    DropshipBehaviorRule {
        condition: "the dropship has already been assigned 12 items",
        outcome: "no additional items can be ordered until the current delivery is completed and the dropship has left",
    },
    DropshipBehaviorRule {
        condition: "the company_cruiser has been ordered",
        outcome: "it uses dropship delivery capacity and blocks additional orders until it is delivered",
    },
    DropshipBehaviorRule {
        condition: "items are ordered while the ship is in orbit or immediately on landing",
        outcome: "the landing sequence starts at about 08:30 AM and the dropship lands at about 08:45 AM",
    },
    DropshipBehaviorRule {
        condition: "the crew recovers the delivery at about 09:00 AM",
        outcome: "the order adds about 1.0 hour of delay",
    },
    DropshipBehaviorRule {
        condition: "landing",
        outcome: "the dropship sound attracts eyeless_dogs",
    },
    DropshipBehaviorRule {
        condition: "an employee stands under the dropship while it lands",
        outcome: "that employee is crushed or killed",
    },
    DropshipBehaviorRule {
        condition: "the company_cruiser is delivered",
        outcome: "the dropship music plays in a faster, distorted version",
    },
];

pub struct DropshipPlugin;

impl Plugin for DropshipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DropshipState>()
            .add_event::<DropshipOrderEvent>()
            .add_event::<DropshipOrderRejectedEvent>()
            .add_event::<DropshipMoonLandingEvent>()
            .add_event::<DropshipLandedEvent>()
            .add_event::<DropshipCompartmentOpenedEvent>()
            .add_event::<DropshipItemRetrievedEvent>()
            .add_event::<DropshipEmployeeUnderLandingZoneEvent>()
            .add_event::<DropshipEmployeeCrushedEvent>()
            .add_event::<DropshipDepartedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    dropship_accept_orders,
                    dropship_start_delivery_on_moon_landing,
                    dropship_land_when_ready,
                    dropship_open_compartments,
                    dropship_retrieve_items,
                    dropship_crush_employees_under_landing_zone,
                    dropship_depart_after_timeout,
                    dropship_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropshipBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct DropshipState {
    pub assigned_items: u8,
    pub retrieved_items: u8,
    pub pending_orbit_items: u8,
    pub company_cruiser_pending: bool,
    pub company_cruiser_delivered: bool,
    pub delivery_target_moon_id: u64,
    pub delivery_sequence_started_tick: u64,
    pub landed_tick: u64,
    pub melody_ends_tick: u64,
    pub departure_tick: u64,
    pub compartments_open: bool,
    pub delivery_active: bool,
    pub landed: bool,
    pub departed_deliveries: u64,
    pub lost_items: u64,
    pub rejected_orders: u64,
    pub eyeless_dog_attraction_pulses: u64,
    pub crushed_employees: u64,
}

impl Default for DropshipState {
    fn default() -> Self {
        Self {
            assigned_items: 0,
            retrieved_items: 0,
            pending_orbit_items: 0,
            company_cruiser_pending: false,
            company_cruiser_delivered: false,
            delivery_target_moon_id: 0,
            delivery_sequence_started_tick: 0,
            landed_tick: 0,
            melody_ends_tick: 0,
            departure_tick: 0,
            compartments_open: false,
            delivery_active: false,
            landed: false,
            departed_deliveries: 0,
            lost_items: 0,
            rejected_orders: 0,
            eyeless_dog_attraction_pulses: 0,
            crushed_employees: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipOrderEvent {
    pub order_id: u64,
    pub moon_id: u64,
    pub item_count: u8,
    pub ordered_while_in_orbit: bool,
    pub includes_company_cruiser: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipOrderRejectedEvent {
    pub order_id: u64,
    pub assigned_items: u8,
    pub requested_items: u8,
    pub company_cruiser_pending: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipMoonLandingEvent {
    pub moon_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipLandedEvent {
    pub moon_id: u64,
    pub assigned_items: u8,
    pub company_cruiser_delivered: bool,
    pub distorted_music: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipCompartmentOpenedEvent {
    pub actor_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipItemRetrievedEvent {
    pub actor_id: u64,
    pub item_count: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipEmployeeUnderLandingZoneEvent {
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipEmployeeCrushedEvent {
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropshipDepartedEvent {
    pub moon_id: u64,
    pub retrieved_items: u8,
    pub lost_items: u8,
}

pub fn dropship_max_delivery_capacity_items() -> u8 {
    DROPSHIP_MAX_DELIVERY_CAPACITY_ITEMS
}

pub fn dropship_landing_melody_seconds() -> u32 {
    DROPSHIP_LANDING_MELODY_SECONDS
}

pub fn dropship_departure_after_landing_seconds() -> u32 {
    DROPSHIP_DEPARTURE_AFTER_LANDING_SECONDS
}

pub fn dropship_company_cruiser_blocks_orders(state: &DropshipState) -> bool {
    state.company_cruiser_pending
}

fn dropship_accept_orders(
    mut order_events: EventReader<DropshipOrderEvent>,
    mut rejected_events: EventWriter<DropshipOrderRejectedEvent>,
    tick: Res<SimTick>,
    mut state: ResMut<DropshipState>,
) {
    for event in order_events.read() {
        let requested_items = event.item_count;
        let total_items = state.assigned_items.saturating_add(requested_items);

        if state.company_cruiser_pending || total_items > DROPSHIP_MAX_DELIVERY_CAPACITY_ITEMS {
            state.rejected_orders = state.rejected_orders.wrapping_add(1);
            rejected_events.send(DropshipOrderRejectedEvent {
                order_id: event.order_id,
                assigned_items: state.assigned_items,
                requested_items,
                company_cruiser_pending: state.company_cruiser_pending,
            });
            continue;
        }

        state.assigned_items = total_items;
        state.delivery_target_moon_id = event.moon_id;

        if event.ordered_while_in_orbit {
            state.pending_orbit_items = state.pending_orbit_items.saturating_add(requested_items);
        } else if !state.delivery_active {
            state.delivery_active = true;
            state.delivery_sequence_started_tick = tick.0;
        }

        if event.includes_company_cruiser {
            state.company_cruiser_pending = true;
        }
    }
}

fn dropship_start_delivery_on_moon_landing(
    mut landing_events: EventReader<DropshipMoonLandingEvent>,
    tick: Res<SimTick>,
    mut state: ResMut<DropshipState>,
) {
    for event in landing_events.read() {
        if state.pending_orbit_items == 0 && state.assigned_items == 0 {
            continue;
        }

        state.delivery_target_moon_id = event.moon_id;
        state.delivery_active = true;
        state.delivery_sequence_started_tick = tick.0;
        state.pending_orbit_items = 0;
    }
}

fn dropship_land_when_ready(
    mut landed_events: EventWriter<DropshipLandedEvent>,
    tick: Res<SimTick>,
    mut state: ResMut<DropshipState>,
) {
    if !state.delivery_active || state.landed || state.assigned_items == 0 {
        return;
    }

    let descent_ticks = minute_delta_ticks(
        DROPSHIP_SEQUENCE_START_MINUTE_OF_DAY,
        DROPSHIP_LANDING_MINUTE_OF_DAY,
    );

    if tick.0 < state.delivery_sequence_started_tick.wrapping_add(descent_ticks) {
        return;
    }

    state.landed = true;
    state.landed_tick = tick.0;
    state.melody_ends_tick = tick
        .0
        .wrapping_add(seconds_to_sim_ticks(DROPSHIP_LANDING_MELODY_SECONDS));
    state.departure_tick = tick
        .0
        .wrapping_add(seconds_to_sim_ticks(DROPSHIP_DEPARTURE_AFTER_LANDING_SECONDS));
    state.eyeless_dog_attraction_pulses = state.eyeless_dog_attraction_pulses.wrapping_add(1);

    let delivered_cruiser = state.company_cruiser_pending;
    if delivered_cruiser {
        state.company_cruiser_pending = false;
        state.company_cruiser_delivered = true;
    }

    landed_events.send(DropshipLandedEvent {
        moon_id: state.delivery_target_moon_id,
        assigned_items: state.assigned_items,
        company_cruiser_delivered: delivered_cruiser,
        distorted_music: delivered_cruiser,
    });
}

fn dropship_open_compartments(
    mut opened_events: EventReader<DropshipCompartmentOpenedEvent>,
    mut state: ResMut<DropshipState>,
) {
    for _event in opened_events.read() {
        if state.landed {
            state.compartments_open = true;
        }
    }
}

fn dropship_retrieve_items(
    mut retrieve_events: EventReader<DropshipItemRetrievedEvent>,
    mut state: ResMut<DropshipState>,
) {
    for event in retrieve_events.read() {
        if !state.landed || !state.compartments_open {
            continue;
        }

        let remaining_items = state.assigned_items.saturating_sub(state.retrieved_items);
        let retrieved_now = event.item_count.min(remaining_items);
        state.retrieved_items = state.retrieved_items.saturating_add(retrieved_now);
    }
}

fn dropship_crush_employees_under_landing_zone(
    mut under_landing_events: EventReader<DropshipEmployeeUnderLandingZoneEvent>,
    mut crushed_events: EventWriter<DropshipEmployeeCrushedEvent>,
    state: Res<DropshipState>,
) {
    if !state.landed {
        return;
    }

    for event in under_landing_events.read() {
        crushed_events.send(DropshipEmployeeCrushedEvent {
            employee_id: event.employee_id,
        });
    }
}

fn dropship_depart_after_timeout(
    mut departed_events: EventWriter<DropshipDepartedEvent>,
    tick: Res<SimTick>,
    mut state: ResMut<DropshipState>,
) {
    if !state.landed || tick.0 < state.departure_tick {
        return;
    }

    let lost_items = state.assigned_items.saturating_sub(state.retrieved_items);
    state.lost_items = state.lost_items.wrapping_add(lost_items as u64);
    state.departed_deliveries = state.departed_deliveries.wrapping_add(1);

    departed_events.send(DropshipDepartedEvent {
        moon_id: state.delivery_target_moon_id,
        retrieved_items: state.retrieved_items,
        lost_items,
    });

    state.assigned_items = 0;
    state.retrieved_items = 0;
    state.delivery_target_moon_id = 0;
    state.delivery_sequence_started_tick = 0;
    state.landed_tick = 0;
    state.melody_ends_tick = 0;
    state.departure_tick = 0;
    state.compartments_open = false;
    state.delivery_active = false;
    state.landed = false;
}

fn dropship_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<DropshipState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(DROPSHIP_SOURCE_REVISION as u64);
    checksum.accumulate(DROPSHIP_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(DROPSHIP_MAX_DELIVERY_CAPACITY_ITEMS as u64);
    checksum.accumulate(DROPSHIP_LANDING_MELODY_SECONDS as u64);
    checksum.accumulate(DROPSHIP_DEPARTURE_AFTER_LANDING_SECONDS as u64);
    checksum.accumulate(DROPSHIP_SEQUENCE_START_MINUTE_OF_DAY as u64);
    checksum.accumulate(DROPSHIP_LANDING_MINUTE_OF_DAY as u64);
    checksum.accumulate(DROPSHIP_RECOVERY_MINUTE_OF_DAY as u64);
    checksum.accumulate(DROPSHIP_RECOVERY_DELAY_HOURS_NUMERATOR as u64);
    checksum.accumulate(DROPSHIP_RECOVERY_DELAY_HOURS_DENOMINATOR as u64);

    for dependency in DROPSHIP_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in DROPSHIP_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    checksum.accumulate(state.assigned_items as u64);
    checksum.accumulate(state.retrieved_items as u64);
    checksum.accumulate(state.pending_orbit_items as u64);
    checksum.accumulate(state.company_cruiser_pending as u64);
    checksum.accumulate(state.company_cruiser_delivered as u64);
    checksum.accumulate(state.delivery_target_moon_id);
    checksum.accumulate(state.delivery_sequence_started_tick);
    checksum.accumulate(state.landed_tick);
    checksum.accumulate(state.melody_ends_tick);
    checksum.accumulate(state.departure_tick);
    checksum.accumulate(state.compartments_open as u64);
    checksum.accumulate(state.delivery_active as u64);
    checksum.accumulate(state.landed as u64);
    checksum.accumulate(state.departed_deliveries);
    checksum.accumulate(state.lost_items);
    checksum.accumulate(state.rejected_orders);
    checksum.accumulate(state.eyeless_dog_attraction_pulses);
    checksum.accumulate(state.crushed_employees);
}

fn seconds_to_sim_ticks(seconds: u32) -> u64 {
    seconds as u64 * 20
}

fn minute_delta_ticks(start_minute: u16, end_minute: u16) -> u64 {
    let delta_minutes = end_minute.saturating_sub(start_minute);
    seconds_to_sim_ticks(delta_minutes as u32 * 60)
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}