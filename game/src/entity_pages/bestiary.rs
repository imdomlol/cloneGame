// Sources: vault/entity_pages/bestiary.md
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::sim::{SimChecksumState, SimTick};

pub const BESTIARY_ID: &str = "bestiary";
pub const BESTIARY_NAME: &str = "Bestiary";
pub const BESTIARY_TYPE: &str = "entity_pages";
pub const BESTIARY_SUBTYPE: &str = "compendium";
pub const BESTIARY_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Bestiary";
pub const BESTIARY_SOURCE_REVISION: u32 = 17787;
pub const BESTIARY_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BESTIARY_CONFIDENCE_BASIS_POINTS: u16 = 95;

pub const BESTIARY_DEPENDS_ON: [&str; 7] = [
    "entities",
    "lethal_company",
    "terminal",
    "echo_scanner",
    "ghost_girl",
    "masked",
    "mask_hornets",
];

pub const BESTIARY_FRONTMATTER_BEHAVIOR: [&str; 3] = [
    "IF an employee opens [[Terminal]] and runs `BESTIARY`, THEN the Bestiary entries are shown.",
    "IF an employee types an entity name followed by `INFO` at [[Terminal]], THEN that entity's entry opens directly.",
    "IF an employee scans an entity with [[Echo Scanner]], THEN new Bestiary entries are unlocked in [[Terminal]].",
];

pub const BESTIARY_BEHAVIORAL_MECHANICS: [BestiaryBehaviorRule; 5] = [
    BestiaryBehaviorRule {
        condition: "an employee opens [[Terminal]] and runs `BESTIARY`",
        outcome: "the Bestiary entries are displayed",
    },
    BestiaryBehaviorRule {
        condition: "an employee types an entity name followed by `INFO` at [[Terminal]]",
        outcome: "the matching entry opens directly",
    },
    BestiaryBehaviorRule {
        condition: "an employee scans an entity with [[Echo Scanner]]",
        outcome: "new Bestiary entries become available in [[Terminal]]",
    },
    BestiaryBehaviorRule {
        condition: "a creature is marked as having no creature file on record",
        outcome: "the page records that the entity has no transcript",
    },
    BestiaryBehaviorRule {
        condition: "[[Mask Hornets]] are scanned",
        outcome: "the page still records no Bestiary log for them",
    },
];

pub const BESTIARY_CREATURE_FILES: [BestiaryCreatureFile; 25] = [
    BestiaryCreatureFile {
        page: "baboon_hawk",
        transcribe: "Hawk",
    },
    BestiaryCreatureFile {
        page: "barber",
        transcribe: "Clay Surgeon",
    },
    BestiaryCreatureFile {
        page: "bracken",
        transcribe: "Bracken",
    },
    BestiaryCreatureFile {
        page: "bunker_spider",
        transcribe: "Spider",
    },
    BestiaryCreatureFile {
        page: "butler",
        transcribe: "Butler",
    },
    BestiaryCreatureFile {
        page: "coil_head",
        transcribe: "Springman",
    },
    BestiaryCreatureFile {
        page: "circuit_bee",
        transcribe: "Bee",
    },
    BestiaryCreatureFile {
        page: "earth_leviathan",
        transcribe: "Worm",
    },
    BestiaryCreatureFile {
        page: "eyeless_dog",
        transcribe: "Dog",
    },
    BestiaryCreatureFile {
        page: "forest_keeper",
        transcribe: "Giant",
    },
    BestiaryCreatureFile {
        page: "hoarding_bug",
        transcribe: "Lootbug",
    },
    BestiaryCreatureFile {
        page: "hygrodere",
        transcribe: "Slime",
    },
    BestiaryCreatureFile {
        page: "jester",
        transcribe: "Jester",
    },
    BestiaryCreatureFile {
        page: "kidnapper_fox",
        transcribe: "Bush Wolf",
    },
    BestiaryCreatureFile {
        page: "maneater",
        transcribe: "Maneater",
    },
    BestiaryCreatureFile {
        page: "manticoil",
        transcribe: "Bird",
    },
    BestiaryCreatureFile {
        page: "mask_hornets",
        transcribe: "Mask Hornets",
    },
    BestiaryCreatureFile {
        page: "nutcracker",
        transcribe: "Nutcracker",
    },
    BestiaryCreatureFile {
        page: "old_bird",
        transcribe: "Radmech",
    },
    BestiaryCreatureFile {
        page: "roaming_locust",
        transcribe: "Locust",
    },
    BestiaryCreatureFile {
        page: "snare_flea",
        transcribe: "Centipede",
    },
    BestiaryCreatureFile {
        page: "spore_lizard",
        transcribe: "Puffer",
    },
    BestiaryCreatureFile {
        page: "thumper",
        transcribe: "Half",
    },
    BestiaryCreatureFile {
        page: "tulip_snake",
        transcribe: "Tulip Snake",
    },
    BestiaryCreatureFile {
        page: "vain_shroud",
        transcribe: "Vain Shroud",
    },
];

pub const BESTIARY_MISSING_CREATURE_FILES: [&str; 2] = ["ghost_girl", "masked"];
pub const BESTIARY_SCANNABLE_WITHOUT_LOG: [&str; 1] = ["mask_hornets"];

pub struct BestiaryPlugin;

impl Plugin for BestiaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BestiaryState>()
            .add_event::<BestiaryTerminalCommandEvent>()
            .add_event::<EchoScannerBestiaryScanEvent>()
            .add_event::<BestiaryEntryUnlockedEvent>()
            .add_event::<BestiaryNoLogScanEvent>()
            .add_systems(
                FixedUpdate,
                (
                    bestiary_unlock_scanned_entities,
                    bestiary_handle_terminal_commands,
                    bestiary_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BestiaryCreatureFile {
    pub page: &'static str,
    pub transcribe: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BestiaryBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct BestiaryState {
    pub unlocked_entries: BTreeSet<&'static str>,
    pub opened_view: BestiaryOpenedView,
    pub transcriptless_entities: BTreeSet<&'static str>,
    pub scannable_without_log_entities: BTreeSet<&'static str>,
}

impl Default for BestiaryState {
    fn default() -> Self {
        Self {
            unlocked_entries: BTreeSet::new(),
            opened_view: BestiaryOpenedView::Closed,
            transcriptless_entities: BESTIARY_MISSING_CREATURE_FILES.into_iter().collect(),
            scannable_without_log_entities: BESTIARY_SCANNABLE_WITHOUT_LOG.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BestiaryOpenedView {
    Closed,
    EntryList,
    EntityEntry {
        page: &'static str,
        transcribe: &'static str,
        is_unlocked: bool,
        has_transcript: bool,
    },
    MissingTranscript {
        page: &'static str,
    },
    NoLog {
        page: &'static str,
    },
    UnknownQuery {
        query: String,
    },
}

impl Default for BestiaryOpenedView {
    fn default() -> Self {
        Self::Closed
    }
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct BestiaryTerminalCommandEvent {
    pub command: BestiaryTerminalCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BestiaryTerminalCommand {
    OpenBestiary,
    OpenEntityInfo { query: String },
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct EchoScannerBestiaryScanEvent {
    pub entity_id: String,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestiaryEntryUnlockedEvent {
    pub page: &'static str,
    pub transcribe: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestiaryNoLogScanEvent {
    pub page: &'static str,
}

fn bestiary_unlock_scanned_entities(
    mut scan_events: EventReader<EchoScannerBestiaryScanEvent>,
    mut unlocked_events: EventWriter<BestiaryEntryUnlockedEvent>,
    mut no_log_events: EventWriter<BestiaryNoLogScanEvent>,
    mut state: ResMut<BestiaryState>,
) {
    for event in scan_events.read() {
        if is_scannable_without_log(&event.entity_id) {
            no_log_events.send(BestiaryNoLogScanEvent {
                page: "mask_hornets",
            });
            continue;
        }

        if let Some(file) = creature_file_by_page(&event.entity_id) {
            if state.unlocked_entries.insert(file.page) {
                unlocked_events.send(BestiaryEntryUnlockedEvent {
                    page: file.page,
                    transcribe: file.transcribe,
                });
            }
        }
    }
}

fn bestiary_handle_terminal_commands(
    mut command_events: EventReader<BestiaryTerminalCommandEvent>,
    mut state: ResMut<BestiaryState>,
) {
    for event in command_events.read() {
        match &event.command {
            BestiaryTerminalCommand::OpenBestiary => {
                state.opened_view = BestiaryOpenedView::EntryList;
            }
            BestiaryTerminalCommand::OpenEntityInfo { query } => {
                state.opened_view = bestiary_view_for_query(query, &state);
            }
        }
    }
}

fn bestiary_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<BestiaryState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(BESTIARY_SOURCE_REVISION as u64);
    checksum.accumulate(BESTIARY_CONFIDENCE_BASIS_POINTS as u64);

    for dependency in BESTIARY_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for behavior in BESTIARY_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x2000, behavior);
    }

    for rule in BESTIARY_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for file in BESTIARY_CREATURE_FILES {
        accumulate_str(&mut checksum, 0x4000, file.page);
        accumulate_str(&mut checksum, 0x4001, file.transcribe);
    }

    for page in &state.unlocked_entries {
        accumulate_str(&mut checksum, 0x5000, page);
    }

    for page in &state.transcriptless_entities {
        accumulate_str(&mut checksum, 0x6000, page);
    }

    for page in &state.scannable_without_log_entities {
        accumulate_str(&mut checksum, 0x7000, page);
    }

    match &state.opened_view {
        BestiaryOpenedView::Closed => checksum.accumulate(0),
        BestiaryOpenedView::EntryList => checksum.accumulate(1),
        BestiaryOpenedView::EntityEntry {
            page,
            transcribe,
            is_unlocked,
            has_transcript,
        } => {
            checksum.accumulate(2);
            accumulate_str(&mut checksum, 0x8000, page);
            accumulate_str(&mut checksum, 0x8001, transcribe);
            checksum.accumulate(*is_unlocked as u64);
            checksum.accumulate(*has_transcript as u64);
        }
        BestiaryOpenedView::MissingTranscript { page } => {
            checksum.accumulate(3);
            accumulate_str(&mut checksum, 0x8002, page);
        }
        BestiaryOpenedView::NoLog { page } => {
            checksum.accumulate(4);
            accumulate_str(&mut checksum, 0x8003, page);
        }
        BestiaryOpenedView::UnknownQuery { query } => {
            checksum.accumulate(5);
            accumulate_str(&mut checksum, 0x8004, query);
        }
    }
}

fn bestiary_view_for_query(query: &str, state: &BestiaryState) -> BestiaryOpenedView {
    if let Some(file) = creature_file_by_query(query) {
        if is_scannable_without_log(file.page) {
            return BestiaryOpenedView::NoLog { page: file.page };
        }

        return BestiaryOpenedView::EntityEntry {
            page: file.page,
            transcribe: file.transcribe,
            is_unlocked: state.unlocked_entries.contains(file.page),
            has_transcript: true,
        };
    }

    if let Some(page) = missing_creature_file_by_query(query) {
        return BestiaryOpenedView::MissingTranscript { page };
    }

    BestiaryOpenedView::UnknownQuery {
        query: query.to_ascii_lowercase(),
    }
}

fn creature_file_by_page(page: &str) -> Option<BestiaryCreatureFile> {
    for file in BESTIARY_CREATURE_FILES {
        if file.page == page {
            return Some(file);
        }
    }

    None
}

fn creature_file_by_query(query: &str) -> Option<BestiaryCreatureFile> {
    for file in BESTIARY_CREATURE_FILES {
        if query_matches(query, file.page) || query_matches(query, file.transcribe) {
            return Some(file);
        }
    }

    None
}

fn missing_creature_file_by_query(query: &str) -> Option<&'static str> {
    for page in BESTIARY_MISSING_CREATURE_FILES {
        if query_matches(query, page) {
            return Some(page);
        }
    }

    None
}

fn is_scannable_without_log(page: &str) -> bool {
    for scannable in BESTIARY_SCANNABLE_WITHOUT_LOG {
        if page == scannable {
            return true;
        }
    }

    false
}

fn query_matches(query: &str, expected: &str) -> bool {
    normalize_terminal_query(query) == normalize_terminal_query(expected)
}

fn normalize_terminal_query(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '_' && *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}