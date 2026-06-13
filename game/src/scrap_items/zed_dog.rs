// Sources: vault/scrap_items/zed_dog.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition, SimTick};

pub const ZED_DOG_ID: &str = "zed_dog";
pub const ZED_DOG_NAME: &str = "Zed Dog";
pub const ZED_DOG_TYPE: &str = "scrap_items";
pub const ZED_DOG_SUBTYPE: &str = "scrap";
pub const ZED_DOG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Zed_Dog";
pub const ZED_DOG_SOURCE_REVISION: u32 = 20944;
pub const ZED_DOG_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const ZED_DOG_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const ZED_DOG_EFFECTS: &str = "Squeaks when picked up or equipped";
pub const ZED_DOG_WEIGHT: I32F32 = I32F32::lit("0");
pub const ZED_DOG_CONDUCTIVE: bool = true;
pub const ZED_DOG_MIN_VALUE: I32F32 = I32F32::lit("0");
pub const ZED_DOG_MAX_VALUE: I32F32 = I32F32::lit("199");
pub const ZED_DOG_TWO_HANDED: bool = false;
// Squeak does not route through NoiseEmittedEvent; eyeless_dog is not alerted by this item.
pub const ZED_DOG_ALERTS_EYELESS_DOG: bool = false;

pub const ZED_DOG_DEPENDS_ON: [&str; 8] = [
    "eyeless_dog",
    "the_company",
    "rend",
    "assurance",
    "offense",
    "march",
    "vow",
    "artifice",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZedDogBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZedDogSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

pub const ZED_DOG_BEHAVIORAL_MECHANICS: [ZedDogBehaviorRule; 4] = [
    ZedDogBehaviorRule {
        condition: "Zed Dog is picked up or equipped",
        outcome: "it plays a unique squeaking sound",
    },
    ZedDogBehaviorRule {
        condition: "Zed Dog is dropped",
        outcome: "it plays a metallic sound",
    },
    ZedDogBehaviorRule {
        condition: "Zed Dog is present as scrap",
        outcome: "it is conductive",
    },
    ZedDogBehaviorRule {
        condition: "Zed Dog is evaluated for noise detection",
        outcome: "it does not alert eyeless_dog",
    },
];

pub const ZED_DOG_SPAWN_CHANCES: [ZedDogSpawnChance; 6] = [
    ZedDogSpawnChance { moon: "rend",      chance: I32F32::lit("0.19") },
    ZedDogSpawnChance { moon: "assurance", chance: I32F32::lit("0.12") },
    ZedDogSpawnChance { moon: "offense",   chance: I32F32::lit("0.12") },
    ZedDogSpawnChance { moon: "march",     chance: I32F32::lit("0.11") },
    ZedDogSpawnChance { moon: "vow",       chance: I32F32::lit("0.10") },
    ZedDogSpawnChance { moon: "artifice",  chance: I32F32::lit("0.07") },
];

pub struct ZedDogPlugin;

impl Plugin for ZedDogPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnZedDogEvent>()
            .add_event::<ZedDogSqueakEvent>()
            .add_event::<ZedDogDropSoundEvent>()
            .add_event::<ZedDogSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_zed_dog,
                    zed_dog_pickup_bridge,
                    zed_dog_sell_bridge,
                    zed_dog_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZedDog {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZedDogScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ZedDogScrap {
    fn default() -> Self {
        Self {
            min_value: ZED_DOG_MIN_VALUE,
            max_value: ZED_DOG_MAX_VALUE,
            weight: ZED_DOG_WEIGHT,
            conductive: ZED_DOG_CONDUCTIVE,
            two_handed: ZED_DOG_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZedDogHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZedDogSoundState {
    pub squeak_count: u64,
    pub last_squeak_tick: u64,
    pub drop_count: u64,
    pub last_drop_tick: u64,
}

#[derive(Bundle)]
pub struct ZedDogBundle {
    pub name: Name,
    pub zed_dog: ZedDog,
    pub scrap: ZedDogScrap,
    pub position: SimPosition,
    pub held_by: ZedDogHeldBy,
    pub sound_state: ZedDogSoundState,
}

impl ZedDogBundle {
    pub fn new(event: SpawnZedDogEvent) -> Self {
        Self {
            name: Name::new(ZED_DOG_NAME),
            zed_dog: ZedDog { stable_id: event.stable_id },
            scrap: ZedDogScrap::default(),
            position: event.position,
            held_by: ZedDogHeldBy::default(),
            sound_state: ZedDogSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnZedDogEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZedDogSqueakEvent {
    pub zed_dog_entity: Entity,
    pub zed_dog_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZedDogDropSoundEvent {
    pub zed_dog_entity: Entity,
    pub zed_dog_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZedDogSoldEvent {
    pub zed_dog_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn zed_dog_value_range() -> (I32F32, I32F32) {
    (ZED_DOG_MIN_VALUE, ZED_DOG_MAX_VALUE)
}

pub fn zed_dog_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    ZED_DOG_SPAWN_CHANCES
        .iter()
        .find(|s| s.moon == moon)
        .map(|s| s.chance)
}

fn spawn_zed_dog(mut commands: Commands, mut events: EventReader<SpawnZedDogEvent>) {
    for event in events.read() {
        commands.spawn(ZedDogBundle::new(*event));
    }
}

fn zed_dog_pickup_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    mut squeak_events: EventWriter<ZedDogSqueakEvent>,
    mut drop_events: EventWriter<ZedDogDropSoundEvent>,
    mut zed_dogs: Query<
        (Entity, &ZedDog, &ZedDogHeldBy, &SimPosition, &mut ZedDogSoundState),
        Changed<ZedDogHeldBy>,
    >,
    tick: Res<SimTick>,
) {
    for (entity, zed_dog, held_by, position, mut sound_state) in &mut zed_dogs {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: ZED_DOG_ID,
                two_handed: ZED_DOG_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
            sound_state.squeak_count = sound_state.squeak_count.wrapping_add(1);
            sound_state.last_squeak_tick = tick.0;
            squeak_events.send(ZedDogSqueakEvent {
                zed_dog_entity: entity,
                zed_dog_stable_id: zed_dog.stable_id,
                employee_id: held_by.employee_id,
                position: *position,
            });
        } else if sound_state.last_squeak_tick > 0 {
            sound_state.drop_count = sound_state.drop_count.wrapping_add(1);
            sound_state.last_drop_tick = tick.0;
            drop_events.send(ZedDogDropSoundEvent {
                zed_dog_entity: entity,
                zed_dog_stable_id: zed_dog.stable_id,
                position: *position,
            });
        }
    }
}

fn zed_dog_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ZedDogSoldEvent>,
    zed_dogs: Query<(&ZedDog, &ZedDogScrap)>,
) {
    for event in sell_events.read() {
        for (zed_dog, scrap) in &zed_dogs {
            if zed_dog.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(ZedDogSoldEvent {
                zed_dog_stable_id: zed_dog.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn zed_dog_checksum(
    mut checksum: ResMut<SimChecksumState>,
    zed_dogs: Query<(&ZedDog, &ZedDogScrap, &SimPosition, &ZedDogHeldBy, &ZedDogSoundState)>,
) {
    accumulate_str(&mut checksum, 0x1000, ZED_DOG_ID);
    accumulate_str(&mut checksum, 0x1001, ZED_DOG_NAME);
    accumulate_str(&mut checksum, 0x1002, ZED_DOG_TYPE);
    accumulate_str(&mut checksum, 0x1003, ZED_DOG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ZED_DOG_EFFECTS);

    checksum.accumulate(ZED_DOG_SOURCE_REVISION as u64);
    checksum.accumulate(ZED_DOG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ZED_DOG_WEIGHT.to_bits() as u64);
    checksum.accumulate(ZED_DOG_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(ZED_DOG_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(ZED_DOG_CONDUCTIVE as u64);
    checksum.accumulate(ZED_DOG_TWO_HANDED as u64);
    checksum.accumulate(ZED_DOG_ALERTS_EYELESS_DOG as u64);

    for dependency in ZED_DOG_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in ZED_DOG_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in ZED_DOG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (zed_dog, scrap, position, held_by, sound_state) in &zed_dogs {
        checksum.accumulate(zed_dog.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(sound_state.squeak_count);
        checksum.accumulate(sound_state.last_squeak_tick);
        checksum.accumulate(sound_state.drop_count);
        checksum.accumulate(sound_state.last_drop_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}