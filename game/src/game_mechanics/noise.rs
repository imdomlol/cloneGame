// Sources: vault/game_mechanics/noise.md, vault/units/soldier.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimHz, SimPosition, SimTick};

const ZERO: I32F32 = I32F32::ZERO;
const ONE: I32F32 = I32F32::lit("1");
const TWO: I32F32 = I32F32::lit("2");
const THREE: I32F32 = I32F32::lit("3");
const FOUR: I32F32 = I32F32::lit("4");
const FIVE: I32F32 = I32F32::lit("5");
const EIGHT: I32F32 = I32F32::lit("8");
const TEN: I32F32 = I32F32::lit("10");
const TWENTY: I32F32 = I32F32::lit("20");
const THIRTY_TWO: I32F32 = I32F32::lit("32");
const FIFTY: I32F32 = I32F32::lit("50");
const FIVE_HUNDRED: I32F32 = I32F32::lit("500");
const ONE_THOUSAND: I32F32 = I32F32::lit("1000");
const MAP_MODIFIER_MAP3: I32F32 = I32F32::lit("0.5");
const MAP_MODIFIER_MAP4: I32F32 = I32F32::lit("1.5");
const ADJACENT_FALLOFF_DIVISOR: I32F32 = I32F32::lit("2");
const LARGE_BREAK_NOISE: I32F32 = ONE_THOUSAND;
const ENTITY_SALT_NOISE_REACTION: u64 = 0x4E4F_4953_455F_5245; // "NOISE_RE"

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseMapKind {
    Map3,
    Map4,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfectedNoiseClass {
    SlowInfected,
    Runner,
    Fatty,
    Venom,
    Harpy,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseSourceKind {
    RangerAttack,
    SoldierAttack,
    GreatBallistaShot,
    SniperAttack,
    TitanAttack,
    ThanatosAttack,
    InfectedStructureAttack,
    InfectedBuildingPopulation,
    InfectedStructureBreak,
    Other,
}

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseSensitive;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedEntityId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseListenerProfile {
    pub class: Option<InfectedNoiseClass>,
    pub alertness: I32F32,
    pub watch_range: I32F32,
    pub map_modifier: I32F32,
}

impl NoiseListenerProfile {
    pub fn slow_infected() -> Self {
        Self {
            class: Some(InfectedNoiseClass::SlowInfected),
            alertness: TWO,
            watch_range: TWENTY,
            map_modifier: ONE,
        }
    }

    pub fn runner() -> Self {
        Self {
            class: Some(InfectedNoiseClass::Runner),
            alertness: THREE,
            watch_range: TWENTY,
            map_modifier: ONE,
        }
    }

    pub fn fatty() -> Self {
        Self {
            class: Some(InfectedNoiseClass::Fatty),
            alertness: THREE,
            watch_range: TWENTY,
            map_modifier: ONE,
        }
    }

    pub fn venom() -> Self {
        Self {
            class: Some(InfectedNoiseClass::Venom),
            alertness: FOUR,
            watch_range: TWENTY,
            map_modifier: ONE,
        }
    }

    pub fn harpy() -> Self {
        Self {
            class: Some(InfectedNoiseClass::Harpy),
            alertness: EIGHT,
            watch_range: THIRTY_TWO,
            map_modifier: ONE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingPopulationNoise {
    pub infected: bool,
    pub worker_or_population_count: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedAttackingStructure;

#[derive(Event, Clone, Copy)]
pub struct NoiseReactionEvent {
    pub entity: Entity,
    pub source_tile_x: i32,
    pub source_tile_y: i32,
    pub activity: I32F32,
    pub threshold_used: I32F32,
    pub roll: u32,
}

#[derive(Event, Clone, Copy)]
pub struct NoiseSourceIntentEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub source_kind: NoiseSourceKind,
}

#[derive(Event, Clone, Copy)]
pub struct StructureBrokenNoiseEvent {
    pub source: Entity,
    pub position: SimPosition,
}

#[derive(Resource, Clone, Copy)]
pub struct NoiseMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for NoiseMechanicData {
    fn default() -> Self {
        Self {
            id: "noise",
            name: "Noise",
            mechanic_type: "noise",
            how_it_works: "Noise is tracked as per-tile activity; sources add activity to their tile and reduced activity to adjacent tiles, and infected evaluate it probabilistically with alertness-dependent thresholds.",
            techniques: "Clustered fire saturates a local area, while kiting spreads the same fire over more tiles with lower activity per tile.",
            applications: "Lure infected into kill zones, steer investigations, and manage or trigger cascades around colony structures.",
            depends_on: &[
                "infected",
                "slow_infected",
                "runners",
                "special_infected",
                "ranger",
                "soldier",
                "great_ballista",
                "sniper",
                "titan",
                "thanatos",
                "tesla_tower",
                "farm",
                "stone_house",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct NoiseMapSettings {
    pub map_kind: NoiseMapKind,
}

impl Default for NoiseMapSettings {
    fn default() -> Self {
        Self {
            map_kind: NoiseMapKind::Other,
        }
    }
}

#[derive(Resource, Default)]
pub struct NoiseTileActivityState {
    pub activity_by_tile: BTreeMap<(i32, i32), I32F32>,
    pub ticks_since_decay: u32,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct NoiseMetrics {
    pub total_activity_added: I32F32,
    pub total_activity_decay_removed: I32F32,
    pub total_reactions: u64,
    pub total_building_population_noise: I32F32,
    pub total_structure_break_noise: I32F32,
    pub total_cascade_noise: I32F32,
}

fn map_modifier_for_kind(kind: NoiseMapKind) -> I32F32 {
    match kind {
        NoiseMapKind::Map3 => MAP_MODIFIER_MAP3,
        NoiseMapKind::Map4 => MAP_MODIFIER_MAP4,
        NoiseMapKind::Other => ONE,
    }
}

fn tile_from_position(position: SimPosition) -> (i32, i32) {
    (position.x.to_num::<i32>(), position.y.to_num::<i32>())
}

fn add_activity_to_tile(
    tiles: &mut BTreeMap<(i32, i32), I32F32>,
    x: i32,
    y: i32,
    amount: I32F32,
) -> I32F32 {
    if amount <= ZERO {
        return ZERO;
    }
    let entry = tiles.entry((x, y)).or_insert(ZERO);
    *entry = *entry + amount;
    amount
}

fn apply_noise_source_intents_system(
    mut intents: EventReader<NoiseSourceIntentEvent>,
    mut writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in intents.read() {
        let amount = match ev.source_kind {
            NoiseSourceKind::RangerAttack => ONE,
            NoiseSourceKind::SoldierAttack => THREE,
            NoiseSourceKind::GreatBallistaShot => FIVE,
            NoiseSourceKind::SniperAttack => TEN,
            NoiseSourceKind::TitanAttack => TWENTY,
            NoiseSourceKind::ThanatosAttack => FIVE_HUNDRED,
            NoiseSourceKind::InfectedStructureAttack => FIFTY,
            NoiseSourceKind::InfectedBuildingPopulation => FIFTY,
            NoiseSourceKind::InfectedStructureBreak => LARGE_BREAK_NOISE,
            NoiseSourceKind::Other => ZERO,
        };

        if amount <= ZERO {
            continue;
        }

        writer.send(NoiseEmittedEvent {
            source: ev.source,
            position: ev.position,
            amount,
        });
    }
}

fn emit_infected_building_population_noise_system(
    mut writer: EventWriter<NoiseEmittedEvent>,
    buildings: Query<(Entity, &SimPosition, &BuildingPopulationNoise)>,
) {
    for (entity, position, profile) in &buildings {
        if !profile.infected {
            continue;
        }

        let amount = FIFTY * I32F32::from_num(profile.worker_or_population_count);
        if amount <= ZERO {
            continue;
        }

        writer.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount,
        });
    }
}

fn emit_infected_structure_attack_noise_system(
    mut writer: EventWriter<NoiseEmittedEvent>,
    attackers: Query<(Entity, &SimPosition), With<InfectedAttackingStructure>>,
) {
    for (entity, position) in &attackers {
        writer.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: FIFTY,
        });
    }
}

fn emit_structure_break_noise_system(
    mut break_events: EventReader<StructureBrokenNoiseEvent>,
    mut writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in break_events.read() {
        writer.send(NoiseEmittedEvent {
            source: ev.source,
            position: ev.position,
            amount: LARGE_BREAK_NOISE,
        });
    }
}

fn apply_noise_emissions_system(
    mut metrics: ResMut<NoiseMetrics>,
    mut state: ResMut<NoiseTileActivityState>,
    mut emissions: EventReader<NoiseEmittedEvent>,
) {
    for ev in emissions.read() {
        let (tx, ty) = tile_from_position(ev.position);

        let added_main = add_activity_to_tile(&mut state.activity_by_tile, tx, ty, ev.amount);
        metrics.total_activity_added = metrics.total_activity_added + added_main;

        let neighbor_amount = ev.amount / ADJACENT_FALLOFF_DIVISOR;
        for ny in (ty - 1)..=(ty + 1) {
            for nx in (tx - 1)..=(tx + 1) {
                if nx == tx && ny == ty {
                    continue;
                }
                let added_neighbor =
                    add_activity_to_tile(&mut state.activity_by_tile, nx, ny, neighbor_amount);
                metrics.total_activity_added = metrics.total_activity_added + added_neighbor;
            }
        }

        if ev.amount == LARGE_BREAK_NOISE {
            metrics.total_structure_break_noise = metrics.total_structure_break_noise + ev.amount;
        }
        if ev.amount == FIFTY {
            metrics.total_cascade_noise = metrics.total_cascade_noise + ev.amount;
            metrics.total_building_population_noise =
                metrics.total_building_population_noise + ev.amount;
        }
    }
}

fn decay_noise_activity_system(
    sim_hz: Res<SimHz>,
    mut metrics: ResMut<NoiseMetrics>,
    mut state: ResMut<NoiseTileActivityState>,
) {
    let mut ticks_per_second = sim_hz.0.to_num::<u32>();
    if ticks_per_second == 0 {
        ticks_per_second = 1;
    }

    state.ticks_since_decay = state.ticks_since_decay.saturating_add(1);
    if state.ticks_since_decay < ticks_per_second {
        return;
    }
    state.ticks_since_decay = 0;

    let mut to_remove: Vec<(i32, i32)> = Vec::new();
    for (tile, value) in &mut state.activity_by_tile {
        let before = *value;
        let after = before / TWO;
        let removed = before - after;
        *value = after;
        metrics.total_activity_decay_removed = metrics.total_activity_decay_removed + removed;

        if after <= ZERO {
            to_remove.push(*tile);
        }
    }

    for tile in to_remove {
        state.activity_by_tile.remove(&tile);
    }
}

fn evaluate_noise_for_infected_system(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    map_settings: Res<NoiseMapSettings>,
    mut metrics: ResMut<NoiseMetrics>,
    state: Res<NoiseTileActivityState>,
    mut reactions: EventWriter<NoiseReactionEvent>,
    listeners: Query<
        (Entity, &ReplicatedEntityId, &SimPosition, &NoiseListenerProfile),
        With<NoiseSensitive>,
    >,
) {
    let map_modifier = map_modifier_for_kind(map_settings.map_kind);

    for (entity, replicated_id, listener_pos, listener_profile) in &listeners {
        let watch_range = listener_profile.watch_range;
        let listener_map_modifier = if listener_profile.map_modifier > ZERO {
            listener_profile.map_modifier
        } else {
            map_modifier
        };

        let search_radius = FOUR * watch_range * listener_map_modifier;
        if search_radius <= ZERO {
            continue;
        }
        let radius_sq = search_radius * search_radius;

        for ((tx, ty), activity) in &state.activity_by_tile {
            if *activity <= ZERO {
                continue;
            }

            let dx = I32F32::from_num(*tx) - listener_pos.x;
            let dy = I32F32::from_num(*ty) - listener_pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq > radius_sq {
                continue;
            }

            let alertness = if listener_profile.alertness > ZERO {
                listener_profile.alertness
            } else {
                ONE
            };

            let base_threshold = ONE_THOUSAND / alertness;

            let proximity = if radius_sq > dist_sq {
                radius_sq - dist_sq
            } else {
                ZERO
            };
            let proximity_reduction = if radius_sq > ZERO {
                (proximity * base_threshold) / radius_sq
            } else {
                ZERO
            };

            let mut effective_threshold = base_threshold - proximity_reduction;
            if effective_threshold < ONE {
                effective_threshold = ONE;
            }

            let chance = if *activity >= effective_threshold {
                1000_u32
            } else {
                ((*activity * ONE_THOUSAND) / effective_threshold).to_num::<u32>()
            };

            let salt = ENTITY_SALT_NOISE_REACTION ^ replicated_id.0;
            let mut rng = tick_rng(game_seed.0, sim_tick.0, salt);
            let roll = rng.next_u32() % 1000;

            if roll < chance {
                reactions.send(NoiseReactionEvent {
                    entity,
                    source_tile_x: *tx,
                    source_tile_y: *ty,
                    activity: *activity,
                    threshold_used: effective_threshold,
                    roll,
                });
                metrics.total_reactions = metrics.total_reactions.saturating_add(1);
            }
        }
    }
}

fn noise_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    state: Res<NoiseTileActivityState>,
    metrics: Res<NoiseMetrics>,
    map_settings: Res<NoiseMapSettings>,
    listeners: Query<(&ReplicatedEntityId, &NoiseListenerProfile), With<NoiseSensitive>>,
    buildings: Query<&BuildingPopulationNoise>,
) {
    checksum.accumulate(ONE.to_bits() as u64);
    checksum.accumulate(THREE.to_bits() as u64);
    checksum.accumulate(FIVE.to_bits() as u64);
    checksum.accumulate(TEN.to_bits() as u64);
    checksum.accumulate(TWENTY.to_bits() as u64);
    checksum.accumulate(FIVE_HUNDRED.to_bits() as u64);
    checksum.accumulate(FIFTY.to_bits() as u64);
    checksum.accumulate(LARGE_BREAK_NOISE.to_bits() as u64);
    checksum.accumulate(TWENTY.to_bits() as u64);
    checksum.accumulate(THIRTY_TWO.to_bits() as u64);
    checksum.accumulate(MAP_MODIFIER_MAP3.to_bits() as u64);
    checksum.accumulate(MAP_MODIFIER_MAP4.to_bits() as u64);

    let map_kind_bits = match map_settings.map_kind {
        NoiseMapKind::Map3 => 3_u64,
        NoiseMapKind::Map4 => 4_u64,
        NoiseMapKind::Other => 0_u64,
    };
    checksum.accumulate(map_kind_bits);

    checksum.accumulate(metrics.total_activity_added.to_bits() as u64);
    checksum.accumulate(metrics.total_activity_decay_removed.to_bits() as u64);
    checksum.accumulate(metrics.total_reactions);
    checksum.accumulate(metrics.total_building_population_noise.to_bits() as u64);
    checksum.accumulate(metrics.total_structure_break_noise.to_bits() as u64);
    checksum.accumulate(metrics.total_cascade_noise.to_bits() as u64);
    checksum.accumulate(state.ticks_since_decay as u64);

    for ((x, y), v) in &state.activity_by_tile {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
        checksum.accumulate(v.to_bits() as u64);
    }

    for (id, profile) in &listeners {
        checksum.accumulate(id.0);

        let class_bits = match profile.class {
            Some(InfectedNoiseClass::SlowInfected) => 1_u64,
            Some(InfectedNoiseClass::Runner) => 2_u64,
            Some(InfectedNoiseClass::Fatty) => 3_u64,
            Some(InfectedNoiseClass::Venom) => 4_u64,
            Some(InfectedNoiseClass::Harpy) => 5_u64,
            Some(InfectedNoiseClass::Custom) => 6_u64,
            None => 0_u64,
        };
        checksum.accumulate(class_bits);

        checksum.accumulate(profile.alertness.to_bits() as u64);
        checksum.accumulate(profile.watch_range.to_bits() as u64);
        checksum.accumulate(profile.map_modifier.to_bits() as u64);
    }

    for b in &buildings {
        checksum.accumulate(u64::from(b.infected));
        checksum.accumulate(b.worker_or_population_count as u64);
    }
}

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoiseMechanicData>()
            .init_resource::<NoiseMapSettings>()
            .init_resource::<NoiseTileActivityState>()
            .init_resource::<NoiseMetrics>()
            .add_event::<NoiseSourceIntentEvent>()
            .add_event::<StructureBrokenNoiseEvent>()
            .add_event::<NoiseReactionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_noise_source_intents_system,
                    emit_infected_building_population_noise_system,
                    emit_infected_structure_attack_noise_system,
                    emit_structure_break_noise_system,
                    apply_noise_emissions_system,
                    decay_noise_activity_system,
                    evaluate_noise_for_infected_system,
                    noise_checksum_system,
                )
                    .chain(),
            );
    }
}