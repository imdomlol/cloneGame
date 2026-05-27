// Sources: vault/buildings/stone_gate.md, vault/buildings/wood_gate.md, vault/buildings/stone_wall.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::buildings::wood_gate::WoodGate;
use crate::sim::{Health, SimChecksumState};

const STONE_GATE_HP: I32F32 = I32F32::lit("1500");
const STONE_GATE_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const STONE_GATE_WATCH_RANGE: I32F32 = I32F32::lit("0");
const STONE_GATE_ENERGY_COST: i32 = 0;
const STONE_GATE_WOOD_COST: i32 = 3;
const STONE_GATE_STONE_COST: i32 = 6;
const STONE_GATE_IRON_COST: i32 = 0;
const STONE_GATE_OIL_COST: i32 = 0;
const STONE_GATE_GOLD_COST: i32 = 200;
const STONE_GATE_BUILD_TIME_SECONDS: i32 = 45;
const STONE_GATE_SIZE_X_TILES: i32 = 3;
const STONE_GATE_SIZE_Y_TILES: i32 = 1;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct StoneGate;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct StoneGateCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for StoneGateCore {
    fn default() -> Self {
        Self {
            defenses_life: STONE_GATE_DEFENSES_LIFE,
            watch_range: STONE_GATE_WATCH_RANGE,
            health: Health::full(STONE_GATE_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneGateEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for StoneGateEconomy {
    fn default() -> Self {
        Self {
            energy_cost: STONE_GATE_ENERGY_COST,
            wood_cost: STONE_GATE_WOOD_COST,
            stone_cost: STONE_GATE_STONE_COST,
            iron_cost: STONE_GATE_IRON_COST,
            oil_cost: STONE_GATE_OIL_COST,
            gold_cost: STONE_GATE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneGateFootprint {
    pub size_x_tiles: i32,
    pub size_y_tiles: i32,
}

impl Default for StoneGateFootprint {
    fn default() -> Self {
        Self {
            size_x_tiles: STONE_GATE_SIZE_X_TILES,
            size_y_tiles: STONE_GATE_SIZE_Y_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct StoneGateBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_wood_gate: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct StoneGatePassageState {
    pub is_open: bool,
    pub friendly_units_in_passage: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct StoneGateBehavior {
    pub can_build_directly: bool,
    pub can_upgrade_from_wood_gate: bool,
    pub auto_open_for_friendlies: bool,
    pub auto_close_after_friendly_passes: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStoneGateEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeWoodGateToStoneGateEvent {
    pub wood_gate_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct SetStoneGateFriendlyUnitsInPassageEvent {
    pub gate_entity: Entity,
    pub friendly_units_in_passage: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageStoneGateEvent {
    pub gate_entity: Entity,
    pub damage: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_stone_gate_system(mut commands: Commands, mut events: EventReader<PlaceStoneGateEvent>) {
    for ev in events.read() {
        commands.spawn((
            StoneGate,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
            StoneGateCore::default(),
            StoneGateEconomy::default(),
            StoneGateFootprint::default(),
            StoneGateBuildState {
                build_ticks_remaining: seconds_to_ticks(STONE_GATE_BUILD_TIME_SECONDS),
                upgrading_from_wood_gate: false,
                completed: false,
            },
            StoneGatePassageState::default(),
            StoneGateBehavior {
                can_build_directly: true,
                can_upgrade_from_wood_gate: true,
                auto_open_for_friendlies: true,
                auto_close_after_friendly_passes: true,
            },
        ));
    }
}

fn upgrade_wood_gate_to_stone_gate_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeWoodGateToStoneGateEvent>,
    wood_gates: Query<&BuildingAnchor, With<WoodGate>>,
) {
    for ev in events.read() {
        let Ok(anchor) = wood_gates.get(ev.wood_gate_entity) else {
            continue;
        };

        commands.entity(ev.wood_gate_entity).remove::<WoodGate>();
        commands.entity(ev.wood_gate_entity).insert((
            StoneGate,
            *anchor,
            StoneGateCore::default(),
            StoneGateEconomy::default(),
            StoneGateFootprint::default(),
            StoneGateBuildState {
                build_ticks_remaining: seconds_to_ticks(STONE_GATE_BUILD_TIME_SECONDS),
                upgrading_from_wood_gate: true,
                completed: false,
            },
            StoneGatePassageState::default(),
            StoneGateBehavior {
                can_build_directly: true,
                can_upgrade_from_wood_gate: true,
                auto_open_for_friendlies: true,
                auto_close_after_friendly_passes: true,
            },
        ));
    }
}

fn stone_gate_build_tick_system(
    mut gates: Query<(&mut StoneGateBuildState, &mut StoneGatePassageState), With<StoneGate>>,
) {
    for (mut build, mut passage) in &mut gates {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
            passage.is_open = false;
            passage.friendly_units_in_passage = 0;
        }
    }
}

fn set_stone_gate_friendly_units_system(
    mut events: EventReader<SetStoneGateFriendlyUnitsInPassageEvent>,
    mut gates: Query<(&StoneGateBuildState, &StoneGateBehavior, &mut StoneGatePassageState), With<StoneGate>>,
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

        if behavior.auto_open_for_friendlies && incoming > 0 {
            passage.is_open = true;
        }

        if behavior.auto_close_after_friendly_passes && incoming == 0 {
            passage.is_open = false;
        }
    }
}

fn damage_stone_gate_system(
    mut commands: Commands,
    mut events: EventReader<DamageStoneGateEvent>,
    mut gates: Query<(Entity, &mut StoneGateCore), With<StoneGate>>,
) {
    for ev in events.read() {
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

fn stone_gate_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    gates: Query<
        (
            Entity,
            &BuildingAnchor,
            &StoneGateCore,
            &StoneGateEconomy,
            &StoneGateFootprint,
            &StoneGateBuildState,
            &StoneGatePassageState,
            &StoneGateBehavior,
        ),
        With<StoneGate>,
    >,
) {
    for (entity, anchor, core, economy, footprint, build, passage, behavior) in &gates {
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
        checksum.accumulate(u64::from(build.upgrading_from_wood_gate));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(u64::from(passage.is_open));
        checksum.accumulate(passage.friendly_units_in_passage as u64);

        checksum.accumulate(u64::from(behavior.can_build_directly));
        checksum.accumulate(u64::from(behavior.can_upgrade_from_wood_gate));
        checksum.accumulate(u64::from(behavior.auto_open_for_friendlies));
        checksum.accumulate(u64::from(behavior.auto_close_after_friendly_passes));
    }
}

pub struct StoneGatePlugin;

impl Plugin for StoneGatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaceStoneGateEvent>()
            .add_event::<UpgradeWoodGateToStoneGateEvent>()
            .add_event::<SetStoneGateFriendlyUnitsInPassageEvent>()
            .add_event::<DamageStoneGateEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_stone_gate_system,
                    upgrade_wood_gate_to_stone_gate_system,
                    stone_gate_build_tick_system,
                    set_stone_gate_friendly_units_system,
                    damage_stone_gate_system,
                    stone_gate_checksum_system,
                )
                    .chain(),
            );
    }
}