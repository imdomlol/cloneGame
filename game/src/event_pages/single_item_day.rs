// Sources: vault/event_pages/single_item_day.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

pub const SINGLE_ITEM_DAY_ID: &str = "single_item_day";
pub const SINGLE_ITEM_DAY_NAME: &str = "Single Item Day";
pub const SINGLE_ITEM_DAY_TYPE: &str = "event_pages";
pub const SINGLE_ITEM_DAY_SUBTYPE: &str = "single_item_day";
pub const SINGLE_ITEM_DAY_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Single_Item_Day";
pub const SINGLE_ITEM_DAY_SOURCE_REVISION: u32 = 19342;
pub const SINGLE_ITEM_DAY_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const SINGLE_ITEM_DAY_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const SINGLE_ITEM_DAY_CHANCE_TENTHS_PERCENT: u16 = 52;
pub const SINGLE_ITEM_DAY_CHANCE_DENOMINATOR: u16 = 1000;
pub const SINGLE_ITEM_DAY_REROLL_LIMIT: u8 = 2;
pub const SINGLE_ITEM_DAY_MIN_ACCEPTABLE_RARITY: u16 = 5;
pub const SINGLE_ITEM_DAY_SKIP_CHANCE_PERCENT: u16 = 60;
pub const SINGLE_ITEM_DAY_SKIP_DENOMINATOR: u16 = 100;
pub const SINGLE_ITEM_DAY_MIN_SCRAP_VALUE: I32F32 = I32F32::lit("50");
pub const SINGLE_ITEM_DAY_MAX_SCRAP_VALUE: I32F32 = I32F32::lit("170");
pub const SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_THRESHOLD: I32F32 = I32F32::lit("4500");
pub const SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_MULTIPLIER: I32F32 = I32F32::lit("0.7");
pub const SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_ONE_HANDED: I32F32 = I32F32::lit("600");
pub const SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_TWO_HANDED: I32F32 = I32F32::lit("1500");
pub const SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_MULTIPLIER: I32F32 = I32F32::lit("1.4");

pub const SINGLE_ITEM_DAY_GIFT_BOX_ID: &str = "gift_box";
pub const SINGLE_ITEM_DAY_LARGE_AXLE_ID: &str = "large_axle";
pub const SINGLE_ITEM_DAY_V_TYPE_ENGINE_ID: &str = "v_type_engine";

const SID_EVENT_ROLL_SALT: u64 = 0x5349_4400_0000_0001;
const SID_INITIAL_PICK_SALT: u64 = 0x5349_4400_0000_0002;
const SID_REROLL_SALT_BASE: u64 = 0x5349_4400_0000_0100;
const SID_SKIP_SALT: u64 = 0x5349_4400_0000_0003;

pub struct SingleItemDayPlugin;

impl Plugin for SingleItemDayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SingleItemDayState>()
            .init_resource::<SingleItemDayLootPool>()
            .add_event::<SingleItemDayRollEvent>()
            .add_event::<SingleItemDayResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    single_item_day_roll_occurrence,
                    single_item_day_reroll_unsuitable_selection,
                    single_item_day_skip_unsuitable_final_selection,
                    single_item_day_set_two_handed_low_threshold,
                    single_item_day_reset_value_multiplier,
                    single_item_day_apply_high_total_multiplier,
                    single_item_day_apply_low_total_multiplier,
                    single_item_day_apply_scrap_identity_and_value,
                    single_item_day_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SingleItemDayState {
    pub day_index: u32,
    pub moon_id: &'static str,
    pub roll_succeeded: bool,
    pub active: bool,
    pub skipped: bool,
    pub selected_scrap_id: &'static str,
    pub selected_rarity: u16,
    pub selected_two_handed: bool,
    pub rerolls_attempted: u8,
    pub moon_resulting_total_scrap_value: I32F32,
    pub low_total_value_threshold: I32F32,
    pub value_multiplier: I32F32,
}

impl Default for SingleItemDayState {
    fn default() -> Self {
        Self {
            day_index: 0,
            moon_id: "",
            roll_succeeded: false,
            active: false,
            skipped: false,
            selected_scrap_id: "",
            selected_rarity: 0,
            selected_two_handed: false,
            rerolls_attempted: 0,
            moon_resulting_total_scrap_value: I32F32::ZERO,
            low_total_value_threshold: SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_ONE_HANDED,
            value_multiplier: I32F32::lit("1"),
        }
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct SingleItemDayLootPool {
    pub entries: Vec<SingleItemDayLootPoolEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleItemDayLootPoolEntry {
    pub scrap_id: &'static str,
    pub rarity: u16,
    pub two_handed: bool,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SingleItemDayScrap {
    pub stable_scrap_id: u64,
    pub item_id: &'static str,
    pub base_value: I32F32,
    pub value: I32F32,
    pub rarity: u16,
    pub two_handed: bool,
    pub applied_day_index: u32,
}

impl SingleItemDayScrap {
    pub fn new(
        stable_scrap_id: u64,
        item_id: &'static str,
        value: I32F32,
        rarity: u16,
        two_handed: bool,
    ) -> Self {
        Self {
            stable_scrap_id,
            item_id,
            base_value: value,
            value,
            rarity,
            two_handed,
            applied_day_index: u32::MAX,
        }
    }

    pub fn can_open_to_random_scrap(&self) -> bool {
        self.item_id == SINGLE_ITEM_DAY_GIFT_BOX_ID
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SingleItemDayRollEvent {
    pub day_index: u32,
    pub moon_id: &'static str,
    pub moon_resulting_total_scrap_value: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleItemDayResolvedEvent {
    pub day_index: u32,
    pub moon_id: &'static str,
    pub result: SingleItemDayResult,
    pub selected_scrap_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleItemDayResult {
    DidNotRoll,
    Active,
    Skipped,
}

fn single_item_day_roll_occurrence(
    mut roll_events: EventReader<SingleItemDayRollEvent>,
    mut resolved_events: EventWriter<SingleItemDayResolvedEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    loot_pool: Res<SingleItemDayLootPool>,
    mut state: ResMut<SingleItemDayState>,
) {
    for event in roll_events.read() {
        reset_for_day(&mut state, *event);

        let mut occurrence_rng = tick_rng(
            game_seed.0,
            tick.0,
            SID_EVENT_ROLL_SALT ^ event.day_index as u64,
        );
        let roll = occurrence_rng.next_u32() % SINGLE_ITEM_DAY_CHANCE_DENOMINATOR as u32;

        if roll >= SINGLE_ITEM_DAY_CHANCE_TENTHS_PERCENT as u32 {
            resolved_events.send(SingleItemDayResolvedEvent {
                day_index: event.day_index,
                moon_id: event.moon_id,
                result: SingleItemDayResult::DidNotRoll,
                selected_scrap_id: "",
            });
            continue;
        }

        state.roll_succeeded = true;

        if let Some(entry) = choose_pool_entry(
            &loot_pool,
            game_seed.0,
            tick.0,
            SID_INITIAL_PICK_SALT ^ event.day_index as u64,
        ) {
            apply_selected_entry(&mut state, entry);
            state.active = true;
        } else {
            state.skipped = true;
            resolved_events.send(SingleItemDayResolvedEvent {
                day_index: event.day_index,
                moon_id: event.moon_id,
                result: SingleItemDayResult::Skipped,
                selected_scrap_id: "",
            });
        }
    }
}

fn single_item_day_reroll_unsuitable_selection(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    loot_pool: Res<SingleItemDayLootPool>,
    mut state: ResMut<SingleItemDayState>,
) {
    if !state.active || !selected_entry_is_unsuitable(&state) {
        return;
    }

    while state.rerolls_attempted < SINGLE_ITEM_DAY_REROLL_LIMIT
        && selected_entry_is_unsuitable(&state)
    {
        let salt = SID_REROLL_SALT_BASE
            ^ state.day_index as u64
            ^ ((state.rerolls_attempted as u64) << 32);
        if let Some(entry) = choose_pool_entry(&loot_pool, game_seed.0, tick.0, salt) {
            apply_selected_entry(&mut state, entry);
        }
        state.rerolls_attempted = state.rerolls_attempted.saturating_add(1);
    }
}

fn single_item_day_skip_unsuitable_final_selection(
    mut resolved_events: EventWriter<SingleItemDayResolvedEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<SingleItemDayState>,
) {
    if !state.active
        || state.rerolls_attempted < SINGLE_ITEM_DAY_REROLL_LIMIT
        || !selected_entry_is_unsuitable(&state)
    {
        return;
    }

    let mut skip_rng = tick_rng(
        game_seed.0,
        tick.0,
        SID_SKIP_SALT ^ state.day_index as u64,
    );
    let roll = skip_rng.next_u32() % SINGLE_ITEM_DAY_SKIP_DENOMINATOR as u32;

    if roll < SINGLE_ITEM_DAY_SKIP_CHANCE_PERCENT as u32 {
        state.active = false;
        state.skipped = true;
        resolved_events.send(SingleItemDayResolvedEvent {
            day_index: state.day_index,
            moon_id: state.moon_id,
            result: SingleItemDayResult::Skipped,
            selected_scrap_id: state.selected_scrap_id,
        });
    } else {
        resolved_events.send(SingleItemDayResolvedEvent {
            day_index: state.day_index,
            moon_id: state.moon_id,
            result: SingleItemDayResult::Active,
            selected_scrap_id: state.selected_scrap_id,
        });
    }
}

fn single_item_day_set_two_handed_low_threshold(mut state: ResMut<SingleItemDayState>) {
    if !state.active {
        return;
    }

    if state.selected_two_handed {
        state.low_total_value_threshold = SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_TWO_HANDED;
    } else {
        state.low_total_value_threshold = SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_ONE_HANDED;
    }
}

fn single_item_day_reset_value_multiplier(mut state: ResMut<SingleItemDayState>) {
    if state.active {
        state.value_multiplier = I32F32::lit("1");
    }
}

fn single_item_day_apply_high_total_multiplier(mut state: ResMut<SingleItemDayState>) {
    if state.active
        && state.moon_resulting_total_scrap_value > SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_THRESHOLD
    {
        state.value_multiplier = SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_MULTIPLIER;
    }
}

fn single_item_day_apply_low_total_multiplier(mut state: ResMut<SingleItemDayState>) {
    if state.active && state.moon_resulting_total_scrap_value < state.low_total_value_threshold {
        state.value_multiplier = SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_MULTIPLIER;
    }
}

fn single_item_day_apply_scrap_identity_and_value(
    state: Res<SingleItemDayState>,
    mut scrap_query: Query<&mut SingleItemDayScrap>,
) {
    if !state.active {
        return;
    }

    for mut scrap in scrap_query.iter_mut() {
        if scrap.applied_day_index == state.day_index {
            continue;
        }

        scrap.item_id = state.selected_scrap_id;
        scrap.rarity = state.selected_rarity;
        scrap.two_handed = state.selected_two_handed;
        scrap.value = clamp_scrap_value(scrap.base_value) * state.value_multiplier;
        scrap.applied_day_index = state.day_index;
    }
}

fn single_item_day_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<SingleItemDayState>,
    loot_pool: Res<SingleItemDayLootPool>,
    scrap_query: Query<&SingleItemDayScrap>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(SINGLE_ITEM_DAY_SOURCE_REVISION as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_CHANCE_TENTHS_PERCENT as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_CHANCE_DENOMINATOR as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_REROLL_LIMIT as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_MIN_ACCEPTABLE_RARITY as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_SKIP_CHANCE_PERCENT as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_SKIP_DENOMINATOR as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_MIN_SCRAP_VALUE.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_MAX_SCRAP_VALUE.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_THRESHOLD.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_HIGH_TOTAL_VALUE_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_ONE_HANDED.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_TWO_HANDED.to_bits() as u64);
    checksum.accumulate(SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_MULTIPLIER.to_bits() as u64);

    checksum.accumulate(state.day_index as u64);
    accumulate_str(&mut checksum, 0x1000, state.moon_id);
    checksum.accumulate(state.roll_succeeded as u64);
    checksum.accumulate(state.active as u64);
    checksum.accumulate(state.skipped as u64);
    accumulate_str(&mut checksum, 0x1001, state.selected_scrap_id);
    checksum.accumulate(state.selected_rarity as u64);
    checksum.accumulate(state.selected_two_handed as u64);
    checksum.accumulate(state.rerolls_attempted as u64);
    checksum.accumulate(state.moon_resulting_total_scrap_value.to_bits() as u64);
    checksum.accumulate(state.low_total_value_threshold.to_bits() as u64);
    checksum.accumulate(state.value_multiplier.to_bits() as u64);

    for entry in &loot_pool.entries {
        accumulate_str(&mut checksum, 0x2000, entry.scrap_id);
        checksum.accumulate(entry.rarity as u64);
        checksum.accumulate(entry.two_handed as u64);
    }

    for scrap in scrap_query.iter() {
        checksum.accumulate(scrap.stable_scrap_id);
        accumulate_str(&mut checksum, 0x3000, scrap.item_id);
        checksum.accumulate(scrap.base_value.to_bits() as u64);
        checksum.accumulate(scrap.value.to_bits() as u64);
        checksum.accumulate(scrap.rarity as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(scrap.applied_day_index as u64);
    }
}

fn reset_for_day(state: &mut SingleItemDayState, event: SingleItemDayRollEvent) {
    state.day_index = event.day_index;
    state.moon_id = event.moon_id;
    state.roll_succeeded = false;
    state.active = false;
    state.skipped = false;
    state.selected_scrap_id = "";
    state.selected_rarity = 0;
    state.selected_two_handed = false;
    state.rerolls_attempted = 0;
    state.moon_resulting_total_scrap_value = event.moon_resulting_total_scrap_value;
    state.low_total_value_threshold = SINGLE_ITEM_DAY_LOW_TOTAL_VALUE_THRESHOLD_ONE_HANDED;
    state.value_multiplier = I32F32::lit("1");
}

fn choose_pool_entry(
    loot_pool: &SingleItemDayLootPool,
    game_seed: u64,
    tick: u64,
    salt: u64,
) -> Option<SingleItemDayLootPoolEntry> {
    if loot_pool.entries.is_empty() {
        return None;
    }

    let mut sorted_indices: Vec<usize> = (0..loot_pool.entries.len()).collect();
    sorted_indices.sort_by(|left, right| {
        let left_entry = loot_pool.entries[*left];
        let right_entry = loot_pool.entries[*right];

        left_entry
            .scrap_id
            .cmp(right_entry.scrap_id)
            .then(left_entry.rarity.cmp(&right_entry.rarity))
            .then(left_entry.two_handed.cmp(&right_entry.two_handed))
    });

    let mut rng = tick_rng(game_seed, tick, salt);
    let selected_sorted_index = (rng.next_u64() % sorted_indices.len() as u64) as usize;
    Some(loot_pool.entries[sorted_indices[selected_sorted_index]])
}

fn apply_selected_entry(state: &mut SingleItemDayState, entry: SingleItemDayLootPoolEntry) {
    state.selected_scrap_id = entry.scrap_id;
    state.selected_rarity = entry.rarity;
    state.selected_two_handed = entry.two_handed;
}

fn selected_entry_is_unsuitable(state: &SingleItemDayState) -> bool {
    state.selected_rarity < SINGLE_ITEM_DAY_MIN_ACCEPTABLE_RARITY || state.selected_two_handed
}

fn clamp_scrap_value(value: I32F32) -> I32F32 {
    if value < SINGLE_ITEM_DAY_MIN_SCRAP_VALUE {
        SINGLE_ITEM_DAY_MIN_SCRAP_VALUE
    } else if value > SINGLE_ITEM_DAY_MAX_SCRAP_VALUE {
        SINGLE_ITEM_DAY_MAX_SCRAP_VALUE
    } else {
        value
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}