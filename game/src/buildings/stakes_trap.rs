// Sources: vault/buildings/stakes_trap.md, vault/buildings/buildings.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const STAKES_TRAP_HP: I32F32 = I32F32::lit("150");
const STAKES_TRAP_DEFENSES_LIFE: I32F32 = I32F32::lit("750");
const STAKES_TRAP_WATCH_RANGE: I32F32 = I32F32::lit("0");
const STAKES_TRAP_ENERGY_COST: i32 = 0;
const STAKES_TRAP_WOOD_COST: i32 = 3;
const STAKES_TRAP_STONE_COST: i32 = 0;
const STAKES_TRAP_IRON_COST: i32 = 0;
const STAKES_TRAP_OIL_COST: i32 = 0;
const STAKES_TRAP_GOLD_COST: i32 = 30;
const STAKES_TRAP_BUILD_TIME_SECONDS: i32 = 15;
const STAKES_TRAP_SIZE_TILES: i32 = 1;
const STAKES_TRAP_DAMAGE_PER_HIT: I32F32 = I32F32::lit("5");
const STAKES_TRAP_HITS_PER_SECOND: I32F32 = I32F32::lit("2");
const STAKES_TRAP_SLOW_PERCENT: I32F32 = I32F32::lit("50");
const STAKES_TRAP_NOISE: i32 = 0;
const STAKES_TRAP_DAMAGE_WHILE_BUILDING: bool = false;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct StakesTrap;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct StakesTrapCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for StakesTrapCore {
    fn default() -> Self {
        Self {
            defenses_life: STAKES_TRAP_DEFENSES_LIFE,
            watch_range: STAKES_TRAP_WATCH_RANGE,
            health: Health::full(STAKES_TRAP_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StakesTrapEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for StakesTrapEconomy {
    fn default() -> Self {
        Self {
            energy_cost: STAKES_TRAP_ENERGY_COST,
            wood_cost: STAKES_TRAP_WOOD_COST,
            stone_cost: STAKES_TRAP_STONE_COST,
            iron_cost: STAKES_TRAP_IRON_COST,
            oil_cost: STAKES_TRAP_OIL_COST,
            gold_cost: STAKES_TRAP_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StakesTrapFootprint {
    pub size_tiles: i32,
}

impl Default for StakesTrapFootprint {
    fn default() -> Self {
        Self {
            size_tiles: STAKES_TRAP_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct StakesTrapBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StakesTrapProfile {
    pub attack_damage: I32F32,
    pub hits_per_second: I32F32,
    pub slow_percent: I32F32,
    pub noise: i32,
    pub damage_while_building: bool,
    pub upgrades_to_wire_fence_trap: bool,
}

impl Default for StakesTrapProfile {
    fn default() -> Self {
        Self {
            attack_damage: STAKES_TRAP_DAMAGE_PER_HIT,
            hits_per_second: STAKES_TRAP_HITS_PER_SECOND,
            slow_percent: STAKES_TRAP_SLOW_PERCENT,
            noise: STAKES_TRAP_NOISE,
            damage_while_building: STAKES_TRAP_DAMAGE_WHILE_BUILDING,
            upgrades_to_wire_fence_trap: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct StakesTrapRuntime {
    pub hit_charge: I32F32,
    pub total_hits_dealt: i64,
    pub total_damage_dealt: I32F32,
    pub total_self_damage_taken: I32F32,
    pub total_noise_generated: i64,
}

#[derive(Component, Clone, Copy, Default)]
pub struct TrapTileOccupant {
    pub tile_x: i32,
    pub tile_y: i32,
    pub infected: bool,
    pub friendly: bool,
}

#[derive(Component, Clone, Copy)]
pub struct TrapTileEffect {
    pub move_speed_multiplier: I32F32,
    pub damage_taken_last_tick: I32F32,
}

impl Default for TrapTileEffect {
    fn default() -> Self {
        Self {
            move_speed_multiplier: I32F32::lit("1"),
            damage_taken_last_tick: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TrapTileHealth {
    pub health: Health,
}

impl Default for TrapTileHealth {
    fn default() -> Self {
        Self {
            health: Health::full(I32F32::ZERO),
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct StakesTrapResearchState {
    pub unlocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct StakesTrapPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetStakesTrapResearchUnlockedEvent {
    pub unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStakesTrapEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct StakesTrapPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetTrapTileOccupantEvent {
    pub entity: Entity,
    pub tile_x: i32,
    pub tile_y: i32,
    pub infected: bool,
    pub friendly: bool,
    pub hp: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn apply_research_unlock_event_system(
    mut events: EventReader<SetStakesTrapResearchUnlockedEvent>,
    mut research: ResMut<StakesTrapResearchState>,
) {
    for ev in events.read() {
        research.unlocked = ev.unlocked;
    }
}

fn place_stakes_trap_system(
    mut commands: Commands,
    mut events: EventReader<PlaceStakesTrapEvent>,
    mut rejected: EventWriter<StakesTrapPlacementRejectedEvent>,
    research: Res<StakesTrapResearchState>,
    mut claims: ResMut<StakesTrapPlacementClaims>,
) {
    for ev in events.read() {
        if !research.unlocked {
            rejected.send(StakesTrapPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                StakesTrap,
                BuildingAnchor {
                    x: ev.tile_x,
                    y: ev.tile_y,
                },
                StakesTrapCore::default(),
                StakesTrapEconomy::default(),
                StakesTrapFootprint::default(),
                StakesTrapBuildState {
                    build_ticks_remaining: seconds_to_ticks(STAKES_TRAP_BUILD_TIME_SECONDS),
                    completed: false,
                },
                StakesTrapProfile::default(),
                StakesTrapRuntime::default(),
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

fn set_trap_tile_occupant_event_system(
    mut commands: Commands,
    mut events: EventReader<SetTrapTileOccupantEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert((
            TrapTileOccupant {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
                infected: ev.infected,
                friendly: ev.friendly,
            },
            TrapTileEffect::default(),
            TrapTileHealth {
                health: Health::full(ev.hp.max(I32F32::ZERO)),
            },
        ));
    }
}

fn stakes_trap_build_tick_system(mut traps: Query<&mut StakesTrapBuildState, With<StakesTrap>>) {
    for mut build in &mut traps {
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

fn reset_trap_tile_effects_system(mut occupants: Query<&mut TrapTileEffect, With<TrapTileOccupant>>) {
    for mut effect in &mut occupants {
        effect.move_speed_multiplier = I32F32::lit("1");
        effect.damage_taken_last_tick = I32F32::ZERO;
    }
}

fn stakes_trap_apply_effects_system(
    mut commands: Commands,
    mut traps: Query<
        (
            Entity,
            &BuildingAnchor,
            &mut StakesTrapCore,
            &StakesTrapBuildState,
            &StakesTrapProfile,
            &mut StakesTrapRuntime,
        ),
        With<StakesTrap>,
    >,
    mut occupants: Query<(Entity, &TrapTileOccupant, &mut TrapTileEffect, &mut TrapTileHealth)>,
    mut claims: ResMut<StakesTrapPlacementClaims>,
) {
    for (trap_entity, anchor, mut core, build, profile, mut runtime) in &mut traps {
        let slow_multiplier = (I32F32::lit("100") - profile.slow_percent) / I32F32::lit("100");
        let can_damage = build.completed || profile.damage_while_building;

        let mut occupant_entities_on_tile = Vec::new();
        for (occupant_entity, occ, mut effect, _) in &mut occupants {
            if occ.tile_x == anchor.x && occ.tile_y == anchor.y {
                effect.move_speed_multiplier = slow_multiplier;
                occupant_entities_on_tile.push(occupant_entity);
            }
        }

        runtime.hit_charge += profile.hits_per_second;
        while runtime.hit_charge >= I32F32::from_num(SIM_HZ) {
            runtime.hit_charge -= I32F32::from_num(SIM_HZ);

            if !can_damage {
                continue;
            }

            let mut self_damage_this_hit = I32F32::ZERO;

            for occupant_entity in &occupant_entities_on_tile {
                let Ok((_, occ, mut effect, mut health)) = occupants.get_mut(*occupant_entity) else {
                    continue;
                };

                if !occ.infected {
                    continue;
                }

                let dmg = profile.attack_damage.max(I32F32::ZERO);
                if dmg <= I32F32::ZERO {
                    continue;
                }

                let applied = health.health.current.min(dmg).max(I32F32::ZERO);
                health.health.current -= applied;
                effect.damage_taken_last_tick += applied;

                runtime.total_hits_dealt += 1;
                runtime.total_damage_dealt += applied;
                self_damage_this_hit += I32F32::lit("1");
            }

            core.health.current -= self_damage_this_hit;
            runtime.total_self_damage_taken += self_damage_this_hit;
            runtime.total_noise_generated += profile.noise as i64;

            if core.health.current <= I32F32::ZERO {
                claims.claims.remove(&trap_entity);
                commands.entity(trap_entity).despawn();
                break;
            }
        }
    }
}

fn stakes_trap_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    traps: Query<
        (
            Entity,
            &BuildingAnchor,
            &StakesTrapCore,
            &StakesTrapEconomy,
            &StakesTrapFootprint,
            &StakesTrapBuildState,
            &StakesTrapProfile,
            &StakesTrapRuntime,
        ),
        With<StakesTrap>,
    >,
    occupants: Query<(Entity, &TrapTileOccupant, &TrapTileEffect, &TrapTileHealth)>,
    research: Res<StakesTrapResearchState>,
    claims: Res<StakesTrapPlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, profile, runtime) in &traps {
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

        checksum.accumulate(footprint.size_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(profile.attack_damage.to_bits() as u64);
        checksum.accumulate(profile.hits_per_second.to_bits() as u64);
        checksum.accumulate(profile.slow_percent.to_bits() as u64);
        checksum.accumulate(profile.noise as u64);
        checksum.accumulate(u64::from(profile.damage_while_building));
        checksum.accumulate(u64::from(profile.upgrades_to_wire_fence_trap));

        checksum.accumulate(runtime.hit_charge.to_bits() as u64);
        checksum.accumulate(runtime.total_hits_dealt as u64);
        checksum.accumulate(runtime.total_damage_dealt.to_bits() as u64);
        checksum.accumulate(runtime.total_self_damage_taken.to_bits() as u64);
        checksum.accumulate(runtime.total_noise_generated as u64);
    }

    for (entity, occ, effect, health) in &occupants {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(occ.tile_x as u64);
        checksum.accumulate(occ.tile_y as u64);
        checksum.accumulate(u64::from(occ.infected));
        checksum.accumulate(u64::from(occ.friendly));
        checksum.accumulate(effect.move_speed_multiplier.to_bits() as u64);
        checksum.accumulate(effect.damage_taken_last_tick.to_bits() as u64);
        checksum.accumulate(health.health.current.to_bits() as u64);
        checksum.accumulate(health.health.max.to_bits() as u64);
    }

    checksum.accumulate(u64::from(research.unlocked));

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct StakesTrapPlugin;

impl Plugin for StakesTrapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StakesTrapResearchState>()
            .init_resource::<StakesTrapPlacementClaims>()
            .add_event::<SetStakesTrapResearchUnlockedEvent>()
            .add_event::<PlaceStakesTrapEvent>()
            .add_event::<StakesTrapPlacementRejectedEvent>()
            .add_event::<SetTrapTileOccupantEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_research_unlock_event_system,
                    place_stakes_trap_system,
                    set_trap_tile_occupant_event_system,
                    stakes_trap_build_tick_system,
                    reset_trap_tile_effects_system,
                    stakes_trap_apply_effects_system,
                    stakes_trap_checksum_system,
                )
                    .chain(),
            );
    }
}