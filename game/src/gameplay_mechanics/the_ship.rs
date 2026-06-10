// Sources: vault/gameplay_mechanics/the_ship.md
use bevy::prelude::*;
use fixed::types::{I16F16, I32F32};

use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const THE_SHIP_ID: &str = "the_ship";
pub const THE_SHIP_NAME: &str = "The Ship";
pub const THE_SHIP_TYPE: &str = "gameplay_mechanics";
pub const THE_SHIP_SUBTYPE: &str = "ship";
pub const THE_SHIP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/The_Ship";
pub const THE_SHIP_SOURCE_REVISION: u32 = 21375;
pub const THE_SHIP_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const THE_SHIP_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const THE_SHIP_LANDING_HOUR: u8 = 8;
pub const THE_SHIP_LANDING_MINUTE: u8 = 0;
pub const THE_SHIP_DEPARTURE_HOUR: u8 = 23;
pub const THE_SHIP_DEPARTURE_MINUTE: u8 = 57;
pub const THE_SHIP_MAIN_DOOR_LOSS_SECONDS: u32 = 30;
pub const THE_SHIP_MAIN_DOOR_REFILL_SECONDS: u32 = 6;
pub const THE_SHIP_TELEPORTER_COOLDOWN_SECONDS: u32 = 10;
pub const THE_SHIP_INVERSE_TELEPORTER_COOLDOWN_SECONDS: u32 = 300;
pub const THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS: usize = 9;
pub const THE_SHIP_SIGNAL_TRANSLATOR_STATED_CHARACTERS: usize = 10;

pub const THE_SHIP_DEPENDS_ON: [&str; 15] = [
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
    "moon",
    "facility",
    "cozy_lights",
];

pub const THE_SHIP_OVERVIEW: &str = "Crew mobile base between moons with the terminal, monitors, storage for scrap and equipment, and room for ship_upgrades.";

pub const THE_SHIP_FRONTMATTER_RULES: [&str; 3] = [
    "The ship is the crew's only transport between moons.",
    "The terminal controls travel, shopping, scanning, and ship systems.",
    "The main pressure door opens when pressure reaches 0% and regains pressure in about 6 seconds.",
];

pub const THE_SHIP_FRONTMATTER_MODIFIERS: [&str; 4] = [
    "The loud_horn can be heard everywhere.",
    "The teleporter drops carried scrap and equipment and has a 10-second cooldown.",
    "The inverse_teleporter drops carried scrap and equipment, sends nearby employees to a random facility location, and has a 5-minute cooldown.",
    "The signal_translator transmits a nine-character message, stated to be ten characters.",
];

pub const THE_SHIP_FRONTMATTER_STRATEGY: [&str; 3] = [
    "Use the terminal to set the next moon before the ship departs.",
    "Use the teleporter to bring a monitored employee back to the ship when the body is still intact.",
    "Use the loud_horn to call teammates back or draw outdoor threats toward the ship.",
];

pub const THE_SHIP_FRONTMATTER_NOTES: [&str; 3] = [
    "Furniture can be moved with B and hidden with X, but not while the ship is arriving or departing.",
    "On departure, employees on the roof or catwalk are pulled inside before the ship leaves.",
    "Some lights stay visible when the ship lights are off, including cozy_lights.",
];

pub const THE_SHIP_BEHAVIORAL_MECHANICS: [TheShipBehaviorRule; 15] = [
    TheShipBehaviorRule {
        condition: "a destination moon is confirmed in the terminal and the front lever is pulled",
        outcome: "the ship lands at 8:00 am and departs at 11:57 pm",
    },
    TheShipBehaviorRule {
        condition: "the brake lever is disengaged",
        outcome: "the ship departs early even if crew members are not on board",
    },
    TheShipBehaviorRule {
        condition: "a unanimous vote to leave succeeds",
        outcome: "the ship departs early and living crew are warned that the exit is due to hazardous conditions",
    },
    TheShipBehaviorRule {
        condition: "all crew members die",
        outcome: "the ship departs automatically and retains equipment, ship upgrades, decor, and credits while saved scrap is lost",
    },
    TheShipBehaviorRule {
        condition: "a crew member is anywhere on the ship before the doors close",
        outcome: "they count as aboard even on the railing, roof, or catwalk",
    },
    TheShipBehaviorRule {
        condition: "the main pressure door has power",
        outcome: "it can stay closed, lose pressure over about 30 seconds, and refill in about 6 seconds",
    },
    TheShipBehaviorRule {
        condition: "main door pressure reaches 0%",
        outcome: "the door opens",
    },
    TheShipBehaviorRule {
        condition: "the light switch is toggled",
        outcome: "the ship's interior lights turn on or off while some fixtures remain lit",
    },
    TheShipBehaviorRule {
        condition: "the loud_horn is activated",
        outcome: "its sound can be heard everywhere and can draw outdoor creatures toward the front of the ship",
    },
    TheShipBehaviorRule {
        condition: "the teleporter is used",
        outcome: "one monitored employee returns to the ship, carried scrap and equipment drop, and the cooldown is 10 seconds",
    },
    TheShipBehaviorRule {
        condition: "the inverse_teleporter is used",
        outcome: "employees standing on its pad are sent to a random facility location, carried scrap and equipment drop on the ship, and the cooldown is 5 minutes",
    },
    TheShipBehaviorRule {
        condition: "the signal_translator receives TRANSMIT [message]",
        outcome: "it sends a nine-character message, stated to be ten characters",
    },
    TheShipBehaviorRule {
        condition: "movable furnishings are repositioned",
        outcome: "they can be hidden with X or moved with B, but not while the ship is arriving or departing",
    },
    TheShipBehaviorRule {
        condition: "ship departure",
        outcome: "employees on the roof or catwalk are pulled inside before the ship leaves",
    },
    TheShipBehaviorRule {
        condition: "some lights are still active after the ship lights go off",
        outcome: "cozy_lights and similar fixtures remain visible",
    },
];

const THE_SHIP_DOOR_FULL_PRESSURE_BITS: i32 = 100 << 16;
const THE_SHIP_DOOR_EMPTY_PRESSURE_BITS: i32 = 0;
const THE_SHIP_GLOBAL_NOISE_AMOUNT: i32 = 32_000;

pub struct TheShipPlugin;

impl Plugin for TheShipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TheShipState>()
            .add_event::<TheShipDestinationConfirmedEvent>()
            .add_event::<TheShipFrontLeverPulledEvent>()
            .add_event::<TheShipBrakeLeverDisengagedEvent>()
            .add_event::<TheShipLeaveVoteSucceededEvent>()
            .add_event::<TheShipAllCrewDeadEvent>()
            .add_event::<TheShipCrewLocationChangedEvent>()
            .add_event::<TheShipMainDoorPowerEvent>()
            .add_event::<TheShipMainDoorToggleEvent>()
            .add_event::<TheShipLightSwitchToggledEvent>()
            .add_event::<TheShipLoudHornActivatedEvent>()
            .add_event::<TheShipTeleporterUsedEvent>()
            .add_event::<TheShipInverseTeleporterUsedEvent>()
            .add_event::<TheShipSignalTranslatorCommandEvent>()
            .add_event::<TheShipSignalTranslatedEvent>()
            .add_event::<TheShipFurnishingCommandEvent>()
            .add_event::<TheShipDepartureEvent>()
            .add_systems(
                FixedUpdate,
                (
                    the_ship_confirm_destination,
                    the_ship_front_lever,
                    the_ship_early_departure_controls,
                    the_ship_all_crew_dead_departure,
                    the_ship_track_crew_location,
                    the_ship_main_door_power,
                    the_ship_main_door_toggle,
                    the_ship_update_main_door_pressure,
                    the_ship_light_switch,
                    the_ship_loud_horn,
                    the_ship_teleporter,
                    the_ship_inverse_teleporter,
                    the_ship_signal_translator,
                    the_ship_furnishing_command,
                    the_ship_departure_roof_pull,
                    the_ship_cooldowns,
                    the_ship_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TheShipBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TheShipPhase {
    InOrbit,
    Landing,
    Landed,
    Departing,
    Departed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TheShipCrewLocation {
    OffShip,
    Interior,
    Railing,
    Roof,
    Catwalk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TheShipFurnishingCommand {
    MoveWithB,
    HideWithX,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct TheShipState {
    pub phase: TheShipPhase,
    pub destination_moon_id: &'static str,
    pub destination_confirmed: bool,
    pub scheduled_landing_hour: u8,
    pub scheduled_landing_minute: u8,
    pub scheduled_departure_hour: u8,
    pub scheduled_departure_minute: u8,
    pub departed_early: bool,
    pub hazardous_departure_warning_sent: bool,
    pub retained_equipment_ship_upgrades_decor_and_credits: bool,
    pub saved_scrap_lost: bool,
    pub crew_aboard_count: u32,
    pub crew_on_roof_or_catwalk_count: u32,
    pub crew_pulled_inside_on_departure: u32,
    pub main_door_has_power: bool,
    pub main_door_closed: bool,
    pub main_door_pressure: I16F16,
    pub interior_lights_on: bool,
    pub cozy_lights_visible: bool,
    pub teleporter_cooldown_ticks_remaining: u32,
    pub inverse_teleporter_cooldown_ticks_remaining: u32,
    pub teleporter_uses: u64,
    pub inverse_teleporter_uses: u64,
    pub loud_horn_activations: u64,
    pub carried_scrap_and_equipment_drops: u64,
    pub last_monitored_employee_id: u64,
    pub last_inverse_pad_group_id: u64,
    pub signal_transmissions: u64,
    pub last_signal_message: [u8; THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS],
    pub last_signal_message_len: usize,
    pub furnishing_moves: u64,
    pub furnishing_hides: u64,
    pub furnishing_commands_rejected_while_arriving_or_departing: u64,
}

impl Default for TheShipState {
    fn default() -> Self {
        Self {
            phase: TheShipPhase::InOrbit,
            destination_moon_id: "",
            destination_confirmed: false,
            scheduled_landing_hour: THE_SHIP_LANDING_HOUR,
            scheduled_landing_minute: THE_SHIP_LANDING_MINUTE,
            scheduled_departure_hour: THE_SHIP_DEPARTURE_HOUR,
            scheduled_departure_minute: THE_SHIP_DEPARTURE_MINUTE,
            departed_early: false,
            hazardous_departure_warning_sent: false,
            retained_equipment_ship_upgrades_decor_and_credits: false,
            saved_scrap_lost: false,
            crew_aboard_count: 0,
            crew_on_roof_or_catwalk_count: 0,
            crew_pulled_inside_on_departure: 0,
            main_door_has_power: true,
            main_door_closed: true,
            main_door_pressure: I16F16::from_bits(THE_SHIP_DOOR_FULL_PRESSURE_BITS),
            interior_lights_on: true,
            cozy_lights_visible: true,
            teleporter_cooldown_ticks_remaining: 0,
            inverse_teleporter_cooldown_ticks_remaining: 0,
            teleporter_uses: 0,
            inverse_teleporter_uses: 0,
            loud_horn_activations: 0,
            carried_scrap_and_equipment_drops: 0,
            last_monitored_employee_id: 0,
            last_inverse_pad_group_id: 0,
            signal_transmissions: 0,
            last_signal_message: [0; THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS],
            last_signal_message_len: 0,
            furnishing_moves: 0,
            furnishing_hides: 0,
            furnishing_commands_rejected_while_arriving_or_departing: 0,
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
    pub location: TheShipCrewLocation,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipMainDoorPowerEvent {
    pub has_power: bool,
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
    pub monitored_employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipInverseTeleporterUsedEvent {
    pub pad_group_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipSignalTranslatorCommandEvent {
    pub message: [u8; THE_SHIP_SIGNAL_TRANSLATOR_STATED_CHARACTERS],
    pub message_len: usize,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipSignalTranslatedEvent {
    pub message: [u8; THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS],
    pub message_len: usize,
    pub stated_character_limit: usize,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipFurnishingCommandEvent {
    pub furnishing_id: u64,
    pub command: TheShipFurnishingCommand,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheShipDepartureEvent {
    pub early: bool,
}

pub fn the_ship_landing_time() -> (u8, u8) {
    (THE_SHIP_LANDING_HOUR, THE_SHIP_LANDING_MINUTE)
}

pub fn the_ship_departure_time() -> (u8, u8) {
    (THE_SHIP_DEPARTURE_HOUR, THE_SHIP_DEPARTURE_MINUTE)
}

pub fn the_ship_teleporter_cooldown_seconds() -> u32 {
    THE_SHIP_TELEPORTER_COOLDOWN_SECONDS
}

pub fn the_ship_inverse_teleporter_cooldown_seconds() -> u32 {
    THE_SHIP_INVERSE_TELEPORTER_COOLDOWN_SECONDS
}

fn the_ship_confirm_destination(
    mut events: EventReader<TheShipDestinationConfirmedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        state.destination_moon_id = event.moon_id;
        state.destination_confirmed = true;
    }
}

fn the_ship_front_lever(
    mut events: EventReader<TheShipFrontLeverPulledEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in events.read() {
        if state.destination_confirmed {
            state.phase = TheShipPhase::Landed;
            state.scheduled_landing_hour = THE_SHIP_LANDING_HOUR;
            state.scheduled_landing_minute = THE_SHIP_LANDING_MINUTE;
            state.scheduled_departure_hour = THE_SHIP_DEPARTURE_HOUR;
            state.scheduled_departure_minute = THE_SHIP_DEPARTURE_MINUTE;
        }
    }
}

fn the_ship_early_departure_controls(
    mut brake_events: EventReader<TheShipBrakeLeverDisengagedEvent>,
    mut vote_events: EventReader<TheShipLeaveVoteSucceededEvent>,
    mut departure_events: EventWriter<TheShipDepartureEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in brake_events.read() {
        state.phase = TheShipPhase::Departing;
        state.departed_early = true;
        departure_events.send(TheShipDepartureEvent { early: true });
    }

    for _event in vote_events.read() {
        state.phase = TheShipPhase::Departing;
        state.departed_early = true;
        state.hazardous_departure_warning_sent = true;
        departure_events.send(TheShipDepartureEvent { early: true });
    }
}

fn the_ship_all_crew_dead_departure(
    mut events: EventReader<TheShipAllCrewDeadEvent>,
    mut departure_events: EventWriter<TheShipDepartureEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in events.read() {
        state.phase = TheShipPhase::Departing;
        state.retained_equipment_ship_upgrades_decor_and_credits = true;
        state.saved_scrap_lost = true;
        departure_events.send(TheShipDepartureEvent { early: true });
    }
}

fn the_ship_track_crew_location(
    mut events: EventReader<TheShipCrewLocationChangedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        match event.location {
            TheShipCrewLocation::Interior
            | TheShipCrewLocation::Railing
            | TheShipCrewLocation::Roof
            | TheShipCrewLocation::Catwalk => {
                state.crew_aboard_count = state.crew_aboard_count.saturating_add(1);
            }
            TheShipCrewLocation::OffShip => {}
        }

        if matches!(
            event.location,
            TheShipCrewLocation::Roof | TheShipCrewLocation::Catwalk
        ) {
            state.crew_on_roof_or_catwalk_count =
                state.crew_on_roof_or_catwalk_count.saturating_add(1);
        }
    }
}

fn the_ship_main_door_power(
    mut events: EventReader<TheShipMainDoorPowerEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        state.main_door_has_power = event.has_power;
    }
}

fn the_ship_main_door_toggle(
    mut events: EventReader<TheShipMainDoorToggleEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        if state.main_door_has_power {
            state.main_door_closed = event.closed;
        }
    }
}

fn the_ship_update_main_door_pressure(mut state: ResMut<TheShipState>) {
    if !state.main_door_has_power {
        return;
    }

    let loss_ticks = I16F16::from_num(THE_SHIP_MAIN_DOOR_LOSS_SECONDS as i32 * 30);
    let refill_ticks = I16F16::from_num(THE_SHIP_MAIN_DOOR_REFILL_SECONDS as i32 * 30);
    let full_pressure = I16F16::from_bits(THE_SHIP_DOOR_FULL_PRESSURE_BITS);
    let empty_pressure = I16F16::from_bits(THE_SHIP_DOOR_EMPTY_PRESSURE_BITS);

    if state.main_door_closed {
        let loss_per_tick = full_pressure / loss_ticks;
        state.main_door_pressure -= loss_per_tick;

        if state.main_door_pressure <= empty_pressure {
            state.main_door_pressure = empty_pressure;
            state.main_door_closed = false;
        }
    } else {
        let refill_per_tick = full_pressure / refill_ticks;
        state.main_door_pressure += refill_per_tick;

        if state.main_door_pressure >= full_pressure {
            state.main_door_pressure = full_pressure;
        }
    }
}

fn the_ship_light_switch(
    mut events: EventReader<TheShipLightSwitchToggledEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in events.read() {
        state.interior_lights_on = !state.interior_lights_on;
        state.cozy_lights_visible = true;
    }
}

fn the_ship_loud_horn(
    mut events: EventReader<TheShipLoudHornActivatedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in events.read() {
        state.loud_horn_activations = state.loud_horn_activations.wrapping_add(1);
        noise_events.send(NoiseEmittedEvent {
            source: Entity::PLACEHOLDER,
            position: SimPosition {
                x: I32F32::from_num(0),
                y: I32F32::from_num(0),
            },
            amount: I32F32::from_num(THE_SHIP_GLOBAL_NOISE_AMOUNT),
        });
    }
}

fn the_ship_teleporter(
    mut events: EventReader<TheShipTeleporterUsedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        if state.teleporter_cooldown_ticks_remaining == 0 {
            state.teleporter_uses = state.teleporter_uses.wrapping_add(1);
            state.carried_scrap_and_equipment_drops =
                state.carried_scrap_and_equipment_drops.wrapping_add(1);
            state.last_monitored_employee_id = event.monitored_employee_id;
            state.teleporter_cooldown_ticks_remaining =
                THE_SHIP_TELEPORTER_COOLDOWN_SECONDS * 30;
        }
    }
}

fn the_ship_inverse_teleporter(
    mut events: EventReader<TheShipInverseTeleporterUsedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        if state.inverse_teleporter_cooldown_ticks_remaining == 0 {
            state.inverse_teleporter_uses = state.inverse_teleporter_uses.wrapping_add(1);
            state.carried_scrap_and_equipment_drops =
                state.carried_scrap_and_equipment_drops.wrapping_add(1);
            state.last_inverse_pad_group_id = event.pad_group_id;
            state.inverse_teleporter_cooldown_ticks_remaining =
                THE_SHIP_INVERSE_TELEPORTER_COOLDOWN_SECONDS * 30;
        }
    }
}

fn the_ship_signal_translator(
    mut commands: EventReader<TheShipSignalTranslatorCommandEvent>,
    mut translated: EventWriter<TheShipSignalTranslatedEvent>,
    mut state: ResMut<TheShipState>,
) {
    for command in commands.read() {
        let mut message = [0; THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS];
        let mut len = command.message_len;

        if len > THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS {
            len = THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS;
        }

        let mut index = 0;
        while index < len {
            message[index] = command.message[index];
            index += 1;
        }

        state.signal_transmissions = state.signal_transmissions.wrapping_add(1);
        state.last_signal_message = message;
        state.last_signal_message_len = len;

        translated.send(TheShipSignalTranslatedEvent {
            message,
            message_len: len,
            stated_character_limit: THE_SHIP_SIGNAL_TRANSLATOR_STATED_CHARACTERS,
        });
    }
}

fn the_ship_furnishing_command(
    mut events: EventReader<TheShipFurnishingCommandEvent>,
    mut state: ResMut<TheShipState>,
) {
    for event in events.read() {
        if matches!(state.phase, TheShipPhase::Landing | TheShipPhase::Departing) {
            state.furnishing_commands_rejected_while_arriving_or_departing = state
                .furnishing_commands_rejected_while_arriving_or_departing
                .wrapping_add(1);
            continue;
        }

        match event.command {
            TheShipFurnishingCommand::MoveWithB => {
                state.furnishing_moves = state.furnishing_moves.wrapping_add(1);
            }
            TheShipFurnishingCommand::HideWithX => {
                state.furnishing_hides = state.furnishing_hides.wrapping_add(1);
            }
        }
    }
}

fn the_ship_departure_roof_pull(
    mut events: EventReader<TheShipDepartureEvent>,
    mut state: ResMut<TheShipState>,
) {
    for _event in events.read() {
        state.crew_pulled_inside_on_departure = state
            .crew_pulled_inside_on_departure
            .saturating_add(state.crew_on_roof_or_catwalk_count);
        state.crew_on_roof_or_catwalk_count = 0;
        state.phase = TheShipPhase::Departed;
    }
}

fn the_ship_cooldowns(mut state: ResMut<TheShipState>) {
    state.teleporter_cooldown_ticks_remaining =
        state.teleporter_cooldown_ticks_remaining.saturating_sub(1);
    state.inverse_teleporter_cooldown_ticks_remaining = state
        .inverse_teleporter_cooldown_ticks_remaining
        .saturating_sub(1);
}

fn the_ship_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<TheShipState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(THE_SHIP_SOURCE_REVISION as u64);
    checksum.accumulate(THE_SHIP_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(THE_SHIP_LANDING_HOUR as u64);
    checksum.accumulate(THE_SHIP_LANDING_MINUTE as u64);
    checksum.accumulate(THE_SHIP_DEPARTURE_HOUR as u64);
    checksum.accumulate(THE_SHIP_DEPARTURE_MINUTE as u64);
    checksum.accumulate(THE_SHIP_MAIN_DOOR_LOSS_SECONDS as u64);
    checksum.accumulate(THE_SHIP_MAIN_DOOR_REFILL_SECONDS as u64);
    checksum.accumulate(THE_SHIP_TELEPORTER_COOLDOWN_SECONDS as u64);
    checksum.accumulate(THE_SHIP_INVERSE_TELEPORTER_COOLDOWN_SECONDS as u64);
    checksum.accumulate(THE_SHIP_SIGNAL_TRANSLATOR_CHARACTERS as u64);
    checksum.accumulate(THE_SHIP_SIGNAL_TRANSLATOR_STATED_CHARACTERS as u64);

    accumulate_str(&mut checksum, 0x0100, THE_SHIP_OVERVIEW);

    for dependency in THE_SHIP_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in THE_SHIP_FRONTMATTER_RULES {
        accumulate_str(&mut checksum, 0x1100, rule);
    }

    for modifier in THE_SHIP_FRONTMATTER_MODIFIERS {
        accumulate_str(&mut checksum, 0x1200, modifier);
    }

    for strategy in THE_SHIP_FRONTMATTER_STRATEGY {
        accumulate_str(&mut checksum, 0x1300, strategy);
    }

    for note in THE_SHIP_FRONTMATTER_NOTES {
        accumulate_str(&mut checksum, 0x1400, note);
    }

    for rule in THE_SHIP_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    checksum.accumulate(match state.phase {
        TheShipPhase::InOrbit => 0,
        TheShipPhase::Landing => 1,
        TheShipPhase::Landed => 2,
        TheShipPhase::Departing => 3,
        TheShipPhase::Departed => 4,
    });
    accumulate_str(&mut checksum, 0x3000, state.destination_moon_id);
    checksum.accumulate(state.destination_confirmed as u64);
    checksum.accumulate(state.scheduled_landing_hour as u64);
    checksum.accumulate(state.scheduled_landing_minute as u64);
    checksum.accumulate(state.scheduled_departure_hour as u64);
    checksum.accumulate(state.scheduled_departure_minute as u64);
    checksum.accumulate(state.departed_early as u64);
    checksum.accumulate(state.hazardous_departure_warning_sent as u64);
    checksum.accumulate(state.retained_equipment_ship_upgrades_decor_and_credits as u64);
    checksum.accumulate(state.saved_scrap_lost as u64);
    checksum.accumulate(state.crew_aboard_count as u64);
    checksum.accumulate(state.crew_on_roof_or_catwalk_count as u64);
    checksum.accumulate(state.crew_pulled_inside_on_departure as u64);
    checksum.accumulate(state.main_door_has_power as u64);
    checksum.accumulate(state.main_door_closed as u64);
    checksum.accumulate(state.main_door_pressure.to_bits() as u64);
    checksum.accumulate(state.interior_lights_on as u64);
    checksum.accumulate(state.cozy_lights_visible as u64);
    checksum.accumulate(state.teleporter_cooldown_ticks_remaining as u64);
    checksum.accumulate(state.inverse_teleporter_cooldown_ticks_remaining as u64);
    checksum.accumulate(state.teleporter_uses);
    checksum.accumulate(state.inverse_teleporter_uses);
    checksum.accumulate(state.loud_horn_activations);
    checksum.accumulate(state.carried_scrap_and_equipment_drops);
    checksum.accumulate(state.last_monitored_employee_id);
    checksum.accumulate(state.last_inverse_pad_group_id);
    checksum.accumulate(state.signal_transmissions);
    checksum.accumulate(state.last_signal_message_len as u64);

    for (index, byte) in state.last_signal_message.iter().enumerate() {
        checksum.accumulate(0x4000 ^ ((index as u64) << 8) ^ *byte as u64);
    }

    checksum.accumulate(state.furnishing_moves);
    checksum.accumulate(state.furnishing_hides);
    checksum.accumulate(state.furnishing_commands_rejected_while_arriving_or_departing);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}