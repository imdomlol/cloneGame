// Sources: vault/buildings/land_mine.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, Health, SimChecksumState, SimTick};

const LAND_MINE_HP: I32F32 = I32F32::lit("0");
const LAND_MINE_WATCH_RANGE: I32F32 = I32F32::lit("0");
const LAND_MINE_GOLD_COST: i32 = 100;
const LAND_MINE_IRON_COST: i32 = 3;
const LAND_MINE_BUILD_TIME_SECONDS: i32 = 1;
const LAND_MINE_SIZE_TILES: i32 = 1;
const LAND_MINE_EXPLOSION_DAMAGE: I32F32 = I32F32::lit("500");
const LAND_MINE_EXPLOSION_RADIUS_TILES: i32 = 1;
const LAND_MINE_TRIGGER_DELAY_SECONDS_MIN: i32 = 0;
const LAND_MINE_TRIGGER_DELAY_SECONDS_MAX: i32 = 3;
const LAND_MINE_RESEARCH_TIER_STONE_WORKSHOP: bool = true;
const SIM_HZ: i32 = 25;
const LAND_MINE_TRIGGER_SALT: u64 = 0xF261_6F4B_8C71_DAE3;

#[derive(Component, Default)]
pub struct LandMine;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct LandMineCore {
    pub watch_range: I32F32,
    pub health: Health,
    pub invincible: bool,
}

impl Default for LandMineCore {
    fn default() -> Self {
        Self {
            watch_range: LAND_MINE_WATCH_RANGE,
            health: Health::full(LAND_MINE_HP),
            invincible: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LandMineEconomy {
    pub gold_cost: i32,
    pub iron_cost: i32,
}

impl Default for LandMineEconomy {
    fn default() -> Self {
        Self {
            gold_cost: LAND_MINE_GOLD_COST,
            iron_cost: LAND_MINE_IRON_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LandMineFootprint {
    pub size_tiles: i32,
}

impl Default for LandMineFootprint {
    fn default() -> Self {
        Self {
            size_tiles: LAND_MINE_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct LandMineBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct LandMineProfile {
    pub explosion_damage: I32F32,
    pub explosion_radius_tiles: i32,
    pub trigger_delay_seconds_min: i32,
    pub trigger_delay_seconds_max: i32,
}

impl Default for LandMineProfile {
    fn default() -> Self {
        Self {
            explosion_damage: LAND_MINE_EXPLOSION_DAMAGE,
            explosion_radius_tiles: LAND_MINE_EXPLOSION_RADIUS_TILES,
            trigger_delay_seconds_min: LAND_MINE_TRIGGER_DELAY_SECONDS_MIN,
            trigger_delay_seconds_max: LAND_MINE_TRIGGER_DELAY_SECONDS_MAX,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct LandMineRuntime {
    pub pending_detonation: bool,
    pub trigger_delay_ticks_remaining: i32,
    pub triggered_by_game_entity_id: i32,
    pub detonation_count: i32,
    pub last_roll_seconds: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct LandMineTileOccupant {
    pub game_entity_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub infected: bool,
    pub friendly: bool,
    pub building: bool,
    pub defensive_barrier: bool,
    pub land_mine: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct LandMineTileEffect {
    pub damage_taken_last_tick: I32F32,
}

#[derive(Component, Clone, Copy)]
pub struct LandMineTileHealth {
    pub health: Health,
}

impl Default for LandMineTileHealth {
    fn default() -> Self {
        Self {
            health: Health::full(I32F32::ZERO),
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct LandMineResearchState {
    pub stone_workshop_unlocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct LandMinePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetLandMineResearchUnlockedEvent {
    pub stone_workshop_unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceLandMineEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct LandMinePlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetLandMineTileOccupantEvent {
    pub entity: Entity,
    pub game_entity_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub infected: bool,
    pub friendly: bool,
    pub building: bool,
    pub defensive_barrier: bool,
    pub land_mine: bool,
    pub hp: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn apply_research_unlock_event_system(
    mut events: EventReader<SetLandMineResearchUnlockedEvent>,
    mut research: ResMut<LandMineResearchState>,
) {
    for ev in events.read() {
        research.stone_workshop_unlocked = ev.stone_workshop_unlocked;
    }
}

fn place_land_mine_system(
    mut commands: Commands,
    mut events: EventReader<PlaceLandMineEvent>,
    mut rejected: EventWriter<LandMinePlacementRejectedEvent>,
    research: Res<LandMineResearchState>,
    mut claims: ResMut<LandMinePlacementClaims>,
) {
    for ev in events.read() {
        if LAND_MINE_RESEARCH_TIER_STONE_WORKSHOP && !research.stone_workshop_unlocked {
            rejected.send(LandMinePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                LandMine,
                BuildingAnchor {
                    x: ev.tile_x,
                    y: ev.tile_y,
                },
                LandMineCore::default(),
                LandMineEconomy::default(),
                LandMineFootprint::default(),
                LandMineBuildState {
                    build_ticks_remaining: seconds_to_ticks(LAND_MINE_BUILD_TIME_SECONDS),
                    completed: false,
                },
                LandMineProfile::default(),
                LandMineRuntime::default(),
            ))
            .id();

        claims.claims.insert(
            entity,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
        );
    }
}

fn set_land_mine_tile_occupant_event_system(
    mut commands: Commands,
    mut events: EventReader<SetLandMineTileOccupantEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert((
            LandMineTileOccupant {
                game_entity_id: ev.game_entity_id,
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
                infected: ev.infected,
                friendly: ev.friendly,
                building: ev.building,
                defensive_barrier: ev.defensive_barrier,
                land_mine: ev.land_mine,
            },
            LandMineTileEffect::default(),
            LandMineTileHealth {
                health: Health::full(ev.hp.max(I32F32::ZERO)),
            },
        ));
    }
}

fn land_mine_build_tick_system(mut mines: Query<&mut LandMineBuildState, With<LandMine>>) {
    for mut build in &mut mines {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn reset_land_mine_tile_effects_system(
    mut occupants: Query<&mut LandMineTileEffect, With<LandMineTileOccupant>>,
) {
    for mut effect in &mut occupants {
        effect.damage_taken_last_tick = I32F32::ZERO;
    }
}

fn land_mine_trigger_system(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    mut mines: Query<(&BuildingAnchor, &LandMineBuildState, &LandMineProfile, &mut LandMineRuntime), With<LandMine>>,
    occupants: Query<&LandMineTileOccupant>,
) {
    for (anchor, build, profile, mut runtime) in &mut mines {
        if !build.completed || runtime.pending_detonation {
            continue;
        }

        let mut trigger_source_game_entity_id = None;
        for occ in &occupants {
            if occ.tile_x == anchor.x && occ.tile_y == anchor.y && occ.infected {
                trigger_source_game_entity_id = Some(occ.game_entity_id);
                break;
            }
        }

        let Some(game_entity_id) = trigger_source_game_entity_id else {
            continue;
        };

        let delay_span = profile.trigger_delay_seconds_max - profile.trigger_delay_seconds_min + 1;
        if delay_span <= 0 {
            continue;
        }

        let salt = LAND_MINE_TRIGGER_SALT
            ^ (game_entity_id as u64)
            ^ ((anchor.x as u64) << 32)
            ^ (anchor.y as u64);
        let mut rng = tick_rng(game_seed.0, sim_tick.0, salt);
        let delay_seconds = profile.trigger_delay_seconds_min + (rng.next_u32() % (delay_span as u32)) as i32;

        runtime.pending_detonation = true;
        runtime.trigger_delay_ticks_remaining = seconds_to_ticks(delay_seconds);
        runtime.triggered_by_game_entity_id = game_entity_id;
        runtime.last_roll_seconds = delay_seconds;
    }
}

fn land_mine_detonation_tick_system(
    mut commands: Commands,
    mut mines: Query<
        (
            Entity,
            &BuildingAnchor,
            &LandMineProfile,
            &mut LandMineRuntime,
        ),
        With<LandMine>,
    >,
    mut occupants: Query<(Entity, &LandMineTileOccupant, &mut LandMineTileEffect, &mut LandMineTileHealth)>,
    mut claims: ResMut<LandMinePlacementClaims>,
) {
    for (mine_entity, anchor, profile, mut runtime) in &mut mines {
        if !runtime.pending_detonation {
            continue;
        }

        if runtime.trigger_delay_ticks_remaining > 0 {
            runtime.trigger_delay_ticks_remaining -= 1;
            continue;
        }

        runtime.trigger_delay_ticks_remaining = 0;

        for (_entity, occ, mut effect, mut hp) in &mut occupants {
            let dx = (occ.tile_x - anchor.x).abs();
            let dy = (occ.tile_y - anchor.y).abs();
            if dx > profile.explosion_radius_tiles || dy > profile.explosion_radius_tiles {
                continue;
            }

            if occ.land_mine {
                continue;
            }

            if occ.building && occ.defensive_barrier {
                continue;
            }

            let dmg = profile.explosion_damage.max(I32F32::ZERO);
            if dmg <= I32F32::ZERO {
                continue;
            }

            let applied = hp.health.current.min(dmg).max(I32F32::ZERO);
            hp.health.current -= applied;
            effect.damage_taken_last_tick += applied;
        }

        runtime.pending_detonation = false;
        runtime.detonation_count += 1;
        claims.claims.remove(&mine_entity);
        commands.entity(mine_entity).despawn();
    }
}

fn land_mine_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    mines: Query<
        (
            Entity,
            &BuildingAnchor,
            &LandMineCore,
            &LandMineEconomy,
            &LandMineFootprint,
            &LandMineBuildState,
            &LandMineProfile,
            &LandMineRuntime,
        ),
        With<LandMine>,
    >,
    occupants: Query<(Entity, &LandMineTileOccupant, &LandMineTileEffect, &LandMineTileHealth)>,
    research: Res<LandMineResearchState>,
    claims: Res<LandMinePlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, profile, runtime) in &mines {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);
        checksum.accumulate(u64::from(core.invincible));

        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(profile.explosion_damage.to_bits() as u64);
        checksum.accumulate(profile.explosion_radius_tiles as u64);
        checksum.accumulate(profile.trigger_delay_seconds_min as u64);
        checksum.accumulate(profile.trigger_delay_seconds_max as u64);

        checksum.accumulate(u64::from(runtime.pending_detonation));
        checksum.accumulate(runtime.trigger_delay_ticks_remaining as u64);
        checksum.accumulate(runtime.triggered_by_game_entity_id as u64);
        checksum.accumulate(runtime.detonation_count as u64);
        checksum.accumulate(runtime.last_roll_seconds as u64);
    }

    for (entity, occ, effect, health) in &occupants {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(occ.game_entity_id as u64);
        checksum.accumulate(occ.tile_x as u64);
        checksum.accumulate(occ.tile_y as u64);
        checksum.accumulate(u64::from(occ.infected));
        checksum.accumulate(u64::from(occ.friendly));
        checksum.accumulate(u64::from(occ.building));
        checksum.accumulate(u64::from(occ.defensive_barrier));
        checksum.accumulate(u64::from(occ.land_mine));
        checksum.accumulate(effect.damage_taken_last_tick.to_bits() as u64);
        checksum.accumulate(health.health.current.to_bits() as u64);
        checksum.accumulate(health.health.max.to_bits() as u64);
    }

    checksum.accumulate(u64::from(research.stone_workshop_unlocked));

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct LandMinePlugin;

impl Plugin for LandMinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandMineResearchState>()
            .init_resource::<LandMinePlacementClaims>()
            .add_event::<SetLandMineResearchUnlockedEvent>()
            .add_event::<PlaceLandMineEvent>()
            .add_event::<LandMinePlacementRejectedEvent>()
            .add_event::<SetLandMineTileOccupantEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_research_unlock_event_system,
                    place_land_mine_system,
                    set_land_mine_tile_occupant_event_system,
                    land_mine_build_tick_system,
                    reset_land_mine_tile_effects_system,
                    land_mine_trigger_system,
                    land_mine_detonation_tick_system,
                    land_mine_checksum_system,
                )
                    .chain(),
            );
    }
}