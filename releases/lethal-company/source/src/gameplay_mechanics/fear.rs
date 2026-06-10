// Sources: vault/gameplay_mechanics/fear.md
use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I16F16;

use crate::sim::{Health, SimChecksumState, SimTick};

pub const FEAR_ID: &str = "fear";
pub const FEAR_NAME: &str = "Fear";
pub const FEAR_TYPE: &str = "gameplay_mechanics";
pub const FEAR_SUBTYPE: &str = "status_effect";
pub const FEAR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Fear";
pub const FEAR_SOURCE_REVISION: u32 = 18982;
pub const FEAR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const FEAR_CONFIDENCE_BASIS_POINTS: u16 = 66;

pub const FEAR_CRITICAL_HEALTH_THRESHOLD: i32 = 20;
pub const FEAR_BASE_INTENSITY_BITS: i32 = 1 << 16;

pub const FEAR_DEPENDS_ON: [&str; 31] = [
    "employee",
    "player_body",
    "teleporter",
    "ship",
    "weather",
    "march",
    "company_monster",
    "masked",
    "earth_leviathan",
    "hoarding_bug",
    "snare_flea",
    "thumper",
    "eyeless_dog",
    "giant_sapsucker",
    "coil_head",
    "bracken",
    "forest_keeper",
    "jester",
    "bunker_spider",
    "mask_hornets",
    "hygrodere",
    "ghost_girl",
    "spore_lizard",
    "nutcracker",
    "baboon_hawk",
    "circuit_bee",
    "tulip_snake",
    "manticoil",
    "roaming_locust",
    "moon",
    "rend",
];

pub const FEAR_RULES: [&str; 3] = [
    "Fear can be triggered by low health, witnessing certain deaths, or specific entity interactions.",
    "Some triggers persist until the related source stops, the employee is freed, or health returns to the recovery threshold.",
    "Some deaths and transformations are explicitly exempt from fear triggers.",
];

pub const FEAR_MODIFIERS: [&str; 3] = [
    "The screen becomes distorted and blurry while the effect is active.",
    "Most of the UI is affected for a few seconds after application.",
    "The sound cue is only heard by the affected employee.",
];

pub const FEAR_STRATEGY: [&str; 2] = [
    "Use the ambience cue from a forest keeper as an early warning in fog, on poor line-of-sight moons, or inside the ship.",
    "Treat fear as an informational alert unless the underlying source is already lethal.",
];

pub const FEAR_NOTES: [&str; 2] = [
    "The wiki text uses 'fear' and 'shocked' interchangeably.",
    "The page is internally inconsistent about whether coil-head is a unique fear trigger or an example of critical-health-only behavior.",
];

pub const FEAR_BEHAVIORAL_MECHANICS: [FearBehaviorRule; 20] = [
    FearBehaviorRule {
        condition: "fear is active",
        outcome: "the affected employee's screen becomes distorted and blurry, most of the UI is harder to read, and a jarring synth-like sound plays for a few seconds after application",
    },
    FearBehaviorRule {
        condition: "an employee's HP drops below 20",
        outcome: "the employee automatically enters critical health and becomes feared until health regenerates back to 20",
    },
    FearBehaviorRule {
        condition: "an employee directly views another employee's player_body",
        outcome: "the viewer becomes feared",
    },
    FearBehaviorRule {
        condition: "a teleporter returns an employee corpse to the ship",
        outcome: "anyone inside the ship or otherwise within vision of the body becomes feared",
    },
    FearBehaviorRule {
        condition: "an employee only sees another employee fall into weather water or march quicksand",
        outcome: "the fear condition does not trigger because the moment of death is not witnessed",
    },
    FearBehaviorRule {
        condition: "an employee sees another employee die to the company_monster, transform into a masked, or get eaten by the earth_leviathan",
        outcome: "the fear condition does not trigger",
    },
    FearBehaviorRule {
        condition: "a hoarding_bug flies near an employee",
        outcome: "the employee becomes feared",
    },
    FearBehaviorRule {
        condition: "a snare_flea drops and attempts to ensnare an employee",
        outcome: "the employee becomes feared",
    },
    FearBehaviorRule {
        condition: "a snare_flea successfully ensnares an employee",
        outcome: "the employee remains feared until freed and far enough away from the flea",
    },
    FearBehaviorRule {
        condition: "a snare_flea fails to ensnare an employee",
        outcome: "the employee remains feared until the flea stops pursuing them",
    },
    FearBehaviorRule {
        condition: "a thumper, eyeless_dog, or giant_sapsucker targets an employee",
        outcome: "the employee becomes feared",
    },
    FearBehaviorRule {
        condition: "a coil_head freezes close enough to an employee",
        outcome: "the employee becomes feared",
    },
    FearBehaviorRule {
        condition: "a bracken is spotted during its hunt",
        outcome: "the employee becomes feared, and the effect intensity scales with the distance to the bracken",
    },
    FearBehaviorRule {
        condition: "a forest_keeper looks in an employee's direction",
        outcome: "the employee hears ambience similar to the fear effect",
    },
    FearBehaviorRule {
        condition: "a forest_keeper spots an employee",
        outcome: "the full fear effect applies",
    },
    FearBehaviorRule {
        condition: "a masked is interrupted while attacking an employee",
        outcome: "the employee becomes feared",
    },
    FearBehaviorRule {
        condition: "a jester pops",
        outcome: "all nearby employees become feared",
    },
    FearBehaviorRule {
        condition: "the entity that caused the fear effect dies",
        outcome: "the effect dissipates",
    },
    FearBehaviorRule {
        condition: "a bunker_spider, mask_hornets, hygrodere, ghost_girl, spore_lizard, nutcracker, baboon_hawk, circuit_bee, tulip_snake, manticoil, or roaming_locust is involved",
        outcome: "the page lists no unique fear interaction for that entity",
    },
    FearBehaviorRule {
        condition: "line of sight is poor on a moon such as rend or inside the ship",
        outcome: "the fear ambience can act as a warning for nearby threats",
    },
];

pub struct FearPlugin;

impl Plugin for FearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FearState>()
            .add_event::<ApplyFearEvent>()
            .add_event::<ClearFearEvent>()
            .add_event::<ClearFearSourceEvent>()
            .add_event::<FearAppliedEvent>()
            .add_event::<FearEndedEvent>()
            .add_event::<FearAmbienceEvent>()
            .add_systems(
                FixedUpdate,
                (
                    fear_apply_events,
                    fear_clear_employee_events,
                    fear_clear_source_events,
                    fear_critical_health,
                    fear_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FearBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EmployeeFearSubject {
    pub employee_id: u64,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct FearState {
    pub active: BTreeMap<u64, ActiveFear>,
    pub total_applications: u64,
    pub total_dissipations: u64,
    pub ambience_warnings: u64,
    pub last_changed_employee_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveFear {
    pub employee_id: u64,
    pub trigger: FearTrigger,
    pub source_entity_id: &'static str,
    pub intensity: I16F16,
    pub started_tick: u64,
    pub last_refreshed_tick: u64,
    pub persists_until_cleared: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FearTrigger {
    CriticalHealth,
    ViewedPlayerBody,
    TeleportedCorpseVisible,
    HoardingBugNearFlight,
    SnareFleaAttemptedEnsnare,
    SnareFleaEnsnared,
    SnareFleaPursuit,
    ThreatTargetedEmployee,
    CoilHeadFrozenNearby,
    BrackenSpottedDuringHunt,
    ForestKeeperSpottedEmployee,
    MaskedInterruptedAttack,
    JesterPoppedNearby,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyFearEvent {
    pub employee_id: u64,
    pub trigger: FearTrigger,
    pub source_entity_id: &'static str,
    pub intensity: I16F16,
    pub persists_until_cleared: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearFearEvent {
    pub employee_id: u64,
    pub trigger: FearTrigger,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearFearSourceEvent {
    pub source_entity_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FearAppliedEvent {
    pub employee_id: u64,
    pub trigger: FearTrigger,
    pub source_entity_id: &'static str,
    pub intensity: I16F16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FearEndedEvent {
    pub employee_id: u64,
    pub trigger: FearTrigger,
    pub source_entity_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FearAmbienceEvent {
    pub employee_id: u64,
    pub source_entity_id: &'static str,
}

pub fn fear_behavioral_mechanics() -> &'static [FearBehaviorRule] {
    &FEAR_BEHAVIORAL_MECHANICS
}

pub fn fear_critical_health_threshold() -> I16F16 {
    I16F16::from_num(FEAR_CRITICAL_HEALTH_THRESHOLD)
}

pub fn fear_base_intensity() -> I16F16 {
    I16F16::from_bits(FEAR_BASE_INTENSITY_BITS)
}

pub fn fear_is_active(state: &FearState, employee_id: u64) -> bool {
    state.active.contains_key(&employee_id)
}

fn fear_apply_events(
    mut apply_events: EventReader<ApplyFearEvent>,
    mut applied_events: EventWriter<FearAppliedEvent>,
    mut ambience_events: EventWriter<FearAmbienceEvent>,
    mut state: ResMut<FearState>,
    tick: Res<SimTick>,
) {
    for event in apply_events.read() {
        state.total_applications = state.total_applications.wrapping_add(1);
        state.last_changed_employee_id = event.employee_id;

        let active = ActiveFear {
            employee_id: event.employee_id,
            trigger: event.trigger,
            source_entity_id: event.source_entity_id,
            intensity: event.intensity,
            started_tick: tick.0,
            last_refreshed_tick: tick.0,
            persists_until_cleared: event.persists_until_cleared,
        };

        state.active.insert(event.employee_id, active);

        applied_events.send(FearAppliedEvent {
            employee_id: event.employee_id,
            trigger: event.trigger,
            source_entity_id: event.source_entity_id,
            intensity: event.intensity,
        });

        if event.source_entity_id == "forest_keeper" {
            state.ambience_warnings = state.ambience_warnings.wrapping_add(1);
            ambience_events.send(FearAmbienceEvent {
                employee_id: event.employee_id,
                source_entity_id: event.source_entity_id,
            });
        }
    }
}

fn fear_clear_employee_events(
    mut clear_events: EventReader<ClearFearEvent>,
    mut ended_events: EventWriter<FearEndedEvent>,
    mut state: ResMut<FearState>,
) {
    for event in clear_events.read() {
        let Some(active) = state.active.get(&event.employee_id).copied() else {
            continue;
        };

        if active.trigger != event.trigger {
            continue;
        }

        state.active.remove(&event.employee_id);
        state.total_dissipations = state.total_dissipations.wrapping_add(1);
        state.last_changed_employee_id = event.employee_id;

        ended_events.send(FearEndedEvent {
            employee_id: active.employee_id,
            trigger: active.trigger,
            source_entity_id: active.source_entity_id,
        });
    }
}

fn fear_clear_source_events(
    mut clear_source_events: EventReader<ClearFearSourceEvent>,
    mut ended_events: EventWriter<FearEndedEvent>,
    mut state: ResMut<FearState>,
) {
    for event in clear_source_events.read() {
        let mut employees_to_clear = Vec::new();

        for (employee_id, active) in state.active.iter() {
            if active.source_entity_id == event.source_entity_id {
                employees_to_clear.push(*employee_id);
            }
        }

        for employee_id in employees_to_clear {
            let Some(active) = state.active.remove(&employee_id) else {
                continue;
            };

            state.total_dissipations = state.total_dissipations.wrapping_add(1);
            state.last_changed_employee_id = employee_id;

            ended_events.send(FearEndedEvent {
                employee_id: active.employee_id,
                trigger: active.trigger,
                source_entity_id: active.source_entity_id,
            });
        }
    }
}

fn fear_critical_health(
    employees: Query<(&EmployeeFearSubject, &Health)>,
    mut apply_events: EventWriter<ApplyFearEvent>,
    mut clear_events: EventWriter<ClearFearEvent>,
    state: Res<FearState>,
) {
    let threshold = fear_critical_health_threshold();

    for (subject, health) in employees.iter() {
        let active = state
            .active
            .get(&subject.employee_id)
            .map(|fear| fear.trigger == FearTrigger::CriticalHealth)
            .unwrap_or(false);

        if health.current < threshold && !active {
            apply_events.send(ApplyFearEvent {
                employee_id: subject.employee_id,
                trigger: FearTrigger::CriticalHealth,
                source_entity_id: "employee",
                intensity: fear_base_intensity(),
                persists_until_cleared: true,
            });
        }

        if health.current >= threshold && active {
            clear_events.send(ClearFearEvent {
                employee_id: subject.employee_id,
                trigger: FearTrigger::CriticalHealth,
            });
        }
    }
}

fn fear_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<FearState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(FEAR_SOURCE_REVISION as u64);
    checksum.accumulate(FEAR_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(FEAR_CRITICAL_HEALTH_THRESHOLD as u64);
    checksum.accumulate(FEAR_BASE_INTENSITY_BITS as u64);

    accumulate_str(&mut checksum, 0x1000, FEAR_ID);
    accumulate_str(&mut checksum, 0x1001, FEAR_NAME);
    accumulate_str(&mut checksum, 0x1002, FEAR_TYPE);
    accumulate_str(&mut checksum, 0x1003, FEAR_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, FEAR_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, FEAR_EXTRACTED_AT);

    for dependency in FEAR_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in FEAR_RULES {
        accumulate_str(&mut checksum, 0x3000, rule);
    }

    for modifier in FEAR_MODIFIERS {
        accumulate_str(&mut checksum, 0x3001, modifier);
    }

    for strategy in FEAR_STRATEGY {
        accumulate_str(&mut checksum, 0x3002, strategy);
    }

    for note in FEAR_NOTES {
        accumulate_str(&mut checksum, 0x3003, note);
    }

    for rule in FEAR_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    checksum.accumulate(state.total_applications);
    checksum.accumulate(state.total_dissipations);
    checksum.accumulate(state.ambience_warnings);
    checksum.accumulate(state.last_changed_employee_id);

    for (employee_id, active) in state.active.iter() {
        checksum.accumulate(*employee_id);
        checksum.accumulate(fear_trigger_checksum(active.trigger));
        accumulate_str(&mut checksum, 0x5000, active.source_entity_id);
        checksum.accumulate(active.intensity.to_bits() as u64);
        checksum.accumulate(active.started_tick);
        checksum.accumulate(active.last_refreshed_tick);
        checksum.accumulate(active.persists_until_cleared as u64);
    }
}

fn fear_trigger_checksum(trigger: FearTrigger) -> u64 {
    match trigger {
        FearTrigger::CriticalHealth => 1,
        FearTrigger::ViewedPlayerBody => 2,
        FearTrigger::TeleportedCorpseVisible => 3,
        FearTrigger::HoardingBugNearFlight => 4,
        FearTrigger::SnareFleaAttemptedEnsnare => 5,
        FearTrigger::SnareFleaEnsnared => 6,
        FearTrigger::SnareFleaPursuit => 7,
        FearTrigger::ThreatTargetedEmployee => 8,
        FearTrigger::CoilHeadFrozenNearby => 9,
        FearTrigger::BrackenSpottedDuringHunt => 10,
        FearTrigger::ForestKeeperSpottedEmployee => 11,
        FearTrigger::MaskedInterruptedAttack => 12,
        FearTrigger::JesterPoppedNearby => 13,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}