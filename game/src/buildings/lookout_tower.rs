// Sources: vault/buildings/lookout_tower.md, vault/buildings/radar_tower.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const LOOKOUT_TOWER_HP: I32F32 = I32F32::lit("300");
const LOOKOUT_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("75");
const LOOKOUT_TOWER_WATCH_RANGE: I32F32 = I32F32::lit("24");
const LOOKOUT_TOWER_ENERGY_COST: i32 = 0;
const LOOKOUT_TOWER_WOOD_COST: i32 = 10;
const LOOKOUT_TOWER_STONE_COST: i32 = 0;
const LOOKOUT_TOWER_IRON_COST: i32 = 0;
const LOOKOUT_TOWER_OIL_COST: i32 = 0;
const LOOKOUT_TOWER_GOLD_COST: i32 = 300;
const LOOKOUT_TOWER_BUILD_TIME_SECONDS: i32 = 39;
const LOOKOUT_TOWER_WORKERS: i32 = 1;
const LOOKOUT_TOWER_GOLD_MAINTENANCE: i32 = 0;
const LOOKOUT_TOWER_ENERGY_MAINTENANCE: i32 = 0;
const LOOKOUT_TOWER_SIZE_TILES: i32 = 1;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct LookoutTower;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct LookoutTowerCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for LookoutTowerCore {
    fn default() -> Self {
        Self {
            defenses_life: LOOKOUT_TOWER_DEFENSES_LIFE,
            watch_range: LOOKOUT_TOWER_WATCH_RANGE,
            health: Health::full(LOOKOUT_TOWER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LookoutTowerEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
    pub energy_maintenance: i32,
}

impl Default for LookoutTowerEconomy {
    fn default() -> Self {
        Self {
            energy_cost: LOOKOUT_TOWER_ENERGY_COST,
            wood_cost: LOOKOUT_TOWER_WOOD_COST,
            stone_cost: LOOKOUT_TOWER_STONE_COST,
            iron_cost: LOOKOUT_TOWER_IRON_COST,
            oil_cost: LOOKOUT_TOWER_OIL_COST,
            gold_cost: LOOKOUT_TOWER_GOLD_COST,
            workers: LOOKOUT_TOWER_WORKERS,
            gold_maintenance: LOOKOUT_TOWER_GOLD_MAINTENANCE,
            energy_maintenance: LOOKOUT_TOWER_ENERGY_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LookoutTowerFootprint {
    pub size_tiles: i32,
}

impl Default for LookoutTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: LOOKOUT_TOWER_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct LookoutTowerBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct LookoutTowerState {
    pub active: bool,
    pub alerts_colony: bool,
    pub scouting_enabled: bool,
    pub reveals_approaching_hordes: bool,
    pub can_upgrade_to_radar_tower: bool,
}

#[derive(Resource, Default, Clone)]
pub struct LookoutTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceLookoutTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct LookoutTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct RemoveLookoutTowerEvent {
    pub tower_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct DamageLookoutTowerEvent {
    pub tower_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct SetLookoutTowerThreatVisibleEvent {
    pub tower_entity: Entity,
    pub threat_visible: bool,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_lookout_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceLookoutTowerEvent>,
    mut rejected: EventWriter<LookoutTowerPlacementRejectedEvent>,
    mut claims: ResMut<LookoutTowerPlacementClaims>,
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
            rejected.send(LookoutTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                LookoutTower,
                anchor,
                LookoutTowerCore::default(),
                LookoutTowerEconomy::default(),
                LookoutTowerFootprint::default(),
                LookoutTowerBuildState {
                    build_ticks_remaining: seconds_to_ticks(LOOKOUT_TOWER_BUILD_TIME_SECONDS),
                    completed: false,
                },
                LookoutTowerState {
                    active: false,
                    alerts_colony: false,
                    scouting_enabled: true,
                    reveals_approaching_hordes: false,
                    can_upgrade_to_radar_tower: true,
                },
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn lookout_tower_build_tick_system(
    mut towers: Query<(&mut LookoutTowerBuildState, &mut LookoutTowerState), With<LookoutTower>>,
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

fn set_lookout_tower_threat_visibility_system(
    mut events: EventReader<SetLookoutTowerThreatVisibleEvent>,
    mut towers: Query<(&LookoutTowerBuildState, &mut LookoutTowerState), With<LookoutTower>>,
) {
    for ev in events.read() {
        let Ok((build, mut state)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        if !build.completed {
            state.alerts_colony = false;
            state.reveals_approaching_hordes = false;
            continue;
        }

        state.reveals_approaching_hordes = ev.threat_visible;
        state.alerts_colony = ev.threat_visible;
    }
}

fn damage_lookout_tower_system(
    mut commands: Commands,
    mut events: EventReader<DamageLookoutTowerEvent>,
    mut towers: Query<(Entity, &mut LookoutTowerCore), With<LookoutTower>>,
    mut claims: ResMut<LookoutTowerPlacementClaims>,
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

fn remove_lookout_tower_system(
    mut commands: Commands,
    mut events: EventReader<RemoveLookoutTowerEvent>,
    towers: Query<(), With<LookoutTower>>,
    mut claims: ResMut<LookoutTowerPlacementClaims>,
) {
    for ev in events.read() {
        if towers.get(ev.tower_entity).is_ok() {
            claims.claims.remove(&ev.tower_entity);
            commands.entity(ev.tower_entity).despawn();
        }
    }
}

fn lookout_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            &BuildingAnchor,
            &LookoutTowerCore,
            &LookoutTowerEconomy,
            &LookoutTowerFootprint,
            &LookoutTowerBuildState,
            &LookoutTowerState,
        ),
        With<LookoutTower>,
    >,
    claims: Res<LookoutTowerPlacementClaims>,
) {
    for (anchor, core, eco, footprint, build, state) in &towers {
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);
        checksum.accumulate(eco.gold_maintenance as u64);
        checksum.accumulate(eco.energy_maintenance as u64);

        checksum.accumulate(footprint.size_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(u64::from(state.active));
        checksum.accumulate(u64::from(state.alerts_colony));
        checksum.accumulate(u64::from(state.scouting_enabled));
        checksum.accumulate(u64::from(state.reveals_approaching_hordes));
        checksum.accumulate(u64::from(state.can_upgrade_to_radar_tower));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct LookoutTowerPlugin;

impl Plugin for LookoutTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LookoutTowerPlacementClaims>()
            .add_event::<PlaceLookoutTowerEvent>()
            .add_event::<LookoutTowerPlacementRejectedEvent>()
            .add_event::<RemoveLookoutTowerEvent>()
            .add_event::<DamageLookoutTowerEvent>()
            .add_event::<SetLookoutTowerThreatVisibleEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_lookout_tower_system,
                    lookout_tower_build_tick_system,
                    set_lookout_tower_threat_visibility_system,
                    damage_lookout_tower_system,
                    remove_lookout_tower_system,
                    lookout_tower_checksum_system,
                )
                    .chain(),
            );
    }
}