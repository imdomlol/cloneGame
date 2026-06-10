// Sources: vault/gameplay_mechanics/lore.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const LORE_ID: &str = "lore";
pub const LORE_NAME: &str = "Lore";
pub const LORE_TYPE: &str = "gameplay_mechanics";
pub const LORE_SUBTYPE: &str = "lore";
pub const LORE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Lore";
pub const LORE_SOURCE_REVISION: u32 = 21353;
pub const LORE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const LORE_CONFIDENCE_BASIS_POINTS: u16 = 84;

pub const LORE_YEAR: u16 = 2532;
pub const LORE_SIGURD_KEYWORD: &str = "sigurd";
pub const LORE_SETTING_REGION_ID: &str = "thistle_nebula";
pub const LORE_COMPANY_ID: &str = "the_company";
pub const LORE_TERMINAL_ID: &str = "terminal";
pub const LORE_MOONS_ID: &str = "moons";
pub const LORE_SIGURDS_LOGS_ID: &str = "sigurds_logs";
pub const LORE_BESTIARY_ID: &str = "bestiary";

pub const LORE_OVERVIEW: &str = "A lore page describing the setting of Lethal Company, the role of the Company, and the way Sigurd's logs expose the game's hidden history.";

pub const LORE_DEPENDS_ON: [&str; 9] = [
    "lethal_company",
    "thistle_nebula",
    "the_company",
    "moons",
    "sigurds_logs",
    "terminal",
    "bestiary",
    "old_bird",
    "bunker_spider",
];

pub const LORE_RULES: [&str; 10] = [
    "The setting is located in the Thistle Nebula and may be part of the Fading Nebulae.",
    "The year is 2532.",
    "Humanity has been space-faring and colonizing for several hundred years.",
    "Accessible moons span multiple star systems and include twin moons, asteroid moons, frozen worlds, and abandoned industrial sites.",
    "The Company's salvage teams transport scrap to the ocean-covered moon where the Company resides.",
    "The terminal can be used to reveal the date and time at the start of a hired run.",
    "The word sigurd unlocks the log list after data chips are collected.",
    "The bestiary is used as evidence that many mundane species were carried across the nebula and later evolved.",
    "The Manual and the Company Cruiser share company trademark branding.",
    "Company equipment is supplied by Halden Electronics and F. Power Co.",
];

pub const LORE_MODIFIERS: [&str; 3] = [
    "Sigurd's entries contain many typos, and later entries appear cleaner.",
    "Several moon colonies are described as mysteriously vanished.",
    "The Company Building is treated as a place where hidden voices and threats are encountered.",
];

pub const LORE_STRATEGY: [&str; 3] = [
    "Use Sigurd's log order to reconstruct the timeline rather than reading the page as a single continuous history.",
    "Treat the moon list as setting context for individual log locations.",
    "Treat the beastiary notes as ecological inference rather than a complete species catalog.",
];

pub const LORE_NOTES: [&str; 3] = [
    "The page keeps some origin claims uncertain, especially around non-Earth life.",
    "The lore includes a legendary Golden Planet story that remains indirect and fragmented.",
    "The page ties the Company to branded tools, transport, and communications.",
];

pub const LORE_BEHAVIORAL_MECHANICS: [LoreBehaviorRule; 5] = [
    LoreBehaviorRule {
        condition: "the player opens the terminal on day 0 after being hired",
        outcome: "the in-universe date display resolves to the year 2532",
    },
    LoreBehaviorRule {
        condition: "the keyword sigurd is entered into the terminal",
        outcome: "the hidden log index becomes available once data chips have been collected from the moons",
    },
    LoreBehaviorRule {
        condition: "reading sigurds_logs",
        outcome: "later entries show fewer typographical errors than earlier entries",
    },
    LoreBehaviorRule {
        condition: "the bestiary is treated as an in-world source",
        outcome: "many creatures are implied to descend from Earth species that were transported and evolved after arrival",
    },
    LoreBehaviorRule {
        condition: "the world is read through the_company's infrastructure",
        outcome: "hidden corporate logistics and moon-based salvage are treated as intentional parts of the setting",
    },
];

pub struct LorePlugin;

impl Plugin for LorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoreState>()
            .add_event::<LoreTerminalOpenedEvent>()
            .add_event::<LoreSigurdKeywordEnteredEvent>()
            .add_event::<LoreSigurdLogReadEvent>()
            .add_event::<LoreBestiarySourceInterpretedEvent>()
            .add_event::<LoreCompanyInfrastructureReadEvent>()
            .add_event::<LoreRuleResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    lore_resolve_terminal_date_display,
                    lore_resolve_sigurd_log_index,
                    lore_resolve_sigurd_log_typography,
                    lore_resolve_bestiary_inference,
                    lore_resolve_company_infrastructure,
                    lore_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoreBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LoreState {
    pub setting_region_id: &'static str,
    pub company_id: &'static str,
    pub terminal_id: &'static str,
    pub moons_id: &'static str,
    pub sigurds_logs_id: &'static str,
    pub bestiary_id: &'static str,
    pub in_universe_year: u16,
    pub date_display_resolved: bool,
    pub hidden_log_index_available: bool,
    pub later_entries_cleaner: bool,
    pub bestiary_ecological_inference_active: bool,
    pub company_logistics_intentional: bool,
    pub terminal_open_requests: u64,
    pub sigurd_keyword_entries: u64,
    pub sigurd_log_reads: u64,
    pub bestiary_interpretations: u64,
    pub company_infrastructure_reads: u64,
    pub resolved_rules: u64,
    pub last_context_id: u64,
}

impl Default for LoreState {
    fn default() -> Self {
        Self {
            setting_region_id: LORE_SETTING_REGION_ID,
            company_id: LORE_COMPANY_ID,
            terminal_id: LORE_TERMINAL_ID,
            moons_id: LORE_MOONS_ID,
            sigurds_logs_id: LORE_SIGURDS_LOGS_ID,
            bestiary_id: LORE_BESTIARY_ID,
            in_universe_year: LORE_YEAR,
            date_display_resolved: false,
            hidden_log_index_available: false,
            later_entries_cleaner: false,
            bestiary_ecological_inference_active: false,
            company_logistics_intentional: false,
            terminal_open_requests: 0,
            sigurd_keyword_entries: 0,
            sigurd_log_reads: 0,
            bestiary_interpretations: 0,
            company_infrastructure_reads: 0,
            resolved_rules: 0,
            last_context_id: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreTerminalOpenedEvent {
    pub context_id: u64,
    pub day: u16,
    pub hired_run_started: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreSigurdKeywordEnteredEvent {
    pub context_id: u64,
    pub keyword: &'static str,
    pub data_chips_collected: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreSigurdLogReadEvent {
    pub context_id: u64,
    pub log_order: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreBestiarySourceInterpretedEvent {
    pub context_id: u64,
    pub treated_as_in_world_source: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreCompanyInfrastructureReadEvent {
    pub context_id: u64,
    pub read_through_company_infrastructure: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoreRuleResolvedEvent {
    pub context_id: u64,
    pub rule_index: u8,
    pub condition: &'static str,
    pub outcome: &'static str,
}

pub fn lore_in_universe_year() -> u16 {
    LORE_YEAR
}

pub fn lore_sigurd_keyword() -> &'static str {
    LORE_SIGURD_KEYWORD
}

pub fn lore_setting_region_id() -> &'static str {
    LORE_SETTING_REGION_ID
}

fn lore_resolve_terminal_date_display(
    mut events: EventReader<LoreTerminalOpenedEvent>,
    mut resolved_events: EventWriter<LoreRuleResolvedEvent>,
    mut state: ResMut<LoreState>,
) {
    for event in events.read() {
        state.terminal_open_requests = state.terminal_open_requests.wrapping_add(1);
        state.last_context_id = event.context_id;

        if event.day == 0 && event.hired_run_started {
            state.date_display_resolved = true;
            state.resolved_rules = state.resolved_rules.wrapping_add(1);
            resolved_events.send(LoreRuleResolvedEvent {
                context_id: event.context_id,
                rule_index: 0,
                condition: LORE_BEHAVIORAL_MECHANICS[0].condition,
                outcome: LORE_BEHAVIORAL_MECHANICS[0].outcome,
            });
        }
    }
}

fn lore_resolve_sigurd_log_index(
    mut events: EventReader<LoreSigurdKeywordEnteredEvent>,
    mut resolved_events: EventWriter<LoreRuleResolvedEvent>,
    mut state: ResMut<LoreState>,
) {
    for event in events.read() {
        state.sigurd_keyword_entries = state.sigurd_keyword_entries.wrapping_add(1);
        state.last_context_id = event.context_id;

        if event.keyword == LORE_SIGURD_KEYWORD && event.data_chips_collected {
            state.hidden_log_index_available = true;
            state.resolved_rules = state.resolved_rules.wrapping_add(1);
            resolved_events.send(LoreRuleResolvedEvent {
                context_id: event.context_id,
                rule_index: 1,
                condition: LORE_BEHAVIORAL_MECHANICS[1].condition,
                outcome: LORE_BEHAVIORAL_MECHANICS[1].outcome,
            });
        }
    }
}

fn lore_resolve_sigurd_log_typography(
    mut events: EventReader<LoreSigurdLogReadEvent>,
    mut resolved_events: EventWriter<LoreRuleResolvedEvent>,
    mut state: ResMut<LoreState>,
) {
    for event in events.read() {
        state.sigurd_log_reads = state.sigurd_log_reads.wrapping_add(1);
        state.last_context_id = event.context_id;
        state.later_entries_cleaner = true;
        state.resolved_rules = state.resolved_rules.wrapping_add(1);

        resolved_events.send(LoreRuleResolvedEvent {
            context_id: event.context_id,
            rule_index: 2,
            condition: LORE_BEHAVIORAL_MECHANICS[2].condition,
            outcome: LORE_BEHAVIORAL_MECHANICS[2].outcome,
        });
    }
}

fn lore_resolve_bestiary_inference(
    mut events: EventReader<LoreBestiarySourceInterpretedEvent>,
    mut resolved_events: EventWriter<LoreRuleResolvedEvent>,
    mut state: ResMut<LoreState>,
) {
    for event in events.read() {
        state.bestiary_interpretations = state.bestiary_interpretations.wrapping_add(1);
        state.last_context_id = event.context_id;

        if event.treated_as_in_world_source {
            state.bestiary_ecological_inference_active = true;
            state.resolved_rules = state.resolved_rules.wrapping_add(1);
            resolved_events.send(LoreRuleResolvedEvent {
                context_id: event.context_id,
                rule_index: 3,
                condition: LORE_BEHAVIORAL_MECHANICS[3].condition,
                outcome: LORE_BEHAVIORAL_MECHANICS[3].outcome,
            });
        }
    }
}

fn lore_resolve_company_infrastructure(
    mut events: EventReader<LoreCompanyInfrastructureReadEvent>,
    mut resolved_events: EventWriter<LoreRuleResolvedEvent>,
    mut state: ResMut<LoreState>,
) {
    for event in events.read() {
        state.company_infrastructure_reads = state.company_infrastructure_reads.wrapping_add(1);
        state.last_context_id = event.context_id;

        if event.read_through_company_infrastructure {
            state.company_logistics_intentional = true;
            state.resolved_rules = state.resolved_rules.wrapping_add(1);
            resolved_events.send(LoreRuleResolvedEvent {
                context_id: event.context_id,
                rule_index: 4,
                condition: LORE_BEHAVIORAL_MECHANICS[4].condition,
                outcome: LORE_BEHAVIORAL_MECHANICS[4].outcome,
            });
        }
    }
}

fn lore_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<LoreState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(LORE_SOURCE_REVISION as u64);
    checksum.accumulate(LORE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(LORE_YEAR as u64);

    accumulate_str(&mut checksum, 0x1000, LORE_ID);
    accumulate_str(&mut checksum, 0x1001, LORE_NAME);
    accumulate_str(&mut checksum, 0x1002, LORE_TYPE);
    accumulate_str(&mut checksum, 0x1003, LORE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, LORE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, LORE_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, LORE_OVERVIEW);
    accumulate_str(&mut checksum, 0x1007, LORE_SIGURD_KEYWORD);

    for dependency in LORE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in LORE_RULES {
        accumulate_str(&mut checksum, 0x3000, rule);
    }

    for modifier in LORE_MODIFIERS {
        accumulate_str(&mut checksum, 0x4000, modifier);
    }

    for strategy in LORE_STRATEGY {
        accumulate_str(&mut checksum, 0x5000, strategy);
    }

    for note in LORE_NOTES {
        accumulate_str(&mut checksum, 0x6000, note);
    }

    for rule in LORE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x7000, rule.condition);
        accumulate_str(&mut checksum, 0x7001, rule.outcome);
    }

    accumulate_str(&mut checksum, 0x8000, state.setting_region_id);
    accumulate_str(&mut checksum, 0x8001, state.company_id);
    accumulate_str(&mut checksum, 0x8002, state.terminal_id);
    accumulate_str(&mut checksum, 0x8003, state.moons_id);
    accumulate_str(&mut checksum, 0x8004, state.sigurds_logs_id);
    accumulate_str(&mut checksum, 0x8005, state.bestiary_id);
    checksum.accumulate(state.in_universe_year as u64);
    checksum.accumulate(state.date_display_resolved as u64);
    checksum.accumulate(state.hidden_log_index_available as u64);
    checksum.accumulate(state.later_entries_cleaner as u64);
    checksum.accumulate(state.bestiary_ecological_inference_active as u64);
    checksum.accumulate(state.company_logistics_intentional as u64);
    checksum.accumulate(state.terminal_open_requests);
    checksum.accumulate(state.sigurd_keyword_entries);
    checksum.accumulate(state.sigurd_log_reads);
    checksum.accumulate(state.bestiary_interpretations);
    checksum.accumulate(state.company_infrastructure_reads);
    checksum.accumulate(state.resolved_rules);
    checksum.accumulate(state.last_context_id);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}