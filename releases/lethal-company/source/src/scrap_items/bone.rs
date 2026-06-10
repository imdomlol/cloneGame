// Sources: vault/scrap_items/bone.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const BONE_ID: &str = "bone";
pub const BONE_NAME: &str = "Bone";
pub const BONE_TYPE: &str = "scrap_items";
pub const BONE_SUBTYPE: &str = "bone";
pub const BONE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Bone";
pub const BONE_SOURCE_REVISION: u32 = 20283;
pub const BONE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BONE_CONFIDENCE_BASIS_POINTS: u16 = 84;

pub const BONE_WEIGHT: I32F32 = I32F32::lit("0");
pub const BONE_CONDUCTIVE: bool = false;
pub const BONE_TWO_HANDED: bool = false;
pub const BONE_MIN_VALUE: I32F32 = I32F32::lit("6");
pub const BONE_MAX_VALUE: I32F32 = I32F32::lit("14");

pub const BONE_DEPENDS_ON: [&str; 1] = ["dine"];

pub const BONE_SPAWN_CHANCES: [BoneSpawnChance; 1] = [BoneSpawnChance {
    moon: "dine",
    chance: I32F32::lit("15.99"),
}];

pub const BONE_BEHAVIORAL_MECHANICS: [BoneBehaviorRule; 2] = [
    BoneBehaviorRule {
        condition: "the item spawns on dine",
        outcome: "its spawn chance is 15.99%",
    },
    BoneBehaviorRule {
        condition: "the item is collected",
        outcome: "it has no documented mechanical use",
    },
];

pub struct BonePlugin;

impl Plugin for BonePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBoneEvent>()
            .add_event::<BoneSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_bone,
                    bone_pickup_item_bar_bridge,
                    bone_sell_bridge,
                    bone_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoneBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoneSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bone {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoneScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for BoneScrap {
    fn default() -> Self {
        Self {
            min_value: BONE_MIN_VALUE,
            max_value: BONE_MAX_VALUE,
            weight: BONE_WEIGHT,
            conductive: BONE_CONDUCTIVE,
            two_handed: BONE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoneHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct BoneBundle {
    pub name: Name,
    pub bone: Bone,
    pub scrap: BoneScrap,
    pub position: SimPosition,
    pub held_by: BoneHeldBy,
}

impl BoneBundle {
    pub fn new(event: SpawnBoneEvent) -> Self {
        Self {
            name: Name::new(BONE_NAME),
            bone: Bone {
                stable_id: event.stable_id,
            },
            scrap: BoneScrap::default(),
            position: event.position,
            held_by: BoneHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnBoneEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoneSoldEvent {
    pub bone_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn bone_value_range() -> (I32F32, I32F32) {
    (BONE_MIN_VALUE, BONE_MAX_VALUE)
}

pub fn bone_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    BONE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_bone(mut commands: Commands, mut events: EventReader<SpawnBoneEvent>) {
    for event in events.read() {
        commands.spawn(BoneBundle::new(*event));
    }
}

fn bone_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    bones: Query<(&Bone, &BoneHeldBy), Changed<BoneHeldBy>>,
) {
    for (bone, held_by) in &bones {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: BONE_ID,
                two_handed: BONE_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = bone.stable_id;
        }
    }
}

fn bone_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<BoneSoldEvent>,
    bones: Query<(&Bone, &BoneScrap)>,
) {
    for event in sell_events.read() {
        for (bone, scrap) in &bones {
            if bone.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(BoneSoldEvent {
                bone_stable_id: bone.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn bone_checksum(
    mut checksum: ResMut<SimChecksumState>,
    bones: Query<(&Bone, &BoneScrap, &SimPosition, &BoneHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, BONE_ID);
    accumulate_str(&mut checksum, 0x1001, BONE_NAME);
    accumulate_str(&mut checksum, 0x1002, BONE_TYPE);
    accumulate_str(&mut checksum, 0x1003, BONE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, BONE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, BONE_EXTRACTED_AT);

    checksum.accumulate(BONE_SOURCE_REVISION as u64);
    checksum.accumulate(BONE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(BONE_WEIGHT.to_bits() as u64);
    checksum.accumulate(BONE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(BONE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(BONE_CONDUCTIVE as u64);
    checksum.accumulate(BONE_TWO_HANDED as u64);

    for dependency in BONE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in BONE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in BONE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (bone, scrap, position, held_by) in &bones {
        checksum.accumulate(bone.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}