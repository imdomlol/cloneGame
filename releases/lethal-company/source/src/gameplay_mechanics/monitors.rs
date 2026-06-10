// Sources: vault/gameplay_mechanics/monitors.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const MONITORS_ID: &str = "monitors";
pub const MONITORS_NAME: &str = "Monitors";
pub const MONITORS_TYPE: &str = "gameplay_mechanics";
pub const MONITORS_SUBTYPE: &str = "equipment";
pub const MONITORS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Monitors";
pub const MONITORS_SOURCE_REVISION: u32 = 13796;
pub const MONITORS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MONITORS_CONFIDENCE_BASIS_POINTS: u16 = 9300;

pub const MONITORS_DEPENDS_ON: [&str; 9] = [
    "autopilot_ship",
    "camera_duty",
    "stormy",
    "terminal",
    "moon",
    "weather",
    "ghost_girl",
    "profit_quota",
    "challenge_moon",
];

pub const MONITORS_RULES: [&str; 5] = [
    "The left radar cam shows a top-down map centered on the currently selected employee.",
    "The white button switches the selected player.",
    "The red button toggles the radar cam on or off.",
    "The right CCTV monitors show the ship interior cameras and the ship entrance camera.",
    "The quota monitors show quota progress on the left and deadline days on the right.",
];

pub const MONITORS_MODIFIERS: [&str; 5] = [
    "If electronics are disabled by a lightning strike, the radar cam can be turned back on.",
    "If the ship is in orbit, the radar cam shows general information about the selected moon and current weather conditions.",
    "If the session starts on a challenge moon, both quota monitors use a bright purple backdrop.",
    "If the session starts on a challenge moon, the left quota monitor shows the moon name.",
    "If the session starts on a challenge moon, the right quota monitor shows AS MUCH PROFIT AS POSSIBLE.",
];

pub const MONITORS_STRATEGY: [&str; 4] = [
    "Use the CCTV monitors to check outside when the door is closed.",
    "Use the CCTV monitors to detect the ghost girl, since she does not appear on the radar cams.",
    "Use the quota monitors to track progress toward the profit quota and the remaining deadline.",
    "Use the terminal to duplicate the radar cam with VIEW MONITOR or switch players with SWITCH.",
];

pub const MONITORS_NOTES: [&str; 2] = [
    "The brake lever for landing and taking off is underneath the monitors.",
    "The monitors are not movable.",
];

pub const MONITORS_BEHAVIORAL_MECHANICS: [MonitorsBehaviorRule; 16] = [
    MonitorsBehaviorRule {
        condition: "the left radar cam is active",
        outcome: "it shows a top-down map around the currently selected employee",
    },
    MonitorsBehaviorRule {
        condition: "the upper white button is pressed",
        outcome: "the selected player changes",
    },
    MonitorsBehaviorRule {
        condition: "the lower red button is pressed",
        outcome: "the left radar cam toggles on or off",
    },
    MonitorsBehaviorRule {
        condition: "a lightning strike from stormy disables the ship's electronics",
        outcome: "the radar cam can be restored by toggling it back on",
    },
    MonitorsBehaviorRule {
        condition: "terminal runs VIEW MONITOR",
        outcome: "it duplicates the left radar cam display",
    },
    MonitorsBehaviorRule {
        condition: "terminal runs SWITCH",
        outcome: "it duplicates the selected-player switching function",
    },
    MonitorsBehaviorRule {
        condition: "terminal runs SWITCH [crew mate]",
        outcome: "it switches directly to the named crew mate",
    },
    MonitorsBehaviorRule {
        condition: "the ship is in orbit",
        outcome: "the radar cam shows general information about the selected moon and current weather conditions",
    },
    MonitorsBehaviorRule {
        condition: "the CCTV monitors are active",
        outcome: "they show the ship's interior cameras and the entrance camera",
    },
    MonitorsBehaviorRule {
        condition: "the door is closed and a crew mate needs to see outside",
        outcome: "the CCTV monitors provide the outside view",
    },
    MonitorsBehaviorRule {
        condition: "a ghost_girl is present",
        outcome: "only the CCTV monitors can reveal her because she does not appear on the radar cams",
    },
    MonitorsBehaviorRule {
        condition: "the quota monitors are active",
        outcome: "the left screen shows current progress toward the profit_quota",
    },
    MonitorsBehaviorRule {
        condition: "the quota monitors are active",
        outcome: "the right screen shows the deadline in days remaining to sell scrap to the company",
    },
    MonitorsBehaviorRule {
        condition: "a session starts on a challenge_moon",
        outcome: "both quota monitors use a bright purple backdrop",
    },
    MonitorsBehaviorRule {
        condition: "a session starts on a challenge_moon",
        outcome: "the left quota screen shows the moon name",
    },
    MonitorsBehaviorRule {
        condition: "a session starts on a challenge_moon",
        outcome: "the right quota screen shows AS MUCH PROFIT AS POSSIBLE",
    },
];

pub const MONITORS_CHALLENGE_BACKDROP: &str = "bright purple";
pub const MONITORS_CHALLENGE_RIGHT_QUOTA_TEXT: &str = "AS MUCH PROFIT AS POSSIBLE";

pub struct MonitorsPlugin;

impl Plugin for MonitorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonitorsState>()
            .add_event::<MonitorButtonPressedEvent>()
            .add_event::<MonitorTerminalCommandEvent>()
            .add_event::<MonitorElectronicsDisabledEvent>()
            .add_event::<MonitorSessionStartedEvent>()
            .add_event::<MonitorShipOrbitChangedEvent>()
            .add_event::<MonitorQuotaUpdatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    monitors_apply_session_start,
                    monitors_apply_orbit_change,
                    monitors_apply_quota_update,
                    monitors_apply_electronics_disabled,
                    monitors_apply_button,
                    monitors_apply_terminal_command,
                    monitors_refresh_display,
                    monitors_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorsBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorButton {
    UpperWhite,
    LowerRed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorTerminalCommand {
    ViewMonitor,
    Switch,
    SwitchToCrewMate { employee_id: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorRadarDisplay {
    Off,
    EmployeeMap { employee_id: u64 },
    OrbitInfo {
        moon_id: &'static str,
        weather_id: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorQuotaBackdrop {
    Standard,
    BrightPurple,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct MonitorsState {
    pub radar_cam_active: bool,
    pub cctv_monitors_active: bool,
    pub quota_monitors_active: bool,
    pub terminal_view_monitor_active: bool,
    pub selected_employee_id: u64,
    pub known_employee_ids: Vec<u64>,
    pub ship_in_orbit: bool,
    pub selected_moon_id: &'static str,
    pub current_weather_id: &'static str,
    pub challenge_moon_session: bool,
    pub challenge_moon_name: &'static str,
    pub quota_progress: u64,
    pub quota_target: u64,
    pub deadline_days_remaining: u32,
    pub radar_display: MonitorRadarDisplay,
    pub quota_backdrop: MonitorQuotaBackdrop,
    pub left_quota_text: &'static str,
    pub right_quota_text: &'static str,
    pub electronics_disabled_events: u64,
    pub button_presses: u64,
    pub terminal_commands: u64,
}

impl Default for MonitorsState {
    fn default() -> Self {
        Self {
            radar_cam_active: true,
            cctv_monitors_active: true,
            quota_monitors_active: true,
            terminal_view_monitor_active: false,
            selected_employee_id: 0,
            known_employee_ids: Vec::new(),
            ship_in_orbit: false,
            selected_moon_id: "moon",
            current_weather_id: "weather",
            challenge_moon_session: false,
            challenge_moon_name: "challenge_moon",
            quota_progress: 0,
            quota_target: 0,
            deadline_days_remaining: 0,
            radar_display: MonitorRadarDisplay::EmployeeMap { employee_id: 0 },
            quota_backdrop: MonitorQuotaBackdrop::Standard,
            left_quota_text: "profit_quota",
            right_quota_text: "deadline days remaining",
            electronics_disabled_events: 0,
            button_presses: 0,
            terminal_commands: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorButtonPressedEvent {
    pub button: MonitorButton,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorTerminalCommandEvent {
    pub command: MonitorTerminalCommand,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorElectronicsDisabledEvent {
    pub source_id: &'static str,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct MonitorSessionStartedEvent {
    pub employee_ids: Vec<u64>,
    pub selected_employee_id: u64,
    pub challenge_moon_session: bool,
    pub challenge_moon_name: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorShipOrbitChangedEvent {
    pub ship_in_orbit: bool,
    pub selected_moon_id: &'static str,
    pub current_weather_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorQuotaUpdatedEvent {
    pub quota_progress: u64,
    pub quota_target: u64,
    pub deadline_days_remaining: u32,
}

pub fn monitors_selected_employee_id(state: &MonitorsState) -> u64 {
    state.selected_employee_id
}

pub fn monitors_radar_display(state: &MonitorsState) -> MonitorRadarDisplay {
    state.radar_display
}

pub fn monitors_quota_backdrop(state: &MonitorsState) -> MonitorQuotaBackdrop {
    state.quota_backdrop
}

fn monitors_apply_session_start(
    mut events: EventReader<MonitorSessionStartedEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.known_employee_ids = event.employee_ids.clone();
        state.known_employee_ids.sort_unstable();
        state.selected_employee_id = event.selected_employee_id;
        state.challenge_moon_session = event.challenge_moon_session;
        state.challenge_moon_name = event.challenge_moon_name;
    }
}

fn monitors_apply_orbit_change(
    mut events: EventReader<MonitorShipOrbitChangedEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.ship_in_orbit = event.ship_in_orbit;
        state.selected_moon_id = event.selected_moon_id;
        state.current_weather_id = event.current_weather_id;
    }
}

fn monitors_apply_quota_update(
    mut events: EventReader<MonitorQuotaUpdatedEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.quota_progress = event.quota_progress;
        state.quota_target = event.quota_target;
        state.deadline_days_remaining = event.deadline_days_remaining;
    }
}

fn monitors_apply_electronics_disabled(
    mut events: EventReader<MonitorElectronicsDisabledEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.electronics_disabled_events = state.electronics_disabled_events.wrapping_add(1);

        if event.source_id == "stormy" {
            state.radar_cam_active = false;
            state.terminal_view_monitor_active = false;
        }
    }
}

fn monitors_apply_button(
    mut events: EventReader<MonitorButtonPressedEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.button_presses = state.button_presses.wrapping_add(1);

        match event.button {
            MonitorButton::UpperWhite => {
                switch_to_next_employee(&mut state);
            }
            MonitorButton::LowerRed => {
                state.radar_cam_active = !state.radar_cam_active;
            }
        }
    }
}

fn monitors_apply_terminal_command(
    mut events: EventReader<MonitorTerminalCommandEvent>,
    mut state: ResMut<MonitorsState>,
) {
    for event in events.read() {
        state.terminal_commands = state.terminal_commands.wrapping_add(1);

        match event.command {
            MonitorTerminalCommand::ViewMonitor => {
                state.terminal_view_monitor_active = state.radar_cam_active;
            }
            MonitorTerminalCommand::Switch => {
                switch_to_next_employee(&mut state);
            }
            MonitorTerminalCommand::SwitchToCrewMate { employee_id } => {
                if state.known_employee_ids.binary_search(&employee_id).is_ok() {
                    state.selected_employee_id = employee_id;
                }
            }
        }
    }
}

fn monitors_refresh_display(mut state: ResMut<MonitorsState>) {
    state.radar_display = if !state.radar_cam_active {
        MonitorRadarDisplay::Off
    } else if state.ship_in_orbit {
        MonitorRadarDisplay::OrbitInfo {
            moon_id: state.selected_moon_id,
            weather_id: state.current_weather_id,
        }
    } else {
        MonitorRadarDisplay::EmployeeMap {
            employee_id: state.selected_employee_id,
        }
    };

    if state.challenge_moon_session {
        state.quota_backdrop = MonitorQuotaBackdrop::BrightPurple;
        state.left_quota_text = state.challenge_moon_name;
        state.right_quota_text = MONITORS_CHALLENGE_RIGHT_QUOTA_TEXT;
    } else {
        state.quota_backdrop = MonitorQuotaBackdrop::Standard;
        state.left_quota_text = "profit_quota";
        state.right_quota_text = "deadline days remaining";
    }
}

fn switch_to_next_employee(state: &mut MonitorsState) {
    if state.known_employee_ids.is_empty() {
        return;
    }

    state.known_employee_ids.sort_unstable();

    let next_index = match state
        .known_employee_ids
        .binary_search(&state.selected_employee_id)
    {
        Ok(index) => (index + 1) % state.known_employee_ids.len(),
        Err(index) => index % state.known_employee_ids.len(),
    };

    state.selected_employee_id = state.known_employee_ids[next_index];
}

fn monitors_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<MonitorsState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(MONITORS_SOURCE_REVISION as u64);
    checksum.accumulate(MONITORS_CONFIDENCE_BASIS_POINTS as u64);

    accumulate_str(&mut checksum, 0x1000, MONITORS_ID);
    accumulate_str(&mut checksum, 0x1001, MONITORS_NAME);
    accumulate_str(&mut checksum, 0x1002, MONITORS_TYPE);
    accumulate_str(&mut checksum, 0x1003, MONITORS_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, MONITORS_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, MONITORS_EXTRACTED_AT);

    for dependency in MONITORS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in MONITORS_RULES {
        accumulate_str(&mut checksum, 0x3000, rule);
    }

    for modifier in MONITORS_MODIFIERS {
        accumulate_str(&mut checksum, 0x4000, modifier);
    }

    for strategy in MONITORS_STRATEGY {
        accumulate_str(&mut checksum, 0x5000, strategy);
    }

    for note in MONITORS_NOTES {
        accumulate_str(&mut checksum, 0x6000, note);
    }

    for rule in MONITORS_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x7000, rule.condition);
        accumulate_str(&mut checksum, 0x7001, rule.outcome);
    }

    checksum.accumulate(state.radar_cam_active as u64);
    checksum.accumulate(state.cctv_monitors_active as u64);
    checksum.accumulate(state.quota_monitors_active as u64);
    checksum.accumulate(state.terminal_view_monitor_active as u64);
    checksum.accumulate(state.selected_employee_id);
    checksum.accumulate(state.ship_in_orbit as u64);
    checksum.accumulate(state.challenge_moon_session as u64);
    checksum.accumulate(state.quota_progress);
    checksum.accumulate(state.quota_target);
    checksum.accumulate(state.deadline_days_remaining as u64);
    checksum.accumulate(state.electronics_disabled_events);
    checksum.accumulate(state.button_presses);
    checksum.accumulate(state.terminal_commands);

    for employee_id in state.known_employee_ids.iter().copied() {
        checksum.accumulate(employee_id);
    }

    accumulate_str(&mut checksum, 0x8000, state.selected_moon_id);
    accumulate_str(&mut checksum, 0x8001, state.current_weather_id);
    accumulate_str(&mut checksum, 0x8002, state.challenge_moon_name);
    accumulate_str(&mut checksum, 0x8003, state.left_quota_text);
    accumulate_str(&mut checksum, 0x8004, state.right_quota_text);

    match state.radar_display {
        MonitorRadarDisplay::Off => {
            checksum.accumulate(0);
        }
        MonitorRadarDisplay::EmployeeMap { employee_id } => {
            checksum.accumulate(1);
            checksum.accumulate(employee_id);
        }
        MonitorRadarDisplay::OrbitInfo {
            moon_id,
            weather_id,
        } => {
            checksum.accumulate(2);
            accumulate_str(&mut checksum, 0x9000, moon_id);
            accumulate_str(&mut checksum, 0x9001, weather_id);
        }
    }

    checksum.accumulate(match state.quota_backdrop {
        MonitorQuotaBackdrop::Standard => 0,
        MonitorQuotaBackdrop::BrightPurple => 1,
    });
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}