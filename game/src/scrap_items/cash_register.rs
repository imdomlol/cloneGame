// Sources: vault/scrap_items/cash_register.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition};

pub const CASH_REGISTER_ID: &str = "cash_register";
pub const CASH_REGISTER_NAME: &str = "Cash register";
pub const CASH_REGISTER_TYPE: &str = "scrap_items";
pub const CASH_REGISTER_SUBTYPE: &str = "scrap";
pub const CASH_REGISTER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Cash_Register";
pub const CASH_REGISTER_SOURCE_REVISION: u32 = 20389;
pub const CASH_REGISTER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const CASH_REGISTER_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const CASH_REGISTER_EFFECTS: &str = "Opens drawer and plays a \"Ding\" sound upon activation";
pub const CASH_REGISTER_WEIGHT: I32F32 = I32F32::lit("84");
pub const CASH_REGISTER_CONDUCTIVE: bool = true;
pub const CASH_REGISTER_SELL: &str = "Can be sold for credits";
pub const CASH_REGISTER_TWO_HANDED: bool = true;
pub const CASH_REGISTER_MIN_VALUE: I32F32 = I32F32::lit("80");
pub const CASH_REGISTER_MAX_VALUE: I32F32 = I32F32::lit("159");
pub const CASH_REGISTER_PAGE_ID: u32 = 23;

pub const CASH_REGISTER_DING_SOUND_AMOUNT: I32F32 = I32F32::lit("80");
pub const CASH_REGISTER_MOVEMENT_SOUND_AMOUNT: I32F32 = I32F32::lit("30");

pub const CASH_REGISTER_DEPENDS_ON: [&str; 17] = [
    "scrap",
    "lethal_company",
    "the_company",
    "credits",
    "eyeless_dog",
    "audible_sounds",
    "weather",
    "baboon_hawk",
    "company_cruiser",
    "the_ship",
    "artifice",
    "rend",
    "adamance",
    "experimentation",
    "vow",
    "assurance",
    "march",
];

pub const CASH_REGISTER_SPAWN_CHANCES: [CashRegisterSpawnChance; 7] = [
    CashRegisterSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("1.92"),
    },
    CashRegisterSpawnChance {
        moon: "rend",
        chance: I32F32::lit("1.48"),
    },
    CashRegisterSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("0.74"),
    },
    CashRegisterSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("0.53"),
    },
    CashRegisterSpawnChance {
        moon: "vow",
        chance: I32F32::lit("0.51"),
    },
    CashRegisterSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("0.35"),
    },
    CashRegisterSpawnChance {
        moon: "march",
        chance: I32F32::lit("0.34"),
    },
];

pub const CASH_REGISTER_BEHAVIORAL_MECHANICS: [CashRegisterBehaviorRule; 14] = [
    CashRegisterBehaviorRule {
        condition: "left-click",
        outcome: "the lever pulls to activate the item",
    },
    CashRegisterBehaviorRule {
        condition: "the item is activated",
        outcome: "the drawer opens and a \"Ding\" sound plays",
    },
    CashRegisterBehaviorRule {
        condition: "the sound is produced",
        outcome: "it can alert eyeless_dog and other entities capable of audible_sounds",
    },
    CashRegisterBehaviorRule {
        condition: "the item is being held and moved",
        outcome: "it produces a separate movement sound",
    },
    CashRegisterBehaviorRule {
        condition: "the item is transported during stormy weather",
        outcome: "its 84 lb weight and conductive property make it risky to bring back",
    },
    CashRegisterBehaviorRule {
        condition: "transport is delayed late into the run",
        outcome: "death or abandonment can occur",
    },
    CashRegisterBehaviorRule {
        condition: "sound is generated near baboon_hawks",
        outcome: "the item can help deter them",
    },
    CashRegisterBehaviorRule {
        condition: "the crew lacks a company_cruiser",
        outcome: "early transport is safer",
    },
    CashRegisterBehaviorRule {
        condition: "artifice",
        outcome: "spawn chance is 1.92%",
    },
    CashRegisterBehaviorRule {
        condition: "rend",
        outcome: "spawn chance is 1.48%",
    },
    CashRegisterBehaviorRule {
        condition: "adamance",
        outcome: "spawn chance is 0.74%",
    },
    CashRegisterBehaviorRule {
        condition: "experimentation",
        outcome: "spawn chance is 0.53%",
    },
    CashRegisterBehaviorRule {
        condition: "vow",
        outcome: "spawn chance is 0.51%",
    },
    CashRegisterBehaviorRule {
        condition: "assurance",
        outcome: "spawn chance is 0.35%",
    },
];

pub struct CashRegisterPlugin;

impl Plugin for CashRegisterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCashRegisterEvent>()
            .add_event::<CashRegisterActivatedEvent>()
            .add_event::<CashRegisterAudibleAlertEvent>()
            .add_event::<CashRegisterMovementSoundEvent>()
            .add_event::<CashRegisterStormTransportRiskEvent>()
            .add_event::<CashRegisterDelayedTransportRiskEvent>()
            .add_event::<CashRegisterBaboonHawkDeterEvent>()
            .add_event::<CashRegisterEarlyTransportSaferEvent>()
            .add_event::<CashRegisterSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_cash_register,
                    cash_register_pickup_item_bar_bridge,
                    cash_register_use_from_item_bar,
                    cash_register_emit_activation_noise,
                    cash_register_emit_movement_noise,
                    cash_register_transport_risk_bridge,
                    cash_register_sell_bridge,
                    cash_register_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CashRegisterBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CashRegisterSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CashRegister {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CashRegisterScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for CashRegisterScrap {
    fn default() -> Self {
        Self {
            min_value: CASH_REGISTER_MIN_VALUE,
            max_value: CASH_REGISTER_MAX_VALUE,
            weight: CASH_REGISTER_WEIGHT,
            conductive: CASH_REGISTER_CONDUCTIVE,
            two_handed: CASH_REGISTER_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CashRegisterHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CashRegisterActivationState {
    pub lever_pulled: bool,
    pub drawer_open: bool,
    pub activations: u64,
    pub movement_sound_count: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CashRegisterTransportState {
    pub stormy_weather: bool,
    pub delayed_late_run: bool,
    pub company_cruiser_available: bool,
    pub storm_risk_reported: bool,
    pub delayed_risk_reported: bool,
    pub early_transport_reported: bool,
}

#[derive(Bundle)]
pub struct CashRegisterBundle {
    pub name: Name,
    pub cash_register: CashRegister,
    pub scrap: CashRegisterScrap,
    pub position: SimPosition,
    pub held_by: CashRegisterHeldBy,
    pub activation_state: CashRegisterActivationState,
    pub transport_state: CashRegisterTransportState,
}

impl CashRegisterBundle {
    pub fn new(event: SpawnCashRegisterEvent) -> Self {
        Self {
            name: Name::new(CASH_REGISTER_NAME),
            cash_register: CashRegister {
                stable_id: event.stable_id,
            },
            scrap: CashRegisterScrap::default(),
            position: event.position,
            held_by: CashRegisterHeldBy::default(),
            activation_state: CashRegisterActivationState::default(),
            transport_state: CashRegisterTransportState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCashRegisterEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterActivatedEvent {
    pub cash_register_entity: Entity,
    pub cash_register_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterAudibleAlertEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterMovementSoundEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterStormTransportRiskEvent {
    pub cash_register_entity: Entity,
    pub cash_register_stable_id: u64,
    pub employee_id: u64,
    pub weight: I32F32,
    pub conductive: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterDelayedTransportRiskEvent {
    pub cash_register_entity: Entity,
    pub cash_register_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterBaboonHawkDeterEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterEarlyTransportSaferEvent {
    pub cash_register_entity: Entity,
    pub cash_register_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashRegisterSoldEvent {
    pub cash_register_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn cash_register_value_range() -> (I32F32, I32F32) {
    (CASH_REGISTER_MIN_VALUE, CASH_REGISTER_MAX_VALUE)
}

pub fn cash_register_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    CASH_REGISTER_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_cash_register(mut commands: Commands, mut events: EventReader<SpawnCashRegisterEvent>) {
    for event in events.read() {
        commands.spawn(CashRegisterBundle::new(*event));
    }
}

fn cash_register_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    cash_registers: Query<(&CashRegister, &CashRegisterHeldBy), Changed<CashRegisterHeldBy>>,
) {
    for (cash_register, held_by) in &cash_registers {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: CASH_REGISTER_ID,
                two_handed: CASH_REGISTER_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = cash_register.stable_id;
        }
    }
}

fn cash_register_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut activated_events: EventWriter<CashRegisterActivatedEvent>,
    mut cash_registers: Query<(
        Entity,
        &CashRegister,
        &CashRegisterHeldBy,
        &SimPosition,
        &mut CashRegisterActivationState,
    )>,
) {
    for event in item_events.read() {
        if event.item_id != CASH_REGISTER_ID
            || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }

        for (entity, cash_register, held_by, position, mut activation_state) in &mut cash_registers
        {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            activation_state.lever_pulled = true;
            activation_state.drawer_open = true;
            activation_state.activations = activation_state.activations.wrapping_add(1);

            activated_events.send(CashRegisterActivatedEvent {
                cash_register_entity: entity,
                cash_register_stable_id: cash_register.stable_id,
                employee_id: event.employee_id,
                position: *position,
            });
        }
    }
}

fn cash_register_emit_activation_noise(
    mut activated_events: EventReader<CashRegisterActivatedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<CashRegisterAudibleAlertEvent>,
    mut deter_events: EventWriter<CashRegisterBaboonHawkDeterEvent>,
) {
    for event in activated_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.cash_register_entity,
            position: event.position,
            amount: CASH_REGISTER_DING_SOUND_AMOUNT,
        });

        alert_events.send(CashRegisterAudibleAlertEvent {
            source: event.cash_register_entity,
            employee_id: event.employee_id,
            position: event.position,
            amount: CASH_REGISTER_DING_SOUND_AMOUNT,
        });

        deter_events.send(CashRegisterBaboonHawkDeterEvent {
            source: event.cash_register_entity,
            employee_id: event.employee_id,
            position: event.position,
        });
    }
}

fn cash_register_emit_movement_noise(
    mut movement_events: EventWriter<CashRegisterMovementSoundEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut cash_registers: Query<
        (
            Entity,
            &CashRegisterHeldBy,
            &SimPosition,
            &mut CashRegisterActivationState,
        ),
        (With<CashRegister>, Changed<SimPosition>),
    >,
) {
    for (entity, held_by, position, mut activation_state) in &mut cash_registers {
        if !held_by.is_held {
            continue;
        }

        activation_state.movement_sound_count =
            activation_state.movement_sound_count.wrapping_add(1);

        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: CASH_REGISTER_MOVEMENT_SOUND_AMOUNT,
        });

        movement_events.send(CashRegisterMovementSoundEvent {
            source: entity,
            employee_id: held_by.employee_id,
            position: *position,
            amount: CASH_REGISTER_MOVEMENT_SOUND_AMOUNT,
        });
    }
}

fn cash_register_transport_risk_bridge(
    mut storm_events: EventWriter<CashRegisterStormTransportRiskEvent>,
    mut delayed_events: EventWriter<CashRegisterDelayedTransportRiskEvent>,
    mut early_events: EventWriter<CashRegisterEarlyTransportSaferEvent>,
    mut cash_registers: Query<(
        Entity,
        &CashRegister,
        &CashRegisterScrap,
        &CashRegisterHeldBy,
        &mut CashRegisterTransportState,
    )>,
) {
    for (entity, cash_register, scrap, held_by, mut transport_state) in &mut cash_registers {
        if !held_by.is_held {
            continue;
        }

        if transport_state.stormy_weather
            && scrap.conductive
            && !transport_state.storm_risk_reported
        {
            transport_state.storm_risk_reported = true;
            storm_events.send(CashRegisterStormTransportRiskEvent {
                cash_register_entity: entity,
                cash_register_stable_id: cash_register.stable_id,
                employee_id: held_by.employee_id,
                weight: scrap.weight,
                conductive: scrap.conductive,
            });
        }

        if transport_state.delayed_late_run && !transport_state.delayed_risk_reported {
            transport_state.delayed_risk_reported = true;
            delayed_events.send(CashRegisterDelayedTransportRiskEvent {
                cash_register_entity: entity,
                cash_register_stable_id: cash_register.stable_id,
                employee_id: held_by.employee_id,
            });
        }

        if !transport_state.company_cruiser_available && !transport_state.early_transport_reported
        {
            transport_state.early_transport_reported = true;
            early_events.send(CashRegisterEarlyTransportSaferEvent {
                cash_register_entity: entity,
                cash_register_stable_id: cash_register.stable_id,
                employee_id: held_by.employee_id,
            });
        }
    }
}

fn cash_register_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<CashRegisterSoldEvent>,
    cash_registers: Query<(&CashRegister, &CashRegisterScrap)>,
) {
    for event in sell_events.read() {
        for (cash_register, scrap) in &cash_registers {
            if cash_register.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(CashRegisterSoldEvent {
                cash_register_stable_id: cash_register.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn cash_register_checksum(
    mut checksum: ResMut<SimChecksumState>,
    cash_registers: Query<(
        &CashRegister,
        &CashRegisterScrap,
        &SimPosition,
        &CashRegisterHeldBy,
        &CashRegisterActivationState,
        &CashRegisterTransportState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, CASH_REGISTER_ID);
    accumulate_str(&mut checksum, 0x1001, CASH_REGISTER_NAME);
    accumulate_str(&mut checksum, 0x1002, CASH_REGISTER_TYPE);
    accumulate_str(&mut checksum, 0x1003, CASH_REGISTER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CASH_REGISTER_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, CASH_REGISTER_SELL);

    checksum.accumulate(CASH_REGISTER_SOURCE_REVISION as u64);
    checksum.accumulate(CASH_REGISTER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CASH_REGISTER_WEIGHT.to_bits() as u64);
    checksum.accumulate(CASH_REGISTER_CONDUCTIVE as u64);
    checksum.accumulate(CASH_REGISTER_TWO_HANDED as u64);
    checksum.accumulate(CASH_REGISTER_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(CASH_REGISTER_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(CASH_REGISTER_PAGE_ID as u64);
    checksum.accumulate(CASH_REGISTER_DING_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(CASH_REGISTER_MOVEMENT_SOUND_AMOUNT.to_bits() as u64);

    for dependency in CASH_REGISTER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in CASH_REGISTER_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in CASH_REGISTER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (cash_register, scrap, position, held_by, activation_state, transport_state) in
        &cash_registers
    {
        checksum.accumulate(cash_register.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(activation_state.lever_pulled as u64);
        checksum.accumulate(activation_state.drawer_open as u64);
        checksum.accumulate(activation_state.activations);
        checksum.accumulate(activation_state.movement_sound_count);
        checksum.accumulate(transport_state.stormy_weather as u64);
        checksum.accumulate(transport_state.delayed_late_run as u64);
        checksum.accumulate(transport_state.company_cruiser_available as u64);
        checksum.accumulate(transport_state.storm_risk_reported as u64);
        checksum.accumulate(transport_state.delayed_risk_reported as u64);
        checksum.accumulate(transport_state.early_transport_reported as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}