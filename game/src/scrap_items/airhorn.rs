// Sources: vault/scrap_items/airhorn.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const AIRHORN_ID: &str = "airhorn";
pub const AIRHORN_NAME: &str = "Airhorn";
pub const AIRHORN_TYPE: &str = "scrap_items";
pub const AIRHORN_SUBTYPE: &str = "scrap";
pub const AIRHORN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Airhorn";
pub const AIRHORN_SOURCE_REVISION: u32 = 20469;
pub const AIRHORN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const AIRHORN_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const AIRHORN_EFFECTS: &str = "Produces airhorn sounds";
pub const AIRHORN_WEIGHT: I32F32 = I32F32::lit("0");
pub const AIRHORN_CONDUCTIVE: bool = false;
pub const AIRHORN_MIN_VALUE: I32F32 = I32F32::lit("52");
pub const AIRHORN_MAX_VALUE: I32F32 = I32F32::lit("71");
pub const AIRHORN_TWO_HANDED: bool = false;
pub const AIRHORN_SOUND_AMOUNT: I32F32 = I32F32::lit("100");
pub const AIRHORN_DETER_BABOON_HAWK_TICKS: u16 = 75;
pub const AIRHORN_RANDOM_PITCH_STEPS: u32 = 8;
pub const AIRHORN_PITCH_ROLL_SALT: u64 = 0x6169_7268_6f72_6e70;

pub const AIRHORN_DEPENDS_ON: [&str; 6] = [
    "clown_horn",
    "eyeless_dog",
    "audible_sounds",
    "the_company",
    "credits",
    "baboon_hawk",
];

pub const AIRHORN_AUDIO: [&str; 2] = [
    "Sound whenever an Airhorn is used.",
    "Sound whenever an Airhorn is used, from far away.",
];

pub const AIRHORN_SPAWN_CHANCES: [AirhornSpawnChance; 10] = [
    AirhornSpawnChance {
        moon: "march",
        chance: I32F32::lit("3.88"),
    },
    AirhornSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("2.88"),
    },
    AirhornSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("2.66"),
    },
    AirhornSpawnChance {
        moon: "titan",
        chance: I32F32::lit("2.58"),
    },
    AirhornSpawnChance {
        moon: "offense",
        chance: I32F32::lit("1.8"),
    },
    AirhornSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.37"),
    },
    AirhornSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.18"),
    },
    AirhornSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.03"),
    },
    AirhornSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.74"),
    },
    AirhornSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("0.53"),
    },
];

pub const AIRHORN_BEHAVIORAL_MECHANICS: [AirhornBehaviorRule; 5] = [
    AirhornBehaviorRule {
        condition: "the Airhorn is used",
        outcome: "it emits loud, randomly pitched airhorn sounds",
    },
    AirhornBehaviorRule {
        condition: "the sound reaches eyeless_dog or another entity capable of hearing audible_sounds",
        outcome: "it can alert them",
    },
    AirhornBehaviorRule {
        condition: "the Airhorn is used",
        outcome: "it behaves similarly to clown_horn",
    },
    AirhornBehaviorRule {
        condition: "the Airhorn is used near baboon_hawk",
        outcome: "the sound can deter it",
    },
    AirhornBehaviorRule {
        condition: "the Airhorn is sold to the_company",
        outcome: "it can be exchanged for credits",
    },
];

pub struct AirhornPlugin;

impl Plugin for AirhornPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnAirhornEvent>()
            .add_event::<AirhornUsedEvent>()
            .add_event::<AirhornAudibleAlertEvent>()
            .add_event::<AirhornBaboonHawkDeterEvent>()
            .add_event::<AirhornSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_airhorn,
                    airhorn_pickup_item_bar_bridge,
                    airhorn_use_from_item_bar,
                    airhorn_emit_noise,
                    airhorn_decay_deter_effect,
                    airhorn_sell_bridge,
                    airhorn_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirhornBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirhornSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Airhorn {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirhornScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for AirhornScrap {
    fn default() -> Self {
        Self {
            min_value: AIRHORN_MIN_VALUE,
            max_value: AIRHORN_MAX_VALUE,
            weight: AIRHORN_WEIGHT,
            conductive: AIRHORN_CONDUCTIVE,
            two_handed: AIRHORN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AirhornHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirhornSoundState {
    pub last_pitch_step: u32,
    pub uses: u64,
    pub last_used_tick: u64,
    pub baboon_hawk_deter_ticks_remaining: u16,
}

impl Default for AirhornSoundState {
    fn default() -> Self {
        Self {
            last_pitch_step: 0,
            uses: 0,
            last_used_tick: 0,
            baboon_hawk_deter_ticks_remaining: 0,
        }
    }
}

#[derive(Bundle)]
pub struct AirhornBundle {
    pub name: Name,
    pub airhorn: Airhorn,
    pub scrap: AirhornScrap,
    pub position: SimPosition,
    pub held_by: AirhornHeldBy,
    pub sound_state: AirhornSoundState,
}

impl AirhornBundle {
    pub fn new(event: SpawnAirhornEvent) -> Self {
        Self {
            name: Name::new(AIRHORN_NAME),
            airhorn: Airhorn {
                stable_id: event.stable_id,
            },
            scrap: AirhornScrap::default(),
            position: event.position,
            held_by: AirhornHeldBy::default(),
            sound_state: AirhornSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnAirhornEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirhornUsedEvent {
    pub airhorn_entity: Entity,
    pub airhorn_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub pitch_step: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirhornAudibleAlertEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirhornBaboonHawkDeterEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub ticks_remaining: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AirhornSoldEvent {
    pub airhorn_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn airhorn_value_range() -> (I32F32, I32F32) {
    (AIRHORN_MIN_VALUE, AIRHORN_MAX_VALUE)
}

pub fn airhorn_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    AIRHORN_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_airhorn(mut commands: Commands, mut events: EventReader<SpawnAirhornEvent>) {
    for event in events.read() {
        commands.spawn(AirhornBundle::new(*event));
    }
}

fn airhorn_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    airhorns: Query<(&Airhorn, &AirhornHeldBy), Changed<AirhornHeldBy>>,
) {
    for (airhorn, held_by) in &airhorns {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: AIRHORN_ID,
                two_handed: AIRHORN_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = airhorn.stable_id;
        }
    }
}

fn airhorn_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut used_events: EventWriter<AirhornUsedEvent>,
    mut airhorns: Query<(
        Entity,
        &Airhorn,
        &AirhornHeldBy,
        &SimPosition,
        &mut AirhornSoundState,
    )>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != AIRHORN_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, airhorn, held_by, position, mut sound_state) in &mut airhorns {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            let salt = AIRHORN_PITCH_ROLL_SALT ^ airhorn.stable_id;
            let mut rng = tick_rng(seed.0, tick.0, salt);
            let pitch_step = rng.next_u32() % AIRHORN_RANDOM_PITCH_STEPS;

            sound_state.uses = sound_state.uses.wrapping_add(1);
            sound_state.last_used_tick = tick.0;
            sound_state.last_pitch_step = pitch_step;
            sound_state.baboon_hawk_deter_ticks_remaining = AIRHORN_DETER_BABOON_HAWK_TICKS;

            used_events.send(AirhornUsedEvent {
                airhorn_entity: entity,
                airhorn_stable_id: airhorn.stable_id,
                employee_id: event.employee_id,
                position: *position,
                pitch_step,
            });
        }
    }
}

fn airhorn_emit_noise(
    mut used_events: EventReader<AirhornUsedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<AirhornAudibleAlertEvent>,
    mut deter_events: EventWriter<AirhornBaboonHawkDeterEvent>,
) {
    for event in used_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.airhorn_entity,
            position: event.position,
            amount: AIRHORN_SOUND_AMOUNT,
        });

        alert_events.send(AirhornAudibleAlertEvent {
            source: event.airhorn_entity,
            employee_id: event.employee_id,
            position: event.position,
            amount: AIRHORN_SOUND_AMOUNT,
        });

        deter_events.send(AirhornBaboonHawkDeterEvent {
            source: event.airhorn_entity,
            employee_id: event.employee_id,
            ticks_remaining: AIRHORN_DETER_BABOON_HAWK_TICKS,
        });
    }
}

fn airhorn_decay_deter_effect(mut airhorns: Query<&mut AirhornSoundState, With<Airhorn>>) {
    for mut sound_state in &mut airhorns {
        if sound_state.baboon_hawk_deter_ticks_remaining > 0 {
            sound_state.baboon_hawk_deter_ticks_remaining -= 1;
        }
    }
}

fn airhorn_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<AirhornSoldEvent>,
    airhorns: Query<(&Airhorn, &AirhornScrap)>,
) {
    for event in sell_events.read() {
        for (airhorn, scrap) in &airhorns {
            if airhorn.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(AirhornSoldEvent {
                airhorn_stable_id: airhorn.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn airhorn_checksum(
    mut checksum: ResMut<SimChecksumState>,
    airhorns: Query<(&Airhorn, &AirhornScrap, &SimPosition, &AirhornHeldBy, &AirhornSoundState)>,
) {
    accumulate_str(&mut checksum, 0x1000, AIRHORN_ID);
    accumulate_str(&mut checksum, 0x1001, AIRHORN_NAME);
    accumulate_str(&mut checksum, 0x1002, AIRHORN_TYPE);
    accumulate_str(&mut checksum, 0x1003, AIRHORN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, AIRHORN_EFFECTS);

    checksum.accumulate(AIRHORN_SOURCE_REVISION as u64);
    checksum.accumulate(AIRHORN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(AIRHORN_WEIGHT.to_bits() as u64);
    checksum.accumulate(AIRHORN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(AIRHORN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(AIRHORN_CONDUCTIVE as u64);
    checksum.accumulate(AIRHORN_TWO_HANDED as u64);
    checksum.accumulate(AIRHORN_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(AIRHORN_DETER_BABOON_HAWK_TICKS as u64);
    checksum.accumulate(AIRHORN_RANDOM_PITCH_STEPS as u64);

    for dependency in AIRHORN_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for audio in AIRHORN_AUDIO {
        accumulate_str(&mut checksum, 0x3000, audio);
    }

    for spawn_chance in AIRHORN_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in AIRHORN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (airhorn, scrap, position, held_by, sound_state) in &airhorns {
        checksum.accumulate(airhorn.stable_id);
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
        checksum.accumulate(sound_state.baboon_hawk_deter_ticks_remaining as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}