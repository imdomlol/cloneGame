// Sources: vault/scrap_items/heart.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition};

pub const HEART_ID: &str = "heart";
pub const HEART_NAME: &str = "Heart";
pub const HEART_TYPE: &str = "scrap_items";
pub const HEART_SUBTYPE: &str = "scrap";
pub const HEART_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Heart";
pub const HEART_SOURCE_REVISION: u32 = 20387;
pub const HEART_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const HEART_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const HEART_EFFECTS: &str = "Pulses a heartbeat based on fear level";
pub const HEART_WEIGHT: I32F32 = I32F32::lit("31");
pub const HEART_CONDUCTIVE: bool = false;
pub const HEART_MIN_VALUE: I32F32 = I32F32::lit("24");
pub const HEART_MAX_VALUE: I32F32 = I32F32::lit("99");
pub const HEART_TWO_HANDED: bool = true;
pub const HEART_BASE_PULSE_INTERVAL_TICKS: u16 = 40;
pub const HEART_SPOTTED_PULSE_INTERVAL_TICKS: u16 = 20;
pub const HEART_FEAR_PULSE_INTERVAL_TICKS: u16 = 10;
pub const HEART_BASE_NOISE_AMOUNT: I32F32 = I32F32::lit("8");
pub const HEART_SPOTTED_NOISE_AMOUNT: I32F32 = I32F32::lit("16");
pub const HEART_FEAR_NOISE_AMOUNT: I32F32 = I32F32::lit("24");

pub const HEART_DEPENDS_ON: [&str; 2] = ["dine", "the_company"];
pub const HEART_ALLOWED_SPAWN_MOONS: [&str; 1] = ["dine"];

pub const HEART_BEHAVIORAL_MECHANICS: [HeartBehaviorRule; 5] = [
    HeartBehaviorRule {
        condition: "the item is held",
        outcome: "it emits a heartbeat pulse",
    },
    HeartBehaviorRule {
        condition: "the holder is spotted",
        outcome: "the heartbeat rate increases",
    },
    HeartBehaviorRule {
        condition: "the holder's fear is triggered",
        outcome: "the heartbeat becomes more noticeable",
    },
    HeartBehaviorRule {
        condition: "the item appears in the loot pool",
        outcome: "it can spawn only on dine",
    },
    HeartBehaviorRule {
        condition: "the item is sold",
        outcome: "it is exchanged with the_company for credits",
    },
];

pub struct HeartPlugin;

impl Plugin for HeartPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHeartEvent>()
            .add_event::<HeartPulseEvent>()
            .add_event::<HeartHolderSpottedEvent>()
            .add_event::<HeartHolderFearTriggeredEvent>()
            .add_event::<HeartSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_heart,
                    heart_pickup_item_bar_bridge,
                    heart_holder_spotted,
                    heart_holder_fear_triggered,
                    heart_pulse,
                    heart_sell_bridge,
                    heart_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeartBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heart {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeartScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for HeartScrap {
    fn default() -> Self {
        Self {
            min_value: HEART_MIN_VALUE,
            max_value: HEART_MAX_VALUE,
            weight: HEART_WEIGHT,
            conductive: HEART_CONDUCTIVE,
            two_handed: HEART_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeartHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeartPulseState {
    pub pulse_timer_ticks: u16,
    pub pulse_interval_ticks: u16,
    pub pulse_count: u64,
    pub current_noise_amount: I32F32,
    pub holder_spotted: bool,
    pub holder_fear_triggered: bool,
}

impl Default for HeartPulseState {
    fn default() -> Self {
        Self {
            pulse_timer_ticks: HEART_BASE_PULSE_INTERVAL_TICKS,
            pulse_interval_ticks: HEART_BASE_PULSE_INTERVAL_TICKS,
            pulse_count: 0,
            current_noise_amount: HEART_BASE_NOISE_AMOUNT,
            holder_spotted: false,
            holder_fear_triggered: false,
        }
    }
}

#[derive(Bundle)]
pub struct HeartBundle {
    pub name: Name,
    pub heart: Heart,
    pub scrap: HeartScrap,
    pub position: SimPosition,
    pub held_by: HeartHeldBy,
    pub pulse_state: HeartPulseState,
}

impl HeartBundle {
    pub fn new(event: SpawnHeartEvent) -> Self {
        Self {
            name: Name::new(HEART_NAME),
            heart: Heart {
                stable_id: event.stable_id,
            },
            scrap: HeartScrap::default(),
            position: event.position,
            held_by: HeartHeldBy::default(),
            pulse_state: HeartPulseState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnHeartEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartPulseEvent {
    pub heart_entity: Entity,
    pub heart_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
    pub pulse_count: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartHolderSpottedEvent {
    pub heart_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartHolderFearTriggeredEvent {
    pub heart_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartSoldEvent {
    pub heart_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn heart_value_range() -> (I32F32, I32F32) {
    (HEART_MIN_VALUE, HEART_MAX_VALUE)
}

pub fn heart_can_spawn_on_moon(moon: &str) -> bool {
    HEART_ALLOWED_SPAWN_MOONS.contains(&moon)
}

fn spawn_heart(mut commands: Commands, mut events: EventReader<SpawnHeartEvent>) {
    for event in events.read() {
        commands.spawn(HeartBundle::new(*event));
    }
}

fn heart_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    hearts: Query<(&Heart, &HeartHeldBy), Changed<HeartHeldBy>>,
) {
    for (heart, held_by) in &hearts {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: HEART_ID,
                two_handed: HEART_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = heart.stable_id;
        }
    }
}

fn heart_holder_spotted(
    mut events: EventReader<HeartHolderSpottedEvent>,
    mut hearts: Query<(&Heart, &HeartHeldBy, &mut HeartPulseState)>,
) {
    for event in events.read() {
        for (heart, held_by, mut pulse_state) in &mut hearts {
            if heart.stable_id != event.heart_stable_id || held_by.employee_id != event.employee_id {
                continue;
            }

            pulse_state.holder_spotted = true;
            pulse_state.pulse_interval_ticks = HEART_SPOTTED_PULSE_INTERVAL_TICKS;
            pulse_state.current_noise_amount = HEART_SPOTTED_NOISE_AMOUNT;
            if pulse_state.pulse_timer_ticks > pulse_state.pulse_interval_ticks {
                pulse_state.pulse_timer_ticks = pulse_state.pulse_interval_ticks;
            }
        }
    }
}

fn heart_holder_fear_triggered(
    mut events: EventReader<HeartHolderFearTriggeredEvent>,
    mut hearts: Query<(&Heart, &HeartHeldBy, &mut HeartPulseState)>,
) {
    for event in events.read() {
        for (heart, held_by, mut pulse_state) in &mut hearts {
            if heart.stable_id != event.heart_stable_id || held_by.employee_id != event.employee_id {
                continue;
            }

            pulse_state.holder_fear_triggered = true;
            pulse_state.pulse_interval_ticks = HEART_FEAR_PULSE_INTERVAL_TICKS;
            pulse_state.current_noise_amount = HEART_FEAR_NOISE_AMOUNT;
            if pulse_state.pulse_timer_ticks > pulse_state.pulse_interval_ticks {
                pulse_state.pulse_timer_ticks = pulse_state.pulse_interval_ticks;
            }
        }
    }
}

fn heart_pulse(
    mut pulse_events: EventWriter<HeartPulseEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut hearts: Query<(Entity, &Heart, &HeartHeldBy, &SimPosition, &mut HeartPulseState)>,
) {
    for (entity, heart, held_by, position, mut pulse_state) in &mut hearts {
        if !held_by.is_held {
            pulse_state.pulse_timer_ticks = pulse_state.pulse_interval_ticks;
            continue;
        }

        if pulse_state.pulse_timer_ticks > 0 {
            pulse_state.pulse_timer_ticks -= 1;
            continue;
        }

        pulse_state.pulse_count = pulse_state.pulse_count.wrapping_add(1);
        pulse_state.pulse_timer_ticks = pulse_state.pulse_interval_ticks;

        pulse_events.send(HeartPulseEvent {
            heart_entity: entity,
            heart_stable_id: heart.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
            amount: pulse_state.current_noise_amount,
            pulse_count: pulse_state.pulse_count,
        });

        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: pulse_state.current_noise_amount,
        });
    }
}

fn heart_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<HeartSoldEvent>,
    hearts: Query<(&Heart, &HeartScrap)>,
) {
    for event in sell_events.read() {
        for (heart, scrap) in &hearts {
            if heart.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(HeartSoldEvent {
                heart_stable_id: heart.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn heart_checksum(
    mut checksum: ResMut<SimChecksumState>,
    hearts: Query<(&Heart, &HeartScrap, &SimPosition, &HeartHeldBy, &HeartPulseState)>,
) {
    accumulate_str(&mut checksum, 0x1000, HEART_ID);
    accumulate_str(&mut checksum, 0x1001, HEART_NAME);
    accumulate_str(&mut checksum, 0x1002, HEART_TYPE);
    accumulate_str(&mut checksum, 0x1003, HEART_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, HEART_EFFECTS);

    checksum.accumulate(HEART_SOURCE_REVISION as u64);
    checksum.accumulate(HEART_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(HEART_WEIGHT.to_bits() as u64);
    checksum.accumulate(HEART_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(HEART_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(HEART_CONDUCTIVE as u64);
    checksum.accumulate(HEART_TWO_HANDED as u64);
    checksum.accumulate(HEART_BASE_PULSE_INTERVAL_TICKS as u64);
    checksum.accumulate(HEART_SPOTTED_PULSE_INTERVAL_TICKS as u64);
    checksum.accumulate(HEART_FEAR_PULSE_INTERVAL_TICKS as u64);
    checksum.accumulate(HEART_BASE_NOISE_AMOUNT.to_bits() as u64);
    checksum.accumulate(HEART_SPOTTED_NOISE_AMOUNT.to_bits() as u64);
    checksum.accumulate(HEART_FEAR_NOISE_AMOUNT.to_bits() as u64);

    for dependency in HEART_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for moon in HEART_ALLOWED_SPAWN_MOONS {
        accumulate_str(&mut checksum, 0x3000, moon);
    }

    for rule in HEART_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (heart, scrap, position, held_by, pulse_state) in &hearts {
        checksum.accumulate(heart.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(pulse_state.pulse_timer_ticks as u64);
        checksum.accumulate(pulse_state.pulse_interval_ticks as u64);
        checksum.accumulate(pulse_state.pulse_count);
        checksum.accumulate(pulse_state.current_noise_amount.to_bits() as u64);
        checksum.accumulate(pulse_state.holder_spotted as u64);
        checksum.accumulate(pulse_state.holder_fear_triggered as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}