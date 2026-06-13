// Sources: vault/destination_pages/61_march.md, vault/entity_pages/entity.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::daytime_entity_pages::circuit_bee::SpawnCircuitBeeEvent;
use crate::harmless_entity_pages::backwater_gunkfish::{
    BackwaterGunkfishFacing, SpawnBackwaterGunkfishEvent,
};
use crate::harmless_entity_pages::manticoil::SpawnManticoilEvent;
use crate::harmless_entity_pages::roaming_locust::SpawnRoamingLocustEvent;
use crate::harmless_entity_pages::tulip_snake::SpawnTulipSnakeEvent;
use crate::indoor_entity_pages::bracken::SpawnBrackenEvent;
use crate::indoor_entity_pages::bunker_spider::SpawnBunkerSpiderEvent;
use crate::indoor_entity_pages::coil_head::SpawnCoilHeadEvent;
use crate::indoor_entity_pages::hoarding_bug::SpawnHoardingBugEvent;
use crate::indoor_entity_pages::hygrodere::SpawnHygrodereEvent;
use crate::indoor_entity_pages::jester::SpawnJesterEvent;
use crate::indoor_entity_pages::maneater::SpawnManeaterEvent;
use crate::indoor_entity_pages::nutcrackers::SpawnNutcrackersEvent;
use crate::indoor_entity_pages::snare_flea::SpawnSnareFleaEvent;
use crate::indoor_entity_pages::spore_lizard::SpawnSporeLizardEvent;
use crate::indoor_entity_pages::thumper::SpawnThumperEvent;
use crate::outdoor_entity_pages::baboon_hawk::SpawnBaboonHawkEvent;
use crate::outdoor_entity_pages::earth_leviathan::SpawnEarthLeviathanEvent;
use crate::outdoor_entity_pages::eyeless_dog::SpawnEyelessDogEvent;
use crate::outdoor_entity_pages::feiopar::SpawnFeioparEvent;
use crate::outdoor_entity_pages::forest_keeper::SpawnForestKeeperEvent;
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick};
use crate::system::game_state_machine::GameState;

pub const ENTITY_SPAWNER_ID: &str = "entity_spawner";
pub const DESTINATION_ID: &str = "61_march";
pub const DESTINATION_NAME: &str = "61-March";
pub const DESTINATION_RISK_LEVEL: &str = "B";
pub const DESTINATION_DIFFICULTY: &str = "Intermediate";
pub const DESTINATION_COST: u16 = 0;
pub const DESTINATION_MAP_LAYOUT: &str = "factory";
pub const DESTINATION_MAP_LAYOUT_PERCENTAGE: u16 = 100;
pub const DESTINATION_MAP_SIZE_MULTIPLIER_BASIS_POINTS: u16 = 175;
pub const DESTINATION_MIN_SCRAP: u16 = 13;
pub const DESTINATION_MAX_SCRAP: u16 = 16;
pub const DESTINATION_POWER_COUNT: u16 = 14;
pub const DESTINATION_OUTSIDE_POWER_COUNT: u16 = 12;
pub const DESTINATION_DAYTIME_POWER_COUNT: u16 = 14;
pub const DESTINATION_INDOOR_SPAWN_DEVIATION: u16 = 4;
pub const DESTINATION_OUTDOOR_SPAWN_DEVIATION: u16 = 3;
pub const DESTINATION_DAYTIME_SPAWN_DEVIATION: u16 = 7;
pub const DESTINATION_DIVERSITY_LEVEL: u16 = 8;
pub const DESTINATION_OUTSIDE_DIVERSITY_LEVEL: u16 = 3;

const SIM_TICKS_PER_HOUR: u64 = 90_000;
const INDOOR_FIRST_DELAY_TICKS: u64 = SIM_TICKS_PER_HOUR;
const DAYTIME_FIRST_DELAY_TICKS: u64 = SIM_TICKS_PER_HOUR / 2;
const OUTDOOR_START_TICK: u64 = SIM_TICKS_PER_HOUR * 6;
const OUTDOOR_PEAK_TICK: u64 = SIM_TICKS_PER_HOUR * 16;
const OUTDOOR_SPIKE_START_TICK: u64 = SIM_TICKS_PER_HOUR * 13;
const OUTDOOR_SPIKE_END_TICK: u64 = SIM_TICKS_PER_HOUR * 15;
const ENTITY_SPAWNER_SALT: u64 = 0x61_0e_57_a9_42_13_24_00;

pub struct EntitySpawnerPlugin;

impl Plugin for EntitySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitySpawnerState>().add_systems(
            FixedUpdate,
            (
                entity_spawner_advance_timers,
                entity_spawner_indoor_rolls,
                entity_spawner_outdoor_rolls,
                entity_spawner_daytime_rolls,
                entity_spawner_checksum,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntitySpawnLocation {
    Indoor,
    Outdoor,
    Daytime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntitySpawnerWeather {
    Clear,
    Foggy,
    Stormy,
    Eclipsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnedEntityKind {
    Thumper,
    BackwaterGunkfish,
    BunkerSpider,
    Bracken,
    SnareFlea,
    HoardingBug,
    Hygrodere,
    CoilHead,
    SporeLizard,
    Maneater,
    Nutcrackers,
    Jester,
    ForestKeeper,
    EyelessDog,
    BaboonHawk,
    Feiopar,
    EarthLeviathan,
    Manticoil,
    CircuitBee,
    RoamingLocust,
    TulipSnake,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntitySpawnProfile {
    pub kind: SpawnedEntityKind,
    pub location: EntitySpawnLocation,
    pub base_spawn_chance_basis_points: u16,
    pub power_level_half_units: u16,
    pub max_spawned: u16,
    pub stunnable: bool,
    pub killable: bool,
}

const INDOOR_PROFILES: [EntitySpawnProfile; 12] = [
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Thumper,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 1818,
        power_level_half_units: 6,
        max_spawned: 4,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::BackwaterGunkfish,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 1791,
        power_level_half_units: 1,
        max_spawned: 6,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::BunkerSpider,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 1763,
        power_level_half_units: 4,
        max_spawned: 1,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Bracken,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 1295,
        power_level_half_units: 6,
        max_spawned: 1,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::SnareFlea,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 1047,
        power_level_half_units: 2,
        max_spawned: 4,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::HoardingBug,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 992,
        power_level_half_units: 2,
        max_spawned: 10,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Hygrodere,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 413,
        power_level_half_units: 2,
        max_spawned: 2,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::CoilHead,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 275,
        power_level_half_units: 2,
        max_spawned: 5,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::SporeLizard,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 248,
        power_level_half_units: 2,
        max_spawned: 2,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Maneater,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 248,
        power_level_half_units: 4,
        max_spawned: 1,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Nutcrackers,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 83,
        power_level_half_units: 2,
        max_spawned: 10,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Jester,
        location: EntitySpawnLocation::Indoor,
        base_spawn_chance_basis_points: 28,
        power_level_half_units: 6,
        max_spawned: 1,
        stunnable: true,
        killable: false,
    },
];

const OUTDOOR_PROFILES: [EntitySpawnProfile; 5] = [
    EntitySpawnProfile {
        kind: SpawnedEntityKind::ForestKeeper,
        location: EntitySpawnLocation::Outdoor,
        base_spawn_chance_basis_points: 3441,
        power_level_half_units: 6,
        max_spawned: 3,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::EyelessDog,
        location: EntitySpawnLocation::Outdoor,
        base_spawn_chance_basis_points: 2634,
        power_level_half_units: 4,
        max_spawned: 8,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::BaboonHawk,
        location: EntitySpawnLocation::Outdoor,
        base_spawn_chance_basis_points: 1882,
        power_level_half_units: 1,
        max_spawned: 15,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Feiopar,
        location: EntitySpawnLocation::Outdoor,
        base_spawn_chance_basis_points: 1505,
        power_level_half_units: 4,
        max_spawned: 6,
        stunnable: true,
        killable: true,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::EarthLeviathan,
        location: EntitySpawnLocation::Outdoor,
        base_spawn_chance_basis_points: 538,
        power_level_half_units: 4,
        max_spawned: 3,
        stunnable: false,
        killable: false,
    },
];

const DAYTIME_PROFILES: [EntitySpawnProfile; 4] = [
    EntitySpawnProfile {
        kind: SpawnedEntityKind::Manticoil,
        location: EntitySpawnLocation::Daytime,
        base_spawn_chance_basis_points: 4301,
        power_level_half_units: 2,
        max_spawned: 16,
        stunnable: true,
        killable: false,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::CircuitBee,
        location: EntitySpawnLocation::Daytime,
        base_spawn_chance_basis_points: 3627,
        power_level_half_units: 2,
        max_spawned: 6,
        stunnable: true,
        killable: false,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::RoamingLocust,
        location: EntitySpawnLocation::Daytime,
        base_spawn_chance_basis_points: 2021,
        power_level_half_units: 2,
        max_spawned: 5,
        stunnable: false,
        killable: false,
    },
    EntitySpawnProfile {
        kind: SpawnedEntityKind::TulipSnake,
        location: EntitySpawnLocation::Daytime,
        base_spawn_chance_basis_points: 52,
        power_level_half_units: 1,
        max_spawned: 12,
        stunnable: true,
        killable: true,
    },
];

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct EntitySpawnerState {
    pub destination_id: &'static str,
    pub weather: EntitySpawnerWeather,
    pub elapsed_ticks: u64,
    pub indoor_next_roll_tick: u64,
    pub outdoor_next_roll_tick: u64,
    pub daytime_next_roll_tick: u64,
    pub indoor_power_used_half_units: u16,
    pub outdoor_power_used_half_units: u16,
    pub daytime_power_used_half_units: u16,
    pub indoor_spawned: [u16; INDOOR_PROFILES.len()],
    pub outdoor_spawned: [u16; OUTDOOR_PROFILES.len()],
    pub daytime_spawned: [u16; DAYTIME_PROFILES.len()],
    pub pending_eclipsed_indoor_spawns: u8,
    pub pending_eclipsed_outdoor_spawns: u8,
    pub next_stable_id: u64,
}

impl Default for EntitySpawnerState {
    fn default() -> Self {
        Self {
            destination_id: DESTINATION_ID,
            weather: EntitySpawnerWeather::Foggy,
            elapsed_ticks: 0,
            indoor_next_roll_tick: INDOOR_FIRST_DELAY_TICKS,
            outdoor_next_roll_tick: OUTDOOR_START_TICK,
            daytime_next_roll_tick: DAYTIME_FIRST_DELAY_TICKS,
            indoor_power_used_half_units: 0,
            outdoor_power_used_half_units: 0,
            daytime_power_used_half_units: 0,
            indoor_spawned: [0; INDOOR_PROFILES.len()],
            outdoor_spawned: [0; OUTDOOR_PROFILES.len()],
            daytime_spawned: [0; DAYTIME_PROFILES.len()],
            pending_eclipsed_indoor_spawns: 0,
            pending_eclipsed_outdoor_spawns: 0,
            next_stable_id: 1,
        }
    }
}

impl EntitySpawnerState {
    pub fn set_weather(&mut self, weather: EntitySpawnerWeather) {
        self.weather = weather;
        if weather == EntitySpawnerWeather::Eclipsed
            && self.elapsed_ticks == 0
            && self.pending_eclipsed_indoor_spawns == 0
            && self.pending_eclipsed_outdoor_spawns == 0
        {
            self.pending_eclipsed_indoor_spawns = 2;
            self.pending_eclipsed_outdoor_spawns = 2;
        }
    }

    fn allocate_stable_id(&mut self) -> u64 {
        let stable_id = self.next_stable_id;
        self.next_stable_id = self.next_stable_id.wrapping_add(1);
        stable_id
    }
}

fn entity_spawner_advance_timers(mut state: ResMut<EntitySpawnerState>) {
    if state.elapsed_ticks == 0 && state.weather == EntitySpawnerWeather::Eclipsed {
        state.pending_eclipsed_indoor_spawns = 2;
        state.pending_eclipsed_outdoor_spawns = 2;
    }

    state.elapsed_ticks = state.elapsed_ticks.wrapping_add(1);
}

fn entity_spawner_indoor_rolls(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<EntitySpawnerState>,
    mut thumper: EventWriter<SpawnThumperEvent>,
    mut backwater_gunkfish: EventWriter<SpawnBackwaterGunkfishEvent>,
    mut bunker_spider: EventWriter<SpawnBunkerSpiderEvent>,
    mut bracken: EventWriter<SpawnBrackenEvent>,
    mut snare_flea: EventWriter<SpawnSnareFleaEvent>,
    mut hoarding_bug: EventWriter<SpawnHoardingBugEvent>,
    mut hygrodere: EventWriter<SpawnHygrodereEvent>,
    mut coil_head: EventWriter<SpawnCoilHeadEvent>,
    mut spore_lizard: EventWriter<SpawnSporeLizardEvent>,
    mut maneater: EventWriter<SpawnManeaterEvent>,
    mut nutcrackers: EventWriter<SpawnNutcrackersEvent>,
    mut jester: EventWriter<SpawnJesterEvent>,
) {
    let forced = state.pending_eclipsed_indoor_spawns > 0;
    if !forced && state.elapsed_ticks < state.indoor_next_roll_tick {
        return;
    }

    if forced {
        state.pending_eclipsed_indoor_spawns -= 1;
    } else {
        state.indoor_next_roll_tick = state
            .elapsed_ticks
            .wrapping_add(indoor_roll_interval_ticks(state.elapsed_ticks));
    }

    let mut rng = tick_rng(seed.0, tick.0, ENTITY_SPAWNER_SALT ^ 0x1000 ^ state.elapsed_ticks);
    let Some(index) = select_spawn_profile(
        &INDOOR_PROFILES,
        &state.indoor_spawned,
        state.indoor_power_used_half_units,
        DESTINATION_POWER_COUNT * 2,
        &mut rng,
    ) else {
        return;
    };

    let profile = INDOOR_PROFILES[index];
    state.indoor_spawned[index] = state.indoor_spawned[index].saturating_add(1);
    state.indoor_power_used_half_units = state
        .indoor_power_used_half_units
        .saturating_add(profile.power_level_half_units);

    let position = spawn_position(EntitySpawnLocation::Indoor, index as u64, &mut rng);
    let flank_a = offset_position(position, -16, 6);
    let flank_b = offset_position(position, 16, -6);
    let stable_id = state.allocate_stable_id();

    match profile.kind {
        SpawnedEntityKind::Thumper => {
            thumper.send(SpawnThumperEvent { position });
        }
        SpawnedEntityKind::BackwaterGunkfish => {
            backwater_gunkfish.send(SpawnBackwaterGunkfishEvent {
                position,
                initial_hiding_spot: flank_a,
                fallback_position: flank_b,
                facing: BackwaterGunkfishFacing::default(),
                search_move_speed: I32F32::lit("2.5"),
            });
        }
        SpawnedEntityKind::BunkerSpider => {
            bunker_spider.send(SpawnBunkerSpiderEvent {
                position,
                roam_area_a: flank_a,
                roam_area_b: flank_b,
            });
        }
        SpawnedEntityKind::Bracken => {
            bracken.send(SpawnBrackenEvent {
                position,
                preferred_room_position: flank_a,
                preferred_room_floor_index: (rng.next_u32() % 4) as i16,
            });
        }
        SpawnedEntityKind::SnareFlea => {
            snare_flea.send(SpawnSnareFleaEvent {
                ceiling_position: position,
            });
        }
        SpawnedEntityKind::HoardingBug => {
            hoarding_bug.send(SpawnHoardingBugEvent {
                position,
                nest_position: flank_a,
                initial_nest_scrap_count: (rng.next_u32() % 4) as u16,
            });
        }
        SpawnedEntityKind::Hygrodere => {
            hygrodere.send(SpawnHygrodereEvent { position });
        }
        SpawnedEntityKind::CoilHead => {
            coil_head.send(SpawnCoilHeadEvent {
                position,
                roam_area_a: flank_a,
                roam_area_b: flank_b,
            });
        }
        SpawnedEntityKind::SporeLizard => {
            spore_lizard.send(SpawnSporeLizardEvent {
                stable_id,
                position,
                room_id: 0x61_0000 ^ stable_id,
            });
        }
        SpawnedEntityKind::Maneater => {
            maneater.send(SpawnManeaterEvent {
                position,
                base_move_speed: I32F32::lit("4.25"),
                watch_range: I32F32::lit("18"),
                outdoors: false,
            });
        }
        SpawnedEntityKind::Nutcrackers => {
            nutcrackers.send(SpawnNutcrackersEvent { position });
        }
        SpawnedEntityKind::Jester => {
            jester.send(SpawnJesterEvent { position });
        }
        _ => {}
    }
}

fn entity_spawner_outdoor_rolls(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<EntitySpawnerState>,
    mut forest_keeper: EventWriter<SpawnForestKeeperEvent>,
    mut eyeless_dog: EventWriter<SpawnEyelessDogEvent>,
    mut baboon_hawk: EventWriter<SpawnBaboonHawkEvent>,
    mut feiopar: EventWriter<SpawnFeioparEvent>,
    mut earth_leviathan: EventWriter<SpawnEarthLeviathanEvent>,
) {
    let forced = state.pending_eclipsed_outdoor_spawns > 0;
    if !forced && state.elapsed_ticks < state.outdoor_next_roll_tick {
        return;
    }

    if forced {
        state.pending_eclipsed_outdoor_spawns -= 1;
    } else {
        state.outdoor_next_roll_tick = state
            .elapsed_ticks
            .wrapping_add(outdoor_roll_interval_ticks(state.elapsed_ticks, state.weather));
    }

    let mut rng = tick_rng(seed.0, tick.0, ENTITY_SPAWNER_SALT ^ 0x2000 ^ state.elapsed_ticks);
    let Some(index) = select_spawn_profile(
        &OUTDOOR_PROFILES,
        &state.outdoor_spawned,
        state.outdoor_power_used_half_units,
        DESTINATION_OUTSIDE_POWER_COUNT * 2,
        &mut rng,
    ) else {
        return;
    };

    let profile = OUTDOOR_PROFILES[index];
    state.outdoor_spawned[index] = state.outdoor_spawned[index].saturating_add(1);
    state.outdoor_power_used_half_units = state
        .outdoor_power_used_half_units
        .saturating_add(profile.power_level_half_units);

    let position = spawn_position(EntitySpawnLocation::Outdoor, index as u64, &mut rng);
    let stable_id = state.allocate_stable_id();

    match profile.kind {
        SpawnedEntityKind::ForestKeeper => {
            forest_keeper.send(SpawnForestKeeperEvent { stable_id, position });
        }
        SpawnedEntityKind::EyelessDog => {
            eyeless_dog.send(SpawnEyelessDogEvent { stable_id, position });
        }
        SpawnedEntityKind::BaboonHawk => {
            baboon_hawk.send(SpawnBaboonHawkEvent {
                stable_id,
                pack_id: stable_id / 4,
                position,
            });
        }
        SpawnedEntityKind::Feiopar => {
            feiopar.send(SpawnFeioparEvent { stable_id, position });
        }
        SpawnedEntityKind::EarthLeviathan => {
            earth_leviathan.send(SpawnEarthLeviathanEvent {
                stable_id,
                position,
                active_in_room: false,
                post_version_50: true,
            });
        }
        _ => {}
    }
}

fn entity_spawner_daytime_rolls(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<EntitySpawnerState>,
    mut manticoil: EventWriter<SpawnManticoilEvent>,
    mut circuit_bee: EventWriter<SpawnCircuitBeeEvent>,
    mut roaming_locust: EventWriter<SpawnRoamingLocustEvent>,
    mut tulip_snake: EventWriter<SpawnTulipSnakeEvent>,
) {
    if state.elapsed_ticks < state.daytime_next_roll_tick {
        return;
    }

    state.daytime_next_roll_tick = state
        .elapsed_ticks
        .wrapping_add(daytime_roll_interval_ticks(state.elapsed_ticks));

    let mut rng = tick_rng(seed.0, tick.0, ENTITY_SPAWNER_SALT ^ 0x3000 ^ state.elapsed_ticks);
    let Some(index) = select_spawn_profile(
        &DAYTIME_PROFILES,
        &state.daytime_spawned,
        state.daytime_power_used_half_units,
        DESTINATION_DAYTIME_POWER_COUNT * 2,
        &mut rng,
    ) else {
        return;
    };

    let profile = DAYTIME_PROFILES[index];
    state.daytime_spawned[index] = state.daytime_spawned[index].saturating_add(1);
    state.daytime_power_used_half_units = state
        .daytime_power_used_half_units
        .saturating_add(profile.power_level_half_units);

    let position = spawn_position(EntitySpawnLocation::Daytime, index as u64, &mut rng);
    let anchor = offset_position(position, 8, 8);
    let stable_id = state.allocate_stable_id();

    match profile.kind {
        SpawnedEntityKind::Manticoil => {
            manticoil.send(SpawnManticoilEvent { position });
        }
        SpawnedEntityKind::CircuitBee => {
            circuit_bee.send(SpawnCircuitBeeEvent {
                position,
                hive_position: anchor,
                guard_radius: I32F32::lit("18"),
                return_radius: I32F32::lit("32"),
                attack_range: I32F32::lit("2"),
                watch_range: I32F32::lit("24"),
                move_speed: I32F32::lit("6"),
            });
        }
        SpawnedEntityKind::RoamingLocust => {
            roaming_locust.send(SpawnRoamingLocustEvent {
                position,
                group_center: anchor,
                group_id: stable_id / 5,
                slot: (stable_id % 5) as u8,
            });
        }
        SpawnedEntityKind::TulipSnake => {
            tulip_snake.send(SpawnTulipSnakeEvent {
                position,
                roam_move_speed: I32F32::lit("4"),
                tire_after_attempts: 3,
            });
        }
        _ => {}
    }
}

fn select_spawn_profile<const N: usize>(
    profiles: &[EntitySpawnProfile; N],
    spawned: &[u16; N],
    power_used_half_units: u16,
    power_limit_half_units: u16,
    rng: &mut impl RngCore,
) -> Option<usize> {
    let mut total_weight = 0_u32;
    for index in 0..N {
        if spawned[index] >= profiles[index].max_spawned {
            continue;
        }

        if power_used_half_units.saturating_add(profiles[index].power_level_half_units)
            > power_limit_half_units
        {
            continue;
        }

        total_weight = total_weight.saturating_add(profiles[index].base_spawn_chance_basis_points as u32);
    }

    if total_weight == 0 {
        return None;
    }

    let mut roll = rng.next_u32() % total_weight;
    for index in 0..N {
        if spawned[index] >= profiles[index].max_spawned {
            continue;
        }

        if power_used_half_units.saturating_add(profiles[index].power_level_half_units)
            > power_limit_half_units
        {
            continue;
        }

        let weight = profiles[index].base_spawn_chance_basis_points as u32;
        if roll < weight {
            return Some(index);
        }

        roll -= weight;
    }

    None
}

fn indoor_roll_interval_ticks(elapsed_ticks: u64) -> u64 {
    let base = SIM_TICKS_PER_HOUR / 2;
    let pressure = (elapsed_ticks / SIM_TICKS_PER_HOUR).min(8);
    base.saturating_sub(pressure * (SIM_TICKS_PER_HOUR / 32))
        .saturating_add((DESTINATION_INDOOR_SPAWN_DEVIATION as u64) * 600)
}

fn outdoor_roll_interval_ticks(elapsed_ticks: u64, weather: EntitySpawnerWeather) -> u64 {
    let mut base = SIM_TICKS_PER_HOUR;
    if elapsed_ticks >= OUTDOOR_SPIKE_START_TICK && elapsed_ticks <= OUTDOOR_SPIKE_END_TICK {
        base /= 2;
    }

    if elapsed_ticks >= OUTDOOR_PEAK_TICK {
        base /= 3;
    }

    if weather == EntitySpawnerWeather::Eclipsed {
        base /= 2;
    }

    base.saturating_add((DESTINATION_OUTDOOR_SPAWN_DEVIATION as u64) * 600)
}

fn daytime_roll_interval_ticks(elapsed_ticks: u64) -> u64 {
    let early_delay = SIM_TICKS_PER_HOUR;
    let reduction = (elapsed_ticks / SIM_TICKS_PER_HOUR).min(6) * (SIM_TICKS_PER_HOUR / 12);
    early_delay
        .saturating_sub(reduction)
        .saturating_add((DESTINATION_DAYTIME_SPAWN_DEVIATION as u64) * 600)
}

fn spawn_position(
    location: EntitySpawnLocation,
    profile_index: u64,
    rng: &mut impl RngCore,
) -> SimPosition {
    let base_x = match location {
        EntitySpawnLocation::Indoor => -128,
        EntitySpawnLocation::Outdoor => 96,
        EntitySpawnLocation::Daytime => 160,
    };
    let base_y = match location {
        EntitySpawnLocation::Indoor => 64,
        EntitySpawnLocation::Outdoor => -96,
        EntitySpawnLocation::Daytime => 96,
    };

    let jitter_x = (rng.next_u32() % 96) as i32 - 48;
    let jitter_y = (rng.next_u32() % 96) as i32 - 48;
    let lane = (profile_index as i32) * 12;

    SimPosition {
        x: fixed_from_quarters(base_x + jitter_x + lane),
        y: fixed_from_quarters(base_y + jitter_y - lane),
    }
}

fn offset_position(position: SimPosition, x_quarters: i32, y_quarters: i32) -> SimPosition {
    SimPosition {
        x: position.x + fixed_from_quarters(x_quarters),
        y: position.y + fixed_from_quarters(y_quarters),
    }
}

fn fixed_from_quarters(value: i32) -> I32F32 {
    I32F32::from_bits((value as i64) << 30)
}

fn entity_spawner_checksum(
    tick: Res<SimTick>,
    state: Res<EntitySpawnerState>,
    mut checksum: ResMut<SimChecksumState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(stable_str_bits(ENTITY_SPAWNER_ID));
    checksum.accumulate(stable_str_bits(state.destination_id));
    checksum.accumulate(stable_str_bits(DESTINATION_NAME));
    checksum.accumulate(stable_str_bits(DESTINATION_RISK_LEVEL));
    checksum.accumulate(stable_str_bits(DESTINATION_DIFFICULTY));
    checksum.accumulate(stable_str_bits(DESTINATION_MAP_LAYOUT));
    checksum.accumulate(DESTINATION_COST as u64);
    checksum.accumulate(DESTINATION_MAP_LAYOUT_PERCENTAGE as u64);
    checksum.accumulate(DESTINATION_MAP_SIZE_MULTIPLIER_BASIS_POINTS as u64);
    checksum.accumulate(DESTINATION_MIN_SCRAP as u64);
    checksum.accumulate(DESTINATION_MAX_SCRAP as u64);
    checksum.accumulate(DESTINATION_POWER_COUNT as u64);
    checksum.accumulate(DESTINATION_OUTSIDE_POWER_COUNT as u64);
    checksum.accumulate(DESTINATION_DAYTIME_POWER_COUNT as u64);
    checksum.accumulate(DESTINATION_INDOOR_SPAWN_DEVIATION as u64);
    checksum.accumulate(DESTINATION_OUTDOOR_SPAWN_DEVIATION as u64);
    checksum.accumulate(DESTINATION_DAYTIME_SPAWN_DEVIATION as u64);
    checksum.accumulate(DESTINATION_DIVERSITY_LEVEL as u64);
    checksum.accumulate(DESTINATION_OUTSIDE_DIVERSITY_LEVEL as u64);
    checksum.accumulate(weather_code(state.weather));
    checksum.accumulate(state.elapsed_ticks);
    checksum.accumulate(state.indoor_next_roll_tick);
    checksum.accumulate(state.outdoor_next_roll_tick);
    checksum.accumulate(state.daytime_next_roll_tick);
    checksum.accumulate(state.indoor_power_used_half_units as u64);
    checksum.accumulate(state.outdoor_power_used_half_units as u64);
    checksum.accumulate(state.daytime_power_used_half_units as u64);
    checksum.accumulate(state.pending_eclipsed_indoor_spawns as u64);
    checksum.accumulate(state.pending_eclipsed_outdoor_spawns as u64);
    checksum.accumulate(state.next_stable_id);

    for index in 0..INDOOR_PROFILES.len() {
        accumulate_profile(&mut checksum, INDOOR_PROFILES[index]);
        checksum.accumulate(0x5100 ^ index as u64 ^ state.indoor_spawned[index] as u64);
    }

    for index in 0..OUTDOOR_PROFILES.len() {
        accumulate_profile(&mut checksum, OUTDOOR_PROFILES[index]);
        checksum.accumulate(0x5200 ^ index as u64 ^ state.outdoor_spawned[index] as u64);
    }

    for index in 0..DAYTIME_PROFILES.len() {
        accumulate_profile(&mut checksum, DAYTIME_PROFILES[index]);
        checksum.accumulate(0x5300 ^ index as u64 ^ state.daytime_spawned[index] as u64);
    }
}

fn accumulate_profile(checksum: &mut SimChecksumState, profile: EntitySpawnProfile) {
    checksum.accumulate(kind_code(profile.kind));
    checksum.accumulate(location_code(profile.location));
    checksum.accumulate(profile.base_spawn_chance_basis_points as u64);
    checksum.accumulate(profile.power_level_half_units as u64);
    checksum.accumulate(profile.max_spawned as u64);
    checksum.accumulate(profile.stunnable as u64);
    checksum.accumulate(profile.killable as u64);
}

fn weather_code(weather: EntitySpawnerWeather) -> u64 {
    match weather {
        EntitySpawnerWeather::Clear => 1,
        EntitySpawnerWeather::Foggy => 2,
        EntitySpawnerWeather::Stormy => 3,
        EntitySpawnerWeather::Eclipsed => 4,
    }
}

fn location_code(location: EntitySpawnLocation) -> u64 {
    match location {
        EntitySpawnLocation::Indoor => 1,
        EntitySpawnLocation::Outdoor => 2,
        EntitySpawnLocation::Daytime => 3,
    }
}

fn kind_code(kind: SpawnedEntityKind) -> u64 {
    match kind {
        SpawnedEntityKind::Thumper => 1,
        SpawnedEntityKind::BackwaterGunkfish => 2,
        SpawnedEntityKind::BunkerSpider => 3,
        SpawnedEntityKind::Bracken => 4,
        SpawnedEntityKind::SnareFlea => 5,
        SpawnedEntityKind::HoardingBug => 6,
        SpawnedEntityKind::Hygrodere => 7,
        SpawnedEntityKind::CoilHead => 8,
        SpawnedEntityKind::SporeLizard => 9,
        SpawnedEntityKind::Maneater => 10,
        SpawnedEntityKind::Nutcrackers => 11,
        SpawnedEntityKind::Jester => 12,
        SpawnedEntityKind::ForestKeeper => 13,
        SpawnedEntityKind::EyelessDog => 14,
        SpawnedEntityKind::BaboonHawk => 15,
        SpawnedEntityKind::Feiopar => 16,
        SpawnedEntityKind::EarthLeviathan => 17,
        SpawnedEntityKind::Manticoil => 18,
        SpawnedEntityKind::CircuitBee => 19,
        SpawnedEntityKind::RoamingLocust => 20,
        SpawnedEntityKind::TulipSnake => 21,
    }
}

fn stable_str_bits(value: &str) -> u64 {
    let mut out = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        out ^= *byte as u64;
        out = out.wrapping_mul(0x0000_0100_0000_01b3);
    }
    out
}