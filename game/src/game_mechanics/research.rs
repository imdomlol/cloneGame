// Sources: vault/research/research.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const RESEARCH_COST: I32F32 = I32F32::lit("0");
const RESEARCH_TIME: I32F32 = I32F32::lit("0");
const SECONDS_PER_DAY: I32F32 = I32F32::lit("95");
const DEFAULT_TIER_1_RESEARCH_TIME_SECONDS: I32F32 = I32F32::lit("75");
const STONE_WORKSHOP_RESEARCH_COST_GOLD: I32F32 = I32F32::lit("500");
const STONE_WORKSHOP_RESEARCH_TIME_SECONDS: I32F32 = I32F32::lit("75");
const FOUNDRY_RESEARCH_COST_GOLD: I32F32 = I32F32::lit("1000");
const FOUNDRY_RESEARCH_TIME_SECONDS: I32F32 = I32F32::lit("75");

#[derive(Resource, Clone, Copy)]
pub struct ResearchMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub research_tier: &'static str,
    pub unlocks: &'static str,
    pub provides_bonus: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for ResearchMechanicData {
    fn default() -> Self {
        Self {
            id: "research",
            name: "Research",
            research_tier: "system",
            unlocks: "Tier 1 via [[wood_workshop]], tier 2 via [[stone_workshop]], tier 3 via [[foundry]]",
            provides_bonus: "Defines the technology progression used to unlock research buildings and units",
            depends_on: &["wood_workshop", "stone_workshop", "foundry"],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ResearchState {
    pub research_cost: I32F32,
    pub research_time: I32F32,
    pub seconds_per_day: I32F32,
    pub default_tier_1_research_time_seconds: I32F32,
    pub tier_1_available: bool,
    pub tier_2_available: bool,
    pub tier_3_available: bool,
    pub stone_workshop_researched: bool,
    pub foundry_researched: bool,
    pub last_selected_tier: u8,
    pub last_selected_time_seconds: I32F32,
    pub last_selected_time_known: bool,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            research_cost: RESEARCH_COST,
            research_time: RESEARCH_TIME,
            seconds_per_day: I32F32::ZERO,
            default_tier_1_research_time_seconds: DEFAULT_TIER_1_RESEARCH_TIME_SECONDS,
            tier_1_available: false,
            tier_2_available: false,
            tier_3_available: false,
            stone_workshop_researched: false,
            foundry_researched: false,
            last_selected_tier: 0,
            last_selected_time_seconds: I32F32::ZERO,
            last_selected_time_known: true,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct ResearchPageViewedEvent;

#[derive(Event, Clone, Copy)]
pub struct WoodWorkshopAvailabilityChangedEvent {
    pub is_available: bool,
}

#[derive(Event, Clone, Copy)]
pub struct TierResearchSelectedEvent {
    pub tier: u8,
    pub listed_time_seconds: I32F32,
    pub listed_time_known: bool,
}

#[derive(Event, Clone, Copy)]
pub struct StoneWorkshopResearchedEvent {
    pub paid_gold: I32F32,
    pub elapsed_seconds: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct FoundryResearchedEvent {
    pub paid_gold: I32F32,
    pub elapsed_seconds: I32F32,
}

fn set_seconds_per_day_from_page_system(
    mut state: ResMut<ResearchState>,
    mut page_viewed_events: EventReader<ResearchPageViewedEvent>,
) {
    for _ in page_viewed_events.read() {
        state.seconds_per_day = SECONDS_PER_DAY;
    }
}

fn enable_tier_1_from_wood_workshop_system(
    mut state: ResMut<ResearchState>,
    mut availability_events: EventReader<WoodWorkshopAvailabilityChangedEvent>,
) {
    for ev in availability_events.read() {
        if ev.is_available {
            state.tier_1_available = true;
        }
    }
}

fn apply_default_time_for_tier_1_selection_system(
    mut state: ResMut<ResearchState>,
    mut tier_selected_events: EventReader<TierResearchSelectedEvent>,
) {
    for ev in tier_selected_events.read() {
        if ev.tier == 1 && ev.listed_time_known {
            state.last_selected_tier = 1;
            state.last_selected_time_seconds = DEFAULT_TIER_1_RESEARCH_TIME_SECONDS;
            state.last_selected_time_known = true;
        }
    }
}

fn unlock_tier_2_from_stone_workshop_research_system(
    mut state: ResMut<ResearchState>,
    mut researched_events: EventReader<StoneWorkshopResearchedEvent>,
) {
    for ev in researched_events.read() {
        if ev.paid_gold == STONE_WORKSHOP_RESEARCH_COST_GOLD
            && ev.elapsed_seconds == STONE_WORKSHOP_RESEARCH_TIME_SECONDS
        {
            state.stone_workshop_researched = true;
            state.tier_2_available = true;
        }
    }
}

fn unlock_tier_3_from_foundry_research_system(
    mut state: ResMut<ResearchState>,
    mut researched_events: EventReader<FoundryResearchedEvent>,
) {
    for ev in researched_events.read() {
        if ev.paid_gold == FOUNDRY_RESEARCH_COST_GOLD
            && ev.elapsed_seconds == FOUNDRY_RESEARCH_TIME_SECONDS
        {
            state.foundry_researched = true;
            state.tier_3_available = true;
        }
    }
}

fn set_unknown_time_on_unknown_entry_system(
    mut state: ResMut<ResearchState>,
    mut tier_selected_events: EventReader<TierResearchSelectedEvent>,
) {
    for ev in tier_selected_events.read() {
        if !ev.listed_time_known {
            state.last_selected_tier = ev.tier;
            state.last_selected_time_seconds = I32F32::ZERO;
            state.last_selected_time_known = false;
        }
    }
}

fn research_checksum_system(mut checksum: ResMut<SimChecksumState>, state: Res<ResearchState>) {
    checksum.accumulate(RESEARCH_COST.to_bits() as u64);
    checksum.accumulate(RESEARCH_TIME.to_bits() as u64);
    checksum.accumulate(SECONDS_PER_DAY.to_bits() as u64);
    checksum.accumulate(DEFAULT_TIER_1_RESEARCH_TIME_SECONDS.to_bits() as u64);
    checksum.accumulate(STONE_WORKSHOP_RESEARCH_COST_GOLD.to_bits() as u64);
    checksum.accumulate(STONE_WORKSHOP_RESEARCH_TIME_SECONDS.to_bits() as u64);
    checksum.accumulate(FOUNDRY_RESEARCH_COST_GOLD.to_bits() as u64);
    checksum.accumulate(FOUNDRY_RESEARCH_TIME_SECONDS.to_bits() as u64);

    checksum.accumulate(state.research_cost.to_bits() as u64);
    checksum.accumulate(state.research_time.to_bits() as u64);
    checksum.accumulate(state.seconds_per_day.to_bits() as u64);
    checksum.accumulate(state.default_tier_1_research_time_seconds.to_bits() as u64);
    checksum.accumulate(u64::from(state.tier_1_available));
    checksum.accumulate(u64::from(state.tier_2_available));
    checksum.accumulate(u64::from(state.tier_3_available));
    checksum.accumulate(u64::from(state.stone_workshop_researched));
    checksum.accumulate(u64::from(state.foundry_researched));
    checksum.accumulate(state.last_selected_tier as u64);
    checksum.accumulate(state.last_selected_time_seconds.to_bits() as u64);
    checksum.accumulate(u64::from(state.last_selected_time_known));
}

pub struct ResearchPlugin;

impl Plugin for ResearchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResearchMechanicData>()
            .init_resource::<ResearchState>()
            .add_event::<ResearchPageViewedEvent>()
            .add_event::<WoodWorkshopAvailabilityChangedEvent>()
            .add_event::<TierResearchSelectedEvent>()
            .add_event::<StoneWorkshopResearchedEvent>()
            .add_event::<FoundryResearchedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    set_seconds_per_day_from_page_system,
                    enable_tier_1_from_wood_workshop_system,
                    apply_default_time_for_tier_1_selection_system,
                    unlock_tier_2_from_stone_workshop_research_system,
                    unlock_tier_3_from_foundry_research_system,
                    set_unknown_time_on_unknown_entry_system,
                    research_checksum_system,
                )
                    .chain(),
            );
    }
}