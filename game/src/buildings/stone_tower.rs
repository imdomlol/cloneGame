// Sources: vault/buildings/stone_tower.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const STONE_TOWER_HP: I32F32 = I32F32::lit("2000");
const STONE_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const STONE_TOWER_WATCH_RANGE_BONUS: I32F32 = I32F32::lit("4");
const STONE_TOWER_ATTACK_RANGE_BONUS: I32F32 = I32F32::lit("4");
const STONE_TOWER_WOOD_COST: i32 = 10;
const STONE_TOWER_STONE_COST: i32 = 10;
const STONE_TOWER_IRON_COST: i32 = 0;
const STONE_TOWER_OIL_COST: i32 = 0;
const STONE_TOWER_GOLD_COST: i32 = 450;
const STONE_TOWER_UPGRADE_WOOD_COST: i32 = 0;
const STONE_TOWER_UPGRADE_STONE_COST: i32 = 0;
const STONE_TOWER_UPGRADE_IRON_COST: i32 = 0;
const STONE_TOWER_UPGRADE_OIL_COST: i32 = 0;
const STONE_TOWER_UPGRADE_GOLD_COST: i32 = 330;
const STONE_TOWER_BUILD_TIME_SECONDS: i32 = 60;
const STONE_TOWER_UPGRADE_TIME_SECONDS: i32 = 23;
const STONE_TOWER_SIZE_TILES: i32 = 1;
const STONE_TOWER_GARRISON_CAPACITY: i32 = 4;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct WoodTower;

#[derive(Component, Default)]
pub struct StoneTower;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingHealth {
    pub hp: I32F32,
    pub defenses_life: I32F32,
}

impl Default for BuildingHealth {
    fn default() -> Self {
        Self {
            hp: STONE_TOWER_HP,
            defenses_life: STONE_TOWER_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneTowerBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_wood_tower: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StoneTowerEconomy {
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub upgrade_wood_cost: i32,
    pub upgrade_stone_cost: i32,
    pub upgrade_iron_cost: i32,
    pub upgrade_oil_cost: i32,
    pub upgrade_gold_cost: i32,
}

impl Default for StoneTowerEconomy {
    fn default() -> Self {
        Self {
            wood_cost: STONE_TOWER_WOOD_COST,
            stone_cost: STONE_TOWER_STONE_COST,
            iron_cost: STONE_TOWER_IRON_COST,
            oil_cost: STONE_TOWER_OIL_COST,
            gold_cost: STONE_TOWER_GOLD_COST,
            upgrade_wood_cost: STONE_TOWER_UPGRADE_WOOD_COST,
            upgrade_stone_cost: STONE_TOWER_UPGRADE_STONE_COST,
            upgrade_iron_cost: STONE_TOWER_UPGRADE_IRON_COST,
            upgrade_oil_cost: STONE_TOWER_UPGRADE_OIL_COST,
            upgrade_gold_cost: STONE_TOWER_UPGRADE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneTowerFootprint {
    pub size_tiles: i32,
    pub watch_range_bonus: I32F32,
    pub attack_range_bonus: I32F32,
    pub garrison_capacity: i32,
}

impl Default for StoneTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: STONE_TOWER_SIZE_TILES,
            watch_range_bonus: STONE_TOWER_WATCH_RANGE_BONUS,
            attack_range_bonus: STONE_TOWER_ATTACK_RANGE_BONUS,
            garrison_capacity: STONE_TOWER_GARRISON_CAPACITY,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GarrisonedUnit {
    pub unit_id: i32,
    pub base_attack_range: I32F32,
    pub base_watch_range: I32F32,
    pub tower_attack_range: I32F32,
    pub tower_watch_range: I32F32,
}

#[derive(Component, Default, Clone)]
pub struct StoneTowerGarrison {
    pub units: BTreeMap<i32, GarrisonedUnit>,
}

#[derive(Resource, Default, Clone)]
pub struct StoneTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStoneTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeWoodTowerToStoneTowerEvent {
    pub wood_tower_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct StoneTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct GarrisonUnitInStoneTowerEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
    pub base_attack_range: I32F32,
    pub base_watch_range: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct GarrisonUnitInStoneTowerRejectedEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UngarrisonUnitFromStoneTowerEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageStoneTowerEvent {
    pub tower_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct StoneTowerDepositedUnitEvent {
    pub unit_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_stone_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceStoneTowerEvent>,
    mut rejected: EventWriter<StoneTowerPlacementRejectedEvent>,
    mut claims: ResMut<StoneTowerPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims.claims.values().any(|existing| existing.x == anchor.x && existing.y == anchor.y) {
            rejected.send(StoneTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                StoneTower,
                anchor,
                BuildingHealth::default(),
                StoneTowerBuildState {
                    build_ticks_remaining: seconds_to_ticks(STONE_TOWER_BUILD_TIME_SECONDS),
                    upgrading_from_wood_tower: false,
                    completed: false,
                },
                StoneTowerEconomy::default(),
                StoneTowerFootprint::default(),
                StoneTowerGarrison::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_wood_tower_to_stone_tower_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeWoodTowerToStoneTowerEvent>,
    wood_towers: Query<(Entity, &BuildingAnchor), With<WoodTower>>,
    mut claims: ResMut<StoneTowerPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((wood_tower_entity, anchor)) = wood_towers.get(ev.wood_tower_entity) else {
            continue;
        };

        commands.entity(wood_tower_entity).remove::<WoodTower>();
        commands.entity(wood_tower_entity).insert((
            StoneTower,
            BuildingHealth::default(),
            StoneTowerBuildState {
                build_ticks_remaining: seconds_to_ticks(STONE_TOWER_UPGRADE_TIME_SECONDS),
                upgrading_from_wood_tower: true,
                completed: false,
            },
            StoneTowerEconomy::default(),
            StoneTowerFootprint::default(),
            StoneTowerGarrison::default(),
        ));

        claims.claims.insert(wood_tower_entity, *anchor);
    }
}

fn stone_tower_build_tick_system(
    mut towers: Query<&mut StoneTowerBuildState, With<StoneTower>>,
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

fn garrison_unit_in_stone_tower_system(
    mut events: EventReader<GarrisonUnitInStoneTowerEvent>,
    mut rejected: EventWriter<GarrisonUnitInStoneTowerRejectedEvent>,
    mut towers: Query<(&StoneTowerBuildState, &StoneTowerFootprint, &mut StoneTowerGarrison), With<StoneTower>>,
) {
    for ev in events.read() {
        let Ok((build, footprint, mut garrison)) = towers.get_mut(ev.tower_entity) else {
            rejected.send(GarrisonUnitInStoneTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        };

        if !build.completed {
            rejected.send(GarrisonUnitInStoneTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        }

        if garrison.units.contains_key(&ev.unit_id) {
            continue;
        }

        if garrison.units.len() as i32 >= footprint.garrison_capacity {
            rejected.send(GarrisonUnitInStoneTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        }

        let tower_attack_range = ev.base_attack_range + footprint.attack_range_bonus;
        let tower_watch_range = ev.base_watch_range + footprint.watch_range_bonus;

        garrison.units.insert(
            ev.unit_id,
            GarrisonedUnit {
                unit_id: ev.unit_id,
                base_attack_range: ev.base_attack_range,
                base_watch_range: ev.base_watch_range,
                tower_attack_range,
                tower_watch_range,
            },
        );
    }
}

fn ungarrison_unit_from_stone_tower_system(
    mut events: EventReader<UngarrisonUnitFromStoneTowerEvent>,
    mut towers: Query<&mut StoneTowerGarrison, With<StoneTower>>,
) {
    for ev in events.read() {
        let Ok(mut garrison) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        garrison.units.remove(&ev.unit_id);
    }
}

fn damage_stone_tower_system(
    mut commands: Commands,
    mut events: EventReader<DamageStoneTowerEvent>,
    mut deposited_units: EventWriter<StoneTowerDepositedUnitEvent>,
    mut towers: Query<(Entity, &BuildingAnchor, &mut BuildingHealth, &StoneTowerGarrison), With<StoneTower>>,
    mut claims: ResMut<StoneTowerPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((tower_entity, anchor, mut hp, garrison)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        if ev.damage <= I32F32::ZERO {
            continue;
        }

        if hp.hp > ev.damage {
            hp.hp -= ev.damage;
            continue;
        }

        hp.hp = I32F32::ZERO;
        for unit in garrison.units.values() {
            deposited_units.send(StoneTowerDepositedUnitEvent {
                unit_id: unit.unit_id,
                tile_x: anchor.x,
                tile_y: anchor.y,
            });
        }

        claims.claims.remove(&tower_entity);
        commands.entity(tower_entity).despawn();
    }
}

fn stone_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            &BuildingAnchor,
            &BuildingHealth,
            &StoneTowerBuildState,
            &StoneTowerEconomy,
            &StoneTowerFootprint,
            &StoneTowerGarrison,
        ),
        With<StoneTower>,
    >,
    claims: Res<StoneTowerPlacementClaims>,
) {
    for (anchor, health, build, economy, footprint, garrison) in &towers {
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.hp.to_bits() as u64);
        checksum.accumulate(health.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_wood_tower));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.upgrade_wood_cost as u64);
        checksum.accumulate(economy.upgrade_stone_cost as u64);
        checksum.accumulate(economy.upgrade_iron_cost as u64);
        checksum.accumulate(economy.upgrade_oil_cost as u64);
        checksum.accumulate(economy.upgrade_gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range_bonus.to_bits() as u64);
        checksum.accumulate(footprint.attack_range_bonus.to_bits() as u64);
        checksum.accumulate(footprint.garrison_capacity as u64);

        checksum.accumulate(garrison.units.len() as u64);
        for (unit_id, unit) in &garrison.units {
            checksum.accumulate(*unit_id as u64);
            checksum.accumulate(unit.base_attack_range.to_bits() as u64);
            checksum.accumulate(unit.base_watch_range.to_bits() as u64);
            checksum.accumulate(unit.tower_attack_range.to_bits() as u64);
            checksum.accumulate(unit.tower_watch_range.to_bits() as u64);
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct StoneTowerPlugin;

impl Plugin for StoneTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StoneTowerPlacementClaims>()
            .add_event::<PlaceStoneTowerEvent>()
            .add_event::<UpgradeWoodTowerToStoneTowerEvent>()
            .add_event::<StoneTowerPlacementRejectedEvent>()
            .add_event::<GarrisonUnitInStoneTowerEvent>()
            .add_event::<GarrisonUnitInStoneTowerRejectedEvent>()
            .add_event::<UngarrisonUnitFromStoneTowerEvent>()
            .add_event::<DamageStoneTowerEvent>()
            .add_event::<StoneTowerDepositedUnitEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_stone_tower_system,
                    upgrade_wood_tower_to_stone_tower_system,
                    stone_tower_build_tick_system,
                    garrison_unit_in_stone_tower_system,
                    ungarrison_unit_from_stone_tower_system,
                    damage_stone_tower_system,
                    stone_tower_checksum_system,
                )
                    .chain(),
            );
    }
}