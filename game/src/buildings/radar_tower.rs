// Sources: vault/buildings/radar_tower.md, vault/buildings/lookout_tower.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const RADAR_TOWER_HP: I32F32 = I32F32::lit("500");
const RADAR_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const RADAR_TOWER_WATCH_RANGE: I32F32 = I32F32::lit("40");
const RADAR_TOWER_ENERGY_COST: i32 = 10;
const RADAR_TOWER_WOOD_COST: i32 = 10;
const RADAR_TOWER_STONE_COST: i32 = 10;
const RADAR_TOWER_IRON_COST: i32 = 10;
const RADAR_TOWER_OIL_COST: i32 = 0;
const RADAR_TOWER_GOLD_COST: i32 = 800;
const RADAR_TOWER_GOLD_MAINTENANCE: i32 = 30;
const RADAR_TOWER_WORKERS: i32 = 2;
const RADAR_TOWER_BUILD_TIME_SECONDS: i32 = 45;
const RADAR_TOWER_UPGRADE_TIME_SECONDS: i32 = 20;
const RADAR_TOWER_SIZE_TILES: i32 = 1;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct LookoutTower;

#[derive(Component, Default)]
pub struct RadarTower;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct RadarTowerCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for RadarTowerCore {
    fn default() -> Self {
        Self {
            defenses_life: RADAR_TOWER_DEFENSES_LIFE,
            watch_range: RADAR_TOWER_WATCH_RANGE,
            health: Health::full(RADAR_TOWER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RadarTowerEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for RadarTowerEconomy {
    fn default() -> Self {
        Self {
            energy_cost: RADAR_TOWER_ENERGY_COST,
            wood_cost: RADAR_TOWER_WOOD_COST,
            stone_cost: RADAR_TOWER_STONE_COST,
            iron_cost: RADAR_TOWER_IRON_COST,
            oil_cost: RADAR_TOWER_OIL_COST,
            gold_cost: RADAR_TOWER_GOLD_COST,
            workers: RADAR_TOWER_WORKERS,
            gold_maintenance: RADAR_TOWER_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RadarTowerFootprint {
    pub size_tiles: i32,
}

impl Default for RadarTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: RADAR_TOWER_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RadarTowerBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_lookout_tower: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct RadarTowerState {
    pub active: bool,
    pub alerts_colony: bool,
    pub threat_visible: bool,
    pub mayor_research_bypass: bool,
    pub can_spawn_as_ruin: bool,
}

#[derive(Resource, Default, Clone)]
pub struct RadarTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceRadarTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeLookoutTowerToRadarTowerEvent {
    pub lookout_tower_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct RadarTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageRadarTowerEvent {
    pub tower_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct SetRadarTowerThreatVisibleEvent {
    pub tower_entity: Entity,
    pub threat_visible: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetRadarTowerMayorResearchBypassEvent {
    pub tower_entity: Entity,
    pub bypass_enabled: bool,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_radar_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceRadarTowerEvent>,
    mut rejected: EventWriter<RadarTowerPlacementRejectedEvent>,
    mut claims: ResMut<RadarTowerPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y)
        {
            rejected.send(RadarTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                RadarTower,
                anchor,
                RadarTowerCore::default(),
                RadarTowerEconomy::default(),
                RadarTowerFootprint::default(),
                RadarTowerBuildState {
                    build_ticks_remaining: seconds_to_ticks(RADAR_TOWER_BUILD_TIME_SECONDS),
                    upgrading_from_lookout_tower: false,
                    completed: false,
                },
                RadarTowerState {
                    active: false,
                    alerts_colony: false,
                    threat_visible: false,
                    mayor_research_bypass: false,
                    can_spawn_as_ruin: true,
                },
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_lookout_tower_to_radar_tower_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeLookoutTowerToRadarTowerEvent>,
    lookout_towers: Query<(Entity, &BuildingAnchor), With<LookoutTower>>,
    mut claims: ResMut<RadarTowerPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, anchor)) = lookout_towers.get(ev.lookout_tower_entity) else {
            continue;
        };

        commands.entity(entity).remove::<LookoutTower>();
        commands.entity(entity).insert((
            RadarTower,
            RadarTowerCore::default(),
            RadarTowerEconomy::default(),
            RadarTowerFootprint::default(),
            RadarTowerBuildState {
                build_ticks_remaining: seconds_to_ticks(RADAR_TOWER_UPGRADE_TIME_SECONDS),
                upgrading_from_lookout_tower: true,
                completed: false,
            },
            RadarTowerState {
                active: false,
                alerts_colony: false,
                threat_visible: false,
                mayor_research_bypass: false,
                can_spawn_as_ruin: true,
            },
        ));

        claims.claims.insert(entity, *anchor);
    }
}

fn radar_tower_build_tick_system(
    mut towers: Query<(&mut RadarTowerBuildState, &mut RadarTowerState), With<RadarTower>>,
) {
    for (mut build, mut state) in &mut towers {
        if build.completed {
            state.active = true;
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
            state.active = true;
        }
    }
}

fn set_radar_tower_threat_visible_system(
    mut events: EventReader<SetRadarTowerThreatVisibleEvent>,
    mut towers: Query<(&RadarTowerBuildState, &mut RadarTowerState), With<RadarTower>>,
) {
    for ev in events.read() {
        let Ok((build, mut state)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        if !build.completed {
            state.threat_visible = false;
            state.alerts_colony = false;
            continue;
        }

        state.threat_visible = ev.threat_visible;
        state.alerts_colony = ev.threat_visible;
    }
}

fn set_radar_tower_mayor_bypass_system(
    mut events: EventReader<SetRadarTowerMayorResearchBypassEvent>,
    mut towers: Query<&mut RadarTowerState, With<RadarTower>>,
) {
    for ev in events.read() {
        let Ok(mut state) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        state.mayor_research_bypass = ev.bypass_enabled;
    }
}

fn damage_radar_tower_system(
    mut commands: Commands,
    mut events: EventReader<DamageRadarTowerEvent>,
    mut towers: Query<(Entity, &mut RadarTowerCore), With<RadarTower>>,
    mut claims: ResMut<RadarTowerPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut core)) = towers.get_mut(ev.tower_entity) else {
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
            if core.health.current > incoming {
                core.health.current -= incoming;
                continue;
            }

            core.health.current = I32F32::ZERO;
            claims.claims.remove(&entity);
            commands.entity(entity).despawn();
        }
    }
}

fn radar_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            &BuildingAnchor,
            &RadarTowerCore,
            &RadarTowerEconomy,
            &RadarTowerFootprint,
            &RadarTowerBuildState,
            &RadarTowerState,
        ),
        With<RadarTower>,
    >,
    claims: Res<RadarTowerPlacementClaims>,
) {
    for (anchor, core, economy, footprint, build, state) in &towers {
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
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.gold_maintenance as u64);

        checksum.accumulate(footprint.size_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_lookout_tower));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(u64::from(state.active));
        checksum.accumulate(u64::from(state.alerts_colony));
        checksum.accumulate(u64::from(state.threat_visible));
        checksum.accumulate(u64::from(state.mayor_research_bypass));
        checksum.accumulate(u64::from(state.can_spawn_as_ruin));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct RadarTowerPlugin;

impl Plugin for RadarTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RadarTowerPlacementClaims>()
            .add_event::<PlaceRadarTowerEvent>()
            .add_event::<UpgradeLookoutTowerToRadarTowerEvent>()
            .add_event::<RadarTowerPlacementRejectedEvent>()
            .add_event::<DamageRadarTowerEvent>()
            .add_event::<SetRadarTowerThreatVisibleEvent>()
            .add_event::<SetRadarTowerMayorResearchBypassEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_radar_tower_system,
                    upgrade_lookout_tower_to_radar_tower_system,
                    radar_tower_build_tick_system,
                    set_radar_tower_threat_visible_system,
                    set_radar_tower_mayor_bypass_system,
                    damage_radar_tower_system,
                    radar_tower_checksum_system,
                )
                    .chain(),
            );
    }
}