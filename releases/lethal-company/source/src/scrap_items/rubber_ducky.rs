// Sources: vault/scrap_items/rubber_ducky.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const RUBBER_DUCKY_ID: &str = "rubber_ducky";
pub const RUBBER_DUCKY_NAME: &str = "Rubber ducky";
pub const RUBBER_DUCKY_TYPE: &str = "scrap_items";
pub const RUBBER_DUCKY_SUBTYPE: &str = "scrap";
pub const RUBBER_DUCKY_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Rubber_Ducky";
pub const RUBBER_DUCKY_SOURCE_REVISION: u32 = 20947;
pub const RUBBER_DUCKY_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const RUBBER_DUCKY_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const RUBBER_DUCKY_EFFECTS: &str = "Quacks when picked up, equipped, or dropped";
pub const RUBBER_DUCKY_WEIGHT: I32F32 = I32F32::lit("0");
pub const RUBBER_DUCKY_CONDUCTIVE: bool = false;
pub const RUBBER_DUCKY_MIN_VALUE: I32F32 = I32F32::lit("2");
pub const RUBBER_DUCKY_MAX_VALUE: I32F32 = I32F32::lit("99");
pub const RUBBER_DUCKY_TWO_HANDED: bool = false;
pub const RUBBER_DUCKY_PAGE_ID: u32 = 49;
pub const RUBBER_DUCKY_QUACK_SOUND_AMOUNT: I32F32 = I32F32::lit("1");
pub const RUBBER_DUCKY_PLASTIC_SOUND_AMOUNT: I32F32 = I32F32::lit("1");

pub const RUBBER_DUCKY_DEPENDS_ON: [&str; 8] = [
    "eyeless_dog",
    "audible_sounds",
    "the_company",
    "artifice",
    "adamance",
    "titan",
    "vow",
    "rend",
];

pub const RUBBER_DUCKY_SPAWN_CHANCES: [RubberDuckySpawnChance; 5] = [
    RubberDuckySpawnChance {
        moon: "artifice",
        chance: I32F32::lit("4.43"),
    },
    RubberDuckySpawnChance {
        moon: "adamance",
        chance: I32F32::lit("2.64"),
    },
    RubberDuckySpawnChance {
        moon: "titan",
        chance: I32F32::lit("2.58"),
    },
    RubberDuckySpawnChance {
        moon: "vow",
        chance: I32F32::lit("2.47"),
    },
    RubberDuckySpawnChance {
        moon: "rend",
        chance: I32F32::lit("1.48"),
    },
];

pub const RUBBER_DUCKY_BEHAVIORAL_MECHANICS: [RubberDuckyBehaviorRule; 8] = [
    RubberDuckyBehaviorRule {
        condition: "picked up",
        outcome: "it emits a quack sound",
    },
    RubberDuckyBehaviorRule {
        condition: "equipped",
        outcome: "it emits a quack sound",
    },
    RubberDuckyBehaviorRule {
        condition: "dropped",
        outcome: "it emits a quack sound and an additional plastic sound",
    },
    RubberDuckyBehaviorRule {
        condition: "its sound reaches eyeless_dog or other entities that respond to audible_sounds",
        outcome: "it can alert them",
    },
    RubberDuckyBehaviorRule {
        condition: "spawned on artifice",
        outcome: "its spawn chance is 4.43%",
    },
    RubberDuckyBehaviorRule {
        condition: "spawned on adamance",
        outcome: "its spawn chance is 2.64%",
    },
    RubberDuckyBehaviorRule {
        condition: "spawned on titan",
        outcome: "its spawn chance is 2.58%",
    },
    RubberDuckyBehaviorRule {
        condition: "spawned on vow",
        outcome: "its spawn chance is 2.47%",
    },
];

pub struct RubberDuckyPlugin;

impl Plugin for RubberDuckyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRubberDuckyEvent>()
            .add_event::<RubberDuckyQuackedEvent>()
            .add_event::<RubberDuckyPlasticSoundEvent>()
            .add_event::<RubberDuckyAudibleAlertEvent>()
            .add_event::<RubberDuckySoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_rubber_ducky,
                    rubber_ducky_held_state_bridge,
                    rubber_ducky_equipped_from_item_bar,
                    rubber_ducky_emit_noise,
                    rubber_ducky_sell_bridge,
                    rubber_ducky_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RubberDuckyBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RubberDuckySpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RubberDucky {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RubberDuckyScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for RubberDuckyScrap {
    fn default() -> Self {
        Self {
            min_value: RUBBER_DUCKY_MIN_VALUE,
            max_value: RUBBER_DUCKY_MAX_VALUE,
            weight: RUBBER_DUCKY_WEIGHT,
            conductive: RUBBER_DUCKY_CONDUCTIVE,
            two_handed: RUBBER_DUCKY_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RubberDuckyHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RubberDuckySoundState {
    pub was_held_last_tick: bool,
    pub quacks: u64,
    pub plastic_sounds: u64,
    pub last_quack_tick: u64,
    pub last_plastic_sound_tick: u64,
}

#[derive(Bundle)]
pub struct RubberDuckyBundle {
    pub name: Name,
    pub rubber_ducky: RubberDucky,
    pub scrap: RubberDuckyScrap,
    pub position: SimPosition,
    pub held_by: RubberDuckyHeldBy,
    pub sound_state: RubberDuckySoundState,
}

impl RubberDuckyBundle {
    pub fn new(event: SpawnRubberDuckyEvent) -> Self {
        Self {
            name: Name::new(RUBBER_DUCKY_NAME),
            rubber_ducky: RubberDucky {
                stable_id: event.stable_id,
            },
            scrap: RubberDuckyScrap::default(),
            position: event.position,
            held_by: RubberDuckyHeldBy::default(),
            sound_state: RubberDuckySoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRubberDuckyEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubberDuckyQuackedEvent {
    pub rubber_ducky_entity: Entity,
    pub rubber_ducky_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub reason: RubberDuckySoundReason,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubberDuckyPlasticSoundEvent {
    pub rubber_ducky_entity: Entity,
    pub rubber_ducky_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubberDuckyAudibleAlertEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
    pub reason: RubberDuckySoundReason,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RubberDuckySoldEvent {
    pub rubber_ducky_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RubberDuckySoundReason {
    PickedUp,
    Equipped,
    Dropped,
}

pub fn rubber_ducky_value_range() -> (I32F32, I32F32) {
    (RUBBER_DUCKY_MIN_VALUE, RUBBER_DUCKY_MAX_VALUE)
}

pub fn rubber_ducky_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    RUBBER_DUCKY_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_rubber_ducky(mut commands: Commands, mut events: EventReader<SpawnRubberDuckyEvent>) {
    for event in events.read() {
        commands.spawn(RubberDuckyBundle::new(*event));
    }
}

fn rubber_ducky_held_state_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    mut quack_events: EventWriter<RubberDuckyQuackedEvent>,
    mut plastic_events: EventWriter<RubberDuckyPlasticSoundEvent>,
    mut rubber_duckies: Query<(
        Entity,
        &RubberDucky,
        &RubberDuckyHeldBy,
        &SimPosition,
        &mut RubberDuckySoundState,
    )>,
    tick: Res<SimTick>,
) {
    for (entity, rubber_ducky, held_by, position, mut sound_state) in &mut rubber_duckies {
        if held_by.is_held && !sound_state.was_held_last_tick {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: RUBBER_DUCKY_ID,
                two_handed: RUBBER_DUCKY_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });

            sound_state.quacks = sound_state.quacks.wrapping_add(1);
            sound_state.last_quack_tick = tick.0;

            quack_events.send(RubberDuckyQuackedEvent {
                rubber_ducky_entity: entity,
                rubber_ducky_stable_id: rubber_ducky.stable_id,
                employee_id: held_by.employee_id,
                position: *position,
                reason: RubberDuckySoundReason::PickedUp,
            });
        }

        if !held_by.is_held && sound_state.was_held_last_tick {
            sound_state.quacks = sound_state.quacks.wrapping_add(1);
            sound_state.plastic_sounds = sound_state.plastic_sounds.wrapping_add(1);
            sound_state.last_quack_tick = tick.0;
            sound_state.last_plastic_sound_tick = tick.0;

            quack_events.send(RubberDuckyQuackedEvent {
                rubber_ducky_entity: entity,
                rubber_ducky_stable_id: rubber_ducky.stable_id,
                employee_id: held_by.employee_id,
                position: *position,
                reason: RubberDuckySoundReason::Dropped,
            });

            plastic_events.send(RubberDuckyPlasticSoundEvent {
                rubber_ducky_entity: entity,
                rubber_ducky_stable_id: rubber_ducky.stable_id,
                employee_id: held_by.employee_id,
                position: *position,
            });
        }

        sound_state.was_held_last_tick = held_by.is_held;
    }
}

fn rubber_ducky_equipped_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut quack_events: EventWriter<RubberDuckyQuackedEvent>,
    mut rubber_duckies: Query<(
        Entity,
        &RubberDucky,
        &RubberDuckyHeldBy,
        &SimPosition,
        &mut RubberDuckySoundState,
    )>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != RUBBER_DUCKY_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, rubber_ducky, held_by, position, mut sound_state) in &mut rubber_duckies {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            sound_state.quacks = sound_state.quacks.wrapping_add(1);
            sound_state.last_quack_tick = tick.0;

            quack_events.send(RubberDuckyQuackedEvent {
                rubber_ducky_entity: entity,
                rubber_ducky_stable_id: rubber_ducky.stable_id,
                employee_id: event.employee_id,
                position: *position,
                reason: RubberDuckySoundReason::Equipped,
            });
        }
    }
}

fn rubber_ducky_emit_noise(
    mut quack_events: EventReader<RubberDuckyQuackedEvent>,
    mut plastic_events: EventReader<RubberDuckyPlasticSoundEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<RubberDuckyAudibleAlertEvent>,
) {
    for event in quack_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.rubber_ducky_entity,
            position: event.position,
            amount: RUBBER_DUCKY_QUACK_SOUND_AMOUNT,
        });

        alert_events.send(RubberDuckyAudibleAlertEvent {
            source: event.rubber_ducky_entity,
            employee_id: event.employee_id,
            position: event.position,
            amount: RUBBER_DUCKY_QUACK_SOUND_AMOUNT,
            reason: event.reason,
        });
    }

    for event in plastic_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.rubber_ducky_entity,
            position: event.position,
            amount: RUBBER_DUCKY_PLASTIC_SOUND_AMOUNT,
        });
    }
}

fn rubber_ducky_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<RubberDuckySoldEvent>,
    rubber_duckies: Query<(&RubberDucky, &RubberDuckyScrap)>,
) {
    for event in sell_events.read() {
        for (rubber_ducky, scrap) in &rubber_duckies {
            if rubber_ducky.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(RubberDuckySoldEvent {
                rubber_ducky_stable_id: rubber_ducky.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn rubber_ducky_checksum(
    mut checksum: ResMut<SimChecksumState>,
    rubber_duckies: Query<(
        &RubberDucky,
        &RubberDuckyScrap,
        &SimPosition,
        &RubberDuckyHeldBy,
        &RubberDuckySoundState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, RUBBER_DUCKY_ID);
    accumulate_str(&mut checksum, 0x1001, RUBBER_DUCKY_NAME);
    accumulate_str(&mut checksum, 0x1002, RUBBER_DUCKY_TYPE);
    accumulate_str(&mut checksum, 0x1003, RUBBER_DUCKY_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, RUBBER_DUCKY_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, RUBBER_DUCKY_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, RUBBER_DUCKY_EXTRACTED_AT);

    checksum.accumulate(RUBBER_DUCKY_SOURCE_REVISION as u64);
    checksum.accumulate(RUBBER_DUCKY_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(RUBBER_DUCKY_WEIGHT.to_bits() as u64);
    checksum.accumulate(RUBBER_DUCKY_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(RUBBER_DUCKY_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(RUBBER_DUCKY_CONDUCTIVE as u64);
    checksum.accumulate(RUBBER_DUCKY_TWO_HANDED as u64);
    checksum.accumulate(RUBBER_DUCKY_PAGE_ID as u64);
    checksum.accumulate(RUBBER_DUCKY_QUACK_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(RUBBER_DUCKY_PLASTIC_SOUND_AMOUNT.to_bits() as u64);

    for dependency in RUBBER_DUCKY_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in RUBBER_DUCKY_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in RUBBER_DUCKY_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (rubber_ducky, scrap, position, held_by, sound_state) in &rubber_duckies {
        checksum.accumulate(rubber_ducky.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(sound_state.was_held_last_tick as u64);
        checksum.accumulate(sound_state.quacks);
        checksum.accumulate(sound_state.plastic_sounds);
        checksum.accumulate(sound_state.last_quack_tick);
        checksum.accumulate(sound_state.last_plastic_sound_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}