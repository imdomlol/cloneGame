// Sources: vault/research/technology_tree.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const TECHNOLOGY_TREE_TOTAL_COST: I32F32 = I32F32::lit("14220");
const TECHNOLOGY_TREE_STARTING_POINTS: I32F32 = I32F32::lit("50");
const TECHNOLOGY_TREE_MAX_POINTS: I32F32 = I32F32::lit("9950");
const GODDESS_OF_DESTINY_BONUS_POINTS: I32F32 = I32F32::lit("1000");

#[derive(Resource, Clone, Copy)]
pub struct TechnologyTreeMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub research_tier: &'static str,
    pub unlocks: &'static str,
    pub provides_bonus: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for TechnologyTreeMechanicData {
    fn default() -> Self {
        Self {
            id: "technology_tree",
            name: "Technology Tree",
            research_tier: "campaign",
            unlocks: "Campaign technology options and colony bonuses.",
            provides_bonus: "Unlocks technologies and bonuses for colony progression using research points.",
            depends_on: &["the_new_empire", "research", "the_goddess_of_destiny"],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct TechnologyTreeState {
    pub available_points: I32F32,
    pub spent_points: I32F32,
    pub earned_points_from_missions_and_loot: I32F32,
    pub recently_researched_option_id: u64,
    pub recently_researched_locked: bool,
    pub fully_completed: bool,
}

impl Default for TechnologyTreeState {
    fn default() -> Self {
        Self {
            available_points: TECHNOLOGY_TREE_STARTING_POINTS,
            spent_points: I32F32::ZERO,
            earned_points_from_missions_and_loot: I32F32::ZERO,
            recently_researched_option_id: 0,
            recently_researched_locked: false,
            fully_completed: false,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct CampaignStartedEvent;

#[derive(Event, Clone, Copy)]
pub struct ColonyMissionCompletedEvent {
    pub mission_completed: bool,
    pub mission_failed: bool,
    pub mission_replayed: bool,
    pub reward_points: I32F32,
    pub is_goddess_of_destiny: bool,
}

#[derive(Event, Clone, Copy)]
pub struct ScoutingLootCollectedEvent {
    pub reward_points: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct TechnologyResearchedEvent {
    pub option_id: u64,
    pub cost_points: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct TechnologyReselectedEvent {
    pub option_id: u64,
}

fn campaign_start_points_system(
    mut state: ResMut<TechnologyTreeState>,
    mut started_events: EventReader<CampaignStartedEvent>,
) {
    for _ in started_events.read() {
        state.available_points = TECHNOLOGY_TREE_STARTING_POINTS;
        state.spent_points = I32F32::ZERO;
        state.earned_points_from_missions_and_loot = I32F32::ZERO;
        state.recently_researched_option_id = 0;
        state.recently_researched_locked = false;
        state.fully_completed = false;
    }
}

fn spend_research_points_system(
    mut state: ResMut<TechnologyTreeState>,
    mut researched_events: EventReader<TechnologyResearchedEvent>,
) {
    for ev in researched_events.read() {
        if state.fully_completed || state.recently_researched_locked {
            continue;
        }
        if ev.cost_points <= I32F32::ZERO || state.available_points < ev.cost_points {
            continue;
        }

        state.available_points = state.available_points - ev.cost_points;
        state.spent_points = state.spent_points + ev.cost_points;
        state.recently_researched_option_id = ev.option_id;
    }
}

fn lock_recently_researched_on_completed_mission_system(
    mut state: ResMut<TechnologyTreeState>,
    mut mission_events: EventReader<ColonyMissionCompletedEvent>,
) {
    for ev in mission_events.read() {
        if ev.mission_completed && !ev.mission_failed && !ev.mission_replayed && state.recently_researched_option_id != 0
        {
            state.recently_researched_locked = true;
        }
    }
}

fn keep_research_available_on_failed_mission_system(
    mut state: ResMut<TechnologyTreeState>,
    mut mission_events: EventReader<ColonyMissionCompletedEvent>,
) {
    for ev in mission_events.read() {
        if ev.mission_failed {
            state.recently_researched_locked = false;
        }
    }
}

fn keep_research_available_on_replayed_mission_system(
    mut state: ResMut<TechnologyTreeState>,
    mut mission_events: EventReader<ColonyMissionCompletedEvent>,
) {
    for ev in mission_events.read() {
        if ev.mission_replayed {
            state.recently_researched_locked = false;
        }
    }
}

fn unlock_recently_researched_after_reselection_system(
    mut state: ResMut<TechnologyTreeState>,
    mut reselected_events: EventReader<TechnologyReselectedEvent>,
) {
    for ev in reselected_events.read() {
        if state.recently_researched_locked && ev.option_id == state.recently_researched_option_id {
            state.recently_researched_locked = false;
        }
    }
}

fn award_points_from_missions_and_loot_system(
    mut state: ResMut<TechnologyTreeState>,
    mut mission_events: EventReader<ColonyMissionCompletedEvent>,
    mut loot_events: EventReader<ScoutingLootCollectedEvent>,
) {
    for ev in mission_events.read() {
        if ev.mission_completed && ev.reward_points > I32F32::ZERO {
            state.available_points = state.available_points + ev.reward_points;
            state.earned_points_from_missions_and_loot =
                state.earned_points_from_missions_and_loot + ev.reward_points;
        }
    }

    for ev in loot_events.read() {
        if ev.reward_points > I32F32::ZERO {
            state.available_points = state.available_points + ev.reward_points;
            state.earned_points_from_missions_and_loot =
                state.earned_points_from_missions_and_loot + ev.reward_points;
        }
    }

    if state.earned_points_from_missions_and_loot > TECHNOLOGY_TREE_MAX_POINTS {
        let overflow = state.earned_points_from_missions_and_loot - TECHNOLOGY_TREE_MAX_POINTS;
        state.earned_points_from_missions_and_loot = TECHNOLOGY_TREE_MAX_POINTS;
        if state.available_points > overflow {
            state.available_points = state.available_points - overflow;
        } else {
            state.available_points = I32F32::ZERO;
        }
    }
}

fn award_goddess_of_destiny_bonus_system(
    mut state: ResMut<TechnologyTreeState>,
    mut mission_events: EventReader<ColonyMissionCompletedEvent>,
) {
    for ev in mission_events.read() {
        if ev.mission_completed && ev.is_goddess_of_destiny {
            state.available_points = state.available_points + GODDESS_OF_DESTINY_BONUS_POINTS;
        }
    }
}

fn complete_tree_at_total_cost_system(mut state: ResMut<TechnologyTreeState>) {
    if state.spent_points >= TECHNOLOGY_TREE_TOTAL_COST {
        state.spent_points = TECHNOLOGY_TREE_TOTAL_COST;
        state.fully_completed = true;
        state.recently_researched_locked = false;
    }
}

fn technology_tree_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    state: Res<TechnologyTreeState>,
) {
    checksum.accumulate(TECHNOLOGY_TREE_TOTAL_COST.to_bits() as u64);
    checksum.accumulate(TECHNOLOGY_TREE_STARTING_POINTS.to_bits() as u64);
    checksum.accumulate(TECHNOLOGY_TREE_MAX_POINTS.to_bits() as u64);
    checksum.accumulate(GODDESS_OF_DESTINY_BONUS_POINTS.to_bits() as u64);

    checksum.accumulate(state.available_points.to_bits() as u64);
    checksum.accumulate(state.spent_points.to_bits() as u64);
    checksum.accumulate(state.earned_points_from_missions_and_loot.to_bits() as u64);
    checksum.accumulate(state.recently_researched_option_id);
    checksum.accumulate(u64::from(state.recently_researched_locked));
    checksum.accumulate(u64::from(state.fully_completed));
}

pub struct TechnologyTreePlugin;

impl Plugin for TechnologyTreePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TechnologyTreeMechanicData>()
            .init_resource::<TechnologyTreeState>()
            .add_event::<CampaignStartedEvent>()
            .add_event::<ColonyMissionCompletedEvent>()
            .add_event::<ScoutingLootCollectedEvent>()
            .add_event::<TechnologyResearchedEvent>()
            .add_event::<TechnologyReselectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    campaign_start_points_system,
                    spend_research_points_system,
                    lock_recently_researched_on_completed_mission_system,
                    keep_research_available_on_failed_mission_system,
                    keep_research_available_on_replayed_mission_system,
                    unlock_recently_researched_after_reselection_system,
                    award_points_from_missions_and_loot_system,
                    award_goddess_of_destiny_bonus_system,
                    complete_tree_at_total_cost_system,
                    technology_tree_checksum_system,
                )
                    .chain(),
            );
    }
}