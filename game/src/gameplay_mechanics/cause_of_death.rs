// Sources: vault/gameplay_mechanics/cause_of_death.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const CAUSE_OF_DEATH_ID: &str = "cause_of_death";
pub const CAUSE_OF_DEATH_NAME: &str = "Cause of Death";
pub const CAUSE_OF_DEATH_TYPE: &str = "gameplay_mechanics";
pub const CAUSE_OF_DEATH_SUBTYPE: &str = "redirect";
pub const CAUSE_OF_DEATH_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Cause_of_Death";
pub const CAUSE_OF_DEATH_SOURCE_REVISION: u32 = 7329;
pub const CAUSE_OF_DEATH_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const CAUSE_OF_DEATH_CONFIDENCE_BASIS_POINTS: u16 = 5;

pub const CAUSE_OF_DEATH_CANONICAL_ENTITY_ID: &str = "employee";
pub const CAUSE_OF_DEATH_CANONICAL_SECTION: &str = "Damage Sources";
pub const CAUSE_OF_DEATH_CANONICAL_DESTINATION: &str = "employee#Damage Sources";

pub const CAUSE_OF_DEATH_DEPENDS_ON: [&str; 1] = ["employee"];

pub const CAUSE_OF_DEATH_BEHAVIORAL_MECHANICS: [CauseOfDeathBehaviorRule; 1] =
    [CauseOfDeathBehaviorRule {
        condition: "you need the cause-of-death information",
        outcome: "use employee#Damage Sources as the canonical destination",
    }];

pub struct CauseOfDeathPlugin;

impl Plugin for CauseOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CauseOfDeathState>()
            .add_event::<CauseOfDeathLookupEvent>()
            .add_event::<CauseOfDeathRedirectResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    cause_of_death_resolve_redirect_lookup,
                    cause_of_death_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CauseOfDeathBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CauseOfDeathState {
    pub canonical_entity_id: &'static str,
    pub canonical_section: &'static str,
    pub canonical_destination: &'static str,
    pub lookup_requests: u64,
    pub redirects_resolved: u64,
    pub last_lookup_context_id: u64,
}

impl Default for CauseOfDeathState {
    fn default() -> Self {
        Self {
            canonical_entity_id: CAUSE_OF_DEATH_CANONICAL_ENTITY_ID,
            canonical_section: CAUSE_OF_DEATH_CANONICAL_SECTION,
            canonical_destination: CAUSE_OF_DEATH_CANONICAL_DESTINATION,
            lookup_requests: 0,
            redirects_resolved: 0,
            last_lookup_context_id: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CauseOfDeathLookupEvent {
    pub context_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CauseOfDeathRedirectResolvedEvent {
    pub context_id: u64,
    pub canonical_entity_id: &'static str,
    pub canonical_section: &'static str,
    pub canonical_destination: &'static str,
}

pub fn cause_of_death_canonical_destination() -> &'static str {
    CAUSE_OF_DEATH_CANONICAL_DESTINATION
}

fn cause_of_death_resolve_redirect_lookup(
    mut lookup_events: EventReader<CauseOfDeathLookupEvent>,
    mut resolved_events: EventWriter<CauseOfDeathRedirectResolvedEvent>,
    mut state: ResMut<CauseOfDeathState>,
) {
    for event in lookup_events.read() {
        state.lookup_requests = state.lookup_requests.wrapping_add(1);
        state.redirects_resolved = state.redirects_resolved.wrapping_add(1);
        state.last_lookup_context_id = event.context_id;

        resolved_events.send(CauseOfDeathRedirectResolvedEvent {
            context_id: event.context_id,
            canonical_entity_id: state.canonical_entity_id,
            canonical_section: state.canonical_section,
            canonical_destination: state.canonical_destination,
        });
    }
}

fn cause_of_death_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<CauseOfDeathState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(CAUSE_OF_DEATH_SOURCE_REVISION as u64);
    checksum.accumulate(CAUSE_OF_DEATH_CONFIDENCE_BASIS_POINTS as u64);

    for dependency in CAUSE_OF_DEATH_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in CAUSE_OF_DEATH_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    accumulate_str(&mut checksum, 0x3000, state.canonical_entity_id);
    accumulate_str(&mut checksum, 0x3001, state.canonical_section);
    accumulate_str(&mut checksum, 0x3002, state.canonical_destination);
    checksum.accumulate(state.lookup_requests);
    checksum.accumulate(state.redirects_resolved);
    checksum.accumulate(state.last_lookup_context_id);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}