// Sources: vault/gameplay_mechanics/terminal.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const TERMINAL_ID: &str = "terminal";
pub const TERMINAL_NAME: &str = "Terminal";
pub const TERMINAL_TYPE: &str = "gameplay_mechanics";
pub const TERMINAL_SUBTYPE: &str = "terminal";
pub const TERMINAL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Terminal";
pub const TERMINAL_SOURCE_REVISION: u32 = 21482;
pub const TERMINAL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const TERMINAL_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const TERMINAL_OVERVIEW: &str = "The [[terminal]] is the ship's command computer inside the [[autopilot_ship]]. It handles routing to [[exomoon]]s, buying from the [[company_store]], creature lookup in the [[bestiary]], log access in [[sigurds_logs]], and ship utilities such as [[radar_booster]] control and [[signal_translator]] messaging.";

pub const TERMINAL_HELP_GROUPS: [&str; 5] = ["MOONS", "STORE", "BESTIARY", "STORAGE", "OTHER"];
pub const TERMINAL_TRANSMIT_MAX_CHARS: u8 = 9;
pub const TERMINAL_RESET_CREDITS_PROFILE_VALUE: u32 = 2500;
pub const TERMINAL_RESET_CREDITS_CALLBACK: u16 = 200;
pub const TERMINAL_SCAN_HOST_NUMERATOR: u32 = 41;
pub const TERMINAL_SCAN_NON_HOST_NUMERATOR: u32 = 50;
pub const TERMINAL_SCAN_DENOMINATOR: u32 = 100;

pub const TERMINAL_DEPENDS_ON: [&str; 13] = [
    "autopilot_ship",
    "bestiary",
    "breaker_box",
    "company_store",
    "exomoon",
    "landmine",
    "profit_quota",
    "radar_booster",
    "secure_door",
    "signal_translator",
    "sigurds_logs",
    "turret",
    "terminal",
];

pub const TERMINAL_BEHAVIORAL_MECHANICS: [TerminalBehaviorRule; 14] = [
    TerminalBehaviorRule {
        condition: "`HELP` is used",
        outcome: "the terminal exposes the command groups `MOONS`, `STORE`, `BESTIARY`, `STORAGE`, and `OTHER`",
    },
    TerminalBehaviorRule {
        condition: "`MOONS` is used",
        outcome: "the terminal shows the moon catalogue and requires `CONFIRM` or `DENY` before routing the [[autopilot_ship]] to orbit a selected [[exomoon]]",
    },
    TerminalBehaviorRule {
        condition: "`STORE` or `BUY` is used",
        outcome: "the terminal opens the [[company_store]] catalogue and can place an order after `CONFIRM` or cancel with `DENY`",
    },
    TerminalBehaviorRule {
        condition: "`BESTIARY` is used",
        outcome: "the terminal opens the wildlife record list for entries that have been scanned",
    },
    TerminalBehaviorRule {
        condition: "`STORAGE` is used",
        outcome: "the terminal shows ship decor and upgrades that have been moved into storage",
    },
    TerminalBehaviorRule {
        condition: "`VIEW MONITOR` is used",
        outcome: "the terminal toggles the main monitor map cam on and off",
    },
    TerminalBehaviorRule {
        condition: "a 2-digit special code is entered",
        outcome: "the terminal can toggle [[secure_door]]s or temporarily disable [[turret]]s and [[landmine]]s",
    },
    TerminalBehaviorRule {
        condition: "`FLASH` is used on a [[radar_booster]]",
        outcome: "the booster emits a flash that stuns nearby creatures and blinds crew mates looking at it",
    },
    TerminalBehaviorRule {
        condition: "`PING` is used on a [[radar_booster]]",
        outcome: "the booster plays a noise that can help guide crew mates",
    },
    TerminalBehaviorRule {
        condition: "`TRANSMIT` is used",
        outcome: "the [[signal_translator]] broadcasts a message up to 9 characters long to all crew mates",
    },
    TerminalBehaviorRule {
        condition: "`SCAN` is used while the ship is on a moon",
        outcome: "the terminal shows the remaining item count and approximate sell value, and the displayed value can be estimated by multiplying by 0.41 for the host or 0.5 for a non-host",
    },
    TerminalBehaviorRule {
        condition: "`SCAN` is used while the ship is in orbit",
        outcome: "the terminal shows the item count and total value inside the ship",
    },
    TerminalBehaviorRule {
        condition: "`EJECT` is confirmed by the lobby host while orbiting a moon",
        outcome: "the ship is emptied into deep space and the run resets as if the crew had missed its [[profit_quota]]",
    },
    TerminalBehaviorRule {
        condition: "`RESET CREDITS` is used on a Steam profile named Zeekerss, Puffo, or Blueray",
        outcome: "credits are set to 2500 and the terminal displays a callback reading 200",
    },
];

pub const TERMINAL_MODIFIERS: [TerminalBehaviorRule; 3] = [
    TerminalBehaviorRule {
        condition: "a command is entered in shortened form",
        outcome: "the terminal may still accept it, such as `C` for `CONFIRM` and `D` for `DENY`",
    },
    TerminalBehaviorRule {
        condition: "the target name is supplied directly",
        outcome: "several commands can resolve without the full command keyword",
    },
    TerminalBehaviorRule {
        condition: "the syntax order is varied or a minor misspelling is entered",
        outcome: "the terminal may still parse the request",
    },
];

pub const TERMINAL_STRATEGY: [TerminalBehaviorRule; 3] = [
    TerminalBehaviorRule {
        condition: "the crew wants to save time",
        outcome: "place store orders before landing so equipment can arrive during the next moon cycle",
    },
    TerminalBehaviorRule {
        condition: "a mission needs quick direction changes",
        outcome: "use `MOONS` and `ROUTE` to set orbit before the landing sequence begins",
    },
    TerminalBehaviorRule {
        condition: "item value matters",
        outcome: "use `SCAN` and adjust the displayed sell value by the host or non-host multiplier before deciding what to haul",
    },
];

pub const TERMINAL_NOTES: [TerminalBehaviorRule; 3] = [
    TerminalBehaviorRule {
        condition: "`BUY` is used with a quantity",
        outcome: "the number can be typed before or after the item name",
    },
    TerminalBehaviorRule {
        condition: "`INFO` is used with a moon or item",
        outcome: "the terminal shows the relevant summary instead of opening a purchase flow",
    },
    TerminalBehaviorRule {
        condition: "`SIGURD` is used",
        outcome: "the terminal opens the collected log entries and `VIEW` can read an individual log",
    },
];

pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerminalState>()
            .add_event::<TerminalCommandEnteredEvent>()
            .add_event::<TerminalCommandResolvedEvent>()
            .add_event::<TerminalMonitorToggledEvent>()
            .add_event::<TerminalUtilityCommandEvent>()
            .add_systems(
                FixedUpdate,
                (terminal_resolve_command, terminal_checksum).chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerminalLocation {
    #[default]
    InOrbit,
    OnMoon,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerminalPendingAction {
    #[default]
    None,
    RouteMoon,
    StoreOrder,
    EjectCrew,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerminalCommandKind {
    #[default]
    Unknown,
    Help,
    Moons,
    Store,
    Bestiary,
    Storage,
    ViewMonitor,
    SpecialCode,
    FlashRadarBooster,
    PingRadarBooster,
    Transmit,
    Scan,
    Eject,
    ResetCredits,
    Confirm,
    Deny,
    Info,
    Sigurd,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct TerminalState {
    pub location: TerminalLocation,
    pub pending_action: TerminalPendingAction,
    pub monitor_map_cam_enabled: bool,
    pub command_requests: u64,
    pub commands_resolved: u64,
    pub commands_denied: u64,
    pub special_codes_used: u64,
    pub radar_booster_flashes: u64,
    pub radar_booster_pings: u64,
    pub transmitted_messages: u64,
    pub scan_requests: u64,
    pub eject_resets: u64,
    pub reset_credit_callbacks: u64,
    pub last_command_context_id: u64,
    pub last_command_kind: TerminalCommandKind,
    pub last_scan_item_count: u32,
    pub last_scan_total_value: u32,
    pub last_scan_displayed_value: u32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            location: TerminalLocation::InOrbit,
            pending_action: TerminalPendingAction::None,
            monitor_map_cam_enabled: false,
            command_requests: 0,
            commands_resolved: 0,
            commands_denied: 0,
            special_codes_used: 0,
            radar_booster_flashes: 0,
            radar_booster_pings: 0,
            transmitted_messages: 0,
            scan_requests: 0,
            eject_resets: 0,
            reset_credit_callbacks: 0,
            last_command_context_id: 0,
            last_command_kind: TerminalCommandKind::Unknown,
            last_scan_item_count: 0,
            last_scan_total_value: 0,
            last_scan_displayed_value: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCommandEnteredEvent {
    pub context_id: u64,
    pub command: &'static str,
    pub argument: &'static str,
    pub is_lobby_host: bool,
    pub is_host_scan: bool,
    pub scan_item_count: u32,
    pub scan_total_value: u32,
    pub steam_profile_name: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCommandResolvedEvent {
    pub context_id: u64,
    pub command_kind: TerminalCommandKind,
    pub pending_action: TerminalPendingAction,
    pub accepted: bool,
    pub callback_value: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalMonitorToggledEvent {
    pub context_id: u64,
    pub enabled: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalUtilityCommandEvent {
    pub context_id: u64,
    pub command_kind: TerminalCommandKind,
    pub target_entity_id: &'static str,
}

pub fn terminal_help_groups() -> &'static [&'static str; 5] {
    &TERMINAL_HELP_GROUPS
}

pub fn terminal_scan_display_value(total_value: u32, is_host: bool) -> u32 {
    let numerator = if is_host {
        TERMINAL_SCAN_HOST_NUMERATOR
    } else {
        TERMINAL_SCAN_NON_HOST_NUMERATOR
    };

    total_value.saturating_mul(numerator) / TERMINAL_SCAN_DENOMINATOR
}

pub fn terminal_transmit_payload(message: &'static str) -> &'static str {
    if message.len() <= TERMINAL_TRANSMIT_MAX_CHARS as usize {
        message
    } else {
        ""
    }
}

fn terminal_resolve_command(
    mut command_events: EventReader<TerminalCommandEnteredEvent>,
    mut resolved_events: EventWriter<TerminalCommandResolvedEvent>,
    mut monitor_events: EventWriter<TerminalMonitorToggledEvent>,
    mut utility_events: EventWriter<TerminalUtilityCommandEvent>,
    mut state: ResMut<TerminalState>,
) {
    for event in command_events.read() {
        state.command_requests = state.command_requests.wrapping_add(1);
        state.last_command_context_id = event.context_id;

        let command_kind = terminal_command_kind(event.command, event.argument);
        state.last_command_kind = command_kind;

        let mut accepted = true;
        let mut callback_value = 0;

        match command_kind {
            TerminalCommandKind::Help => {}
            TerminalCommandKind::Moons => {
                state.pending_action = TerminalPendingAction::RouteMoon;
            }
            TerminalCommandKind::Store => {
                state.pending_action = TerminalPendingAction::StoreOrder;
            }
            TerminalCommandKind::Bestiary => {}
            TerminalCommandKind::Storage => {}
            TerminalCommandKind::ViewMonitor => {
                state.monitor_map_cam_enabled = !state.monitor_map_cam_enabled;
                monitor_events.send(TerminalMonitorToggledEvent {
                    context_id: event.context_id,
                    enabled: state.monitor_map_cam_enabled,
                });
            }
            TerminalCommandKind::SpecialCode => {
                state.special_codes_used = state.special_codes_used.wrapping_add(1);
                utility_events.send(TerminalUtilityCommandEvent {
                    context_id: event.context_id,
                    command_kind,
                    target_entity_id: "secure_door",
                });
                utility_events.send(TerminalUtilityCommandEvent {
                    context_id: event.context_id,
                    command_kind,
                    target_entity_id: "turret",
                });
                utility_events.send(TerminalUtilityCommandEvent {
                    context_id: event.context_id,
                    command_kind,
                    target_entity_id: "landmine",
                });
            }
            TerminalCommandKind::FlashRadarBooster => {
                state.radar_booster_flashes = state.radar_booster_flashes.wrapping_add(1);
                utility_events.send(TerminalUtilityCommandEvent {
                    context_id: event.context_id,
                    command_kind,
                    target_entity_id: "radar_booster",
                });
            }
            TerminalCommandKind::PingRadarBooster => {
                state.radar_booster_pings = state.radar_booster_pings.wrapping_add(1);
                utility_events.send(TerminalUtilityCommandEvent {
                    context_id: event.context_id,
                    command_kind,
                    target_entity_id: "radar_booster",
                });
            }
            TerminalCommandKind::Transmit => {
                if terminal_transmit_payload(event.argument).is_empty() && !event.argument.is_empty()
                {
                    accepted = false;
                } else {
                    state.transmitted_messages = state.transmitted_messages.wrapping_add(1);
                    utility_events.send(TerminalUtilityCommandEvent {
                        context_id: event.context_id,
                        command_kind,
                        target_entity_id: "signal_translator",
                    });
                }
            }
            TerminalCommandKind::Scan => {
                state.scan_requests = state.scan_requests.wrapping_add(1);
                state.last_scan_item_count = event.scan_item_count;
                state.last_scan_total_value = event.scan_total_value;
                state.last_scan_displayed_value = match state.location {
                    TerminalLocation::OnMoon => {
                        terminal_scan_display_value(event.scan_total_value, event.is_host_scan)
                    }
                    TerminalLocation::InOrbit => event.scan_total_value,
                };
            }
            TerminalCommandKind::Eject => {
                state.pending_action = TerminalPendingAction::EjectCrew;
            }
            TerminalCommandKind::ResetCredits => {
                if terminal_reset_profile_allowed(event.steam_profile_name) {
                    state.reset_credit_callbacks = state.reset_credit_callbacks.wrapping_add(1);
                    callback_value = TERMINAL_RESET_CREDITS_CALLBACK;
                } else {
                    accepted = false;
                }
            }
            TerminalCommandKind::Confirm => {
                if state.pending_action == TerminalPendingAction::EjectCrew
                    && state.location == TerminalLocation::OnMoon
                    && event.is_lobby_host
                {
                    state.eject_resets = state.eject_resets.wrapping_add(1);
                }

                state.pending_action = TerminalPendingAction::None;
            }
            TerminalCommandKind::Deny => {
                state.commands_denied = state.commands_denied.wrapping_add(1);
                state.pending_action = TerminalPendingAction::None;
            }
            TerminalCommandKind::Info => {}
            TerminalCommandKind::Sigurd => {}
            TerminalCommandKind::Unknown => {
                accepted = false;
            }
        }

        if accepted {
            state.commands_resolved = state.commands_resolved.wrapping_add(1);
        }

        resolved_events.send(TerminalCommandResolvedEvent {
            context_id: event.context_id,
            command_kind,
            pending_action: state.pending_action,
            accepted,
            callback_value,
        });
    }
}

fn terminal_command_kind(command: &str, argument: &str) -> TerminalCommandKind {
    let command = command.trim();

    if is_two_digit_special_code(command) {
        return TerminalCommandKind::SpecialCode;
    }

    if command_matches(command, "HELP") {
        TerminalCommandKind::Help
    } else if command_matches(command, "MOONS") || command_matches(command, "ROUTE") {
        TerminalCommandKind::Moons
    } else if command_matches(command, "STORE") || command_matches(command, "BUY") {
        TerminalCommandKind::Store
    } else if command_matches(command, "BESTIARY") {
        TerminalCommandKind::Bestiary
    } else if command_matches(command, "STORAGE") {
        TerminalCommandKind::Storage
    } else if command_matches_two_words(command, "VIEW", "MONITOR") {
        TerminalCommandKind::ViewMonitor
    } else if command_matches(command, "FLASH") {
        TerminalCommandKind::FlashRadarBooster
    } else if command_matches(command, "PING") {
        TerminalCommandKind::PingRadarBooster
    } else if command_matches(command, "TRANSMIT") {
        TerminalCommandKind::Transmit
    } else if command_matches(command, "SCAN") {
        TerminalCommandKind::Scan
    } else if command_matches(command, "EJECT") {
        TerminalCommandKind::Eject
    } else if command_matches_two_words(command, "RESET", "CREDITS") {
        TerminalCommandKind::ResetCredits
    } else if command_matches(command, "CONFIRM") {
        TerminalCommandKind::Confirm
    } else if command_matches(command, "DENY") {
        TerminalCommandKind::Deny
    } else if command_matches(command, "INFO") {
        TerminalCommandKind::Info
    } else if command_matches(command, "SIGURD") || command_matches(argument, "SIGURD") {
        TerminalCommandKind::Sigurd
    } else {
        TerminalCommandKind::Unknown
    }
}

fn command_matches(input: &str, expected: &str) -> bool {
    let input = input.trim();
    !input.is_empty() && expected.starts_with(input) && input.len() <= expected.len()
}

fn command_matches_two_words(input: &str, first: &str, second: &str) -> bool {
    let mut words = input.split_whitespace();
    let Some(first_input) = words.next() else {
        return false;
    };
    let Some(second_input) = words.next() else {
        return false;
    };

    command_matches(first_input, first) && command_matches(second_input, second)
}

fn is_two_digit_special_code(input: &str) -> bool {
    let mut bytes = input.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    let Some(second) = bytes.next() else {
        return false;
    };

    bytes.next().is_none() && first.is_ascii_digit() && second.is_ascii_digit()
}

fn terminal_reset_profile_allowed(profile_name: &str) -> bool {
    profile_name == "Zeekerss" || profile_name == "Puffo" || profile_name == "Blueray"
}

fn terminal_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<TerminalState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(TERMINAL_SOURCE_REVISION as u64);
    checksum.accumulate(TERMINAL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TERMINAL_TRANSMIT_MAX_CHARS as u64);
    checksum.accumulate(TERMINAL_RESET_CREDITS_PROFILE_VALUE as u64);
    checksum.accumulate(TERMINAL_RESET_CREDITS_CALLBACK as u64);
    checksum.accumulate(TERMINAL_SCAN_HOST_NUMERATOR as u64);
    checksum.accumulate(TERMINAL_SCAN_NON_HOST_NUMERATOR as u64);
    checksum.accumulate(TERMINAL_SCAN_DENOMINATOR as u64);

    accumulate_str(&mut checksum, 0x1000, TERMINAL_OVERVIEW);

    for dependency in TERMINAL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for group in TERMINAL_HELP_GROUPS {
        accumulate_str(&mut checksum, 0x3000, group);
    }

    for rule in TERMINAL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for modifier in TERMINAL_MODIFIERS {
        accumulate_str(&mut checksum, 0x5000, modifier.condition);
        accumulate_str(&mut checksum, 0x5001, modifier.outcome);
    }

    for strategy in TERMINAL_STRATEGY {
        accumulate_str(&mut checksum, 0x6000, strategy.condition);
        accumulate_str(&mut checksum, 0x6001, strategy.outcome);
    }

    for note in TERMINAL_NOTES {
        accumulate_str(&mut checksum, 0x7000, note.condition);
        accumulate_str(&mut checksum, 0x7001, note.outcome);
    }

    checksum.accumulate(state.location as u64);
    checksum.accumulate(state.pending_action as u64);
    checksum.accumulate(state.monitor_map_cam_enabled as u64);
    checksum.accumulate(state.command_requests);
    checksum.accumulate(state.commands_resolved);
    checksum.accumulate(state.commands_denied);
    checksum.accumulate(state.special_codes_used);
    checksum.accumulate(state.radar_booster_flashes);
    checksum.accumulate(state.radar_booster_pings);
    checksum.accumulate(state.transmitted_messages);
    checksum.accumulate(state.scan_requests);
    checksum.accumulate(state.eject_resets);
    checksum.accumulate(state.reset_credit_callbacks);
    checksum.accumulate(state.last_command_context_id);
    checksum.accumulate(state.last_command_kind as u64);
    checksum.accumulate(state.last_scan_item_count as u64);
    checksum.accumulate(state.last_scan_total_value as u64);
    checksum.accumulate(state.last_scan_displayed_value as u64);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}