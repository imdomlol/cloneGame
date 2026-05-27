// Sources: vault/buildings/wood_gate.md, vault/buildings/stone_gate.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const WOOD_GATE_HP: I32F32 = I32F32::lit("600");
const WOOD_GATE_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const WOOD_GATE_WATCH_RANGE: I32F32 = I32F32::lit("0");
const WOOD_GATE_ENERGY_COST: i32 = 0;
const WOOD_GATE_WOOD_COST: i32 = 10;
const WOOD_GATE_STONE_COST: i32 = 0;
const WOOD_GATE_IRON_COST: i32 = 0;
const WOOD_GATE_OIL_COST: i32 = 0;
const WOOD_GATE_GOLD_COST: i32 = 50;
const WOOD_GATE_BUILD_TIME_SECONDS: i32 = 30;
const WOOD_GATE_SIZE_X_TILES: i32 = 3;
const WOOD_GATE_SIZE_Y_TILES: i32 = 1;
const WOOD_GATE_CONSTRUCTION_HP_PER_SECOND: I32F32 = I32F32::lit("20");
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct WoodGate;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct WoodGateCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for WoodGateCore {
    fn default() -> Self {
        Self {
            defenses_life: WOOD_GATE_DEFENSES_LIFE,
            watch_range: WOOD_GATE_WATCH_RANGE,
            health: Health {
                current: I32F32::ZERO,
                max: WOOD_GATE_HP,
            },
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodGateEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for WoodGateEconomy {
    fn default() -> Self {
        Self {
            energy_cost: WOOD_GATE_ENERGY_COST,
            wood_cost: WOOD_GATE_WOOD_COST,
            stone_cost: WOOD_GATE_STONE_COST,
            iron_cost: WOOD_GATE_IRON_COST,
            oil_cost: WOOD_GATE_OIL_COST,
            gold_cost: WOOD_GATE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodGateFootprint {
    pub size_x_tiles: i32,
    pub size_y_tiles: i32,
}

impl Default for WoodGateFootprint {
    fn default() -> Self {
        Self {
            size_x_tiles: WOOD_GATE_SIZE_X_TILES,
            size_y_tiles: WOOD_GATE_SIZE_Y_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodGateBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodGatePassageState {
    pub is_open: bool,
    pub friendly_units_in_passage: i32,
    pub infected_waiting_to_pass: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodGateBehavior {
    pub friendly_can_pass_without_breaking: bool,
    pub infected_must_break_before_passing: bool,
    pub can_upgrade_to_stone_gate: bool,
    pub frontline_less_resistant_than_wood_wall: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodGatePlacementState {
    pub placed_on_existing_wall: bool,
    pub wall_refund_applied: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWoodGateEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub placed_on_existing_wall: bool,
    pub recovered_wood: i32,
    pub recovered_stone: i32,
    pub recovered_iron: i32,
    pub recovered_oil: i32,
    pub recovered_gold: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetWoodGateFriendlyUnitsInPassageEvent {
    pub gate_entity: Entity,
    pub friendly_units_in_passage: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetWoodGateInfectedWaitingEvent {
    pub gate_entity: Entity,
    pub infected_waiting_to_pass: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DamageWoodGateEvent {
    pub gate_entity: Entity,
    pub damage: I32F32,
    pub attacker_is_lucifer: bool,
    pub through_single_gate: bool,
}

#[derive(Event, Clone, Copy)]
pub struct WoodGateWallRefundAppliedEvent {
    pub gate_entity: Entity,
    pub recovered_wood: i32,
    pub recovered_stone: i32,
    pub recovered_iron: i32,
    pub recovered_oil: i32,
    pub recovered_gold: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_wood_gate_system(mut commands: Commands, mut events: EventReader<PlaceWoodGateEvent>) {
    for ev in events.read() {
        commands.spawn((
            WoodGate,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
            WoodGateCore::default(),
            WoodGateEconomy::default(),
            WoodGateFootprint::default(),
            WoodGateBuildState {
                build_ticks_remaining: seconds_to_ticks(WOOD_GATE_BUILD_TIME_SECONDS),
                completed: false,
            },
            WoodGatePassageState::default(),
            WoodGateBehavior {
                friendly_can_pass_without_breaking: true,
                infected_must_break_before_passing: true,
                can_upgrade_to_stone_gate: true,
                frontline_less_resistant_than_wood_wall: true,
            },
            WoodGatePlacementState {
                placed_on_existing_wall: ev.placed_on_existing_wall,
                wall_refund_applied: false,
            },
        ));
    }
}

fn wood_gate_build_tick_system(
    mut gates: Query<(&mut WoodGateCore, &mut WoodGateBuildState), With<WoodGate>>,
) {
    let hp_per_tick = WOOD_GATE_CONSTRUCTION_HP_PER_SECOND / I32F32::from_num(SIM_HZ);

    for (mut core, mut build) in &mut gates {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        core.health.current = (core.health.current + hp_per_tick).min(core.health.max);

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn set_wood_gate_friendly_units_system(
    mut events: EventReader<SetWoodGateFriendlyUnitsInPassageEvent>,
    mut gates: Query<(&WoodGateBuildState, &WoodGateBehavior, &mut WoodGatePassageState), With<WoodGate>>,
) {
    for ev in events.read() {
        let Ok((build, behavior, mut passage)) = gates.get_mut(ev.gate_entity) else {
            continue;
        };

        if !build.completed {
            passage.is_open = false;
            passage.friendly_units_in_passage = 0;
            continue;
        }

        let incoming = ev.friendly_units_in_passage.max(0);
        passage.friendly_units_in_passage = incoming;

        if behavior.friendly_can_pass_without_breaking && incoming > 0 {
            passage.is_open = true;
        } else if incoming == 0 {
            passage.is_open = false;
        }
    }
}

fn set_wood_gate_infected_waiting_system(
    mut events: EventReader<SetWoodGateInfectedWaitingEvent>,
    mut gates: Query<(&WoodGateBuildState, &WoodGateBehavior, &mut WoodGatePassageState), With<WoodGate>>,
) {
    for ev in events.read() {
        let Ok((build, behavior, mut passage)) = gates.get_mut(ev.gate_entity) else {
            continue;
        };

        if !build.completed {
            passage.infected_waiting_to_pass = false;
            continue;
        }

        passage.infected_waiting_to_pass = ev.infected_waiting_to_pass;

        if behavior.infected_must_break_before_passing && ev.infected_waiting_to_pass {
            passage.is_open = false;
        }
    }
}

fn damage_wood_gate_system(
    mut commands: Commands,
    mut events: EventReader<DamageWoodGateEvent>,
    mut gates: Query<(Entity, &mut WoodGateCore), With<WoodGate>>,
) {
    for ev in events.read() {
        if ev.attacker_is_lucifer && ev.through_single_gate {
            continue;
        }

        let Ok((entity, mut core)) = gates.get_mut(ev.gate_entity) else {
            continue;
        };

        let mut incoming = ev.damage;
        if incoming <= I32F32::ZERO {
            continue;
        }

        if core.defenses_life > I32F32::ZERO {
            let absorbed = incoming.min(core.defenses_life);
            core.defenses_life -= absorbed;
            incoming -= absorbed;
        }

        if incoming > I32F32::ZERO {
            core.health.current -= incoming;
            if core.health.current <= I32F32::ZERO {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn apply_wall_refund_on_placement_system(
    mut place_events: EventReader<PlaceWoodGateEvent>,
    mut refund_events: EventWriter<WoodGateWallRefundAppliedEvent>,
    mut gates: Query<(Entity, &mut WoodGatePlacementState), With<WoodGate>>,
) {
    for place_ev in place_events.read() {
        if !place_ev.placed_on_existing_wall {
            continue;
        }

        for (entity, mut placement) in &mut gates {
            if placement.placed_on_existing_wall && !placement.wall_refund_applied {
                placement.wall_refund_applied = true;
                refund_events.send(WoodGateWallRefundAppliedEvent {
                    gate_entity: entity,
                    recovered_wood: place_ev.recovered_wood.max(0),
                    recovered_stone: place_ev.recovered_stone.max(0),
                    recovered_iron: place_ev.recovered_iron.max(0),
                    recovered_oil: place_ev.recovered_oil.max(0),
                    recovered_gold: place_ev.recovered_gold.max(0),
                });
                break;
            }
        }
    }
}

fn wood_gate_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    gates: Query<
        (
            Entity,
            &BuildingAnchor,
            &WoodGateCore,
            &WoodGateEconomy,
            &WoodGateFootprint,
            &WoodGateBuildState,
            &WoodGatePassageState,
            &WoodGateBehavior,
            &WoodGatePlacementState,
        ),
        With<WoodGate>,
    >,
) {
    for (entity, anchor, core, economy, footprint, build, passage, behavior, placement) in &gates {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);

        checksum.accumulate(footprint.size_x_tiles as u64);
        checksum.accumulate(footprint.size_y_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(u64::from(passage.is_open));
        checksum.accumulate(passage.friendly_units_in_passage as u64);
        checksum.accumulate(u64::from(passage.infected_waiting_to_pass));

        checksum.accumulate(u64::from(behavior.friendly_can_pass_without_breaking));
        checksum.accumulate(u64::from(behavior.infected_must_break_before_passing));
        checksum.accumulate(u64::from(behavior.can_upgrade_to_stone_gate));
        checksum.accumulate(u64::from(behavior.frontline_less_resistant_than_wood_wall));

        checksum.accumulate(u64::from(placement.placed_on_existing_wall));
        checksum.accumulate(u64::from(placement.wall_refund_applied));
    }
}

pub struct WoodGatePlugin;

impl Plugin for WoodGatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaceWoodGateEvent>()
            .add_event::<SetWoodGateFriendlyUnitsInPassageEvent>()
            .add_event::<SetWoodGateInfectedWaitingEvent>()
            .add_event::<DamageWoodGateEvent>()
            .add_event::<WoodGateWallRefundAppliedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_wood_gate_system,
                    wood_gate_build_tick_system,
                    set_wood_gate_friendly_units_system,
                    set_wood_gate_infected_waiting_system,
                    damage_wood_gate_system,
                    apply_wall_refund_on_placement_system,
                    wood_gate_checksum_system,
                )
                    .chain(),
            );
    }
}