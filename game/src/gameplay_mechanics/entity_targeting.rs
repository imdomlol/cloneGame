// Sources: vault/item_index_pages/items.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const ENTITY_TARGETING_ID: &str = "entity_targeting";
pub const ENTITY_TARGETING_NAME: &str = "Entity Targeting";
pub const ENTITY_TARGETING_TYPE: &str = "gameplay_mechanics";
pub const ENTITY_TARGETING_SUBTYPE: &str = "threat_level";
pub const ENTITY_TARGETING_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Items";
pub const ENTITY_TARGETING_SOURCE_REVISION: u32 = 21248;
pub const ENTITY_TARGETING_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ENTITY_TARGETING_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const ENTITY_TARGETING_WEAPON_ITEM_CLASSIFICATION: &str = "weapon";
pub const ENTITY_TARGETING_HELD_WEAPON_THREAT_DELTA: i16 = 1;
pub const ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA: i16 = 0;

pub const ENTITY_TARGETING_DEPENDS_ON: [&str; 3] = ["weapon", "entity", "employee"];

pub const ENTITY_TARGETING_BEHAVIORAL_MECHANICS: [EntityTargetingBehaviorRule; 1] =
    [EntityTargetingBehaviorRule {
        condition: "an item is classified as a weapon",
        outcome: "holding it changes entity_targeting threat-level calculations",
    }];

pub struct EntityTargetingPlugin;

impl Plugin for EntityTargetingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityTargetingState>()
            .add_event::<EntityTargetingHeldItemChangedEvent>()
            .add_event::<EntityThreatLevelChangedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    entity_targeting_apply_held_item_changes,
                    entity_targeting_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityTargetingBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityTargetingItemClassification {
    Scrap,
    Store,
    Weapon,
    Special,
}

impl EntityTargetingItemClassification {
    pub fn threat_delta(self) -> i16 {
        match self {
            Self::Weapon => ENTITY_TARGETING_HELD_WEAPON_THREAT_DELTA,
            Self::Scrap | Self::Store | Self::Special => ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Scrap => "scrap",
            Self::Store => "store",
            Self::Weapon => "weapon",
            Self::Special => "special",
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EntityTargetingState {
    pub weapon_item_classification: &'static str,
    pub held_weapon_threat_delta: i16,
    pub no_weapon_threat_delta: i16,
    pub held_item_change_events: u64,
    pub threat_level_change_events: u64,
    pub last_employee_entity_id: u64,
    pub last_held_item_id: u64,
    pub last_threat_delta: i16,
}

impl Default for EntityTargetingState {
    fn default() -> Self {
        Self {
            weapon_item_classification: ENTITY_TARGETING_WEAPON_ITEM_CLASSIFICATION,
            held_weapon_threat_delta: ENTITY_TARGETING_HELD_WEAPON_THREAT_DELTA,
            no_weapon_threat_delta: ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA,
            held_item_change_events: 0,
            threat_level_change_events: 0,
            last_employee_entity_id: 0,
            last_held_item_id: 0,
            last_threat_delta: ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTargetingHeldItemChangedEvent {
    pub employee_entity_id: u64,
    pub held_item_id: u64,
    pub classification: EntityTargetingItemClassification,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityThreatLevelChangedEvent {
    pub employee_entity_id: u64,
    pub held_item_id: u64,
    pub threat_delta: i16,
}

pub fn entity_targeting_threat_delta_for_classification(
    classification: EntityTargetingItemClassification,
) -> i16 {
    classification.threat_delta()
}

pub fn entity_targeting_weapon_item_classification() -> &'static str {
    ENTITY_TARGETING_WEAPON_ITEM_CLASSIFICATION
}

fn entity_targeting_apply_held_item_changes(
    mut held_item_events: EventReader<EntityTargetingHeldItemChangedEvent>,
    mut threat_events: EventWriter<EntityThreatLevelChangedEvent>,
    mut state: ResMut<EntityTargetingState>,
) {
    for event in held_item_events.read() {
        let threat_delta = event.classification.threat_delta();

        state.held_item_change_events = state.held_item_change_events.wrapping_add(1);
        state.last_employee_entity_id = event.employee_entity_id;
        state.last_held_item_id = event.held_item_id;
        state.last_threat_delta = threat_delta;

        if threat_delta != ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA {
            state.threat_level_change_events = state.threat_level_change_events.wrapping_add(1);

            threat_events.send(EntityThreatLevelChangedEvent {
                employee_entity_id: event.employee_entity_id,
                held_item_id: event.held_item_id,
                threat_delta,
            });
        }
    }
}

fn entity_targeting_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<EntityTargetingState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ENTITY_TARGETING_SOURCE_REVISION as u64);
    checksum.accumulate(ENTITY_TARGETING_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ENTITY_TARGETING_HELD_WEAPON_THREAT_DELTA as u64);
    checksum.accumulate(ENTITY_TARGETING_NO_WEAPON_THREAT_DELTA as u64);

    for dependency in ENTITY_TARGETING_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in ENTITY_TARGETING_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    accumulate_str(&mut checksum, 0x3000, state.weapon_item_classification);
    checksum.accumulate(state.held_weapon_threat_delta as u64);
    checksum.accumulate(state.no_weapon_threat_delta as u64);
    checksum.accumulate(state.held_item_change_events);
    checksum.accumulate(state.threat_level_change_events);
    checksum.accumulate(state.last_employee_entity_id);
    checksum.accumulate(state.last_held_item_id);
    checksum.accumulate(state.last_threat_delta as u64);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}