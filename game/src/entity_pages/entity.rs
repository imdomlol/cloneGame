// Sources: vault/entity_pages/entity.md
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::sim::{SimChecksumState, SimTick};

pub const ENTITY_ID: &str = "entity";
pub const ENTITY_NAME: &str = "Entity";
pub const ENTITY_TYPE: &str = "entity_pages";
pub const ENTITY_SUBTYPE: &str = "overview";
pub const ENTITY_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Entity";
pub const ENTITY_SOURCE_REVISION: u32 = 21382;
pub const ENTITY_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ENTITY_CONFIDENCE_BASIS_POINTS: u16 = 78;

pub const ENTITY_DEPENDS_ON: [&str; 0] = [];

pub const ENTITY_FRONTMATTER_BEHAVIOR: [&str; 6] = [
    "Groups wildlife and creatures into indoor, outdoor, daytime, and scrapped entities.",
    "Indoor entities generally spawn inside the Facility and use indoor spawn timing.",
    "Outdoor entities generally spawn outside and can begin appearing later in the day or during eclipsed weather.",
    "Daytime entities generally spawn outdoors during daylight and tend to appear and leave earlier than other outdoor entities.",
    "Some entities can be stunned by stun equipment, while invincible or unstunnable entities ignore stun effects.",
    "Some entities can be killed by weapons, lightning, or other entities, but a few cannot be dispatched normally.",
];

pub const ENTITY_BEHAVIORAL_MECHANICS: [EntityBehaviorRule; 10] = [
    EntityBehaviorRule {
        condition: "an entity is classified as indoor",
        outcome: "it spawns inside the Facility and uses indoor spawn timing",
    },
    EntityBehaviorRule {
        condition: "an entity is classified as outdoor",
        outcome: "it spawns outside and uses the outdoor spawn curve",
    },
    EntityBehaviorRule {
        condition: "an entity is classified as daytime",
        outcome: "it spawns outdoors during the day and tends to appear earlier and leave earlier than other outdoor entities",
    },
    EntityBehaviorRule {
        condition: "the spawn cycle executes for indoor entities",
        outcome: "their appearance is delayed by the cycle timing rather than spawning immediately",
    },
    EntityBehaviorRule {
        condition: "the spawn cycle executes for outdoor entities",
        outcome: "they can spawn immediately when the cycle runs",
    },
    EntityBehaviorRule {
        condition: "an entity can be stunned",
        outcome: "stun duration is scaled by its stun multiplier",
    },
    EntityBehaviorRule {
        condition: "an entity is marked invincible",
        outcome: "normal damage does not reduce its health",
    },
    EntityBehaviorRule {
        condition: "an entity is marked as unable to be stunned",
        outcome: "stun equipment does not affect it",
    },
    EntityBehaviorRule {
        condition: "an entity uses a special health rule",
        outcome: "its damage handling can differ from standard health values",
    },
    EntityBehaviorRule {
        condition: "an entity is one of the scrapped entries",
        outcome: "it exists in game files but does not appear in regular gameplay",
    },
];

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityPageState>()
            .add_event::<EntityClassificationRegisteredEvent>()
            .add_event::<EntitySpawnCycleEvent>()
            .add_event::<EntityStunRuleEvent>()
            .add_event::<EntityDamageRuleEvent>()
            .add_systems(
                FixedUpdate,
                (
                    entity_register_classifications,
                    entity_apply_spawn_cycle_rules,
                    entity_apply_stun_rules,
                    entity_apply_damage_rules,
                    entity_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityClassification {
    Indoor,
    Outdoor,
    Daytime,
    Scrapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntitySpawnTiming {
    IndoorDelayed,
    OutdoorImmediate,
    DaytimeEarly,
    ScrappedUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityStunHandling {
    ScaledByMultiplier,
    Ignored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityDamageHandling {
    StandardHealth,
    Invincible,
    SpecialHealthRule,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EntityPageState {
    pub classifications: BTreeSet<EntityClassification>,
    pub indoor_spawn_timing: EntitySpawnTiming,
    pub outdoor_spawn_timing: EntitySpawnTiming,
    pub daytime_spawn_timing: EntitySpawnTiming,
    pub scrapped_spawn_timing: EntitySpawnTiming,
    pub stun_handling: EntityStunHandling,
    pub unstunnable_handling: EntityStunHandling,
    pub damage_handling: EntityDamageHandling,
    pub invincible_damage_handling: EntityDamageHandling,
    pub special_health_damage_handling: EntityDamageHandling,
}

impl Default for EntityPageState {
    fn default() -> Self {
        Self {
            classifications: BTreeSet::new(),
            indoor_spawn_timing: EntitySpawnTiming::IndoorDelayed,
            outdoor_spawn_timing: EntitySpawnTiming::OutdoorImmediate,
            daytime_spawn_timing: EntitySpawnTiming::DaytimeEarly,
            scrapped_spawn_timing: EntitySpawnTiming::ScrappedUnavailable,
            stun_handling: EntityStunHandling::ScaledByMultiplier,
            unstunnable_handling: EntityStunHandling::Ignored,
            damage_handling: EntityDamageHandling::StandardHealth,
            invincible_damage_handling: EntityDamageHandling::Invincible,
            special_health_damage_handling: EntityDamageHandling::SpecialHealthRule,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityClassificationRegisteredEvent {
    pub classification: EntityClassification,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntitySpawnCycleEvent {
    pub classification: EntityClassification,
    pub timing: EntitySpawnTiming,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityStunRuleEvent {
    pub can_be_stunned: bool,
    pub handling: EntityStunHandling,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityDamageRuleEvent {
    pub is_invincible: bool,
    pub uses_special_health_rule: bool,
    pub handling: EntityDamageHandling,
}

pub fn spawn_timing_for_classification(
    classification: EntityClassification,
) -> EntitySpawnTiming {
    match classification {
        EntityClassification::Indoor => EntitySpawnTiming::IndoorDelayed,
        EntityClassification::Outdoor => EntitySpawnTiming::OutdoorImmediate,
        EntityClassification::Daytime => EntitySpawnTiming::DaytimeEarly,
        EntityClassification::Scrapped => EntitySpawnTiming::ScrappedUnavailable,
    }
}

pub fn stun_handling_for_entity(can_be_stunned: bool) -> EntityStunHandling {
    if can_be_stunned {
        EntityStunHandling::ScaledByMultiplier
    } else {
        EntityStunHandling::Ignored
    }
}

pub fn damage_handling_for_entity(
    is_invincible: bool,
    uses_special_health_rule: bool,
) -> EntityDamageHandling {
    if is_invincible {
        return EntityDamageHandling::Invincible;
    }

    if uses_special_health_rule {
        return EntityDamageHandling::SpecialHealthRule;
    }

    EntityDamageHandling::StandardHealth
}

fn entity_register_classifications(
    mut registered_events: EventWriter<EntityClassificationRegisteredEvent>,
    mut state: ResMut<EntityPageState>,
) {
    for classification in [
        EntityClassification::Indoor,
        EntityClassification::Outdoor,
        EntityClassification::Daytime,
        EntityClassification::Scrapped,
    ] {
        if state.classifications.insert(classification) {
            registered_events.send(EntityClassificationRegisteredEvent { classification });
        }
    }
}

fn entity_apply_spawn_cycle_rules(
    mut spawn_events: EventWriter<EntitySpawnCycleEvent>,
    state: Res<EntityPageState>,
) {
    for classification in &state.classifications {
        spawn_events.send(EntitySpawnCycleEvent {
            classification: *classification,
            timing: spawn_timing_for_classification(*classification),
        });
    }
}

fn entity_apply_stun_rules(
    mut stun_events: EventWriter<EntityStunRuleEvent>,
    state: Res<EntityPageState>,
) {
    stun_events.send(EntityStunRuleEvent {
        can_be_stunned: true,
        handling: state.stun_handling,
    });
    stun_events.send(EntityStunRuleEvent {
        can_be_stunned: false,
        handling: state.unstunnable_handling,
    });
}

fn entity_apply_damage_rules(
    mut damage_events: EventWriter<EntityDamageRuleEvent>,
    state: Res<EntityPageState>,
) {
    damage_events.send(EntityDamageRuleEvent {
        is_invincible: false,
        uses_special_health_rule: false,
        handling: state.damage_handling,
    });
    damage_events.send(EntityDamageRuleEvent {
        is_invincible: true,
        uses_special_health_rule: false,
        handling: state.invincible_damage_handling,
    });
    damage_events.send(EntityDamageRuleEvent {
        is_invincible: false,
        uses_special_health_rule: true,
        handling: state.special_health_damage_handling,
    });
}

fn entity_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<EntityPageState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ENTITY_SOURCE_REVISION as u64);
    checksum.accumulate(ENTITY_CONFIDENCE_BASIS_POINTS as u64);

    for dependency in ENTITY_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for behavior in ENTITY_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x2000, behavior);
    }

    for rule in ENTITY_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for classification in &state.classifications {
        checksum.accumulate(0x4000 ^ entity_classification_code(*classification));
        checksum.accumulate(
            0x4001 ^ entity_spawn_timing_code(spawn_timing_for_classification(*classification)),
        );
    }

    checksum.accumulate(0x5000 ^ entity_spawn_timing_code(state.indoor_spawn_timing));
    checksum.accumulate(0x5001 ^ entity_spawn_timing_code(state.outdoor_spawn_timing));
    checksum.accumulate(0x5002 ^ entity_spawn_timing_code(state.daytime_spawn_timing));
    checksum.accumulate(0x5003 ^ entity_spawn_timing_code(state.scrapped_spawn_timing));
    checksum.accumulate(0x6000 ^ entity_stun_handling_code(state.stun_handling));
    checksum.accumulate(0x6001 ^ entity_stun_handling_code(state.unstunnable_handling));
    checksum.accumulate(0x7000 ^ entity_damage_handling_code(state.damage_handling));
    checksum.accumulate(0x7001 ^ entity_damage_handling_code(state.invincible_damage_handling));
    checksum.accumulate(0x7002 ^ entity_damage_handling_code(state.special_health_damage_handling));
}

fn entity_classification_code(classification: EntityClassification) -> u64 {
    match classification {
        EntityClassification::Indoor => 1,
        EntityClassification::Outdoor => 2,
        EntityClassification::Daytime => 3,
        EntityClassification::Scrapped => 4,
    }
}

fn entity_spawn_timing_code(timing: EntitySpawnTiming) -> u64 {
    match timing {
        EntitySpawnTiming::IndoorDelayed => 1,
        EntitySpawnTiming::OutdoorImmediate => 2,
        EntitySpawnTiming::DaytimeEarly => 3,
        EntitySpawnTiming::ScrappedUnavailable => 4,
    }
}

fn entity_stun_handling_code(handling: EntityStunHandling) -> u64 {
    match handling {
        EntityStunHandling::ScaledByMultiplier => 1,
        EntityStunHandling::Ignored => 2,
    }
}

fn entity_damage_handling_code(handling: EntityDamageHandling) -> u64 {
    match handling {
        EntityDamageHandling::StandardHealth => 1,
        EntityDamageHandling::Invincible => 2,
        EntityDamageHandling::SpecialHealthRule => 3,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}