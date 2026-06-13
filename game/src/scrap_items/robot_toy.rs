// Sources: vault/scrap_items/robot_toy.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const ROBOT_TOY_ID: &str = "robot_toy";
pub const ROBOT_TOY_NAME: &str = "Robot Toy";
pub const ROBOT_TOY_TYPE: &str = "scrap_items";
pub const ROBOT_TOY_SUBTYPE: &str = "scrap";
pub const ROBOT_TOY_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Toy_Robot";
pub const ROBOT_TOY_SOURCE_REVISION: u32 = 20360;
pub const ROBOT_TOY_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ROBOT_TOY_CONFIDENCE_BASIS_POINTS: u16 = 86;
pub const ROBOT_TOY_WIKI_ID: u32 = 48;

pub const ROBOT_TOY_EFFECTS: &str = "Can make robotic noises when picked up";
pub const ROBOT_TOY_WEIGHT: I32F32 = I32F32::lit("21");
pub const ROBOT_TOY_CONDUCTIVE: bool = true;
pub const ROBOT_TOY_MIN_VALUE: I32F32 = I32F32::lit("56");
pub const ROBOT_TOY_MAX_VALUE: I32F32 = I32F32::lit("87");
pub const ROBOT_TOY_TWO_HANDED: bool = false;

pub const ROBOT_TOY_NOISE_AMOUNT: I32F32 = I32F32::lit("50");
// 50% activation chance: roll % 2 == 0
pub const ROBOT_TOY_NOISE_ACTIVATION_DENOM: u32 = 2;
pub const ROBOT_TOY_NOISE_EMIT_INTERVAL_TICKS: u64 = 60;
pub const ROBOT_TOY_NOISE_STOP_PICKUP_CYCLES: u32 = 3;
pub const ROBOT_TOY_NOISE_ROLL_SALT: u64 = 0x726f626f745f7479;

pub const ROBOT_TOY_DEPENDS_ON: [&str; 3] = ["the_company", "eyeless_dog", "audible_sounds"];

pub const ROBOT_TOY_SPAWN_CHANCES: [RobotToySpawnChance; 9] = [
    RobotToySpawnChance { moon: "embrion", chance: I32F32::lit("4.98") },
    RobotToySpawnChance { moon: "artifice", chance: I32F32::lit("4.73") },
    RobotToySpawnChance { moon: "rend", chance: I32F32::lit("4.26") },
    RobotToySpawnChance { moon: "titan", chance: I32F32::lit("3.77") },
    RobotToySpawnChance { moon: "offense", chance: I32F32::lit("1.08") },
    RobotToySpawnChance { moon: "adamance", chance: I32F32::lit("0.95") },
    RobotToySpawnChance { moon: "assurance", chance: I32F32::lit("0.71") },
    RobotToySpawnChance { moon: "vow", chance: I32F32::lit("0.62") },
    RobotToySpawnChance { moon: "march", chance: I32F32::lit("0.23") },
];

pub const ROBOT_TOY_BEHAVIORAL_MECHANICS: [RobotToyBehaviorRule; 3] = [
    RobotToyBehaviorRule {
        condition: "the item is picked up or held",
        outcome: "it can emit robotic noises with an unspecified chance while held or stowed in inventory",
    },
    RobotToyBehaviorRule {
        condition: "the noise is active",
        outcome: "it can alert eyeless_dog and other entities capable of hearing audible_sounds",
    },
    RobotToyBehaviorRule {
        condition: "you drop and repeatedly pick up the item",
        outcome: "the noise can stop after several pickup cycles",
    },
];

pub struct RobotToyPlugin;

impl Plugin for RobotToyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRobotToyEvent>()
            .add_event::<RobotToyNoiseEmittedEvent>()
            .add_event::<RobotToyAudibleAlertEvent>()
            .add_event::<RobotToySoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_robot_toy,
                    robot_toy_pickup_item_bar_bridge,
                    robot_toy_noise_roll_on_pickup,
                    robot_toy_emit_periodic_noise,
                    robot_toy_alert_on_noise,
                    robot_toy_sell_bridge,
                    robot_toy_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RobotToyBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RobotToySpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RobotToy {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RobotToyScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for RobotToyScrap {
    fn default() -> Self {
        Self {
            min_value: ROBOT_TOY_MIN_VALUE,
            max_value: ROBOT_TOY_MAX_VALUE,
            weight: ROBOT_TOY_WEIGHT,
            conductive: ROBOT_TOY_CONDUCTIVE,
            two_handed: ROBOT_TOY_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RobotToyHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RobotToyNoiseState {
    pub is_active: bool,
    pub pickup_count: u32,
    pub last_emit_tick: u64,
}

impl Default for RobotToyNoiseState {
    fn default() -> Self {
        Self {
            is_active: false,
            pickup_count: 0,
            last_emit_tick: 0,
        }
    }
}

#[derive(Bundle)]
pub struct RobotToyBundle {
    pub name: Name,
    pub robot_toy: RobotToy,
    pub scrap: RobotToyScrap,
    pub position: SimPosition,
    pub held_by: RobotToyHeldBy,
    pub noise_state: RobotToyNoiseState,
}

impl RobotToyBundle {
    pub fn new(event: SpawnRobotToyEvent) -> Self {
        Self {
            name: Name::new(ROBOT_TOY_NAME),
            robot_toy: RobotToy { stable_id: event.stable_id },
            scrap: RobotToyScrap::default(),
            position: event.position,
            held_by: RobotToyHeldBy::default(),
            noise_state: RobotToyNoiseState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRobotToyEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotToyNoiseEmittedEvent {
    pub robot_toy_entity: Entity,
    pub robot_toy_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotToyAudibleAlertEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotToySoldEvent {
    pub robot_toy_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn robot_toy_value_range() -> (I32F32, I32F32) {
    (ROBOT_TOY_MIN_VALUE, ROBOT_TOY_MAX_VALUE)
}

pub fn robot_toy_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    ROBOT_TOY_SPAWN_CHANCES
        .iter()
        .find(|s| s.moon == moon)
        .map(|s| s.chance)
}

fn spawn_robot_toy(mut commands: Commands, mut events: EventReader<SpawnRobotToyEvent>) {
    for event in events.read() {
        commands.spawn(RobotToyBundle::new(*event));
    }
}

fn robot_toy_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    toys: Query<(&RobotToy, &RobotToyHeldBy), Changed<RobotToyHeldBy>>,
) {
    for (toy, held_by) in &toys {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: ROBOT_TOY_ID,
                two_handed: ROBOT_TOY_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = toy.stable_id;
        }
    }
}

fn robot_toy_noise_roll_on_pickup(
    mut toys: Query<
        (&RobotToy, &RobotToyHeldBy, &mut RobotToyNoiseState),
        Changed<RobotToyHeldBy>,
    >,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for (toy, held_by, mut noise_state) in &mut toys {
        if !held_by.is_held {
            continue;
        }

        noise_state.pickup_count = noise_state.pickup_count.saturating_add(1);

        // Rule 3: stop noise after several pickup cycles while active
        if noise_state.is_active
            && noise_state.pickup_count >= ROBOT_TOY_NOISE_STOP_PICKUP_CYCLES
        {
            noise_state.is_active = false;
            continue;
        }

        // Rule 1: roll for noise activation if not already active
        if !noise_state.is_active {
            let salt = ROBOT_TOY_NOISE_ROLL_SALT ^ toy.stable_id;
            let mut rng = tick_rng(seed.0, tick.0, salt);
            if rng.next_u32() % ROBOT_TOY_NOISE_ACTIVATION_DENOM == 0 {
                noise_state.is_active = true;
            }
        }
    }
}

fn robot_toy_emit_periodic_noise(
    mut toys: Query<(Entity, &RobotToy, &SimPosition, &mut RobotToyNoiseState)>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut toy_noise_events: EventWriter<RobotToyNoiseEmittedEvent>,
    tick: Res<SimTick>,
) {
    for (entity, toy, position, mut noise_state) in &mut toys {
        if !noise_state.is_active {
            continue;
        }
        if tick.0.saturating_sub(noise_state.last_emit_tick) < ROBOT_TOY_NOISE_EMIT_INTERVAL_TICKS
        {
            continue;
        }

        noise_state.last_emit_tick = tick.0;

        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: ROBOT_TOY_NOISE_AMOUNT,
        });

        toy_noise_events.send(RobotToyNoiseEmittedEvent {
            robot_toy_entity: entity,
            robot_toy_stable_id: toy.stable_id,
            position: *position,
        });
    }
}

fn robot_toy_alert_on_noise(
    mut toy_noise_events: EventReader<RobotToyNoiseEmittedEvent>,
    mut alert_events: EventWriter<RobotToyAudibleAlertEvent>,
) {
    for event in toy_noise_events.read() {
        alert_events.send(RobotToyAudibleAlertEvent {
            source: event.robot_toy_entity,
            position: event.position,
            amount: ROBOT_TOY_NOISE_AMOUNT,
        });
    }
}

fn robot_toy_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<RobotToySoldEvent>,
    toys: Query<(&RobotToy, &RobotToyScrap)>,
) {
    for event in sell_events.read() {
        for (toy, scrap) in &toys {
            if toy.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(RobotToySoldEvent {
                robot_toy_stable_id: toy.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn robot_toy_checksum(
    mut checksum: ResMut<SimChecksumState>,
    toys: Query<(
        &RobotToy,
        &RobotToyScrap,
        &SimPosition,
        &RobotToyHeldBy,
        &RobotToyNoiseState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, ROBOT_TOY_ID);
    accumulate_str(&mut checksum, 0x1001, ROBOT_TOY_NAME);
    accumulate_str(&mut checksum, 0x1002, ROBOT_TOY_TYPE);
    accumulate_str(&mut checksum, 0x1003, ROBOT_TOY_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ROBOT_TOY_EFFECTS);

    checksum.accumulate(ROBOT_TOY_SOURCE_REVISION as u64);
    checksum.accumulate(ROBOT_TOY_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ROBOT_TOY_WIKI_ID as u64);
    checksum.accumulate(ROBOT_TOY_WEIGHT.to_bits() as u64);
    checksum.accumulate(ROBOT_TOY_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(ROBOT_TOY_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(ROBOT_TOY_CONDUCTIVE as u64);
    checksum.accumulate(ROBOT_TOY_TWO_HANDED as u64);
    checksum.accumulate(ROBOT_TOY_NOISE_AMOUNT.to_bits() as u64);
    checksum.accumulate(ROBOT_TOY_NOISE_ACTIVATION_DENOM as u64);
    checksum.accumulate(ROBOT_TOY_NOISE_EMIT_INTERVAL_TICKS);
    checksum.accumulate(ROBOT_TOY_NOISE_STOP_PICKUP_CYCLES as u64);

    for dependency in ROBOT_TOY_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in ROBOT_TOY_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in ROBOT_TOY_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (toy, scrap, position, held_by, noise_state) in &toys {
        checksum.accumulate(toy.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(noise_state.is_active as u64);
        checksum.accumulate(noise_state.pickup_count as u64);
        checksum.accumulate(noise_state.last_emit_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}