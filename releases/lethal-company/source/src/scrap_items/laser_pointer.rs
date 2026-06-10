// Sources: vault/scrap_items/laser_pointer.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{SimChecksumState, SimPosition};

pub const LASER_POINTER_ID: &str = "laser_pointer";
pub const LASER_POINTER_NAME: &str = "Laser pointer";
pub const LASER_POINTER_TYPE: &str = "scrap_items";
pub const LASER_POINTER_SUBTYPE: &str = "scrap";
pub const LASER_POINTER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Laser_Pointer";
pub const LASER_POINTER_SOURCE_REVISION: u32 = 20242;
pub const LASER_POINTER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const LASER_POINTER_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const LASER_POINTER_EFFECTS: &str = "Projects a red-orange laser beam";
pub const LASER_POINTER_WEIGHT: I32F32 = I32F32::lit("0");
pub const LASER_POINTER_CONDUCTIVE: bool = false;
pub const LASER_POINTER_TWO_HANDED: bool = false;
pub const LASER_POINTER_BATTERY_LIFE_SECONDS: u32 = 300;
pub const LASER_POINTER_BATTERY_LIFE_TICKS: u32 = LASER_POINTER_BATTERY_LIFE_SECONDS * 30;
pub const LASER_POINTER_MIN_VALUE: I32F32 = I32F32::lit("32");
pub const LASER_POINTER_MAX_VALUE: I32F32 = I32F32::lit("99");

pub const LASER_POINTER_DEPENDS_ON: [&str; 16] = [
    "electric_coil",
    "the_ship",
    "the_company",
    "credits",
    "flashlight",
    "turret",
    "offense",
    "titan",
    "assurance",
    "adamance",
    "vow",
    "artifice",
    "experimentation",
    "embrion",
    "rend",
    "march",
];

pub const LASER_POINTER_SPAWN_CHANCES: [LaserPointerSpawnChance; 10] = [
    LaserPointerSpawnChance {
        moon: "offense",
        chance: I32F32::lit("1.2"),
    },
    LaserPointerSpawnChance {
        moon: "titan",
        chance: I32F32::lit("1.08"),
    },
    LaserPointerSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("0.94"),
    },
    LaserPointerSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("0.84"),
    },
    LaserPointerSpawnChance {
        moon: "vow",
        chance: I32F32::lit("0.82"),
    },
    LaserPointerSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("0.74"),
    },
    LaserPointerSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("0.7"),
    },
    LaserPointerSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("0.69"),
    },
    LaserPointerSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.46"),
    },
    LaserPointerSpawnChance {
        moon: "march",
        chance: I32F32::lit("0.34"),
    },
];

pub const LASER_POINTER_BEHAVIORAL_MECHANICS: [LaserPointerBehaviorRule; 14] = [
    LaserPointerBehaviorRule {
        condition: "the item is activated",
        outcome: "it emits a red-orange laser beam",
    },
    LaserPointerBehaviorRule {
        condition: "the item is activated continuously",
        outcome: "its battery lasts 300 seconds",
    },
    LaserPointerBehaviorRule {
        condition: "the left mouse button is pressed",
        outcome: "the laser pointer toggles on or off",
    },
    LaserPointerBehaviorRule {
        condition: "the item is recharged in the_ship with an electric_coil",
        outcome: "its battery is restored",
    },
    LaserPointerBehaviorRule {
        condition: "the item is sold to the_company",
        outcome: "it converts into credits",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on offense",
        outcome: "its spawn chance is 1.2%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on titan",
        outcome: "its spawn chance is 1.08%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on assurance",
        outcome: "its spawn chance is 0.94%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on adamance",
        outcome: "its spawn chance is 0.84%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on vow",
        outcome: "its spawn chance is 0.82%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on artifice",
        outcome: "its spawn chance is 0.74%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on experimentation",
        outcome: "its spawn chance is 0.7%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on embrion",
        outcome: "its spawn chance is 0.69%",
    },
    LaserPointerBehaviorRule {
        condition: "the item is found on rend",
        outcome: "its spawn chance is 0.46%",
    },
];

pub struct LaserPointerPlugin;

impl Plugin for LaserPointerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnLaserPointerEvent>()
            .add_event::<LaserPointerToggledEvent>()
            .add_event::<LaserPointerBeamEmittedEvent>()
            .add_event::<LaserPointerRechargedEvent>()
            .add_event::<LaserPointerSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_laser_pointer,
                    laser_pointer_pickup_item_bar_bridge,
                    laser_pointer_use_from_item_bar,
                    laser_pointer_emit_beam,
                    laser_pointer_drain_battery,
                    laser_pointer_recharge,
                    laser_pointer_sell_bridge,
                    laser_pointer_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaserPointerBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaserPointerSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaserPointer {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaserPointerScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for LaserPointerScrap {
    fn default() -> Self {
        Self {
            min_value: LASER_POINTER_MIN_VALUE,
            max_value: LASER_POINTER_MAX_VALUE,
            weight: LASER_POINTER_WEIGHT,
            conductive: LASER_POINTER_CONDUCTIVE,
            two_handed: LASER_POINTER_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaserPointerHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaserPointerBattery {
    pub ticks_remaining: u32,
    pub max_ticks: u32,
}

impl Default for LaserPointerBattery {
    fn default() -> Self {
        Self {
            ticks_remaining: LASER_POINTER_BATTERY_LIFE_TICKS,
            max_ticks: LASER_POINTER_BATTERY_LIFE_TICKS,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaserPointerBeamState {
    pub active: bool,
    pub activations: u64,
    pub beam_ticks: u64,
}

#[derive(Bundle)]
pub struct LaserPointerBundle {
    pub name: Name,
    pub laser_pointer: LaserPointer,
    pub scrap: LaserPointerScrap,
    pub position: SimPosition,
    pub held_by: LaserPointerHeldBy,
    pub battery: LaserPointerBattery,
    pub beam_state: LaserPointerBeamState,
}

impl LaserPointerBundle {
    pub fn new(event: SpawnLaserPointerEvent) -> Self {
        Self {
            name: Name::new(LASER_POINTER_NAME),
            laser_pointer: LaserPointer {
                stable_id: event.stable_id,
            },
            scrap: LaserPointerScrap::default(),
            position: event.position,
            held_by: LaserPointerHeldBy::default(),
            battery: LaserPointerBattery::default(),
            beam_state: LaserPointerBeamState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnLaserPointerEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaserPointerToggledEvent {
    pub laser_pointer_entity: Entity,
    pub laser_pointer_stable_id: u64,
    pub employee_id: u64,
    pub active: bool,
    pub battery_ticks_remaining: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaserPointerBeamEmittedEvent {
    pub laser_pointer_entity: Entity,
    pub laser_pointer_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub battery_ticks_remaining: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaserPointerRechargedEvent {
    pub laser_pointer_stable_id: u64,
    pub battery_ticks_remaining: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaserPointerSoldEvent {
    pub laser_pointer_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn laser_pointer_value_range() -> (I32F32, I32F32) {
    (LASER_POINTER_MIN_VALUE, LASER_POINTER_MAX_VALUE)
}

pub fn laser_pointer_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    LASER_POINTER_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_laser_pointer(mut commands: Commands, mut events: EventReader<SpawnLaserPointerEvent>) {
    for event in events.read() {
        commands.spawn(LaserPointerBundle::new(*event));
    }
}

fn laser_pointer_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    laser_pointers: Query<(&LaserPointer, &LaserPointerHeldBy), Changed<LaserPointerHeldBy>>,
) {
    for (laser_pointer, held_by) in &laser_pointers {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: LASER_POINTER_ID,
                two_handed: LASER_POINTER_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = laser_pointer.stable_id;
        }
    }
}

fn laser_pointer_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut toggled_events: EventWriter<LaserPointerToggledEvent>,
    mut laser_pointers: Query<(
        Entity,
        &LaserPointer,
        &LaserPointerHeldBy,
        &LaserPointerBattery,
        &mut LaserPointerBeamState,
    )>,
) {
    for event in item_events.read() {
        if event.item_id != LASER_POINTER_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, laser_pointer, held_by, battery, mut beam_state) in &mut laser_pointers {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            if battery.ticks_remaining == 0 {
                beam_state.active = false;
            } else {
                beam_state.active = !beam_state.active;
                if beam_state.active {
                    beam_state.activations = beam_state.activations.wrapping_add(1);
                }
            }

            toggled_events.send(LaserPointerToggledEvent {
                laser_pointer_entity: entity,
                laser_pointer_stable_id: laser_pointer.stable_id,
                employee_id: event.employee_id,
                active: beam_state.active,
                battery_ticks_remaining: battery.ticks_remaining,
            });
        }
    }
}

fn laser_pointer_emit_beam(
    mut beam_events: EventWriter<LaserPointerBeamEmittedEvent>,
    laser_pointers: Query<(
        Entity,
        &LaserPointer,
        &LaserPointerHeldBy,
        &SimPosition,
        &LaserPointerBattery,
        &LaserPointerBeamState,
    )>,
) {
    for (entity, laser_pointer, held_by, position, battery, beam_state) in &laser_pointers {
        if !beam_state.active || !held_by.is_held || battery.ticks_remaining == 0 {
            continue;
        }

        beam_events.send(LaserPointerBeamEmittedEvent {
            laser_pointer_entity: entity,
            laser_pointer_stable_id: laser_pointer.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
            battery_ticks_remaining: battery.ticks_remaining,
        });
    }
}

fn laser_pointer_drain_battery(
    mut laser_pointers: Query<(&mut LaserPointerBattery, &mut LaserPointerBeamState), With<LaserPointer>>,
) {
    for (mut battery, mut beam_state) in &mut laser_pointers {
        if !beam_state.active {
            continue;
        }

        if battery.ticks_remaining > 0 {
            battery.ticks_remaining -= 1;
            beam_state.beam_ticks = beam_state.beam_ticks.wrapping_add(1);
        }

        if battery.ticks_remaining == 0 {
            beam_state.active = false;
        }
    }
}

fn laser_pointer_recharge(
    mut recharge_events: EventReader<LaserPointerRechargedEvent>,
    mut laser_pointers: Query<(&LaserPointer, &mut LaserPointerBattery)>,
) {
    for event in recharge_events.read() {
        for (laser_pointer, mut battery) in &mut laser_pointers {
            if laser_pointer.stable_id != event.laser_pointer_stable_id {
                continue;
            }

            battery.ticks_remaining = battery.max_ticks;
        }
    }
}

fn laser_pointer_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<LaserPointerSoldEvent>,
    laser_pointers: Query<(&LaserPointer, &LaserPointerScrap)>,
) {
    for event in sell_events.read() {
        for (laser_pointer, scrap) in &laser_pointers {
            if laser_pointer.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(LaserPointerSoldEvent {
                laser_pointer_stable_id: laser_pointer.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn laser_pointer_checksum(
    mut checksum: ResMut<SimChecksumState>,
    laser_pointers: Query<(
        &LaserPointer,
        &LaserPointerScrap,
        &SimPosition,
        &LaserPointerHeldBy,
        &LaserPointerBattery,
        &LaserPointerBeamState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, LASER_POINTER_ID);
    accumulate_str(&mut checksum, 0x1001, LASER_POINTER_NAME);
    accumulate_str(&mut checksum, 0x1002, LASER_POINTER_TYPE);
    accumulate_str(&mut checksum, 0x1003, LASER_POINTER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, LASER_POINTER_EFFECTS);

    checksum.accumulate(LASER_POINTER_SOURCE_REVISION as u64);
    checksum.accumulate(LASER_POINTER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(LASER_POINTER_WEIGHT.to_bits() as u64);
    checksum.accumulate(LASER_POINTER_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(LASER_POINTER_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(LASER_POINTER_CONDUCTIVE as u64);
    checksum.accumulate(LASER_POINTER_TWO_HANDED as u64);
    checksum.accumulate(LASER_POINTER_BATTERY_LIFE_SECONDS as u64);
    checksum.accumulate(LASER_POINTER_BATTERY_LIFE_TICKS as u64);

    for dependency in LASER_POINTER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in LASER_POINTER_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in LASER_POINTER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (laser_pointer, scrap, position, held_by, battery, beam_state) in &laser_pointers {
        checksum.accumulate(laser_pointer.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(battery.ticks_remaining as u64);
        checksum.accumulate(battery.max_ticks as u64);
        checksum.accumulate(beam_state.active as u64);
        checksum.accumulate(beam_state.activations);
        checksum.accumulate(beam_state.beam_ticks);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}