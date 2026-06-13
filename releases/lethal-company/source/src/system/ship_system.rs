// Sources: vault/gameplay_mechanics/the_ship.md, vault/item_index_pages/company_store.md
use bevy::prelude::*;
use fixed::types::I64F64;
use std::collections::BTreeMap;

use crate::sim::{SimChecksumState, SimTick};
use crate::system::game_state_machine::GameState;

pub const SHIP_SYSTEM_ID: &str = "ship_system";
pub const THE_SHIP_ID: &str = "the_ship";
pub const THE_SHIP_NAME: &str = "The Ship";
pub const THE_SHIP_TYPE: &str = "gameplay_mechanics";
pub const THE_SHIP_SUBTYPE: &str = "ship";
pub const THE_SHIP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/The_Ship";
pub const THE_SHIP_SOURCE_REVISION: u32 = 21375;
pub const THE_SHIP_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const COMPANY_STORE_ID: &str = "company_store";
pub const COMPANY_STORE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Store";
pub const COMPANY_STORE_SOURCE_REVISION: u32 = 21369;
pub const COMPANY_STORE_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const SHIP_SIM_TICKS_PER_SECOND: u64 = 25;
pub const SHIP_SIM_TICKS_PER_HOUR: u64 = SHIP_SIM_TICKS_PER_SECOND * 60 * 60;
pub const SHIP_LANDING_TIME_TICK: u64 = SHIP_SIM_TICKS_PER_HOUR * 8;
pub const SHIP_DEPARTURE_TIME_TICK: u64 = (SHIP_SIM_TICKS_PER_HOUR * 23) + (SHIP_SIM_TICKS_PER_SECOND * 60 * 57);
pub const MAIN_DOOR_DRAIN_TICKS: u16 = 30 * 25;
pub const MAIN_DOOR_REFILL_TICKS: u16 = 6 * 25;
pub const MAIN_DOOR_REFILL_STEP: u16 = MAIN_DOOR_DRAIN_TICKS / MAIN_DOOR_REFILL_TICKS;
pub const TELEPORTER_COOLDOWN_TICKS: u32 = 10 * 25;
pub const INVERSE_TELEPORTER_COOLDOWN_TICKS: u32 = 5 * 60 * 25;
pub const DROPSHIP_RETRIEVAL_TICKS: u32 = 30 * 25;
pub const STORE_MAX_ITEMS_PER_DROPSHIP: u16 = 12;
pub const SIGNAL_TRANSLATOR_MESSAGE_CHARS: usize = 9;

pub const THE_SHIP_DEPENDS_ON: [&str; 12] = [
    "moon",
    "terminal",
    "monitors",
    "scrap",
    "equipment",
    "ship_upgrade",
    "loud_horn",
    "teleporter",
    "inverse_teleporter",
    "signal_translator",
    "facility",
    "cozy_lights",
];

pub const COMPANY_STORE_DEPENDS_ON: [&str; 13] = [
    "company_credits",
    "scrap",
    "company_building",
    "terminal",
    "moon",
    "dropship",
    "profit_quota",
    "company",
    "advertisements",
    "ship_upgrade",
    "decor",
    "cozy_lights",
    "storage",
];

pub const THE_SHIP_BEHAVIORAL_MECHANICS: [ShipRule; 15] = [
    ShipRule {
        condition: "a destination moon is confirmed in the terminal and the front lever is pulled",
        outcome: "the ship lands at 8:00 am and departs at 11:57 pm",
    },
    ShipRule {
        condition: "the brake lever is disengaged",
        outcome: "the ship departs early even if crew members are not on board",
    },
    ShipRule {
        condition: "a unanimous vote to leave succeeds",
        outcome: "the ship departs early and living crew are warned that the exit is due to hazardous conditions",
    },
    ShipRule {
        condition: "all crew members die",
        outcome: "the ship departs automatically and retains equipment, ship upgrades, decor, and credits while saved scrap is lost",
    },
    ShipRule {
        condition: "a crew member is anywhere on the ship before the doors close",
        outcome: "they count as aboard even on the railing, roof, or catwalk",
    },
    ShipRule {
        condition: "the main pressure door has power",
        outcome: "it can stay closed, lose pressure over about 30 seconds, and refill in about 6 seconds",
    },
    ShipRule {
        condition: "main door pressure reaches 0%",
        outcome: "the door opens",
    },
    ShipRule {
        condition: "the light switch is toggled",
        outcome: "the ship interior lights turn on or off while some fixtures remain lit",
    },
    ShipRule {
        condition: "the loud_horn is activated",
        outcome: "its sound can be heard everywhere and can draw outdoor creatures toward the front of the ship",
    },
    ShipRule {
        condition: "the teleporter is used",
        outcome: "one monitored employee returns to the ship, carried scrap and equipment drop, and the cooldown is 10 seconds",
    },
    ShipRule {
        condition: "the inverse_teleporter is used",
        outcome: "employees standing on its pad are sent to a random facility location, carried scrap and equipment drop on the ship, and the cooldown is 5 minutes",
    },
    ShipRule {
        condition: "the signal_translator receives `TRANSMIT [message]`",
        outcome: "it sends a nine-character message, stated to be ten characters",
    },
    ShipRule {
        condition: "movable furnishings are repositioned",
        outcome: "they can be hidden with `X` or moved with `B`, but not while the ship is arriving or departing",
    },
    ShipRule {
        condition: "ship departure occurs",
        outcome: "employees on the roof or catwalk are pulled inside before the ship leaves",
    },
    ShipRule {
        condition: "some lights are still active after the ship lights go off",
        outcome: "cozy_lights and similar fixtures remain visible",
    },
];

pub const COMPANY_STORE_BEHAVIORAL_MECHANICS: [ShipRule; 16] = [
    ShipRule {
        condition: "`STORE` is entered in the terminal",
        outcome: "the store prints the catalog and the current profit_quota rotation",
    },
    ShipRule {
        condition: "an item name or at least its first 3 letters are entered",
        outcome: "the terminal opens a confirm or deny prompt",
    },
    ShipRule {
        condition: "`C` is entered at the prompt",
        outcome: "the order is confirmed",
    },
    ShipRule {
        condition: "`D` is entered at the prompt",
        outcome: "the order is denied",
    },
    ShipRule {
        condition: "a number is typed before or after a single item name",
        outcome: "multiple copies of that item are ordered in one command",
    },
    ShipRule {
        condition: "the player tries to combine multiple different items into one command",
        outcome: "the purchase is rejected as a multi-item order",
    },
    ShipRule {
        condition: "the purchase is made from orbit",
        outcome: "the dropship waits until landing before descent begins",
    },
    ShipRule {
        condition: "multiple purchases are made fast enough or before landing",
        outcome: "they are batched into one shipment",
    },
    ShipRule {
        condition: "the delivery vehicle lands",
        outcome: "the crew has 30 seconds to retrieve the items",
    },
    ShipRule {
        condition: "30 seconds elapse after landing",
        outcome: "the vehicle leaves regardless of recovery progress",
    },
    ShipRule {
        condition: "items remain undelivered",
        outcome: "the company does not refund the shipment",
    },
    ShipRule {
        condition: "an order contains more than 12 items",
        outcome: "the delivery is split across multiple vehicles",
    },
    ShipRule {
        condition: "there is a delay between multiple purchases",
        outcome: "the delivery is split across multiple vehicles",
    },
    ShipRule {
        condition: "a daily discount is applied",
        outcome: "the discount severity and the number of discounted items vary by day, and almost every item can be discounted",
    },
    ShipRule {
        condition: "an item is discounted or new decor enters the rotation",
        outcome: "an advertisement may appear on the player's visor",
    },
    ShipRule {
        condition: "a ship upgrade or most decor is purchased",
        outcome: "it remains available until the crew is terminated or the run ends",
    },
];

pub struct ShipSystemPlugin;

impl Plugin for ShipSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShipSystemCargoLoadedEvent>()
            .add_event::<ShipSystemPurchaseAppliedEvent>()
            .add_event::<ShipSystemMissionCompletedEvent>()
            .add_event::<ShipSystemDropshipBatchResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    ship_system_confirm_destination,
                    ship_system_pull_front_lever,
                    ship_system_tick_mission_clock,
                    ship_system_update_main_door,
                    ship_system_update_cooldowns,
                    ship_system_update_dropships,
                    ship_system_apply_store_purchases,
                    ship_system_load_cargo,
                    ship_system_apply_upgrade_effects,
                    ship_system_use_ship_devices,
                    ship_system_handle_departure_requests,
                    ship_system_handle_all_crew_dead,
                    ship_system_complete_departure,
                    ship_system_checksum,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShipRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShipLocation {
    #[default]
    Orbit = 0,
    Landing = 1,
    Landed = 2,
    Departing = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShipCrewLocation {
    #[default]
    OffShip = 0,
    Interior = 1,
    Railing = 2,
    Roof = 3,
    Catwalk = 4,
    Facility = 5,
    Dead = 6,
}

impl ShipCrewLocation {
    pub const fn counts_as_aboard(self) -> bool {
        matches!(
            self,
            Self::Interior | Self::Railing | Self::Roof | Self::Catwalk
        )
    }

    pub const fn is_exterior_ship_surface(self) -> bool {
        matches!(self, Self::Railing | Self::Roof | Self::Catwalk)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShipUpgradeKind {
    #[default]
    LoudHorn = 0,
    Teleporter = 1,
    InverseTeleporter = 2,
    SignalTranslator = 3,
    CozyLights = 4,
    Decor = 5,
    Equipment = 6,
}

impl ShipUpgradeKind {
    pub const fn retained_after_crew_wipe(self) -> bool {
        matches!(
            self,
            Self::LoudHorn
                | Self::Teleporter
                | Self::InverseTeleporter
                | Self::SignalTranslator
                | Self::CozyLights
                | Self::Decor
                | Self::Equipment
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShipCargoKind {
    #[default]
    Scrap = 0,
    Equipment = 1,
    Decor = 2,
    Upgrade = 3,
}

impl ShipCargoKind {
    pub const fn lost_when_all_crew_dead(self) -> bool {
        matches!(self, Self::Scrap)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShipDepartureReason {
    #[default]
    Scheduled = 0,
    BrakeLever = 1,
    LeaveVote = 2,
    AllCrewDead = 3,
    NoCrewAboardAtMidnight = 4,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShipCrewRecord {
    pub employee_id: u64,
    pub alive: bool,
    pub location: ShipCrewLocation,
    pub carried_scrap_count: u16,
    pub carried_equipment_count: u16,
    pub monitored: bool,
    pub standing_on_inverse_pad: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShipCargoRecord {
    pub cargo_id: u64,
    pub item_id: &'static str,
    pub kind: ShipCargoKind,
    pub value: I64F64,
    pub saved_on_ship: bool,
    pub delivered: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShipUpgradeRecord {
    pub upgrade_id: &'static str,
    pub kind: ShipUpgradeKind,
    pub installed: bool,
    pub stored: bool,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShipDropshipBatch {
    pub batch_id: u64,
    pub item_count: u16,
    pub landed: bool,
    pub retrieval_ticks_remaining: u32,
    pub unrecovered_items: u16,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct TheShipState {
    pub location: ShipLocation,
    pub confirmed_destination: Option<&'static str>,
    pub mission_clock_tick: u64,
    pub has_departed_this_day: bool,
    pub departure_reason: ShipDepartureReason,
    pub door_powered: bool,
    pub main_door_closed: bool,
    pub main_door_pressure_ticks: u16,
    pub interior_lights_on: bool,
    pub persistent_fixture_lights_on: bool,
    pub teleporter_cooldown_ticks: u32,
    pub inverse_teleporter_cooldown_ticks: u32,
    pub credits_retained: bool,
    pub saved_scrap_lost: bool,
    pub living_crew_warned: bool,
    pub mission_completed: bool,
    pub cargo: BTreeMap<u64, ShipCargoRecord>,
    pub upgrades: BTreeMap<&'static str, ShipUpgradeRecord>,
    pub crew: BTreeMap<u64, ShipCrewRecord>,
    pub dropship_batches: BTreeMap<u64, ShipDropshipBatch>,
    pub next_dropship_batch_id: u64,
    pub loaded_scrap_value: I64F64,
    pub loaded_scrap_count: u64,
    pub retained_upgrade_count: u64,
    pub departed_days: u64,
    pub loud_horn_uses: u64,
    pub teleporter_uses: u64,
    pub inverse_teleporter_uses: u64,
    pub signal_translator_messages: u64,
}

impl Default for TheShipState {
    fn default() -> Self {
        let mut upgrades = BTreeMap::new();
        upgrades.insert(
            "loud_horn",
            ShipUpgradeRecord {
                upgrade_id: "loud_horn",
                kind: ShipUpgradeKind::LoudHorn,
                installed: false,
                stored: false,
                active: false,
            },
        );
        upgrades.insert(
            "teleporter",
            ShipUpgradeRecord {
                upgrade_id: "teleporter",
                kind: ShipUpgradeKind::Teleporter,
                installed: false,
                stored: false,
                active: false,
            },
        );
        upgrades.insert(
            "inverse_teleporter",
            ShipUpgradeRecord {
                upgrade_id: "inverse_teleporter",
                kind: ShipUpgradeKind::InverseTeleporter,
                installed: false,
                stored: false,
                active: false,
            },
        );
        upgrades.insert(
            "signal_translator",
            ShipUpgradeRecord {
                upgrade_id: "signal_translator",
                kind: ShipUpgradeKind::SignalTranslator,
                installed: false,
                stored: false,
                active: false,
            },
        );
        upgrades.insert(
            "cozy_lights",
            ShipUpgradeRecord {
                upgrade_id: "cozy_lights",
                kind: ShipUpgradeKind::CozyLights,
                installed: false,
                stored: false,
                active: true,
            },
        );

        Self {
            location: ShipLocation::Orbit,
            confirmed_destination: None,
            mission_clock_tick: 0,
            has_departed_this_day: false,
            departure_reason: ShipDepartureReason::Scheduled,
            door_powered: true,
            main_door_closed: true,
            main_door_pressure_ticks: MAIN_DOOR_DRAIN_TICKS,
            interior_lights_on: true,
            persistent_fixture_lights_on: true,
            teleporter_cooldown_ticks: 0,
            inverse_teleporter_cooldown_ticks: 0,
            credits_retained: true,
            saved_scrap_lost: false,
            living_crew_warned: false,
            mission_completed: false,
            cargo: BTreeMap::new(),
            upgrades,
            crew: BTreeMap::new(),
            dropship_batches: BTreeMap::new(),
            next_dropship_batch_id: 1,
            loaded_scrap_value: I64F64::ZERO,
            loaded_scrap_count: 0,
            retained_upgrade_count: 0,
            departed_days: 0,
            loud_horn_uses: 0,
            teleporter_uses: 0,
            inverse_teleporter_uses: 0,
            signal_translator_messages: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipDestinationConfirmedEvent {
    pub moon_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipFrontLeverPulledEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipBrakeLeverDisengagedEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipLeaveVoteSucceededEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipAllCrewDeadEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipCrewLocationChangedEvent {
    pub employee_id: u64,
    pub alive: bool,
    pub location: ShipCrewLocation,
    pub carried_scrap_count: u16,
    pub carried_equipment_count: u16,
    pub monitored: bool,
    pub standing_on_inverse_pad: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipMainDoorPowerEvent {
    pub powered: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipMainDoorToggleEvent {
    pub closed: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipLightSwitchToggledEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipLoudHornActivatedEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipTeleporterUsedEvent {
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipInverseTeleporterUsedEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipSignalTranslatorCommandEvent {
    pub message: [u8; SIGNAL_TRANSLATOR_MESSAGE_CHARS],
    pub len: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipSignalTranslatedEvent {
    pub message: [u8; SIGNAL_TRANSLATOR_MESSAGE_CHARS],
    pub len: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipFurnishingCommandEvent {
    pub furnishing_id: &'static str,
    pub hidden: bool,
    pub moved_with_b: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipDepartureEvent {
    pub reason: ShipDepartureReason,
    pub living_crew_warned: bool,
    pub saved_scrap_lost: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipSystemCargoLoadedEvent {
    pub cargo_id: u64,
    pub item_id: &'static str,
    pub kind: ShipCargoKind,
    pub value: I64F64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipSystemPurchaseAppliedEvent {
    pub purchase_id: u64,
    pub item_id: &'static str,
    pub kind: ShipUpgradeKind,
    pub quantity: u16,
    pub from_orbit: bool,
    pub delayed_from_previous_purchase: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipSystemDropshipBatchResolvedEvent {
    pub batch_id: u64,
    pub item_count: u16,
    pub split_for_capacity: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipSystemMissionCompletedEvent {
    pub reason: ShipDepartureReason,
    pub scrap_value_loaded: I64F64,
    pub scrap_count_loaded: u64,
    pub retained_upgrade_count: u64,
}

fn ship_system_confirm_destination(
    mut state: ResMut<TheShipState>,
    mut events: EventReader<TheShipDestinationConfirmedEvent>,
) {
    for event in events.read() {
        state.confirmed_destination = Some(event.moon_id);
    }
}

fn ship_system_pull_front_lever(
    mut state: ResMut<TheShipState>,
    mut events: EventReader<TheShipFrontLeverPulledEvent>,
) {
    for _event in events.read() {
        if state.location == ShipLocation::Orbit && state.confirmed_destination.is_some() {
            state.location = ShipLocation::Landed;
            state.mission_clock_tick = SHIP_LANDING_TIME_TICK;
            state.has_departed_this_day = false;
            state.departure_reason = ShipDepartureReason::Scheduled;
            state.mission_completed = false;
            state.saved_scrap_lost = false;
            state.living_crew_warned = false;
        }
    }
}

fn ship_system_tick_mission_clock(
    mut state: ResMut<TheShipState>,
    mut departure_events: EventWriter<TheShipDepartureEvent>,
) {
    if state.location != ShipLocation::Landed {
        return;
    }

    state.mission_clock_tick = state.mission_clock_tick.saturating_add(1);

    if state.mission_clock_tick >= SHIP_DEPARTURE_TIME_TICK {
        let any_crew_aboard = state
            .crew
            .values()
            .any(|crew| crew.alive && crew.location.counts_as_aboard());

        let reason = if any_crew_aboard {
            ShipDepartureReason::Scheduled
        } else {
            ShipDepartureReason::NoCrewAboardAtMidnight
        };

        begin_departure(&mut state, reason, &mut departure_events);
    }
}

fn ship_system_update_main_door(
    mut state: ResMut<TheShipState>,
    mut power_events: EventReader<TheShipMainDoorPowerEvent>,
    mut toggle_events: EventReader<TheShipMainDoorToggleEvent>,
) {
    for event in power_events.read() {
        state.door_powered = event.powered;
    }

    for event in toggle_events.read() {
        if state.door_powered {
            state.main_door_closed = event.closed;
        }
    }

    if !state.door_powered {
        return;
    }

    if state.main_door_closed {
        state.main_door_pressure_ticks = state.main_door_pressure_ticks.saturating_sub(1);
        if state.main_door_pressure_ticks == 0 {
            state.main_door_closed = false;
        }
    } else {
        state.main_door_pressure_ticks = state
            .main_door_pressure_ticks
            .saturating_add(MAIN_DOOR_REFILL_STEP)
            .min(MAIN_DOOR_DRAIN_TICKS);
    }
}

fn ship_system_update_cooldowns(mut state: ResMut<TheShipState>) {
    state.teleporter_cooldown_ticks = state.teleporter_cooldown_ticks.saturating_sub(1);
    state.inverse_teleporter_cooldown_ticks =
        state.inverse_teleporter_cooldown_ticks.saturating_sub(1);
}

fn ship_system_update_dropships(
    mut state: ResMut<TheShipState>,
    mut resolved: EventWriter<ShipSystemDropshipBatchResolvedEvent>,
) {
    let ship_landed = state.location == ShipLocation::Landed;

    for batch in state.dropship_batches.values_mut() {
        if !batch.landed && ship_landed {
            batch.landed = true;
            batch.retrieval_ticks_remaining = DROPSHIP_RETRIEVAL_TICKS;
        }

        if batch.landed && batch.retrieval_ticks_remaining > 0 {
            batch.retrieval_ticks_remaining = batch.retrieval_ticks_remaining.saturating_sub(1);
        }

        if batch.landed && batch.retrieval_ticks_remaining == 0 && batch.unrecovered_items > 0 {
            batch.unrecovered_items = 0;
            resolved.send(ShipSystemDropshipBatchResolvedEvent {
                batch_id: batch.batch_id,
                item_count: batch.item_count,
                split_for_capacity: batch.item_count > STORE_MAX_ITEMS_PER_DROPSHIP,
            });
        }
    }
}

fn ship_system_apply_store_purchases(
    mut state: ResMut<TheShipState>,
    mut purchases: EventReader<ShipSystemPurchaseAppliedEvent>,
) {
    for purchase in purchases.read() {
        let mut remaining = purchase.quantity;
        let ship_landed = state.location == ShipLocation::Landed;
        let batch_landed = ship_landed && !purchase.from_orbit;
        let retrieval_ticks_remaining = if batch_landed {
            DROPSHIP_RETRIEVAL_TICKS
        } else {
            0
        };

        while remaining > 0 {
            let batch_count = remaining.min(STORE_MAX_ITEMS_PER_DROPSHIP);
            let batch_id = state.next_dropship_batch_id;
            state.next_dropship_batch_id = state.next_dropship_batch_id.wrapping_add(1);

            state.dropship_batches.insert(
                batch_id,
                ShipDropshipBatch {
                    batch_id,
                    item_count: batch_count,
                    landed: batch_landed,
                    retrieval_ticks_remaining,
                    unrecovered_items: batch_count,
                },
            );

            remaining -= batch_count;
        }

        let record = ShipUpgradeRecord {
            upgrade_id: purchase.item_id,
            kind: purchase.kind,
            installed: true,
            stored: false,
            active: purchase.kind != ShipUpgradeKind::Decor,
        };

        if purchase.kind != ShipUpgradeKind::Equipment {
            state.upgrades.insert(purchase.item_id, record);
        }
    }
}

fn ship_system_load_cargo(
    mut state: ResMut<TheShipState>,
    mut events: EventReader<ShipSystemCargoLoadedEvent>,
) {
    for event in events.read() {
        let record = ShipCargoRecord {
            cargo_id: event.cargo_id,
            item_id: event.item_id,
            kind: event.kind,
            value: event.value,
            saved_on_ship: true,
            delivered: true,
        };

        if event.kind == ShipCargoKind::Scrap && !state.cargo.contains_key(&event.cargo_id) {
            state.loaded_scrap_count = state.loaded_scrap_count.saturating_add(1);
            state.loaded_scrap_value = state.loaded_scrap_value.saturating_add(event.value);
        }

        state.cargo.insert(event.cargo_id, record);
    }
}

fn ship_system_apply_upgrade_effects(
    mut state: ResMut<TheShipState>,
    mut light_events: EventReader<TheShipLightSwitchToggledEvent>,
    mut furnishing_events: EventReader<TheShipFurnishingCommandEvent>,
) {
    for _event in light_events.read() {
        state.interior_lights_on = !state.interior_lights_on;
        state.persistent_fixture_lights_on = state
            .upgrades
            .get("cozy_lights")
            .map(|upgrade| upgrade.installed || upgrade.active)
            .unwrap_or(true);
    }

    for event in furnishing_events.read() {
        if matches!(state.location, ShipLocation::Landing | ShipLocation::Departing) {
            continue;
        }

        let kind = if event.furnishing_id == "cozy_lights" {
            ShipUpgradeKind::CozyLights
        } else {
            ShipUpgradeKind::Decor
        };

        state.upgrades.insert(
            event.furnishing_id,
            ShipUpgradeRecord {
                upgrade_id: event.furnishing_id,
                kind,
                installed: !event.hidden,
                stored: event.hidden && event.furnishing_id != "cozy_lights",
                active: !event.hidden,
            },
        );
    }
}

fn ship_system_use_ship_devices(
    mut state: ResMut<TheShipState>,
    mut horn_events: EventReader<TheShipLoudHornActivatedEvent>,
    mut teleporter_events: EventReader<TheShipTeleporterUsedEvent>,
    mut inverse_events: EventReader<TheShipInverseTeleporterUsedEvent>,
    mut signal_commands: EventReader<TheShipSignalTranslatorCommandEvent>,
    mut signal_translated: EventWriter<TheShipSignalTranslatedEvent>,
) {
    for _event in horn_events.read() {
        if upgrade_installed(&state, "loud_horn") {
            state.loud_horn_uses = state.loud_horn_uses.saturating_add(1);
        }
    }

    for event in teleporter_events.read() {
        if !upgrade_installed(&state, "teleporter") || state.teleporter_cooldown_ticks > 0 {
            continue;
        }

        if let Some(crew) = state.crew.get_mut(&event.employee_id) {
            if crew.alive && crew.monitored {
                crew.location = ShipCrewLocation::Interior;
                crew.carried_scrap_count = 0;
                crew.carried_equipment_count = 0;
                state.teleporter_cooldown_ticks = TELEPORTER_COOLDOWN_TICKS;
                state.teleporter_uses = state.teleporter_uses.saturating_add(1);
            }
        }
    }

    for _event in inverse_events.read() {
        if !upgrade_installed(&state, "inverse_teleporter")
            || state.inverse_teleporter_cooldown_ticks > 0
        {
            continue;
        }

        for crew in state.crew.values_mut() {
            if crew.alive && crew.standing_on_inverse_pad {
                crew.location = ShipCrewLocation::Facility;
                crew.carried_scrap_count = 0;
                crew.carried_equipment_count = 0;
            }
        }

        state.inverse_teleporter_cooldown_ticks = INVERSE_TELEPORTER_COOLDOWN_TICKS;
        state.inverse_teleporter_uses = state.inverse_teleporter_uses.saturating_add(1);
    }

    for event in signal_commands.read() {
        if upgrade_installed(&state, "signal_translator") {
            let len = event.len.min(SIGNAL_TRANSLATOR_MESSAGE_CHARS as u8);
            signal_translated.send(TheShipSignalTranslatedEvent {
                message: event.message,
                len,
            });
            state.signal_translator_messages = state.signal_translator_messages.saturating_add(1);
        }
    }
}

fn ship_system_handle_departure_requests(
    mut state: ResMut<TheShipState>,
    mut brake_events: EventReader<TheShipBrakeLeverDisengagedEvent>,
    mut vote_events: EventReader<TheShipLeaveVoteSucceededEvent>,
    mut crew_location_events: EventReader<TheShipCrewLocationChangedEvent>,
    mut departure_events: EventWriter<TheShipDepartureEvent>,
) {
    for event in crew_location_events.read() {
        state.crew.insert(
            event.employee_id,
            ShipCrewRecord {
                employee_id: event.employee_id,
                alive: event.alive,
                location: event.location,
                carried_scrap_count: event.carried_scrap_count,
                carried_equipment_count: event.carried_equipment_count,
                monitored: event.monitored,
                standing_on_inverse_pad: event.standing_on_inverse_pad,
            },
        );
    }

    for _event in brake_events.read() {
        begin_departure(
            &mut state,
            ShipDepartureReason::BrakeLever,
            &mut departure_events,
        );
    }

    for _event in vote_events.read() {
        begin_departure(
            &mut state,
            ShipDepartureReason::LeaveVote,
            &mut departure_events,
        );
    }
}

fn ship_system_handle_all_crew_dead(
    mut state: ResMut<TheShipState>,
    mut events: EventReader<TheShipAllCrewDeadEvent>,
    mut departure_events: EventWriter<TheShipDepartureEvent>,
) {
    for _event in events.read() {
        begin_departure(
            &mut state,
            ShipDepartureReason::AllCrewDead,
            &mut departure_events,
        );
    }
}

fn ship_system_complete_departure(
    mut state: ResMut<TheShipState>,
    mut completed: EventWriter<ShipSystemMissionCompletedEvent>,
) {
    if state.location != ShipLocation::Departing || state.mission_completed {
        return;
    }

    for crew in state.crew.values_mut() {
        if crew.alive && crew.location.is_exterior_ship_surface() {
            crew.location = ShipCrewLocation::Interior;
        }
    }

    let mut retained_upgrade_count = 0_u64;
    for upgrade in state.upgrades.values() {
        if upgrade.installed && upgrade.kind.retained_after_crew_wipe() {
            retained_upgrade_count = retained_upgrade_count.saturating_add(1);
        }
    }
    state.retained_upgrade_count = retained_upgrade_count;

    if state.departure_reason == ShipDepartureReason::AllCrewDead {
        for cargo in state.cargo.values_mut() {
            if cargo.kind.lost_when_all_crew_dead() {
                cargo.saved_on_ship = false;
            }
        }
    }

    state.location = ShipLocation::Orbit;
    state.confirmed_destination = None;
    state.has_departed_this_day = true;
    state.departed_days = state.departed_days.saturating_add(1);
    state.mission_completed = true;

    completed.send(ShipSystemMissionCompletedEvent {
        reason: state.departure_reason,
        scrap_value_loaded: state.loaded_scrap_value,
        scrap_count_loaded: state.loaded_scrap_count,
        retained_upgrade_count: state.retained_upgrade_count,
    });
}

fn begin_departure(
    state: &mut TheShipState,
    reason: ShipDepartureReason,
    departure_events: &mut EventWriter<TheShipDepartureEvent>,
) {
    if matches!(state.location, ShipLocation::Orbit | ShipLocation::Departing) {
        return;
    }

    state.location = ShipLocation::Departing;
    state.departure_reason = reason;
    state.living_crew_warned = reason == ShipDepartureReason::LeaveVote;
    state.saved_scrap_lost = matches!(
        reason,
        ShipDepartureReason::AllCrewDead | ShipDepartureReason::NoCrewAboardAtMidnight
    );

    departure_events.send(TheShipDepartureEvent {
        reason,
        living_crew_warned: state.living_crew_warned,
        saved_scrap_lost: state.saved_scrap_lost,
    });
}

fn upgrade_installed(state: &TheShipState, upgrade_id: &'static str) -> bool {
    state
        .upgrades
        .get(upgrade_id)
        .map(|upgrade| upgrade.installed && !upgrade.stored)
        .unwrap_or(false)
}

fn ship_system_checksum(
    state: Res<TheShipState>,
    tick: Res<SimTick>,
    mut checksum: ResMut<SimChecksumState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(SHIP_SYSTEM_ID.len() as u64);
    checksum.accumulate(state.location as u64);
    checksum.accumulate(state.mission_clock_tick);
    checksum.accumulate(state.has_departed_this_day as u64);
    checksum.accumulate(state.departure_reason as u64);
    checksum.accumulate(state.door_powered as u64);
    checksum.accumulate(state.main_door_closed as u64);
    checksum.accumulate(state.main_door_pressure_ticks as u64);
    checksum.accumulate(state.interior_lights_on as u64);
    checksum.accumulate(state.persistent_fixture_lights_on as u64);
    checksum.accumulate(state.teleporter_cooldown_ticks as u64);
    checksum.accumulate(state.inverse_teleporter_cooldown_ticks as u64);
    checksum.accumulate(state.credits_retained as u64);
    checksum.accumulate(state.saved_scrap_lost as u64);
    checksum.accumulate(state.living_crew_warned as u64);
    checksum.accumulate(state.mission_completed as u64);
    checksum.accumulate(state.loaded_scrap_value.to_bits() as u64);
    checksum.accumulate(state.loaded_scrap_count);
    checksum.accumulate(state.retained_upgrade_count);
    checksum.accumulate(state.departed_days);
    checksum.accumulate(state.loud_horn_uses);
    checksum.accumulate(state.teleporter_uses);
    checksum.accumulate(state.inverse_teleporter_uses);
    checksum.accumulate(state.signal_translator_messages);

    for (cargo_id, cargo) in state.cargo.iter() {
        checksum.accumulate(*cargo_id);
        checksum.accumulate(cargo.kind as u64);
        checksum.accumulate(cargo.value.to_bits() as u64);
        checksum.accumulate(cargo.saved_on_ship as u64);
        checksum.accumulate(cargo.delivered as u64);
        checksum.accumulate(cargo.item_id.len() as u64);
    }

    for (upgrade_id, upgrade) in state.upgrades.iter() {
        checksum.accumulate(upgrade_id.len() as u64);
        checksum.accumulate(upgrade.kind as u64);
        checksum.accumulate(upgrade.installed as u64);
        checksum.accumulate(upgrade.stored as u64);
        checksum.accumulate(upgrade.active as u64);
    }

    for (employee_id, crew) in state.crew.iter() {
        checksum.accumulate(*employee_id);
        checksum.accumulate(crew.alive as u64);
        checksum.accumulate(crew.location as u64);
        checksum.accumulate(crew.carried_scrap_count as u64);
        checksum.accumulate(crew.carried_equipment_count as u64);
        checksum.accumulate(crew.monitored as u64);
        checksum.accumulate(crew.standing_on_inverse_pad as u64);
    }

    for (batch_id, batch) in state.dropship_batches.iter() {
        checksum.accumulate(*batch_id);
        checksum.accumulate(batch.item_count as u64);
        checksum.accumulate(batch.landed as u64);
        checksum.accumulate(batch.retrieval_ticks_remaining as u64);
        checksum.accumulate(batch.unrecovered_items as u64);
    }
}