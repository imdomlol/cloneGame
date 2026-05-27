// Sources: vault/mayors/mayors.md

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

const LEVEL_I_COLONISTS: I32F32 = I32F32::lit("30");
const LEVEL_II_COLONISTS: I32F32 = I32F32::lit("200");
const LEVEL_III_COLONISTS: I32F32 = I32F32::lit("600");
const LEVEL_IV_COLONISTS: I32F32 = I32F32::lit("1200");

const MAYOR_ROLL_SALT: u64 = 0x4D41_594F_525F_524F;
const RESEARCH_UNLOCK_SALT: u64 = 0x4D41_594F_525F_5445;
const FREE_REWARD_SALT: u64 = 0x4D41_594F_525F_4652;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayorLevel {
    LevelI,
    LevelII,
    LevelIII,
    LevelIV,
}

impl MayorLevel {
    fn as_bits(self) -> u64 {
        match self {
            MayorLevel::LevelI => 1,
            MayorLevel::LevelII => 2,
            MayorLevel::LevelIII => 3,
            MayorLevel::LevelIV => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayorBonusKind {
    Building,
    Unit,
    Buff,
    Resource,
    Research,
    Yield,
}

impl MayorBonusKind {
    fn as_bits(self) -> u64 {
        match self {
            MayorBonusKind::Building => 1,
            MayorBonusKind::Unit => 2,
            MayorBonusKind::Buff => 3,
            MayorBonusKind::Resource => 4,
            MayorBonusKind::Research => 5,
            MayorBonusKind::Yield => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockpileResourceKind {
    Gold,
    Wood,
    Stone,
    Iron,
    Oil,
}

impl StockpileResourceKind {
    fn as_bits(self) -> u64 {
        match self {
            StockpileResourceKind::Gold => 1,
            StockpileResourceKind::Wood => 2,
            StockpileResourceKind::Stone => 3,
            StockpileResourceKind::Iron => 4,
            StockpileResourceKind::Oil => 5,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MayorCandidateDefinition {
    pub id: &'static str,
    pub level: MayorLevel,
    pub bonus_kind: MayorBonusKind,
    pub bonus_target: &'static str,
    pub bonus_value: I32F32,
    pub immediate_resource_kind: Option<StockpileResourceKind>,
    pub immediate_resource_amount: I32F32,
    pub grants_research: bool,
    pub free_reward_available: bool,
    pub command_center_yield_bonus: I32F32,
}

const MAYOR_POOL: [MayorCandidateDefinition; 16] = [
    MayorCandidateDefinition {
        id: "level_i_farm_output",
        level: MayorLevel::LevelI,
        bonus_kind: MayorBonusKind::Building,
        bonus_target: "farm",
        bonus_value: I32F32::lit("0.20"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_i_ranger_recruitment",
        level: MayorLevel::LevelI,
        bonus_kind: MayorBonusKind::Unit,
        bonus_target: "ranger",
        bonus_value: I32F32::lit("-0.15"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_i_reserve_wood",
        level: MayorLevel::LevelI,
        bonus_kind: MayorBonusKind::Resource,
        bonus_target: "wood",
        bonus_value: I32F32::lit("100"),
        immediate_resource_kind: Some(StockpileResourceKind::Wood),
        immediate_resource_amount: I32F32::lit("100"),
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_i_command_center_yield",
        level: MayorLevel::LevelI,
        bonus_kind: MayorBonusKind::Yield,
        bonus_target: "command_center",
        bonus_value: I32F32::lit("0.05"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::lit("0.05"),
    },
    MayorCandidateDefinition {
        id: "level_ii_quarry_output",
        level: MayorLevel::LevelII,
        bonus_kind: MayorBonusKind::Building,
        bonus_target: "quarry",
        bonus_value: I32F32::lit("0.20"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_ii_soldier_training",
        level: MayorLevel::LevelII,
        bonus_kind: MayorBonusKind::Unit,
        bonus_target: "soldier",
        bonus_value: I32F32::lit("-0.15"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_ii_reserve_stone",
        level: MayorLevel::LevelII,
        bonus_kind: MayorBonusKind::Resource,
        bonus_target: "stone",
        bonus_value: I32F32::lit("80"),
        immediate_resource_kind: Some(StockpileResourceKind::Stone),
        immediate_resource_amount: I32F32::lit("80"),
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_ii_unlock_research",
        level: MayorLevel::LevelII,
        bonus_kind: MayorBonusKind::Research,
        bonus_target: "technology",
        bonus_value: I32F32::lit("1"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: true,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iii_power_plant_output",
        level: MayorLevel::LevelIII,
        bonus_kind: MayorBonusKind::Building,
        bonus_target: "power_plant",
        bonus_value: I32F32::lit("0.20"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iii_sniper_training",
        level: MayorLevel::LevelIII,
        bonus_kind: MayorBonusKind::Unit,
        bonus_target: "sniper",
        bonus_value: I32F32::lit("-0.15"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iii_reserve_iron",
        level: MayorLevel::LevelIII,
        bonus_kind: MayorBonusKind::Resource,
        bonus_target: "iron",
        bonus_value: I32F32::lit("60"),
        immediate_resource_kind: Some(StockpileResourceKind::Iron),
        immediate_resource_amount: I32F32::lit("60"),
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iii_free_reward",
        level: MayorLevel::LevelIII,
        bonus_kind: MayorBonusKind::Buff,
        bonus_target: "command_center_bonus_interface",
        bonus_value: I32F32::lit("1"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: true,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iv_foundry_output",
        level: MayorLevel::LevelIV,
        bonus_kind: MayorBonusKind::Building,
        bonus_target: "foundry",
        bonus_value: I32F32::lit("0.20"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iv_titan_deployment",
        level: MayorLevel::LevelIV,
        bonus_kind: MayorBonusKind::Unit,
        bonus_target: "titan",
        bonus_value: I32F32::lit("-0.10"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iv_reserve_gold",
        level: MayorLevel::LevelIV,
        bonus_kind: MayorBonusKind::Resource,
        bonus_target: "gold",
        bonus_value: I32F32::lit("1000"),
        immediate_resource_kind: Some(StockpileResourceKind::Gold),
        immediate_resource_amount: I32F32::lit("1000"),
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::ZERO,
    },
    MayorCandidateDefinition {
        id: "level_iv_command_center_yield",
        level: MayorLevel::LevelIV,
        bonus_kind: MayorBonusKind::Yield,
        bonus_target: "command_center",
        bonus_value: I32F32::lit("0.10"),
        immediate_resource_kind: None,
        immediate_resource_amount: I32F32::ZERO,
        grants_research: false,
        free_reward_available: false,
        command_center_yield_bonus: I32F32::lit("0.10"),
    },
];

#[derive(Resource, Clone, Copy)]
pub struct MayorsMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub subtype: &'static str,
    pub selection_level: &'static str,
    pub bonus_type: &'static str,
    pub bonus_target: &'static str,
    pub bonus_value: &'static str,
    pub mayor_type_category: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for MayorsMechanicData {
    fn default() -> Self {
        Self {
            id: "mayors",
            name: "Mayors",
            subtype: "mayor_system",
            selection_level: "Level I (30 colonists), Level II (200 colonists), Level III (600 colonists), Level IV (1200 colonists)",
            bonus_type: "building, unit, buff, resource, research, yield",
            bonus_target: "structures, units, resources, command center stats, technologies",
            bonus_value: "varies by mayor",
            mayor_type_category: "colony election system",
            depends_on: &[],
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct PendingElectionChoices {
    pub level: Option<MayorLevel>,
    pub option_a: Option<MayorCandidateDefinition>,
    pub option_b: Option<MayorCandidateDefinition>,
}

#[derive(Clone, Copy, Default)]
pub struct ColonyThresholdsReached {
    pub level_i: bool,
    pub level_ii: bool,
    pub level_iii: bool,
    pub level_iv: bool,
}

#[derive(Clone, Copy, Default)]
pub struct StockpileReserves {
    pub gold: I32F32,
    pub wood: I32F32,
    pub stone: I32F32,
    pub iron: I32F32,
    pub oil: I32F32,
}

#[derive(Clone, Copy, Default)]
pub struct CommandCenterYieldBonuses {
    pub flat_bonus_total: I32F32,
    pub election_bonus_count: u64,
}

#[derive(Resource, Default)]
pub struct MayorState {
    pub colony_colonists: I32F32,
    pub is_survival_mode: bool,
    pub thresholds_reached: ColonyThresholdsReached,
    pub pending_queue: Vec<MayorLevel>,
    pub pending_choices: PendingElectionChoices,
    pub chosen_mayor_ids: Vec<u64>,
    pub active_persistent_bonuses: Vec<ActivePersistentBonus>,
    pub reserves: StockpileReserves,
    pub command_center_yield: CommandCenterYieldBonuses,
    pub research_unlocks_granted: u64,
    pub free_rewards_granted: u64,
    pub elections_offered: u64,
    pub elections_resolved: u64,
}

#[derive(Clone, Copy, Default)]
pub struct ActivePersistentBonus {
    pub mayor_id_hash: u64,
    pub level: u64,
    pub bonus_kind: u64,
    pub target_hash: u64,
    pub value_bits: u64,
}

#[derive(Event, Clone, Copy)]
pub struct ColonyPopulationUpdatedEvent {
    pub colonists: I32F32,
    pub is_survival_mode: bool,
}

#[derive(Event, Clone, Copy)]
pub struct MayorElectionOfferedEvent {
    pub level: MayorLevel,
    pub option_a_id_hash: u64,
    pub option_b_id_hash: u64,
}

#[derive(Event, Clone, Copy)]
pub struct MayorSelectedEvent {
    pub selected_mayor_id_hash: u64,
}

#[derive(Event, Clone, Copy)]
pub struct MayorResearchUnlockedEvent {
    pub technology_id_hash: u64,
}

#[derive(Event, Clone, Copy)]
pub struct MayorFreeRewardAvailableEvent {
    pub reward_id_hash: u64,
}

fn stable_hash_str(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

fn pool_indices_for_level(level: MayorLevel, out: &mut [usize; 4]) {
    let mut write_idx = 0usize;
    let mut i = 0usize;
    while i < MAYOR_POOL.len() {
        if MAYOR_POOL[i].level == level {
            out[write_idx] = i;
            write_idx += 1;
            if write_idx == out.len() {
                break;
            }
        }
        i += 1;
    }
}

fn roll_two_candidates(level: MayorLevel, game_seed: u64, tick: u64, offer_count: u64) -> (MayorCandidateDefinition, MayorCandidateDefinition) {
    let mut pool = [0usize; 4];
    pool_indices_for_level(level, &mut pool);

    let level_salt = MAYOR_ROLL_SALT ^ level.as_bits() ^ offer_count;
    let mut rng = tick_rng(game_seed, tick, level_salt);

    let first_pick = (rng.next_u32() as usize) % pool.len();
    let mut second_pick = (rng.next_u32() as usize) % pool.len();
    while second_pick == first_pick {
        second_pick = (rng.next_u32() as usize) % pool.len();
    }

    (MAYOR_POOL[pool[first_pick]], MAYOR_POOL[pool[second_pick]])
}

fn update_colony_population_state_system(
    mut state: ResMut<MayorState>,
    mut population_events: EventReader<ColonyPopulationUpdatedEvent>,
) {
    for ev in population_events.read() {
        state.colony_colonists = ev.colonists;
        state.is_survival_mode = ev.is_survival_mode;
    }
}

fn enqueue_mayor_elections_on_thresholds_system(mut state: ResMut<MayorState>) {
    if !state.is_survival_mode {
        return;
    }

    if !state.thresholds_reached.level_i && state.colony_colonists >= LEVEL_I_COLONISTS {
        state.thresholds_reached.level_i = true;
        state.pending_queue.push(MayorLevel::LevelI);
    }
    if !state.thresholds_reached.level_ii && state.colony_colonists >= LEVEL_II_COLONISTS {
        state.thresholds_reached.level_ii = true;
        state.pending_queue.push(MayorLevel::LevelII);
    }
    if !state.thresholds_reached.level_iii && state.colony_colonists >= LEVEL_III_COLONISTS {
        state.thresholds_reached.level_iii = true;
        state.pending_queue.push(MayorLevel::LevelIII);
    }
    if !state.thresholds_reached.level_iv && state.colony_colonists >= LEVEL_IV_COLONISTS {
        state.thresholds_reached.level_iv = true;
        state.pending_queue.push(MayorLevel::LevelIV);
    }
}

fn offer_mayor_election_system(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<MayorState>,
    mut offered_writer: EventWriter<MayorElectionOfferedEvent>,
) {
    if state.pending_choices.level.is_some() {
        return;
    }

    if state.pending_queue.is_empty() {
        return;
    }

    let level = state.pending_queue.remove(0);
    let (option_a, option_b) = roll_two_candidates(level, game_seed.0, tick.0, state.elections_offered);

    state.pending_choices.level = Some(level);
    state.pending_choices.option_a = Some(option_a);
    state.pending_choices.option_b = Some(option_b);
    state.elections_offered = state.elections_offered.saturating_add(1);

    offered_writer.send(MayorElectionOfferedEvent {
        level,
        option_a_id_hash: stable_hash_str(option_a.id),
        option_b_id_hash: stable_hash_str(option_b.id),
    });
}

fn apply_mayor_choice_effects_system(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<MayorState>,
    mut selected_events: EventReader<MayorSelectedEvent>,
    mut research_writer: EventWriter<MayorResearchUnlockedEvent>,
    mut reward_writer: EventWriter<MayorFreeRewardAvailableEvent>,
) {
    for ev in selected_events.read() {
        let Some(level) = state.pending_choices.level else {
            continue;
        };
        let Some(option_a) = state.pending_choices.option_a else {
            continue;
        };
        let Some(option_b) = state.pending_choices.option_b else {
            continue;
        };

        let selected = if stable_hash_str(option_a.id) == ev.selected_mayor_id_hash {
            option_a
        } else if stable_hash_str(option_b.id) == ev.selected_mayor_id_hash {
            option_b
        } else {
            continue;
        };

        state.elections_resolved = state.elections_resolved.saturating_add(1);
        state.chosen_mayor_ids.push(stable_hash_str(selected.id));

        if let Some(resource_kind) = selected.immediate_resource_kind {
            if selected.immediate_resource_amount > I32F32::ZERO {
                match resource_kind {
                    StockpileResourceKind::Gold => {
                        state.reserves.gold = state.reserves.gold + selected.immediate_resource_amount;
                    }
                    StockpileResourceKind::Wood => {
                        state.reserves.wood = state.reserves.wood + selected.immediate_resource_amount;
                    }
                    StockpileResourceKind::Stone => {
                        state.reserves.stone = state.reserves.stone + selected.immediate_resource_amount;
                    }
                    StockpileResourceKind::Iron => {
                        state.reserves.iron = state.reserves.iron + selected.immediate_resource_amount;
                    }
                    StockpileResourceKind::Oil => {
                        state.reserves.oil = state.reserves.oil + selected.immediate_resource_amount;
                    }
                }
            }
        }

        if selected.command_center_yield_bonus != I32F32::ZERO {
            state.command_center_yield.flat_bonus_total =
                state.command_center_yield.flat_bonus_total + selected.command_center_yield_bonus;
            state.command_center_yield.election_bonus_count =
                state.command_center_yield.election_bonus_count.saturating_add(1);
        }

        if selected.grants_research {
            let tech_hash = stable_hash_str(selected.id) ^ RESEARCH_UNLOCK_SALT ^ tick.0 ^ game_seed.0;
            research_writer.send(MayorResearchUnlockedEvent {
                technology_id_hash: tech_hash,
            });
            state.research_unlocks_granted = state.research_unlocks_granted.saturating_add(1);
        }

        if selected.free_reward_available {
            let reward_hash = stable_hash_str(selected.id) ^ FREE_REWARD_SALT ^ tick.0 ^ game_seed.0;
            reward_writer.send(MayorFreeRewardAvailableEvent {
                reward_id_hash: reward_hash,
            });
            state.free_rewards_granted = state.free_rewards_granted.saturating_add(1);
        }

        state.active_persistent_bonuses.push(ActivePersistentBonus {
            mayor_id_hash: stable_hash_str(selected.id),
            level: level.as_bits(),
            bonus_kind: selected.bonus_kind.as_bits(),
            target_hash: stable_hash_str(selected.bonus_target),
            value_bits: selected.bonus_value.to_bits() as u64,
        });

        state.pending_choices.level = None;
        state.pending_choices.option_a = None;
        state.pending_choices.option_b = None;
    }
}

fn mayors_checksum_system(mut checksum: ResMut<SimChecksumState>, state: Res<MayorState>) {
    checksum.accumulate(LEVEL_I_COLONISTS.to_bits() as u64);
    checksum.accumulate(LEVEL_II_COLONISTS.to_bits() as u64);
    checksum.accumulate(LEVEL_III_COLONISTS.to_bits() as u64);
    checksum.accumulate(LEVEL_IV_COLONISTS.to_bits() as u64);

    checksum.accumulate(state.colony_colonists.to_bits() as u64);
    checksum.accumulate(u64::from(state.is_survival_mode));
    checksum.accumulate(u64::from(state.thresholds_reached.level_i));
    checksum.accumulate(u64::from(state.thresholds_reached.level_ii));
    checksum.accumulate(u64::from(state.thresholds_reached.level_iii));
    checksum.accumulate(u64::from(state.thresholds_reached.level_iv));

    checksum.accumulate(state.pending_queue.len() as u64);
    for level in &state.pending_queue {
        checksum.accumulate(level.as_bits());
    }

    let pending_level_bits = match state.pending_choices.level {
        Some(level) => level.as_bits(),
        None => 0,
    };
    checksum.accumulate(pending_level_bits);

    let option_a_bits = match state.pending_choices.option_a {
        Some(def) => stable_hash_str(def.id),
        None => 0,
    };
    checksum.accumulate(option_a_bits);

    let option_b_bits = match state.pending_choices.option_b {
        Some(def) => stable_hash_str(def.id),
        None => 0,
    };
    checksum.accumulate(option_b_bits);

    checksum.accumulate(state.reserves.gold.to_bits() as u64);
    checksum.accumulate(state.reserves.wood.to_bits() as u64);
    checksum.accumulate(state.reserves.stone.to_bits() as u64);
    checksum.accumulate(state.reserves.iron.to_bits() as u64);
    checksum.accumulate(state.reserves.oil.to_bits() as u64);

    checksum.accumulate(state.command_center_yield.flat_bonus_total.to_bits() as u64);
    checksum.accumulate(state.command_center_yield.election_bonus_count);

    checksum.accumulate(state.research_unlocks_granted);
    checksum.accumulate(state.free_rewards_granted);
    checksum.accumulate(state.elections_offered);
    checksum.accumulate(state.elections_resolved);

    checksum.accumulate(state.chosen_mayor_ids.len() as u64);
    for id in &state.chosen_mayor_ids {
        checksum.accumulate(*id);
    }

    checksum.accumulate(state.active_persistent_bonuses.len() as u64);
    for bonus in &state.active_persistent_bonuses {
        checksum.accumulate(bonus.mayor_id_hash);
        checksum.accumulate(bonus.level);
        checksum.accumulate(bonus.bonus_kind);
        checksum.accumulate(bonus.target_hash);
        checksum.accumulate(bonus.value_bits);
    }

    for def in MAYOR_POOL {
        checksum.accumulate(stable_hash_str(def.id));
        checksum.accumulate(def.level.as_bits());
        checksum.accumulate(def.bonus_kind.as_bits());
        checksum.accumulate(stable_hash_str(def.bonus_target));
        checksum.accumulate(def.bonus_value.to_bits() as u64);
        let resource_bits = match def.immediate_resource_kind {
            Some(kind) => kind.as_bits(),
            None => 0,
        };
        checksum.accumulate(resource_bits);
        checksum.accumulate(def.immediate_resource_amount.to_bits() as u64);
        checksum.accumulate(u64::from(def.grants_research));
        checksum.accumulate(u64::from(def.free_reward_available));
        checksum.accumulate(def.command_center_yield_bonus.to_bits() as u64);
    }
}

pub struct MayorsPlugin;

impl Plugin for MayorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MayorsMechanicData>()
            .init_resource::<MayorState>()
            .add_event::<ColonyPopulationUpdatedEvent>()
            .add_event::<MayorElectionOfferedEvent>()
            .add_event::<MayorSelectedEvent>()
            .add_event::<MayorResearchUnlockedEvent>()
            .add_event::<MayorFreeRewardAvailableEvent>()
            .add_systems(
                FixedUpdate,
                (
                    update_colony_population_state_system,
                    enqueue_mayor_elections_on_thresholds_system,
                    offer_mayor_election_system,
                    apply_mayor_choice_effects_system,
                    mayors_checksum_system,
                )
                    .chain(),
            );
    }
}