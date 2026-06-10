// Sources: vault/gameplay_mechanics/scanner.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const SCANNER_ID: &str = "scanner";
pub const SCANNER_NAME: &str = "Scanner";
pub const SCANNER_TYPE: &str = "gameplay_mechanics";
pub const SCANNER_SUBTYPE: &str = "feature";
pub const SCANNER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Scanner";
pub const SCANNER_SOURCE_REVISION: u32 = 21474;
pub const SCANNER_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const SCANNER_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const SCANNER_SCRAP_HIGHLIGHT_COLOR: &str = "green";
pub const SCANNER_ENTITY_HIGHLIGHT_COLOR: &str = "red";
pub const SCANNER_POINT_OF_INTEREST_HIGHLIGHT_COLOR: &str = "blue";
pub const SCANNER_APPARATUS_DISPLAY_VALUE: &str = "???";
pub const SCANNER_APPARATUS_COMPANY_VALUE_CREDITS: u32 = 80;

pub const SCANNER_DEPENDS_ON: [&str; 17] = [
    "lethal_company",
    "scrap",
    "entities",
    "bestiary",
    "the_ship",
    "terminal",
    "employee",
    "rend",
    "dine",
    "apparatus",
    "gift_box",
    "ghost_girl",
    "masked",
    "key",
    "sticky_note",
    "clipboard",
    "lethal_company",
];

pub const SCANNER_BEHAVIORAL_MECHANICS: [ScannerBehaviorRule; 14] = [
    ScannerBehaviorRule {
        condition: "activated with RMB",
        outcome: "the scanner highlights scrap with green circles and shows the item's value in Company Credits",
    },
    ScannerBehaviorRule {
        condition: "multiple pieces of scrap are within view",
        outcome: "the scanner displays their combined value",
    },
    ScannerBehaviorRule {
        condition: "activated with RMB",
        outcome: "the scanner highlights entities with red circles",
    },
    ScannerBehaviorRule {
        condition: "a new entities is scanned",
        outcome: "its creature file is uploaded to the terminal's bestiary",
    },
    ScannerBehaviorRule {
        condition: "activated with RMB",
        outcome: "the scanner highlights points of interest with blue circles, including the_ship and the main entrance",
    },
    ScannerBehaviorRule {
        condition: "used on an employee corpse",
        outcome: "the scanner can read the cause of death",
    },
    ScannerBehaviorRule {
        condition: "the corpse result reads Gunshot",
        outcome: "the actual cause may be a turret or a nutcracker",
    },
    ScannerBehaviorRule {
        condition: "scanning apparatus",
        outcome: "the displayed value is ??? and the item is always sold to the company for 80 credits",
    },
    ScannerBehaviorRule {
        condition: "scanning a closed gift_box",
        outcome: "its value is hidden until the box is opened",
    },
    ScannerBehaviorRule {
        condition: "scanning ghost_girl or masked",
        outcome: "no data is recorded in the terminal",
    },
    ScannerBehaviorRule {
        condition: "scanning items such as key, sticky_note, or clipboard",
        outcome: "the displayed value may differ from the actual selling price",
    },
    ScannerBehaviorRule {
        condition: "on rend or dine and near the main entrance",
        outcome: "the scanner may show the wrong entrance location",
    },
    ScannerBehaviorRule {
        condition: "following the service manual",
        outcome: "its color assignment for scrap and points of interest is incorrect relative to the scanner's behavior",
    },
    ScannerBehaviorRule {
        condition: "used in darkness",
        outcome: "the scanner provides a very weak light for hauling and gap spotting",
    },
];

pub struct ScannerPlugin;

impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScannerState>()
            .add_event::<ScannerActivationEvent>()
            .add_event::<ScannerScanResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    scanner_resolve_activation,
                    scanner_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScannerBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScannerTargetKind {
    Scrap,
    Entity,
    PointOfInterest,
    EmployeeCorpse,
    Apparatus,
    ClosedGiftBox,
    GhostGirl,
    Masked,
    Key,
    StickyNote,
    Clipboard,
}

impl ScannerTargetKind {
    pub fn highlight_color(self) -> Option<&'static str> {
        match self {
            Self::Scrap
            | Self::Apparatus
            | Self::ClosedGiftBox
            | Self::Key
            | Self::StickyNote
            | Self::Clipboard => Some(SCANNER_SCRAP_HIGHLIGHT_COLOR),
            Self::Entity | Self::EmployeeCorpse => Some(SCANNER_ENTITY_HIGHLIGHT_COLOR),
            Self::PointOfInterest => Some(SCANNER_POINT_OF_INTEREST_HIGHLIGHT_COLOR),
            Self::GhostGirl | Self::Masked => None,
        }
    }

    pub fn records_bestiary_data(self) -> bool {
        match self {
            Self::GhostGirl | Self::Masked => false,
            Self::Entity => true,
            Self::Scrap
            | Self::PointOfInterest
            | Self::EmployeeCorpse
            | Self::Apparatus
            | Self::ClosedGiftBox
            | Self::Key
            | Self::StickyNote
            | Self::Clipboard => false,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ScannerState {
    pub activation_requests: u64,
    pub scans_resolved: u64,
    pub total_visible_scrap_value_credits: u32,
    pub last_scan_context_id: u64,
    pub last_target_id: u64,
    pub last_highlight_color: Option<&'static str>,
    pub last_display_value: Option<&'static str>,
    pub last_bestiary_upload_allowed: bool,
    pub last_cause_of_death_readable: bool,
    pub last_wrong_entrance_location_possible: bool,
    pub weak_light_available: bool,
}

impl Default for ScannerState {
    fn default() -> Self {
        Self {
            activation_requests: 0,
            scans_resolved: 0,
            total_visible_scrap_value_credits: 0,
            last_scan_context_id: 0,
            last_target_id: 0,
            last_highlight_color: None,
            last_display_value: None,
            last_bestiary_upload_allowed: false,
            last_cause_of_death_readable: false,
            last_wrong_entrance_location_possible: false,
            weak_light_available: true,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerActivationEvent {
    pub context_id: u64,
    pub target_id: u64,
    pub target_kind: ScannerTargetKind,
    pub displayed_value_credits: Option<u32>,
    pub visible_scrap_value_credits: u32,
    pub is_new_entity_scan: bool,
    pub is_on_rend_or_dine_near_main_entrance: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerScanResolvedEvent {
    pub context_id: u64,
    pub target_id: u64,
    pub target_kind: ScannerTargetKind,
    pub highlight_color: Option<&'static str>,
    pub displayed_value_credits: Option<u32>,
    pub displayed_text_value: Option<&'static str>,
    pub combined_scrap_value_credits: u32,
    pub bestiary_upload_allowed: bool,
    pub cause_of_death_readable: bool,
    pub wrong_entrance_location_possible: bool,
    pub weak_light_available: bool,
}

pub fn scanner_highlight_color(target_kind: ScannerTargetKind) -> Option<&'static str> {
    target_kind.highlight_color()
}

pub fn scanner_records_bestiary_data(target_kind: ScannerTargetKind) -> bool {
    target_kind.records_bestiary_data()
}

pub fn scanner_apparatus_company_value_credits() -> u32 {
    SCANNER_APPARATUS_COMPANY_VALUE_CREDITS
}

fn scanner_resolve_activation(
    mut activation_events: EventReader<ScannerActivationEvent>,
    mut resolved_events: EventWriter<ScannerScanResolvedEvent>,
    mut state: ResMut<ScannerState>,
) {
    for event in activation_events.read() {
        let highlight_color = event.target_kind.highlight_color();
        let bestiary_upload_allowed =
            event.is_new_entity_scan && event.target_kind.records_bestiary_data();
        let cause_of_death_readable = event.target_kind == ScannerTargetKind::EmployeeCorpse;
        let displayed_text_value = match event.target_kind {
            ScannerTargetKind::Apparatus => Some(SCANNER_APPARATUS_DISPLAY_VALUE),
            ScannerTargetKind::ClosedGiftBox => None,
            ScannerTargetKind::GhostGirl | ScannerTargetKind::Masked => None,
            ScannerTargetKind::Scrap
            | ScannerTargetKind::Entity
            | ScannerTargetKind::PointOfInterest
            | ScannerTargetKind::EmployeeCorpse
            | ScannerTargetKind::Key
            | ScannerTargetKind::StickyNote
            | ScannerTargetKind::Clipboard => None,
        };
        let displayed_value_credits = match event.target_kind {
            ScannerTargetKind::Apparatus
            | ScannerTargetKind::ClosedGiftBox
            | ScannerTargetKind::GhostGirl
            | ScannerTargetKind::Masked => None,
            ScannerTargetKind::Scrap
            | ScannerTargetKind::Entity
            | ScannerTargetKind::PointOfInterest
            | ScannerTargetKind::EmployeeCorpse
            | ScannerTargetKind::Key
            | ScannerTargetKind::StickyNote
            | ScannerTargetKind::Clipboard => event.displayed_value_credits,
        };

        state.activation_requests = state.activation_requests.wrapping_add(1);
        state.scans_resolved = state.scans_resolved.wrapping_add(1);
        state.total_visible_scrap_value_credits = event.visible_scrap_value_credits;
        state.last_scan_context_id = event.context_id;
        state.last_target_id = event.target_id;
        state.last_highlight_color = highlight_color;
        state.last_display_value = displayed_text_value;
        state.last_bestiary_upload_allowed = bestiary_upload_allowed;
        state.last_cause_of_death_readable = cause_of_death_readable;
        state.last_wrong_entrance_location_possible =
            event.is_on_rend_or_dine_near_main_entrance;

        resolved_events.send(ScannerScanResolvedEvent {
            context_id: event.context_id,
            target_id: event.target_id,
            target_kind: event.target_kind,
            highlight_color,
            displayed_value_credits,
            displayed_text_value,
            combined_scrap_value_credits: event.visible_scrap_value_credits,
            bestiary_upload_allowed,
            cause_of_death_readable,
            wrong_entrance_location_possible: event.is_on_rend_or_dine_near_main_entrance,
            weak_light_available: state.weak_light_available,
        });
    }
}

fn scanner_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<ScannerState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(SCANNER_SOURCE_REVISION as u64);
    checksum.accumulate(SCANNER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(SCANNER_APPARATUS_COMPANY_VALUE_CREDITS as u64);

    accumulate_str(&mut checksum, 0x1000, SCANNER_SCRAP_HIGHLIGHT_COLOR);
    accumulate_str(&mut checksum, 0x1001, SCANNER_ENTITY_HIGHLIGHT_COLOR);
    accumulate_str(&mut checksum, 0x1002, SCANNER_POINT_OF_INTEREST_HIGHLIGHT_COLOR);
    accumulate_str(&mut checksum, 0x1003, SCANNER_APPARATUS_DISPLAY_VALUE);

    for dependency in SCANNER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in SCANNER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    checksum.accumulate(state.activation_requests);
    checksum.accumulate(state.scans_resolved);
    checksum.accumulate(state.total_visible_scrap_value_credits as u64);
    checksum.accumulate(state.last_scan_context_id);
    checksum.accumulate(state.last_target_id);
    checksum.accumulate(state.last_bestiary_upload_allowed as u64);
    checksum.accumulate(state.last_cause_of_death_readable as u64);
    checksum.accumulate(state.last_wrong_entrance_location_possible as u64);
    checksum.accumulate(state.weak_light_available as u64);

    if let Some(color) = state.last_highlight_color {
        accumulate_str(&mut checksum, 0x4000, color);
    }

    if let Some(display_value) = state.last_display_value {
        accumulate_str(&mut checksum, 0x4001, display_value);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}