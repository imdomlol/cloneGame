// Sources: vault/game_mechanics/villages_of_doom.md

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimHz, SimTick};

const ZERO: I32F32 = I32F32::ZERO;
const HALF: I32F32 = I32F32::lit("0.5");
const EIGHT_TENTHS: I32F32 = I32F32::lit("0.8");
const ONE: I32F32 = I32F32::lit("1");
const THREE: I32F32 = I32F32::lit("3");

const INFECTION_FACTOR_0: I32F32 = I32F32::lit("0.38");
const INFECTION_FACTOR_1: I32F32 = I32F32::lit("0.4");
const INFECTION_FACTOR_2: I32F32 = I32F32::lit("0.41");
const INFECTION_FACTOR_3: I32F32 = I32F32::lit("0.42");

const SIZE_FACTOR_SMALL: I32F32 = I32F32::lit("1");
const SIZE_FACTOR_MEDIUM: I32F32 = I32F32::lit("2");
const SIZE_FACTOR_LARGE: I32F32 = I32F32::lit("5");

const MAX_GENERATION_SMALL: u32 = 40;
const MAX_GENERATION_MEDIUM: u32 = 150;
const MAX_GENERATION_LARGE: u32 = 500;

const HOURS_BETWEEN_DRY_UP_SPAWNS: u32 = 12;
const DAYS_FOR_LONG_WAVE_PHASE: u32 = 20;
const DAYS_BETWEEN_LONG_WAVES: u32 = 5;
const HOURS_FOR_LAST_HOUR_PHASE: u32 = 12;
const HOURS_BETWEEN_LAST_HOUR_WAVES: u32 = 12;
const FRAMES_TO_HALF_DISTURBANCE: u32 = 120;

const ENTITY_SALT_VILLAGE_SPAWN: u64 = 0x5649_4C4C_4147_455F; // "VILLAGE_"
const ENTITY_SALT_LARGE_POOL: u64 = 0x4C41_5247_455F_504F; // "LARGE_PO"

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VillageBuildingSize {
    #[default]
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VillageSpawnKind {
    InfectedAged,
    InfectedDecrepit,
    InfectedYoung,
    InfectedFresh,
    InfectedColonist,
    InfectedExecutive,
    InfectedChubby,
}

#[derive(Component, Clone, Copy, Default)]
pub struct VillageOfDoom {
    pub size: VillageBuildingSize,
    pub infection_factor_index: u8,
    pub dried_up: bool,
    pub destroyed: bool,
    pub nearby_infected_insufficient: bool,
    pub under_attack: bool,
    pub final_wave_active: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct VillageSpawnRuntime {
    pub disturbance_level: I32F32,
    pub elapsed_frames: u32,
    pub elapsed_hours: u32,
    pub elapsed_days: u32,
    pub ticks_accum_hour: u64,
    pub ticks_accum_day: u64,
    pub generated_total: u32,
}

#[derive(Event, Clone, Copy)]
pub struct VillageSpawnResolvedEvent {
    pub entity: Entity,
    pub spawn_count: u32,
    pub disturbance_level: I32F32,
    pub infection_factor: I32F32,
    pub size_factor: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct VillageInfectedSpawnedEvent {
    pub entity: Entity,
    pub kind: VillageSpawnKind,
}

#[derive(Event, Clone, Copy)]
pub struct VillageDestroyedEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct VillageResourceCacheDroppedEvent {
    pub entity: Entity,
    pub stashes: u32,
}

#[derive(Resource, Clone, Copy)]
pub struct VillagesOfDoomMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for VillagesOfDoomMechanicData {
    fn default() -> Self {
        Self {
            id: "villages_of_doom",
            name: "Villages of Doom",
            mechanic_type: "enemy_nest",
            how_it_works: "Villages of Doom are map-generated infected nests that spawn infected when disturbed by noise or damage, then drop resource caches when destroyed.",
            techniques: "Use low-noise defenses, kite spawns away from the village, and prefer Great Ballistas or Lucifer for clearing.",
            applications: "Early clearing for resource caches and controlled frontier expansion.",
            depends_on: &[
                "infected_aged",
                "infected_chubby",
                "infected_colonist",
                "infected_decrepit",
                "infected_executive",
                "infected_fresh",
                "infected_harpy",
                "infected_venom",
                "infected_young",
                "lucifer",
                "noise",
                "sniper",
                "titan",
                "great_ballista",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct VillagesOfDoomRules {
    pub size_factor_small: I32F32,
    pub max_generation_small: u32,
    pub size_factor_medium: I32F32,
    pub max_generation_medium: u32,
    pub size_factor_large: I32F32,
    pub max_generation_large: u32,
    pub infection_factors: [I32F32; 4],
}

impl Default for VillagesOfDoomRules {
    fn default() -> Self {
        Self {
            size_factor_small: SIZE_FACTOR_SMALL,
            max_generation_small: MAX_GENERATION_SMALL,
            size_factor_medium: SIZE_FACTOR_MEDIUM,
            max_generation_medium: MAX_GENERATION_MEDIUM,
            size_factor_large: SIZE_FACTOR_LARGE,
            max_generation_large: MAX_GENERATION_LARGE,
            infection_factors: [
                INFECTION_FACTOR_0,
                INFECTION_FACTOR_1,
                INFECTION_FACTOR_2,
                INFECTION_FACTOR_3,
            ],
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct VillagesOfDoomMetrics {
    pub total_spawns_triggered: u64,
    pub total_infected_spawned: u64,
    pub total_spawn_rolls: u64,
    pub total_resource_caches_dropped: u64,
}

fn village_size_factor(rules: &VillagesOfDoomRules, size: VillageBuildingSize) -> I32F32 {
    match size {
        VillageBuildingSize::Small => rules.size_factor_small,
        VillageBuildingSize::Medium => rules.size_factor_medium,
        VillageBuildingSize::Large => rules.size_factor_large,
    }
}

fn village_max_generation(rules: &VillagesOfDoomRules, size: VillageBuildingSize) -> u32 {
    match size {
        VillageBuildingSize::Small => rules.max_generation_small,
        VillageBuildingSize::Medium => rules.max_generation_medium,
        VillageBuildingSize::Large => rules.max_generation_large,
    }
}

fn village_cache_stashes(size: VillageBuildingSize) -> u32 {
    match size {
        VillageBuildingSize::Small => 2,
        VillageBuildingSize::Medium => 4,
        VillageBuildingSize::Large => 6,
    }
}

fn infection_factor_from_index(rules: &VillagesOfDoomRules, index: u8) -> I32F32 {
    match index {
        0 => rules.infection_factors[0],
        1 => rules.infection_factors[1],
        2 => rules.infection_factors[2],
        3 => rules.infection_factors[3],
        _ => rules.infection_factors[0],
    }
}

fn ceil_fixed_to_u32(value: I32F32) -> u32 {
    if value <= ZERO {
        return 0;
    }
    let int_part = value.to_num::<u32>();
    if value > I32F32::from_num(int_part) {
        int_part.saturating_add(1)
    } else {
        int_part
    }
}

fn resolve_disturbance_level(
    village: &VillageOfDoom,
    runtime: &VillageSpawnRuntime,
    just_advanced_hour: bool,
    just_advanced_day: bool,
) -> I32F32 {
    let mut disturbance_level = ZERO;

    if village.under_attack {
        disturbance_level = THREE;
    }

    if village.final_wave_active {
        if disturbance_level < EIGHT_TENTHS {
            disturbance_level = EIGHT_TENTHS;
        }
    }

    if runtime.elapsed_days > DAYS_FOR_LONG_WAVE_PHASE
        && just_advanced_day
        && runtime.elapsed_days % DAYS_BETWEEN_LONG_WAVES == 0
    {
        if disturbance_level < ONE {
            disturbance_level = ONE;
        }
    }

    if runtime.elapsed_hours > HOURS_FOR_LAST_HOUR_PHASE
        && just_advanced_hour
        && runtime.elapsed_hours % HOURS_BETWEEN_LAST_HOUR_WAVES == 0
    {
        if disturbance_level < ONE {
            disturbance_level = ONE;
        }
    }

    if runtime.elapsed_frames > 0 && runtime.elapsed_frames % FRAMES_TO_HALF_DISTURBANCE == 0 {
        if disturbance_level < HALF {
            disturbance_level = HALF;
        }
    }

    disturbance_level
}

fn select_spawn_kind_for_village(
    size: VillageBuildingSize,
    rng_u32: u32,
    roll_index: u32,
) -> VillageSpawnKind {
    match size {
        VillageBuildingSize::Small => match (rng_u32.wrapping_add(roll_index)) % 3 {
            0 => VillageSpawnKind::InfectedAged,
            1 => VillageSpawnKind::InfectedDecrepit,
            _ => VillageSpawnKind::InfectedYoung,
        },
        VillageBuildingSize::Medium => {
            if (rng_u32.wrapping_add(roll_index)) % 2 == 0 {
                VillageSpawnKind::InfectedFresh
            } else {
                VillageSpawnKind::InfectedColonist
            }
        }
        VillageBuildingSize::Large => {
            let r = (rng_u32.wrapping_add(roll_index)) % 100;
            if r < 39 {
                VillageSpawnKind::InfectedFresh
            } else if r < 78 {
                VillageSpawnKind::InfectedColonist
            } else if r < 97 {
                VillageSpawnKind::InfectedExecutive
            } else {
                VillageSpawnKind::InfectedChubby
            }
        }
    }
}

fn villages_of_doom_spawn_system(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    rules: Res<VillagesOfDoomRules>,
    mut metrics: ResMut<VillagesOfDoomMetrics>,
    mut spawn_resolved_writer: EventWriter<VillageSpawnResolvedEvent>,
    mut spawned_writer: EventWriter<VillageInfectedSpawnedEvent>,
    mut villages: Query<(Entity, &VillageOfDoom, &mut VillageSpawnRuntime)>,
) {
    let mut ticks_per_hour = (sim_hz.0 * I32F32::lit("3600")).to_num::<u64>();
    if ticks_per_hour == 0 {
        ticks_per_hour = 1;
    }
    let ticks_per_day = ticks_per_hour.saturating_mul(24);

    for (entity, village, mut runtime) in &mut villages {
        if village.destroyed || village.dried_up {
            continue;
        }

        runtime.elapsed_frames = runtime.elapsed_frames.saturating_add(1);
        runtime.ticks_accum_hour = runtime.ticks_accum_hour.saturating_add(1);
        runtime.ticks_accum_day = runtime.ticks_accum_day.saturating_add(1);

        let mut just_advanced_hour = false;
        let mut just_advanced_day = false;

        if runtime.ticks_accum_hour >= ticks_per_hour {
            runtime.ticks_accum_hour = 0;
            runtime.elapsed_hours = runtime.elapsed_hours.saturating_add(1);
            just_advanced_hour = true;
        }

        if runtime.ticks_accum_day >= ticks_per_day {
            runtime.ticks_accum_day = 0;
            runtime.elapsed_days = runtime.elapsed_days.saturating_add(1);
            just_advanced_day = true;
        }

        runtime.disturbance_level =
            resolve_disturbance_level(village, &runtime, just_advanced_hour, just_advanced_day);

        let mut should_spawn = false;

        if !village.dried_up
            && village.nearby_infected_insufficient
            && just_advanced_hour
            && runtime.elapsed_hours % HOURS_BETWEEN_DRY_UP_SPAWNS == 0
        {
            should_spawn = true;
        }

        if runtime.elapsed_days > DAYS_FOR_LONG_WAVE_PHASE
            && just_advanced_day
            && runtime.elapsed_days % DAYS_BETWEEN_LONG_WAVES == 0
        {
            should_spawn = true;
        }

        if !should_spawn || runtime.disturbance_level <= ZERO {
            continue;
        }

        let size_factor = village_size_factor(&rules, village.size);
        let infection_factor = infection_factor_from_index(&rules, village.infection_factor_index);

        let spawn_salt = ENTITY_SALT_VILLAGE_SPAWN ^ (entity.index() as u64);
        let mut rng = tick_rng(game_seed.0, sim_tick.0, spawn_salt);

        let rand_thousand = rng.next_u32() % 1001;
        let rand_factor = HALF + (I32F32::from_num(rand_thousand) / I32F32::lit("1000"));

        let spawn_fp = rand_factor * size_factor * infection_factor * runtime.disturbance_level;
        let mut spawn_count = ceil_fixed_to_u32(spawn_fp);

        let max_generation = village_max_generation(&rules, village.size);
        if runtime.generated_total >= max_generation {
            spawn_count = 0;
        } else {
            let remaining = max_generation - runtime.generated_total;
            if spawn_count > remaining {
                spawn_count = remaining;
            }
        }

        if spawn_count == 0 {
            continue;
        }

        metrics.total_spawns_triggered = metrics.total_spawns_triggered.saturating_add(1);
        metrics.total_spawn_rolls = metrics.total_spawn_rolls.saturating_add(1);
        metrics.total_infected_spawned = metrics
            .total_infected_spawned
            .saturating_add(spawn_count as u64);

        runtime.generated_total = runtime.generated_total.saturating_add(spawn_count);
        if runtime.generated_total >= max_generation {
            runtime.generated_total = max_generation;
        }

        spawn_resolved_writer.send(VillageSpawnResolvedEvent {
            entity,
            spawn_count,
            disturbance_level: runtime.disturbance_level,
            infection_factor,
            size_factor,
        });

        let pool_salt = ENTITY_SALT_LARGE_POOL ^ (entity.index() as u64);
        let mut pool_rng = tick_rng(game_seed.0, sim_tick.0, pool_salt);

        for i in 0..spawn_count {
            let draw = pool_rng.next_u32();
            let kind = select_spawn_kind_for_village(village.size, draw, i);
            spawned_writer.send(VillageInfectedSpawnedEvent { entity, kind });
        }
    }
}

fn villages_of_doom_destroyed_system(
    mut metrics: ResMut<VillagesOfDoomMetrics>,
    mut destroyed_events: EventReader<VillageDestroyedEvent>,
    villages: Query<&VillageOfDoom>,
    mut cache_writer: EventWriter<VillageResourceCacheDroppedEvent>,
) {
    for ev in destroyed_events.read() {
        let Ok(village) = villages.get(ev.entity) else {
            continue;
        };

        let stashes = village_cache_stashes(village.size);
        cache_writer.send(VillageResourceCacheDroppedEvent {
            entity: ev.entity,
            stashes,
        });
        metrics.total_resource_caches_dropped = metrics
            .total_resource_caches_dropped
            .saturating_add(stashes as u64);
    }
}

fn villages_of_doom_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    data: Res<VillagesOfDoomMechanicData>,
    rules: Res<VillagesOfDoomRules>,
    metrics: Res<VillagesOfDoomMetrics>,
    villages: Query<(&VillageOfDoom, &VillageSpawnRuntime)>,
) {
    checksum.accumulate(data.id.len() as u64);
    checksum.accumulate(data.name.len() as u64);
    checksum.accumulate(data.mechanic_type.len() as u64);
    checksum.accumulate(data.how_it_works.len() as u64);
    checksum.accumulate(data.techniques.len() as u64);
    checksum.accumulate(data.applications.len() as u64);
    checksum.accumulate(data.depends_on.len() as u64);

    checksum.accumulate(rules.size_factor_small.to_bits() as u64);
    checksum.accumulate(rules.size_factor_medium.to_bits() as u64);
    checksum.accumulate(rules.size_factor_large.to_bits() as u64);
    checksum.accumulate(rules.max_generation_small as u64);
    checksum.accumulate(rules.max_generation_medium as u64);
    checksum.accumulate(rules.max_generation_large as u64);
    checksum.accumulate(rules.infection_factors[0].to_bits() as u64);
    checksum.accumulate(rules.infection_factors[1].to_bits() as u64);
    checksum.accumulate(rules.infection_factors[2].to_bits() as u64);
    checksum.accumulate(rules.infection_factors[3].to_bits() as u64);

    checksum.accumulate(metrics.total_spawns_triggered);
    checksum.accumulate(metrics.total_infected_spawned);
    checksum.accumulate(metrics.total_spawn_rolls);
    checksum.accumulate(metrics.total_resource_caches_dropped);

    for (village, runtime) in &villages {
        let size_bits = match village.size {
            VillageBuildingSize::Small => 1_u64,
            VillageBuildingSize::Medium => 2_u64,
            VillageBuildingSize::Large => 3_u64,
        };
        checksum.accumulate(size_bits);
        checksum.accumulate(village.infection_factor_index as u64);
        checksum.accumulate(u64::from(village.dried_up));
        checksum.accumulate(u64::from(village.destroyed));
        checksum.accumulate(u64::from(village.nearby_infected_insufficient));
        checksum.accumulate(u64::from(village.under_attack));
        checksum.accumulate(u64::from(village.final_wave_active));

        checksum.accumulate(runtime.disturbance_level.to_bits() as u64);
        checksum.accumulate(runtime.elapsed_frames as u64);
        checksum.accumulate(runtime.elapsed_hours as u64);
        checksum.accumulate(runtime.elapsed_days as u64);
        checksum.accumulate(runtime.ticks_accum_hour);
        checksum.accumulate(runtime.ticks_accum_day);
        checksum.accumulate(runtime.generated_total as u64);
    }
}

pub struct VillagesOfDoomPlugin;

impl Plugin for VillagesOfDoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VillagesOfDoomMechanicData>()
            .init_resource::<VillagesOfDoomRules>()
            .init_resource::<VillagesOfDoomMetrics>()
            .add_event::<VillageSpawnResolvedEvent>()
            .add_event::<VillageInfectedSpawnedEvent>()
            .add_event::<VillageDestroyedEvent>()
            .add_event::<VillageResourceCacheDroppedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    villages_of_doom_spawn_system,
                    villages_of_doom_destroyed_system,
                    villages_of_doom_checksum_system,
                )
                    .chain(),
            );
    }
}