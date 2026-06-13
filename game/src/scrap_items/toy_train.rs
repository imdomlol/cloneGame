// Sources: vault/scrap_items/toy_train.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition, SimTick};

pub const TOY_TRAIN_ID: &str = "toy_train";
pub const TOY_TRAIN_NAME: &str = "Toy train";
pub const TOY_TRAIN_TYPE: &str = "scrap_items";
pub const TOY_TRAIN_SUBTYPE: &str = "scrap";
pub const TOY_TRAIN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Toy_Train";
pub const TOY_TRAIN_SOURCE_REVISION: u32 = 20434;
pub const TOY_TRAIN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const TOY_TRAIN_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const TOY_TRAIN_EFFECTS: &str = "Makes chu-chu noises when picked up, equipped or dropped";
pub const TOY_TRAIN_WEIGHT: I32F32 = I32F32::lit("21");
pub const TOY_TRAIN_CONDUCTIVE: bool = false;
pub const TOY_TRAIN_MIN_VALUE: I32F32 = I32F32::lit("52");
pub const TOY_TRAIN_MAX_VALUE: I32F32 = I32F32::lit("83");
pub const TOY_TRAIN_TWO_HANDED: bool = false;

pub const TOY_TRAIN_DEPENDS_ON: [&str; 9] = [
    "scrap",
    "lethal_company",
    "eyeless_dog",
    "audible_sounds",
    "the_company",
    "credits",
    "rend",
    "artifice",
    "offense",
];

pub const TOY_TRAIN_SPAWN_CHANCES: [ToyTrainSpawnChance; 3] = [
    ToyTrainSpawnChance {
        moon: "rend",
        chance: I32F32::lit("3.24"),
    },
    ToyTrainSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("1.62"),
    },
    ToyTrainSpawnChance {
        moon: "offense",
        chance: I32F32::lit("0.72"),
    },
];

pub const TOY_TRAIN_BEHAVIORAL_MECHANICS: [ToyTrainBehaviorRule; 5] = [
    ToyTrainBehaviorRule {
        condition: "picked up",
        outcome: "the item plays a chu-chu sound",
    },
    ToyTrainBehaviorRule {
        condition: "equipped",
        outcome: "the item plays a chu-chu sound",
    },
    ToyTrainBehaviorRule {
        condition: "dropped",
        outcome: "the item plays a chu-chu sound",
    },
    ToyTrainBehaviorRule {
        condition: "the sound reaches eyeless_dog or another entity that uses audible_sounds",
        outcome: "it does not alert them",
    },
    ToyTrainBehaviorRule {
        condition: "sold to the_company",
        outcome: "it returns 52 to 83 credits",
    },
];

pub struct ToyTrainPlugin;

impl Plugin for ToyTrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnToyTrainEvent>()
            .add_event::<ToyTrainSoundEvent>()
            .add_event::<ToyTrainSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_toy_train,
                    toy_train_pickup_item_bar_bridge,
                    toy_train_sound_on_held_changed,
                    toy_train_sell_bridge,
                    toy_train_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToyTrainBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToyTrainSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToyTrainSoundTrigger {
    Pickup,
    Drop,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToyTrain {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToyTrainScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ToyTrainScrap {
    fn default() -> Self {
        Self {
            min_value: TOY_TRAIN_MIN_VALUE,
            max_value: TOY_TRAIN_MAX_VALUE,
            weight: TOY_TRAIN_WEIGHT,
            conductive: TOY_TRAIN_CONDUCTIVE,
            two_handed: TOY_TRAIN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToyTrainHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToyTrainSoundState {
    pub total_sounds: u64,
    pub last_sound_tick: u64,
}

#[derive(Bundle)]
pub struct ToyTrainBundle {
    pub name: Name,
    pub toy_train: ToyTrain,
    pub scrap: ToyTrainScrap,
    pub position: SimPosition,
    pub held_by: ToyTrainHeldBy,
    pub sound_state: ToyTrainSoundState,
}

impl ToyTrainBundle {
    pub fn new(event: SpawnToyTrainEvent) -> Self {
        Self {
            name: Name::new(TOY_TRAIN_NAME),
            toy_train: ToyTrain {
                stable_id: event.stable_id,
            },
            scrap: ToyTrainScrap::default(),
            position: event.position,
            held_by: ToyTrainHeldBy::default(),
            sound_state: ToyTrainSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnToyTrainEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToyTrainSoundEvent {
    pub toy_train_entity: Entity,
    pub toy_train_stable_id: u64,
    pub trigger: ToyTrainSoundTrigger,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToyTrainSoldEvent {
    pub toy_train_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn toy_train_value_range() -> (I32F32, I32F32) {
    (TOY_TRAIN_MIN_VALUE, TOY_TRAIN_MAX_VALUE)
}

pub fn toy_train_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    TOY_TRAIN_SPAWN_CHANCES
        .iter()
        .find(|s| s.moon == moon)
        .map(|s| s.chance)
}

fn spawn_toy_train(mut commands: Commands, mut events: EventReader<SpawnToyTrainEvent>) {
    for event in events.read() {
        commands.spawn(ToyTrainBundle::new(*event));
    }
}

fn toy_train_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    toy_trains: Query<(&ToyTrain, &ToyTrainHeldBy), Changed<ToyTrainHeldBy>>,
) {
    for (toy_train, held_by) in &toy_trains {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: TOY_TRAIN_ID,
                two_handed: TOY_TRAIN_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = toy_train.stable_id;
        }
    }
}

fn toy_train_sound_on_held_changed(
    mut sound_events: EventWriter<ToyTrainSoundEvent>,
    mut toy_trains: Query<
        (
            Entity,
            &ToyTrain,
            &ToyTrainHeldBy,
            &SimPosition,
            &mut ToyTrainSoundState,
        ),
        Changed<ToyTrainHeldBy>,
    >,
    tick: Res<SimTick>,
) {
    for (entity, toy_train, held_by, position, mut sound_state) in &mut toy_trains {
        let trigger = if held_by.is_held {
            ToyTrainSoundTrigger::Pickup
        } else {
            ToyTrainSoundTrigger::Drop
        };

        sound_state.total_sounds = sound_state.total_sounds.wrapping_add(1);
        sound_state.last_sound_tick = tick.0;

        sound_events.send(ToyTrainSoundEvent {
            toy_train_entity: entity,
            toy_train_stable_id: toy_train.stable_id,
            trigger,
            position: *position,
        });
    }
}

fn toy_train_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ToyTrainSoldEvent>,
    toy_trains: Query<(&ToyTrain, &ToyTrainScrap)>,
) {
    for event in sell_events.read() {
        for (toy_train, scrap) in &toy_trains {
            if toy_train.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(ToyTrainSoldEvent {
                toy_train_stable_id: toy_train.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn toy_train_checksum(
    mut checksum: ResMut<SimChecksumState>,
    toy_trains: Query<(
        &ToyTrain,
        &ToyTrainScrap,
        &SimPosition,
        &ToyTrainHeldBy,
        &ToyTrainSoundState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, TOY_TRAIN_ID);
    accumulate_str(&mut checksum, 0x1001, TOY_TRAIN_NAME);
    accumulate_str(&mut checksum, 0x1002, TOY_TRAIN_TYPE);
    accumulate_str(&mut checksum, 0x1003, TOY_TRAIN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TOY_TRAIN_EFFECTS);

    checksum.accumulate(TOY_TRAIN_SOURCE_REVISION as u64);
    checksum.accumulate(TOY_TRAIN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TOY_TRAIN_WEIGHT.to_bits() as u64);
    checksum.accumulate(TOY_TRAIN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(TOY_TRAIN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(TOY_TRAIN_CONDUCTIVE as u64);
    checksum.accumulate(TOY_TRAIN_TWO_HANDED as u64);

    for dependency in TOY_TRAIN_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in TOY_TRAIN_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in TOY_TRAIN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (toy_train, scrap, position, held_by, sound_state) in &toy_trains {
        checksum.accumulate(toy_train.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(sound_state.total_sounds);
        checksum.accumulate(sound_state.last_sound_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}