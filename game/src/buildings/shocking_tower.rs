// Sources: vault/buildings/shocking_tower.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const SHOCKING_TOWER_HP: I32F32 = I32F32::lit("1000");
const SHOCKING_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const SHOCKING_TOWER_WATCH_RANGE: I32F32 = I32F32::lit("8");
const SHOCKING_TOWER_ATTACK_DAMAGE: I32F32 = I32F32::lit("60");
const SHOCKING_TOWER_ATTACK_SPEED: I32F32 = I32F32::lit("0.25");
const SHOCKING_TOWER_ATTACK_RANGE: I32F32 = I32F32::lit("6");
const SHOCKING_TOWER_AOE_RADIUS: I32F32 = I32F32::lit("6");

const SHOCKING_TOWER_WOOD_COST: i32 = 30;
const SHOCKING_TOWER_STONE_COST: i32 = 30;
const SHOCKING_TOWER_IRON_COST: i32 = 0;
const SHOCKING_TOWER_OIL_COST: i32 = 0;
const SHOCKING_TOWER_GOLD_COST: i32 = 600;
const SHOCKING_TOWER_ENERGY_COST: i32 = 20;
const SHOCKING_TOWER_WORKERS_COST: i32 = 2;
const SHOCKING_TOWER_MAINTENANCE_GOLD: i32 = 20;
const SHOCKING_TOWER_RESEARCH_GOLD_COST: i32 = 900;
const SHOCKING_TOWER_BUILD_TIME_SECONDS: i32 = 45;
const SHOCKING_TOWER_SIZE_TILES: i32 = 2;
const SHOCKING_TOWER_MIN_SEPARATION_CELLS: i32 = 3;
const SHOCKING_TOWER_NOISE: i32 = 20;

const SIM_HZ: i32 = 25;
const SHOCKING_TOWER_RECHARGE_TICKS: i32 = 100;

#[derive(Component, Default)]
pub struct ShockingTower;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct ShockingTowerHealth {
    pub hp: I32F32,
    pub defenses_life: I32F32,
}

impl Default for ShockingTowerHealth {
    fn default() -> Self {
        Self {
            hp: SHOCKING_TOWER_HP,
            defenses_life: SHOCKING_TOWER_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ShockingTowerBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct ShockingTowerEconomy {
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub maintenance_gold: i32,
    pub research_gold_cost: i32,
}

impl Default for ShockingTowerEconomy {
    fn default() -> Self {
        Self {
            wood_cost: SHOCKING_TOWER_WOOD_COST,
            stone_cost: SHOCKING_TOWER_STONE_COST,
            iron_cost: SHOCKING_TOWER_IRON_COST,
            oil_cost: SHOCKING_TOWER_OIL_COST,
            gold_cost: SHOCKING_TOWER_GOLD_COST,
            energy_cost: SHOCKING_TOWER_ENERGY_COST,
            workers_cost: SHOCKING_TOWER_WORKERS_COST,
            maintenance_gold: SHOCKING_TOWER_MAINTENANCE_GOLD,
            research_gold_cost: SHOCKING_TOWER_RESEARCH_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ShockingTowerFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub min_separation_cells: i32,
}

impl Default for ShockingTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: SHOCKING_TOWER_SIZE_TILES,
            watch_range: SHOCKING_TOWER_WATCH_RANGE,
            min_separation_cells: SHOCKING_TOWER_MIN_SEPARATION_CELLS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ShockingTowerAttack {
    pub damage_before_armor: I32F32,
    pub attack_speed: I32F32,
    pub attack_range: I32F32,
    pub aoe_radius: I32F32,
    pub noise: i32,
    pub recharge_ticks: i32,
    pub ticks_until_next_pulse: i32,
}

impl Default for ShockingTowerAttack {
    fn default() -> Self {
        Self {
            damage_before_armor: SHOCKING_TOWER_ATTACK_DAMAGE,
            attack_speed: SHOCKING_TOWER_ATTACK_SPEED,
            attack_range: SHOCKING_TOWER_ATTACK_RANGE,
            aoe_radius: SHOCKING_TOWER_AOE_RADIUS,
            noise: SHOCKING_TOWER_NOISE,
            recharge_ticks: SHOCKING_TOWER_RECHARGE_TICKS,
            ticks_until_next_pulse: SHOCKING_TOWER_RECHARGE_TICKS,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct ShockingTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Default, Clone)]
pub struct ShockingTowerTargets {
    pub targets_by_tower: BTreeMap<Entity, BTreeSet<i32>>,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ShockingTowerResearchState {
    pub researched: bool,
}

#[derive(Event, Clone, Copy)]
pub struct ResearchShockingTowerEvent;

#[derive(Event, Clone, Copy)]
pub struct ShockingTowerResearchedEvent;

#[derive(Event, Clone, Copy)]
pub struct PlaceShockingTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct ShockingTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpdateShockingTowerTargetEvent {
    pub tower_entity: Entity,
    pub enemy_unit_id: i32,
    pub in_range: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DamageShockingTowerEvent {
    pub tower_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct ShockingTowerPulseHitEvent {
    pub tower_entity: Entity,
    pub enemy_unit_id: i32,
    pub damage_before_armor: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn are_anchors_too_close(a: BuildingAnchor, b: BuildingAnchor, min_separation_cells: i32) -> bool {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx < min_separation_cells && dy < min_separation_cells
}

fn research_shocking_tower_system(
    mut events: EventReader<ResearchShockingTowerEvent>,
    mut researched_events: EventWriter<ShockingTowerResearchedEvent>,
    mut research_state: ResMut<ShockingTowerResearchState>,
) {
    for _ in events.read() {
        if research_state.researched {
            continue;
        }

        research_state.researched = true;
        researched_events.send(ShockingTowerResearchedEvent);
    }
}

fn place_shocking_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceShockingTowerEvent>,
    mut rejected: EventWriter<ShockingTowerPlacementRejectedEvent>,
    research_state: Res<ShockingTowerResearchState>,
    mut claims: ResMut<ShockingTowerPlacementClaims>,
) {
    for ev in events.read() {
        if !research_state.researched {
            rejected.send(ShockingTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims
            .claims
            .values()
            .any(|existing| are_anchors_too_close(*existing, anchor, SHOCKING_TOWER_MIN_SEPARATION_CELLS))
        {
            rejected.send(ShockingTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                ShockingTower,
                anchor,
                ShockingTowerHealth::default(),
                ShockingTowerBuildState {
                    build_ticks_remaining: seconds_to_ticks(SHOCKING_TOWER_BUILD_TIME_SECONDS),
                    completed: false,
                },
                ShockingTowerEconomy::default(),
                ShockingTowerFootprint::default(),
                ShockingTowerAttack::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn shocking_tower_build_tick_system(
    mut towers: Query<&mut ShockingTowerBuildState, With<ShockingTower>>,
) {
    for mut state in &mut towers {
        if state.completed {
            continue;
        }

        if state.build_ticks_remaining > 0 {
            state.build_ticks_remaining -= 1;
        }

        if state.build_ticks_remaining <= 0 {
            state.build_ticks_remaining = 0;
            state.completed = true;
        }
    }
}

fn update_shocking_tower_targets_system(
    mut events: EventReader<UpdateShockingTowerTargetEvent>,
    towers: Query<(), With<ShockingTower>>,
    mut targets: ResMut<ShockingTowerTargets>,
) {
    for ev in events.read() {
        if towers.get(ev.tower_entity).is_err() {
            continue;
        }

        let set = targets
            .targets_by_tower
            .entry(ev.tower_entity)
            .or_default();

        if ev.in_range {
            set.insert(ev.enemy_unit_id);
        } else {
            set.remove(&ev.enemy_unit_id);
        }
    }
}

fn shocking_tower_attack_system(
    mut pulse_hits: EventWriter<ShockingTowerPulseHitEvent>,
    mut towers: Query<(Entity, &ShockingTowerBuildState, &mut ShockingTowerAttack), With<ShockingTower>>,
    targets: Res<ShockingTowerTargets>,
) {
    for (tower_entity, build, mut attack) in &mut towers {
        if !build.completed {
            continue;
        }

        if attack.ticks_until_next_pulse > 0 {
            attack.ticks_until_next_pulse -= 1;
        }

        if attack.ticks_until_next_pulse > 0 {
            continue;
        }

        if let Some(in_range_targets) = targets.targets_by_tower.get(&tower_entity) {
            for enemy_unit_id in in_range_targets {
                pulse_hits.send(ShockingTowerPulseHitEvent {
                    tower_entity,
                    enemy_unit_id: *enemy_unit_id,
                    damage_before_armor: attack.damage_before_armor,
                });
            }
        }

        attack.ticks_until_next_pulse = attack.recharge_ticks;
    }
}

fn damage_shocking_tower_system(
    mut commands: Commands,
    mut events: EventReader<DamageShockingTowerEvent>,
    mut towers: Query<(Entity, &mut ShockingTowerHealth), With<ShockingTower>>,
    mut claims: ResMut<ShockingTowerPlacementClaims>,
    mut targets: ResMut<ShockingTowerTargets>,
) {
    for ev in events.read() {
        let Ok((tower_entity, mut health)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        if ev.damage <= I32F32::ZERO {
            continue;
        }

        if health.hp > ev.damage {
            health.hp -= ev.damage;
            continue;
        }

        health.hp = I32F32::ZERO;
        claims.claims.remove(&tower_entity);
        targets.targets_by_tower.remove(&tower_entity);
        commands.entity(tower_entity).despawn();
    }
}

fn shocking_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            Entity,
            &BuildingAnchor,
            &ShockingTowerHealth,
            &ShockingTowerBuildState,
            &ShockingTowerEconomy,
            &ShockingTowerFootprint,
            &ShockingTowerAttack,
        ),
        With<ShockingTower>,
    >,
    claims: Res<ShockingTowerPlacementClaims>,
    targets: Res<ShockingTowerTargets>,
    research_state: Res<ShockingTowerResearchState>,
) {
    checksum.accumulate(u64::from(research_state.researched));

    for (entity, anchor, health, build, economy, footprint, attack) in &towers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.hp.to_bits() as u64);
        checksum.accumulate(health.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.workers_cost as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.research_gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.min_separation_cells as u64);

        checksum.accumulate(attack.damage_before_armor.to_bits() as u64);
        checksum.accumulate(attack.attack_speed.to_bits() as u64);
        checksum.accumulate(attack.attack_range.to_bits() as u64);
        checksum.accumulate(attack.aoe_radius.to_bits() as u64);
        checksum.accumulate(attack.noise as u64);
        checksum.accumulate(attack.recharge_ticks as u64);
        checksum.accumulate(attack.ticks_until_next_pulse as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(targets.targets_by_tower.len() as u64);
    for (tower_entity, unit_ids) in &targets.targets_by_tower {
        checksum.accumulate(tower_entity.to_bits() as u64);
        checksum.accumulate(unit_ids.len() as u64);
        for unit_id in unit_ids {
            checksum.accumulate(*unit_id as u64);
        }
    }
}

pub struct ShockingTowerPlugin;

impl Plugin for ShockingTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShockingTowerPlacementClaims>()
            .init_resource::<ShockingTowerTargets>()
            .init_resource::<ShockingTowerResearchState>()
            .add_event::<ResearchShockingTowerEvent>()
            .add_event::<ShockingTowerResearchedEvent>()
            .add_event::<PlaceShockingTowerEvent>()
            .add_event::<ShockingTowerPlacementRejectedEvent>()
            .add_event::<UpdateShockingTowerTargetEvent>()
            .add_event::<DamageShockingTowerEvent>()
            .add_event::<ShockingTowerPulseHitEvent>()
            .add_systems(
                FixedUpdate,
                (
                    research_shocking_tower_system,
                    place_shocking_tower_system,
                    shocking_tower_build_tick_system,
                    update_shocking_tower_targets_system,
                    shocking_tower_attack_system,
                    damage_shocking_tower_system,
                    shocking_tower_checksum_system,
                )
                    .chain(),
            );
    }
}