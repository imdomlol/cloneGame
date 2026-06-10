// Sources: vault/scrap_items/clown_horn.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const CLOWN_HORN_ID: &str = "clown_horn";
pub const CLOWN_HORN_NAME: &str = "Clown horn";
pub const CLOWN_HORN_TYPE: &str = "scrap_items";
pub const CLOWN_HORN_SUBTYPE: &str = "scrap";
pub const CLOWN_HORN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Clown_Horn";
pub const CLOWN_HORN_SOURCE_REVISION: u32 = 20280;
pub const CLOWN_HORN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const CLOWN_HORN_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const CLOWN_HORN_EFFECTS: &str = "Produces clown horn honking sounds";
pub const CLOWN_HORN_WEIGHT: I32F32 = I32F32::lit("0");
pub const CLOWN_HORN_CONDUCTIVE: bool = true;
pub const CLOWN_HORN_MIN_VALUE: I32F32 = I32F32::lit("52");
pub const CLOWN_HORN_MAX_VALUE: I32F32 = I32F32::lit("71");
pub const CLOWN_HORN_TWO_HANDED: bool = false;
pub const CLOWN_HORN_SOUND_AMOUNT: I32F32 = I32F32::lit("100");
pub const CLOWN_HORN_RANDOM_PITCH_STEPS: u32 = 8;
pub const CLOWN_HORN_PITCH_ROLL_SALT: u64 = 0x636c_6f77_6e68_6f70;

pub const CLOWN_HORN_DEPENDS_ON: [&str; 12] = [
    "eyeless_dog",
    "airhorn",
    "march",
    "vow",
    "artifice",
    "offense",
    "embrion",
    "titan",
    "assurance",
    "adamance",
    "rend",
    "experimentation",
];

pub const CLOWN_HORN_SPAWN_CHANCES: [ClownHornSpawnChance; 10] = [
    ClownHornSpawnChance {
        moon: "march",
        chance: I32F32::lit("4.91"),
    },
    ClownHornSpawnChance {
        moon: "vow",
        chance: I32F32::lit("3.6"),
    },
    ClownHornSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("2.29"),
    },
    ClownHornSpawnChance {
        moon: "offense",
        chance: I32F32::lit("2.16"),
    },
    ClownHornSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("2.08"),
    },
    ClownHornSpawnChance {
        moon: "titan",
        chance: I32F32::lit("1.72"),
    },
    ClownHornSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.18"),
    },
    ClownHornSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("0.95"),
    },
    ClownHornSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.74"),
    },
    ClownHornSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("0.53"),
    },
];

pub const CLOWN_HORN_BEHAVIORAL_MECHANICS: [ClownHornBehaviorRule; 3] = [
    ClownHornBehaviorRule {
        condition: "LMB is used",
        outcome: "the item emits loud, randomly pitched honking sounds",
    },
    ClownHornBehaviorRule {
        condition: "the horn is used",
        outcome: "the sound can alert eyeless_dog",
    },
    ClownHornBehaviorRule {
        condition: "the horn is used",
        outcome: "the sound behaves similarly to airhorn",
    },
];

pub struct ClownHornPlugin;

impl Plugin for ClownHornPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnClownHornEvent>()
            .add_event::<ClownHornUsedEvent>()
            .add_event::<ClownHornAudibleAlertEvent>()
            .add_event::<ClownHornSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_clown_horn,
                    clown_horn_pickup_item_bar_bridge,
                    clown_horn_use_from_item_bar,
                    clown_horn_emit_noise,
                    clown_horn_sell_bridge,
                    clown_horn_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClownHornBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClownHornSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClownHorn {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClownHornScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ClownHornScrap {
    fn default() -> Self {
        Self {
            min_value: CLOWN_HORN_MIN_VALUE,
            max_value: CLOWN_HORN_MAX_VALUE,
            weight: CLOWN_HORN_WEIGHT,
            conductive: CLOWN_HORN_CONDUCTIVE,
            two_handed: CLOWN_HORN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClownHornHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClownHornSoundState {
    pub last_pitch_step: u32,
    pub uses: u64,
    pub last_used_tick: u64,
}

impl Default for ClownHornSoundState {
    fn default() -> Self {
        Self {
            last_pitch_step: 0,
            uses: 0,
            last_used_tick: 0,
        }
    }
}

#[derive(Bundle)]
pub struct ClownHornBundle {
    pub name: Name,
    pub clown_horn: ClownHorn,
    pub scrap: ClownHornScrap,
    pub position: SimPosition,
    pub held_by: ClownHornHeldBy,
    pub sound_state: ClownHornSoundState,
}

impl ClownHornBundle {
    pub fn new(event: SpawnClownHornEvent) -> Self {
        Self {
            name: Name::new(CLOWN_HORN_NAME),
            clown_horn: ClownHorn {
                stable_id: event.stable_id,
            },
            scrap: ClownHornScrap::default(),
            position: event.position,
            held_by: ClownHornHeldBy::default(),
            sound_state: ClownHornSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnClownHornEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClownHornUsedEvent {
    pub clown_horn_entity: Entity,
    pub clown_horn_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub pitch_step: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClownHornAudibleAlertEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClownHornSoldEvent {
    pub clown_horn_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn clown_horn_value_range() -> (I32F32, I32F32) {
    (CLOWN_HORN_MIN_VALUE, CLOWN_HORN_MAX_VALUE)
}

pub fn clown_horn_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    CLOWN_HORN_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_clown_horn(mut commands: Commands, mut events: EventReader<SpawnClownHornEvent>) {
    for event in events.read() {
        commands.spawn(ClownHornBundle::new(*event));
    }
}

fn clown_horn_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    clown_horns: Query<(&ClownHorn, &ClownHornHeldBy), Changed<ClownHornHeldBy>>,
) {
    for (clown_horn, held_by) in &clown_horns {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: CLOWN_HORN_ID,
                two_handed: CLOWN_HORN_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = clown_horn.stable_id;
        }
    }
}

fn clown_horn_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut used_events: EventWriter<ClownHornUsedEvent>,
    mut clown_horns: Query<(
        Entity,
        &ClownHorn,
        &ClownHornHeldBy,
        &SimPosition,
        &mut ClownHornSoundState,
    )>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != CLOWN_HORN_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, clown_horn, held_by, position, mut sound_state) in &mut clown_horns {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            let salt = CLOWN_HORN_PITCH_ROLL_SALT ^ clown_horn.stable_id;
            let mut rng = tick_rng(seed.0, tick.0, salt);
            let pitch_step = rng.next_u32() % CLOWN_HORN_RANDOM_PITCH_STEPS;

            sound_state.uses = sound_state.uses.wrapping_add(1);
            sound_state.last_used_tick = tick.0;
            sound_state.last_pitch_step = pitch_step;

            used_events.send(ClownHornUsedEvent {
                clown_horn_entity: entity,
                clown_horn_stable_id: clown_horn.stable_id,
                employee_id: event.employee_id,
                position: *position,
                pitch_step,
            });
        }
    }
}

fn clown_horn_emit_noise(
    mut used_events: EventReader<ClownHornUsedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<ClownHornAudibleAlertEvent>,
) {
    for event in used_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.clown_horn_entity,
            position: event.position,
            amount: CLOWN_HORN_SOUND_AMOUNT,
        });

        alert_events.send(ClownHornAudibleAlertEvent {
            source: event.clown_horn_entity,
            employee_id: event.employee_id,
            position: event.position,
            amount: CLOWN_HORN_SOUND_AMOUNT,
        });
    }
}

fn clown_horn_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ClownHornSoldEvent>,
    clown_horns: Query<(&ClownHorn, &ClownHornScrap)>,
) {
    for event in sell_events.read() {
        for (clown_horn, scrap) in &clown_horns {
            if clown_horn.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ClownHornSoldEvent {
                clown_horn_stable_id: clown_horn.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn clown_horn_checksum(
    mut checksum: ResMut<SimChecksumState>,
    clown_horns: Query<(
        &ClownHorn,
        &ClownHornScrap,
        &SimPosition,
        &ClownHornHeldBy,
        &ClownHornSoundState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, CLOWN_HORN_ID);
    accumulate_str(&mut checksum, 0x1001, CLOWN_HORN_NAME);
    accumulate_str(&mut checksum, 0x1002, CLOWN_HORN_TYPE);
    accumulate_str(&mut checksum, 0x1003, CLOWN_HORN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CLOWN_HORN_EFFECTS);

    checksum.accumulate(CLOWN_HORN_SOURCE_REVISION as u64);
    checksum.accumulate(CLOWN_HORN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CLOWN_HORN_WEIGHT.to_bits() as u64);
    checksum.accumulate(CLOWN_HORN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(CLOWN_HORN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(CLOWN_HORN_CONDUCTIVE as u64);
    checksum.accumulate(CLOWN_HORN_TWO_HANDED as u64);
    checksum.accumulate(CLOWN_HORN_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(CLOWN_HORN_RANDOM_PITCH_STEPS as u64);

    for dependency in CLOWN_HORN_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in CLOWN_HORN_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in CLOWN_HORN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (clown_horn, scrap, position, held_by, sound_state) in &clown_horns {
        checksum.accumulate(clown_horn.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(sound_state.last_pitch_step as u64);
        checksum.accumulate(sound_state.uses);
        checksum.accumulate(sound_state.last_used_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}