// Sources: vault/gameplay_mechanics/credits.md
use bevy::prelude::*;
use fixed::types::I64F64;

use crate::sim::{SimChecksumState, SimTick};

pub const CREDITS_ID: &str = "credits";
pub const CREDITS_NAME: &str = "Credits";
pub const CREDITS_TYPE: &str = "gameplay_mechanics";
pub const CREDITS_SUBTYPE: &str = "currency";
pub const CREDITS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Credits";
pub const CREDITS_SOURCE_REVISION: u32 = 15573;
pub const CREDITS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const CREDITS_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const CREDITS_STARTING_RESERVE: i64 = 60;
pub const CREDITS_DEATH_FINE_PERCENT: i64 = 8;
pub const CREDITS_UNRECOVERED_BODY_FINE_PERCENT: i64 = 12;
pub const CREDITS_FULL_DEATH_FINE_PERCENT: i64 = 20;

pub const CREDITS_DEPENDS_ON: [&str; 10] = [
    "lethal_company",
    "scrap",
    "company_building",
    "store",
    "profit_quota",
    "moons",
    "player_body",
    "teleport",
    "the_ship",
    "teleporter",
];

pub const CREDITS_RULES: [CreditsRule; 11] = [
    CreditsRule {
        condition: "a new contract starts",
        outcome: "the crew receives 60 Credits to spend",
    },
    CreditsRule {
        condition: "scrap is sold at the company_building before the profit_quota deadline",
        outcome: "its Credit value is reduced by the remaining days",
    },
    CreditsRule {
        condition: "3 days remain before the profit_quota deadline",
        outcome: "buying value is 30%",
    },
    CreditsRule {
        condition: "2 days remain before the profit_quota deadline",
        outcome: "buying value is 53%",
    },
    CreditsRule {
        condition: "1 day remains before the profit_quota deadline",
        outcome: "buying value is 77%",
    },
    CreditsRule {
        condition: "0 days remain before the profit_quota deadline",
        outcome: "buying value is 100%",
    },
    CreditsRule {
        condition: "a crew exceeds the profit_quota",
        outcome: "the crew receives an overtime bonus of extra Credits",
    },
    CreditsRule {
        condition: "an employee dies",
        outcome: "the crew loses 8% of its total Credit amount per death",
    },
    CreditsRule {
        condition: "a player_body is not recovered",
        outcome: "the crew loses an additional 12% of its total Credit amount per unrecovered body",
    },
    CreditsRule {
        condition: "a dead crew mate is teleported home and the body is dropped after being grabbed",
        outcome: "the body can register inside the the_ship",
    },
    CreditsRule {
        condition: "a body becomes irrecoverable through the teleporter",
        outcome: "the crew still receives the full 20% death fine",
    },
];

pub const CREDITS_MODIFIERS: [CreditsModifier; 5] = [
    CreditsModifier {
        condition: "3 days remain before the profit_quota deadline",
        outcome: "scrap sale value is multiplied by 0.30",
    },
    CreditsModifier {
        condition: "2 days remain before the profit_quota deadline",
        outcome: "scrap sale value is multiplied by 0.53",
    },
    CreditsModifier {
        condition: "1 day remains before the profit_quota deadline",
        outcome: "scrap sale value is multiplied by 0.77",
    },
    CreditsModifier {
        condition: "0 days remain before the profit_quota deadline",
        outcome: "scrap sale value is multiplied by 1.00",
    },
    CreditsModifier {
        condition: "a crew has deaths or unrecovered bodies",
        outcome: "Credits are reduced by 8% per death and 12% per unrecovered body",
    },
];

pub const CREDITS_STRATEGY: [CreditsRule; 3] = [
    CreditsRule {
        condition: "a crew wants to maximize Credits",
        outcome: "sell scrap at the company_building on or after the profit_quota deadline",
    },
    CreditsRule {
        condition: "crew mates die",
        outcome: "teleport them home when possible to reduce the unrecovered-body penalty",
    },
    CreditsRule {
        condition: "no teleport is available",
        outcome: "carry dead crew mates so they can register inside the the_ship",
    },
];

pub const CREDITS_NOTES: [CreditsRule; 2] = [
    CreditsRule {
        condition: "a new contract begins",
        outcome: "the starting Credit reserve is 60",
    },
    CreditsRule {
        condition: "the crew fails to fulfill the profit_quota",
        outcome: "Credits do not persist into a successful continuation because the employees are discharged",
    },
];

pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CreditsState>()
            .add_event::<StartCreditsContractEvent>()
            .add_event::<SellScrapForCreditsEvent>()
            .add_event::<AwardOvertimeCreditsEvent>()
            .add_event::<ApplyEmployeeDeathFineEvent>()
            .add_event::<SpendCreditsEvent>()
            .add_event::<CreditsChangedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    credits_start_contract,
                    credits_sell_scrap,
                    credits_award_overtime,
                    credits_apply_employee_death_fine,
                    credits_spend,
                    credits_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreditsRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreditsModifier {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CreditsState {
    pub balance: I64F64,
    pub contracts_started: u64,
    pub scrap_sales: u64,
    pub overtime_awards: u64,
    pub employee_deaths: u64,
    pub unrecovered_bodies: u64,
    pub credits_spent: I64F64,
    pub credits_earned: I64F64,
    pub credits_lost_to_fines: I64F64,
    pub last_sale_days_remaining: u8,
    pub last_sale_base_value: I64F64,
    pub last_sale_credit_value: I64F64,
}

impl Default for CreditsState {
    fn default() -> Self {
        Self {
            balance: I64F64::ZERO,
            contracts_started: 0,
            scrap_sales: 0,
            overtime_awards: 0,
            employee_deaths: 0,
            unrecovered_bodies: 0,
            credits_spent: I64F64::ZERO,
            credits_earned: I64F64::ZERO,
            credits_lost_to_fines: I64F64::ZERO,
            last_sale_days_remaining: 0,
            last_sale_base_value: I64F64::ZERO,
            last_sale_credit_value: I64F64::ZERO,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartCreditsContractEvent;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellScrapForCreditsEvent {
    pub scrap_entity_id: u64,
    pub base_credit_value: I64F64,
    pub days_remaining_before_quota_deadline: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwardOvertimeCreditsEvent {
    pub extra_credits: I64F64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyEmployeeDeathFineEvent {
    pub employee_deaths: u64,
    pub unrecovered_bodies: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendCreditsEvent {
    pub item_entity_id: u64,
    pub cost: I64F64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreditsChangedEvent {
    pub previous_balance: I64F64,
    pub new_balance: I64F64,
    pub delta: I64F64,
}

pub fn credits_starting_reserve() -> I64F64 {
    I64F64::from_num(CREDITS_STARTING_RESERVE)
}

pub fn credits_sale_multiplier(days_remaining_before_quota_deadline: u8) -> I64F64 {
    match days_remaining_before_quota_deadline {
        3 => I64F64::from_num(30) / I64F64::from_num(100),
        2 => I64F64::from_num(53) / I64F64::from_num(100),
        1 => I64F64::from_num(77) / I64F64::from_num(100),
        0 => I64F64::ONE,
        _ => I64F64::ONE,
    }
}

pub fn credits_sale_value(
    base_credit_value: I64F64,
    days_remaining_before_quota_deadline: u8,
) -> I64F64 {
    base_credit_value * credits_sale_multiplier(days_remaining_before_quota_deadline)
}

pub fn credits_death_fine_multiplier(employee_deaths: u64, unrecovered_bodies: u64) -> I64F64 {
    let death_percent = I64F64::from_num(CREDITS_DEATH_FINE_PERCENT)
        * I64F64::from_num(employee_deaths);
    let body_percent = I64F64::from_num(CREDITS_UNRECOVERED_BODY_FINE_PERCENT)
        * I64F64::from_num(unrecovered_bodies);
    (death_percent + body_percent) / I64F64::from_num(100)
}

fn credits_start_contract(
    mut events: EventReader<StartCreditsContractEvent>,
    mut changed_events: EventWriter<CreditsChangedEvent>,
    mut state: ResMut<CreditsState>,
) {
    for _event in events.read() {
        let previous_balance = state.balance;
        state.balance = credits_starting_reserve();
        state.contracts_started = state.contracts_started.wrapping_add(1);
        state.credits_earned = state.credits_earned + state.balance;

        changed_events.send(CreditsChangedEvent {
            previous_balance,
            new_balance: state.balance,
            delta: state.balance - previous_balance,
        });
    }
}

fn credits_sell_scrap(
    mut events: EventReader<SellScrapForCreditsEvent>,
    mut changed_events: EventWriter<CreditsChangedEvent>,
    mut state: ResMut<CreditsState>,
) {
    for event in events.read() {
        let previous_balance = state.balance;
        let sale_value = credits_sale_value(
            event.base_credit_value,
            event.days_remaining_before_quota_deadline,
        );

        state.balance = state.balance + sale_value;
        state.scrap_sales = state.scrap_sales.wrapping_add(1);
        state.credits_earned = state.credits_earned + sale_value;
        state.last_sale_days_remaining = event.days_remaining_before_quota_deadline;
        state.last_sale_base_value = event.base_credit_value;
        state.last_sale_credit_value = sale_value;

        changed_events.send(CreditsChangedEvent {
            previous_balance,
            new_balance: state.balance,
            delta: sale_value,
        });
    }
}

fn credits_award_overtime(
    mut events: EventReader<AwardOvertimeCreditsEvent>,
    mut changed_events: EventWriter<CreditsChangedEvent>,
    mut state: ResMut<CreditsState>,
) {
    for event in events.read() {
        let previous_balance = state.balance;

        state.balance = state.balance + event.extra_credits;
        state.overtime_awards = state.overtime_awards.wrapping_add(1);
        state.credits_earned = state.credits_earned + event.extra_credits;

        changed_events.send(CreditsChangedEvent {
            previous_balance,
            new_balance: state.balance,
            delta: event.extra_credits,
        });
    }
}

fn credits_apply_employee_death_fine(
    mut events: EventReader<ApplyEmployeeDeathFineEvent>,
    mut changed_events: EventWriter<CreditsChangedEvent>,
    mut state: ResMut<CreditsState>,
) {
    for event in events.read() {
        let previous_balance = state.balance;
        let fine_multiplier =
            credits_death_fine_multiplier(event.employee_deaths, event.unrecovered_bodies);
        let fine = state.balance * fine_multiplier;

        state.balance = state.balance - fine;
        state.employee_deaths = state.employee_deaths.wrapping_add(event.employee_deaths);
        state.unrecovered_bodies = state
            .unrecovered_bodies
            .wrapping_add(event.unrecovered_bodies);
        state.credits_lost_to_fines = state.credits_lost_to_fines + fine;

        changed_events.send(CreditsChangedEvent {
            previous_balance,
            new_balance: state.balance,
            delta: I64F64::ZERO - fine,
        });
    }
}

fn credits_spend(
    mut events: EventReader<SpendCreditsEvent>,
    mut changed_events: EventWriter<CreditsChangedEvent>,
    mut state: ResMut<CreditsState>,
) {
    for event in events.read() {
        if state.balance < event.cost {
            continue;
        }

        let previous_balance = state.balance;
        state.balance = state.balance - event.cost;
        state.credits_spent = state.credits_spent + event.cost;

        changed_events.send(CreditsChangedEvent {
            previous_balance,
            new_balance: state.balance,
            delta: I64F64::ZERO - event.cost,
        });
    }
}

fn credits_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<CreditsState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(CREDITS_SOURCE_REVISION as u64);
    checksum.accumulate(CREDITS_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CREDITS_STARTING_RESERVE as u64);
    checksum.accumulate(CREDITS_DEATH_FINE_PERCENT as u64);
    checksum.accumulate(CREDITS_UNRECOVERED_BODY_FINE_PERCENT as u64);
    checksum.accumulate(CREDITS_FULL_DEATH_FINE_PERCENT as u64);

    for dependency in CREDITS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in CREDITS_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for modifier in CREDITS_MODIFIERS {
        accumulate_str(&mut checksum, 0x3000, modifier.condition);
        accumulate_str(&mut checksum, 0x3001, modifier.outcome);
    }

    for rule in CREDITS_STRATEGY {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for rule in CREDITS_NOTES {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    checksum.accumulate(state.balance.to_bits() as u64);
    checksum.accumulate(state.contracts_started);
    checksum.accumulate(state.scrap_sales);
    checksum.accumulate(state.overtime_awards);
    checksum.accumulate(state.employee_deaths);
    checksum.accumulate(state.unrecovered_bodies);
    checksum.accumulate(state.credits_spent.to_bits() as u64);
    checksum.accumulate(state.credits_earned.to_bits() as u64);
    checksum.accumulate(state.credits_lost_to_fines.to_bits() as u64);
    checksum.accumulate(state.last_sale_days_remaining as u64);
    checksum.accumulate(state.last_sale_base_value.to_bits() as u64);
    checksum.accumulate(state.last_sale_credit_value.to_bits() as u64);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}