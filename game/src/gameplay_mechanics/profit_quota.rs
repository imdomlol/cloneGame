// Sources: vault/gameplay_mechanics/profit_quota.md, vault/store_items/dog_house.md, vault/scrap_items/wedding_ring.md
use bevy::prelude::*;
use fixed::types::I64F64;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::AwardOvertimeCreditsEvent;
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

pub const PROFIT_QUOTA_ID: &str = "profit_quota";
pub const PROFIT_QUOTA_NAME: &str = "Profit Quota";
pub const PROFIT_QUOTA_TYPE: &str = "gameplay_mechanics";
pub const PROFIT_QUOTA_SUBTYPE: &str = "quota";
pub const PROFIT_QUOTA_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Profit_Quota";
pub const PROFIT_QUOTA_SOURCE_REVISION: u32 = 21372;
pub const PROFIT_QUOTA_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const PROFIT_QUOTA_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const PROFIT_QUOTA_FIRST_QUOTA: i64 = 130;
pub const PROFIT_QUOTA_CYCLE_DAYS_TOTAL: i8 = 4;
pub const PROFIT_QUOTA_EXPLORE_DAYS: i8 = 3;
pub const PROFIT_QUOTA_INCREASE_BASE: i64 = 200;
pub const PROFIT_QUOTA_TIMES_FULFILLED_SQUARE_DIVISOR: i64 = 4;
pub const PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_NUMERATOR: i64 = 15;
pub const PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_DENOMINATOR: i64 = 10;
pub const PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_NUMERATOR: i64 = 2043;
pub const PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_DENOMINATOR: i64 = 10000;
pub const PROFIT_QUOTA_MAX_RANDOM_REDUCTION_NUMERATOR: i64 = 30645;
pub const PROFIT_QUOTA_MAX_RANDOM_REDUCTION_DENOMINATOR: i64 = 100000;
pub const PROFIT_QUOTA_CURVE_MEAN_NUMERATOR: i64 = 18;
pub const PROFIT_QUOTA_CURVE_MEAN_DENOMINATOR: i64 = 1000;
pub const PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_NUMERATOR: i64 = 121;
pub const PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_DENOMINATOR: i64 = 1000;
pub const PROFIT_QUOTA_CITED_PROBABILITY_NUMERATOR: i64 = 5871;
pub const PROFIT_QUOTA_CITED_PROBABILITY_DENOMINATOR: i64 = 10000;
pub const PROFIT_QUOTA_OVERTIME_SCRAP_DIVISOR: i64 = 5;
pub const PROFIT_QUOTA_OVERTIME_DAY_BONUS: i64 = 15;
pub const PROFIT_QUOTA_TARGET_TOTAL_MULTIPLIER: i64 = 5;
pub const PROFIT_QUOTA_TARGET_TOTAL_DENOMINATOR: i64 = 6;
pub const PROFIT_QUOTA_TARGET_TOTAL_OFFSET: i64 = 75;
pub const PROFIT_QUOTA_RNG_SALT: u64 = 0x7072_6f66_6974_7100;

pub const DOG_HOUSE_ID: &str = "dog_house";
pub const DOG_HOUSE_NAME: &str = "Dog house";
pub const DOG_HOUSE_BUY_CREDITS: i64 = 80;
pub const DOG_HOUSE_LUCK_NUMERATOR: i64 = 7;
pub const DOG_HOUSE_LUCK_DENOMINATOR: i64 = 1000;

pub const WEDDING_RING_ID: &str = "wedding_ring";
pub const WEDDING_RING_NAME: &str = "Wedding ring";
pub const WEDDING_RING_WEIGHT: i64 = 16;
pub const WEDDING_RING_CONDUCTIVE: bool = true;
pub const WEDDING_RING_MIN_CREDITS: i64 = 52;
pub const WEDDING_RING_MAX_CREDITS: i64 = 79;
pub const WEDDING_RING_TWO_HANDED: bool = false;

pub const PROFIT_QUOTA_DEPENDS_ON: [&str; 24] = [
    "lethal_company",
    "moons",
    "scrap",
    "the_company",
    "signal_translator",
    "disco_ball",
    "television",
    "electric_chair",
    "shower",
    "jack_o_lantern",
    "toilet",
    "microwave",
    "fridge",
    "sofa_chair",
    "dog_house",
    "classic_painting",
    "goldfish",
    "cozy_lights",
    "record_player",
    "romantic_table",
    "table",
    "inverse_teleporter",
    "welcome_mat",
    "loud_horn",
];

pub const PROFIT_QUOTA_RULES: [ProfitQuotaRule; 3] = [
    ProfitQuotaRule {
        condition: "the first quota is generated",
        outcome: "the quota is always 130",
    },
    ProfitQuotaRule {
        condition: "a quota cycle begins",
        outcome: "it lasts 4 days total, including the day with 0 days remaining",
    },
    ProfitQuotaRule {
        condition: "the crew starts a cycle",
        outcome: "it has 3 days to explore and collect scrap before the deadline",
    },
];

pub const PROFIT_QUOTA_MODIFIERS: [ProfitQuotaRule; 6] = [
    ProfitQuotaRule {
        condition: "ship furniture contributes luck",
        outcome: "that luck is applied to the next quota calculation, not the current one",
    },
    ProfitQuotaRule {
        condition: "quota increase is computed",
        outcome: "use 200 * (1 + timesFulfilled^2 / 4) * (randomizerCurve.eval(clamp(random(0,1) - (totalLuck * 1.5), 0, 1)) + 1)",
    },
    ProfitQuotaRule {
        condition: "the random input is evaluated",
        outcome: "random(0,1) is reduced by totalLuck * 1.5 before clamping",
    },
    ProfitQuotaRule {
        condition: "all ship furniture except signal_translator is present",
        outcome: "maximum luck is 0.2043 and random input is reduced by 0.30645",
    },
    ProfitQuotaRule {
        condition: "the curve is approximated as a bell-shaped distribution",
        outcome: "use mean 0.018 and standard deviation 0.121 for rough probability work",
    },
    ProfitQuotaRule {
        condition: "the cited example probability is needed",
        outcome: "the chance to roll a value in [-0.1;0.1] is 58.71%",
    },
];

pub const PROFIT_QUOTA_STRATEGY: [ProfitQuotaRule; 2] = [
    ProfitQuotaRule {
        condition: "a target credit total is desired with 0 days until deadline",
        outcome: "use quotaFulfilled = (5 * total + profitQuota + 75) / 6",
    },
    ProfitQuotaRule {
        condition: "the result of the target-sale equation is not a whole number",
        outcome: "round up before selling scrap",
    },
];

pub const PROFIT_QUOTA_NOTES: [ProfitQuotaRule; 3] = [
    ProfitQuotaRule {
        condition: "timesFulfilled increases",
        outcome: "quota increase grows faster than linearly because timesFulfilled^2 / 4 is quadratic",
    },
    ProfitQuotaRule {
        condition: "furniture is placed during quota n",
        outcome: "it only affects the luck used for quota n + 1",
    },
    ProfitQuotaRule {
        condition: "the average quota table is used",
        outcome: "treat the values as estimates because randomizerCurve.eval can change them significantly",
    },
];

pub const PROFIT_QUOTA_BEHAVIORAL_MECHANICS: [ProfitQuotaRule; 20] = [
    ProfitQuotaRule {
        condition: "a quota cycle begins",
        outcome: "it lasts 4 days total, including the day with 0 days remaining",
    },
    ProfitQuotaRule {
        condition: "the crew starts a cycle",
        outcome: "it has 3 days to explore moons and collect scrap",
    },
    ProfitQuotaRule {
        condition: "the crew fails to meet the quota before the deadline",
        outcome: "the company discharges the entire crew by jettisoning them into deep space",
    },
    ProfitQuotaRule {
        condition: "the crew cannot land on Gordion on day 0",
        outcome: "selling scrap becomes impossible and the run ends in game over",
    },
    ProfitQuotaRule {
        condition: "the first quota is generated",
        outcome: "it is always 130",
    },
    ProfitQuotaRule {
        condition: "quota increase is computed",
        outcome: "use 200 * (1 + timesFulfilled^2 / 4) * (randomizerCurve.eval(clamp(random(0,1) - (totalLuck * 1.5), 0, 1)) + 1)",
    },
    ProfitQuotaRule {
        condition: "timesFulfilled increases",
        outcome: "the quota increase grows faster than linearly because the timesFulfilled^2 / 4 term is quadratic",
    },
    ProfitQuotaRule {
        condition: "ship furniture contributes luck",
        outcome: "that luck is read from the previous quota calculation rather than the current one",
    },
    ProfitQuotaRule {
        condition: "the random input is evaluated",
        outcome: "random(0,1) - (totalLuck * 1.5) is clamped into the range [0, 1] before randomizerCurve.eval runs",
    },
    ProfitQuotaRule {
        condition: "all ship furniture except signal_translator is present",
        outcome: "maximum luck is 0.2043 and the random input is reduced by 0.30645 to an effective range of [0, 0.69355]",
    },
    ProfitQuotaRule {
        condition: "the curve is approximated as a bell-shaped distribution",
        outcome: "use mean 0.018 and standard deviation 0.121 for rough probability work",
    },
    ProfitQuotaRule {
        condition: "you need the cited example probability",
        outcome: "the chance to roll a value in [-0.1;0.1] is 58.71%",
    },
    ProfitQuotaRule {
        condition: "the crew over-delivers",
        outcome: "overtimeBonus = floor((quotaFulfilled - profitQuota) / 5 + 15 * daysUntilDeadline)",
    },
    ProfitQuotaRule {
        condition: "the calculated overtime bonus would be negative",
        outcome: "it is capped at 0 and credits cannot be lost",
    },
    ProfitQuotaRule {
        condition: "the crew sells on day 3",
        outcome: "daysUntilDeadline is 2 and the efficiency bonus is 30",
    },
    ProfitQuotaRule {
        condition: "the crew sells on day 2",
        outcome: "daysUntilDeadline is 1 and the efficiency bonus is 15",
    },
    ProfitQuotaRule {
        condition: "the crew sells on day 1",
        outcome: "daysUntilDeadline is 0 and the efficiency bonus is 0",
    },
    ProfitQuotaRule {
        condition: "the crew sells on day 0",
        outcome: "daysUntilDeadline is -1 and the efficiency penalty is -15 before the cap",
    },
    ProfitQuotaRule {
        condition: "a target credit total is desired with 0 days until deadline",
        outcome: "quotaFulfilled = (5 * total + profitQuota + 75) / 6",
    },
    ProfitQuotaRule {
        condition: "the result of the target-sale equation is not a whole number",
        outcome: "round up before selling scrap",
    },
];

pub struct ProfitQuotaPlugin;

impl Plugin for ProfitQuotaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProfitQuotaState>()
            .add_event::<StartProfitQuotaCycleEvent>()
            .add_event::<AdvanceProfitQuotaDayEvent>()
            .add_event::<SetProfitQuotaFurnitureLuckEvent>()
            .add_event::<FulfillProfitQuotaEvent>()
            .add_event::<ProfitQuotaGeneratedEvent>()
            .add_event::<ProfitQuotaFulfilledEvent>()
            .add_event::<ProfitQuotaFailedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    profit_quota_start_cycle,
                    profit_quota_advance_day,
                    profit_quota_set_furniture_luck,
                    profit_quota_fulfill,
                    profit_quota_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfitQuotaRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ProfitQuotaState {
    pub current_quota: I64F64,
    pub days_until_deadline: i8,
    pub times_fulfilled: u64,
    pub cycles_started: u64,
    pub quota_fulfilled_this_cycle: I64F64,
    pub queued_furniture_luck: I64F64,
    pub last_random_unit: I64F64,
    pub last_luck_adjusted_random: I64F64,
    pub last_curve_value: I64F64,
    pub last_quota_increase: I64F64,
    pub last_overtime_bonus: I64F64,
    pub fulfilled_cycles: u64,
    pub failed_cycles: u64,
    pub game_over: bool,
}

impl Default for ProfitQuotaState {
    fn default() -> Self {
        Self {
            current_quota: profit_quota_first_quota(),
            days_until_deadline: PROFIT_QUOTA_EXPLORE_DAYS,
            times_fulfilled: 0,
            cycles_started: 0,
            quota_fulfilled_this_cycle: I64F64::ZERO,
            queued_furniture_luck: I64F64::ZERO,
            last_random_unit: I64F64::ZERO,
            last_luck_adjusted_random: I64F64::ZERO,
            last_curve_value: I64F64::ZERO,
            last_quota_increase: I64F64::ZERO,
            last_overtime_bonus: I64F64::ZERO,
            fulfilled_cycles: 0,
            failed_cycles: 0,
            game_over: false,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartProfitQuotaCycleEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceProfitQuotaDayEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetProfitQuotaFurnitureLuckEvent {
    pub total_luck: I64F64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FulfillProfitQuotaEvent {
    pub quota_fulfilled: I64F64,
    pub can_land_at_company: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfitQuotaGeneratedEvent {
    pub quota: I64F64,
    pub days_until_deadline: i8,
    pub times_fulfilled: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfitQuotaFulfilledEvent {
    pub quota_fulfilled: I64F64,
    pub required_quota: I64F64,
    pub overtime_bonus: I64F64,
    pub next_quota: I64F64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfitQuotaFailedEvent {
    pub required_quota: I64F64,
    pub quota_fulfilled: I64F64,
    pub can_land_at_company: bool,
}

pub fn profit_quota_first_quota() -> I64F64 {
    I64F64::from_num(PROFIT_QUOTA_FIRST_QUOTA)
}

pub fn profit_quota_dog_house_luck() -> I64F64 {
    fixed_ratio(DOG_HOUSE_LUCK_NUMERATOR, DOG_HOUSE_LUCK_DENOMINATOR)
}

pub fn profit_quota_max_luck_without_signal_translator() -> I64F64 {
    fixed_ratio(
        PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_NUMERATOR,
        PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_DENOMINATOR,
    )
}

pub fn profit_quota_max_random_reduction_without_signal_translator() -> I64F64 {
    fixed_ratio(
        PROFIT_QUOTA_MAX_RANDOM_REDUCTION_NUMERATOR,
        PROFIT_QUOTA_MAX_RANDOM_REDUCTION_DENOMINATOR,
    )
}

pub fn profit_quota_curve_mean() -> I64F64 {
    fixed_ratio(
        PROFIT_QUOTA_CURVE_MEAN_NUMERATOR,
        PROFIT_QUOTA_CURVE_MEAN_DENOMINATOR,
    )
}

pub fn profit_quota_curve_standard_deviation() -> I64F64 {
    fixed_ratio(
        PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_NUMERATOR,
        PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_DENOMINATOR,
    )
}

pub fn profit_quota_cited_probability() -> I64F64 {
    fixed_ratio(
        PROFIT_QUOTA_CITED_PROBABILITY_NUMERATOR,
        PROFIT_QUOTA_CITED_PROBABILITY_DENOMINATOR,
    )
}

pub fn profit_quota_luck_random_reduction(total_luck: I64F64) -> I64F64 {
    total_luck
        * fixed_ratio(
            PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_NUMERATOR,
            PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_DENOMINATOR,
        )
}

pub fn profit_quota_clamped_random_input(random_unit: I64F64, total_luck: I64F64) -> I64F64 {
    clamp_fixed_unit(random_unit - profit_quota_luck_random_reduction(total_luck))
}

pub fn profit_quota_randomizer_curve_eval_estimate(clamped_input: I64F64) -> I64F64 {
    let centered = clamped_input - fixed_ratio(1, 2);
    profit_quota_curve_mean() + centered * profit_quota_curve_standard_deviation() * I64F64::from_num(2)
}

pub fn profit_quota_increase_from_curve_value(
    times_fulfilled: u64,
    randomizer_curve_value: I64F64,
) -> I64F64 {
    let fulfilled = I64F64::from_num(times_fulfilled);
    let growth = I64F64::ONE
        + (fulfilled * fulfilled)
            / I64F64::from_num(PROFIT_QUOTA_TIMES_FULFILLED_SQUARE_DIVISOR);

    I64F64::from_num(PROFIT_QUOTA_INCREASE_BASE)
        * growth
        * (randomizer_curve_value + I64F64::ONE)
}

pub fn profit_quota_increase(
    times_fulfilled: u64,
    total_luck: I64F64,
    random_unit: I64F64,
) -> I64F64 {
    let clamped_input = profit_quota_clamped_random_input(random_unit, total_luck);
    let curve_value = profit_quota_randomizer_curve_eval_estimate(clamped_input);
    profit_quota_increase_from_curve_value(times_fulfilled, curve_value)
}

pub fn profit_quota_overtime_bonus(
    quota_fulfilled: I64F64,
    profit_quota: I64F64,
    days_until_deadline: i8,
) -> I64F64 {
    let raw_bonus = ((quota_fulfilled - profit_quota)
        / I64F64::from_num(PROFIT_QUOTA_OVERTIME_SCRAP_DIVISOR))
        + I64F64::from_num(PROFIT_QUOTA_OVERTIME_DAY_BONUS)
            * I64F64::from_num(days_until_deadline);

    if raw_bonus < I64F64::ZERO {
        I64F64::ZERO
    } else {
        raw_bonus.floor()
    }
}

pub fn profit_quota_target_sale_for_total(
    target_total: I64F64,
    profit_quota: I64F64,
) -> I64F64 {
    ((I64F64::from_num(PROFIT_QUOTA_TARGET_TOTAL_MULTIPLIER) * target_total)
        + profit_quota
        + I64F64::from_num(PROFIT_QUOTA_TARGET_TOTAL_OFFSET))
        / I64F64::from_num(PROFIT_QUOTA_TARGET_TOTAL_DENOMINATOR)
}

pub fn profit_quota_target_sale_for_total_rounded_up(
    target_total: I64F64,
    profit_quota: I64F64,
) -> I64F64 {
    profit_quota_target_sale_for_total(target_total, profit_quota).ceil()
}

fn profit_quota_start_cycle(
    mut events: EventReader<StartProfitQuotaCycleEvent>,
    mut generated_events: EventWriter<ProfitQuotaGeneratedEvent>,
    mut state: ResMut<ProfitQuotaState>,
) {
    for _event in events.read() {
        state.current_quota = profit_quota_first_quota();
        state.days_until_deadline = PROFIT_QUOTA_EXPLORE_DAYS;
        state.quota_fulfilled_this_cycle = I64F64::ZERO;
        state.last_random_unit = I64F64::ZERO;
        state.last_luck_adjusted_random = I64F64::ZERO;
        state.last_curve_value = I64F64::ZERO;
        state.last_quota_increase = I64F64::ZERO;
        state.last_overtime_bonus = I64F64::ZERO;
        state.game_over = false;
        state.cycles_started = state.cycles_started.wrapping_add(1);

        generated_events.send(ProfitQuotaGeneratedEvent {
            quota: state.current_quota,
            days_until_deadline: state.days_until_deadline,
            times_fulfilled: state.times_fulfilled,
        });
    }
}

fn profit_quota_advance_day(
    mut events: EventReader<AdvanceProfitQuotaDayEvent>,
    mut failed_events: EventWriter<ProfitQuotaFailedEvent>,
    mut state: ResMut<ProfitQuotaState>,
) {
    for _event in events.read() {
        state.days_until_deadline = state.days_until_deadline.saturating_sub(1);

        if state.days_until_deadline < 0 && state.quota_fulfilled_this_cycle < state.current_quota {
            state.failed_cycles = state.failed_cycles.wrapping_add(1);
            state.game_over = true;

            failed_events.send(ProfitQuotaFailedEvent {
                required_quota: state.current_quota,
                quota_fulfilled: state.quota_fulfilled_this_cycle,
                can_land_at_company: false,
            });
        }
    }
}

fn profit_quota_set_furniture_luck(
    mut events: EventReader<SetProfitQuotaFurnitureLuckEvent>,
    mut state: ResMut<ProfitQuotaState>,
) {
    for event in events.read() {
        state.queued_furniture_luck = event.total_luck;
    }
}

fn profit_quota_fulfill(
    mut events: EventReader<FulfillProfitQuotaEvent>,
    mut generated_events: EventWriter<ProfitQuotaGeneratedEvent>,
    mut fulfilled_events: EventWriter<ProfitQuotaFulfilledEvent>,
    mut failed_events: EventWriter<ProfitQuotaFailedEvent>,
    mut overtime_events: EventWriter<AwardOvertimeCreditsEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<ProfitQuotaState>,
) {
    for event in events.read() {
        state.quota_fulfilled_this_cycle = event.quota_fulfilled;

        if !event.can_land_at_company && state.days_until_deadline <= 0 {
            state.failed_cycles = state.failed_cycles.wrapping_add(1);
            state.game_over = true;

            failed_events.send(ProfitQuotaFailedEvent {
                required_quota: state.current_quota,
                quota_fulfilled: event.quota_fulfilled,
                can_land_at_company: event.can_land_at_company,
            });
            continue;
        }

        if event.quota_fulfilled < state.current_quota {
            if state.days_until_deadline <= 0 {
                state.failed_cycles = state.failed_cycles.wrapping_add(1);
                state.game_over = true;

                failed_events.send(ProfitQuotaFailedEvent {
                    required_quota: state.current_quota,
                    quota_fulfilled: event.quota_fulfilled,
                    can_land_at_company: event.can_land_at_company,
                });
            }
            continue;
        }

        let overtime_bonus = profit_quota_overtime_bonus(
            event.quota_fulfilled,
            state.current_quota,
            state.days_until_deadline - 1,
        );

        if overtime_bonus > I64F64::ZERO {
            overtime_events.send(AwardOvertimeCreditsEvent {
                extra_credits: overtime_bonus,
            });
        }

        state.times_fulfilled = state.times_fulfilled.wrapping_add(1);
        state.fulfilled_cycles = state.fulfilled_cycles.wrapping_add(1);
        state.last_overtime_bonus = overtime_bonus;

        let mut rng = tick_rng(
            game_seed.0,
            tick.0,
            PROFIT_QUOTA_RNG_SALT ^ state.times_fulfilled,
        );
        let random_draw = rng.next_u32() % 1_000_001;
        let random_unit = I64F64::from_num(random_draw) / I64F64::from_num(1_000_000);
        let luck_adjusted_random =
            profit_quota_clamped_random_input(random_unit, state.queued_furniture_luck);
        let curve_value = profit_quota_randomizer_curve_eval_estimate(luck_adjusted_random);
        let quota_increase =
            profit_quota_increase_from_curve_value(state.times_fulfilled, curve_value);
        let next_quota = state.current_quota + quota_increase.floor();

        state.last_random_unit = random_unit;
        state.last_luck_adjusted_random = luck_adjusted_random;
        state.last_curve_value = curve_value;
        state.last_quota_increase = quota_increase;
        state.current_quota = next_quota;
        state.days_until_deadline = PROFIT_QUOTA_EXPLORE_DAYS;
        state.quota_fulfilled_this_cycle = I64F64::ZERO;
        state.cycles_started = state.cycles_started.wrapping_add(1);

        fulfilled_events.send(ProfitQuotaFulfilledEvent {
            quota_fulfilled: event.quota_fulfilled,
            required_quota: next_quota - quota_increase.floor(),
            overtime_bonus,
            next_quota,
        });

        generated_events.send(ProfitQuotaGeneratedEvent {
            quota: next_quota,
            days_until_deadline: state.days_until_deadline,
            times_fulfilled: state.times_fulfilled,
        });
    }
}

fn fixed_ratio(numerator: i64, denominator: i64) -> I64F64 {
    I64F64::from_num(numerator) / I64F64::from_num(denominator)
}

fn clamp_fixed_unit(value: I64F64) -> I64F64 {
    if value < I64F64::ZERO {
        I64F64::ZERO
    } else if value > I64F64::ONE {
        I64F64::ONE
    } else {
        value
    }
}

fn profit_quota_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<ProfitQuotaState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(PROFIT_QUOTA_SOURCE_REVISION as u64);
    checksum.accumulate(PROFIT_QUOTA_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(PROFIT_QUOTA_FIRST_QUOTA as u64);
    checksum.accumulate(PROFIT_QUOTA_CYCLE_DAYS_TOTAL as u64);
    checksum.accumulate(PROFIT_QUOTA_EXPLORE_DAYS as u64);
    checksum.accumulate(PROFIT_QUOTA_INCREASE_BASE as u64);
    checksum.accumulate(PROFIT_QUOTA_TIMES_FULFILLED_SQUARE_DIVISOR as u64);
    checksum.accumulate(PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_LUCK_RANDOM_REDUCTION_DENOMINATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_MAX_LUCK_WITHOUT_SIGNAL_TRANSLATOR_DENOMINATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_MAX_RANDOM_REDUCTION_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_MAX_RANDOM_REDUCTION_DENOMINATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CURVE_MEAN_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CURVE_MEAN_DENOMINATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CURVE_STANDARD_DEVIATION_DENOMINATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CITED_PROBABILITY_NUMERATOR as u64);
    checksum.accumulate(PROFIT_QUOTA_CITED_PROBABILITY_DENOMINATOR as u64);
    checksum.accumulate(DOG_HOUSE_BUY_CREDITS as u64);
    checksum.accumulate(DOG_HOUSE_LUCK_NUMERATOR as u64);
    checksum.accumulate(DOG_HOUSE_LUCK_DENOMINATOR as u64);
    checksum.accumulate(WEDDING_RING_WEIGHT as u64);
    checksum.accumulate(WEDDING_RING_CONDUCTIVE as u64);
    checksum.accumulate(WEDDING_RING_MIN_CREDITS as u64);
    checksum.accumulate(WEDDING_RING_MAX_CREDITS as u64);
    checksum.accumulate(WEDDING_RING_TWO_HANDED as u64);

    for dependency in PROFIT_QUOTA_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in PROFIT_QUOTA_RULES {
        accumulate_rule(&mut checksum, 0x2000, rule);
    }

    for rule in PROFIT_QUOTA_MODIFIERS {
        accumulate_rule(&mut checksum, 0x3000, rule);
    }

    for rule in PROFIT_QUOTA_STRATEGY {
        accumulate_rule(&mut checksum, 0x4000, rule);
    }

    for rule in PROFIT_QUOTA_NOTES {
        accumulate_rule(&mut checksum, 0x5000, rule);
    }

    for rule in PROFIT_QUOTA_BEHAVIORAL_MECHANICS {
        accumulate_rule(&mut checksum, 0x6000, rule);
    }

    accumulate_str(&mut checksum, 0x7000, DOG_HOUSE_ID);
    accumulate_str(&mut checksum, 0x7001, DOG_HOUSE_NAME);
    accumulate_str(&mut checksum, 0x7002, WEDDING_RING_ID);
    accumulate_str(&mut checksum, 0x7003, WEDDING_RING_NAME);

    checksum.accumulate(state.current_quota.to_bits() as u64);
    checksum.accumulate(state.days_until_deadline as u64);
    checksum.accumulate(state.times_fulfilled);
    checksum.accumulate(state.cycles_started);
    checksum.accumulate(state.quota_fulfilled_this_cycle.to_bits() as u64);
    checksum.accumulate(state.queued_furniture_luck.to_bits() as u64);
    checksum.accumulate(state.last_random_unit.to_bits() as u64);
    checksum.accumulate(state.last_luck_adjusted_random.to_bits() as u64);
    checksum.accumulate(state.last_curve_value.to_bits() as u64);
    checksum.accumulate(state.last_quota_increase.to_bits() as u64);
    checksum.accumulate(state.last_overtime_bonus.to_bits() as u64);
    checksum.accumulate(state.fulfilled_cycles);
    checksum.accumulate(state.failed_cycles);
    checksum.accumulate(state.game_over as u64);
}

fn accumulate_rule(checksum: &mut SimChecksumState, salt: u64, rule: ProfitQuotaRule) {
    accumulate_str(checksum, salt, rule.condition);
    accumulate_str(checksum, salt ^ 1, rule.outcome);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}