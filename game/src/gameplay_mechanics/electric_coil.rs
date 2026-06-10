// Sources: vault/gameplay_mechanics/electric_coil.md
use bevy::prelude::*;
use fixed::types::I16F16;

use crate::sim::{SimChecksumState, SimTick};

pub const ELECTRIC_COIL_ID: &str = "electric_coil";
pub const ELECTRIC_COIL_NAME: &str = "Electric Coil";
pub const ELECTRIC_COIL_TYPE: &str = "gameplay_mechanics";
pub const ELECTRIC_COIL_SUBTYPE: &str = "device";
pub const ELECTRIC_COIL_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Electric_Coil";
pub const ELECTRIC_COIL_SOURCE_REVISION: u32 = 19222;
pub const ELECTRIC_COIL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ELECTRIC_COIL_CONFIDENCE_BASIS_POINTS: u16 = 95;

pub const ELECTRIC_COIL_FULL_CHARGE: I16F16 = I16F16::from_bits(1 << 16);

pub const ELECTRIC_COIL_DEPENDS_ON: [&str; 11] = [
    "ship",
    "flashlight",
    "pro_flashlight",
    "laser_pointer",
    "walkie_talkie",
    "hairdryer",
    "jetpack",
    "zap_gun",
    "boombox",
    "items",
    "scrap",
];

pub const ELECTRIC_COIL_RECHARGEABLE_ITEMS: [&str; 8] = [
    "flashlight",
    "pro_flashlight",
    "laser_pointer",
    "walkie_talkie",
    "hairdryer",
    "jetpack",
    "zap_gun",
    "boombox",
];

pub const ELECTRIC_COIL_RULES: [ElectricCoilBehaviorRule; 6] = [
    ElectricCoilBehaviorRule {
        condition: "only items with a battery can be used at the electric_coil",
        outcome: "the electric_coil accepts only battery-powered items",
    },
    ElectricCoilBehaviorRule {
        condition: "a battery-powered item is placed at the electric_coil",
        outcome: "it recharges instantly",
    },
    ElectricCoilBehaviorRule {
        condition: "the held item is one of the rechargeable items listed for the electric_coil",
        outcome: "it can be charged there",
    },
    ElectricCoilBehaviorRule {
        condition: "the held item is not battery-powered",
        outcome: "the electric_coil does not accept it",
    },
    ElectricCoilBehaviorRule {
        condition: "the player is holding a chargeable item, switches to a different inventory slot, and quickly presses the charge button",
        outcome: "the character still plays the charging animation without the sound effect",
    },
    ElectricCoilBehaviorRule {
        condition: "the player is holding an item that is not normally chargeable, then switches to a chargeable item",
        outcome: "the item is briefly unable to charge",
    },
];

pub const ELECTRIC_COIL_STRATEGY_RULES: [ElectricCoilBehaviorRule; 1] =
    [ElectricCoilBehaviorRule {
        condition: "you need to restore battery-powered gear quickly",
        outcome: "use the electric_coil on the ship instead of waiting for gradual recharge",
    }];

pub const ELECTRIC_COIL_NOTE_RULES: [ElectricCoilBehaviorRule; 2] = [
    ElectricCoilBehaviorRule {
        condition: "the charge-button slot-swap behavior occurs",
        outcome: "it is likely a bug rather than intended behavior",
    },
    ElectricCoilBehaviorRule {
        condition: "the inverse slot-swap behavior occurs",
        outcome: "it is likely a bug rather than intended behavior",
    },
];

pub struct ElectricCoilPlugin;

impl Plugin for ElectricCoilPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ElectricCoilState>()
            .add_event::<ElectricCoilChargeRequestEvent>()
            .add_event::<ElectricCoilChargedEvent>()
            .add_event::<ElectricCoilRejectedEvent>()
            .add_event::<ElectricCoilSlotSwapAnimationEvent>()
            .add_event::<ElectricCoilBriefChargeLockoutEvent>()
            .add_systems(
                FixedUpdate,
                (
                    electric_coil_process_charge_requests,
                    electric_coil_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElectricCoilBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElectricCoil;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryPoweredItem {
    pub item_id: &'static str,
    pub current_charge: I16F16,
    pub max_charge: I16F16,
    pub can_charge_at_electric_coil: bool,
}

impl Default for BatteryPoweredItem {
    fn default() -> Self {
        Self {
            item_id: "",
            current_charge: ELECTRIC_COIL_FULL_CHARGE,
            max_charge: ELECTRIC_COIL_FULL_CHARGE,
            can_charge_at_electric_coil: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElectricCoilChargeLockout {
    pub remaining_ticks: u32,
}

#[derive(Bundle, Debug, Clone, Default)]
pub struct ElectricCoilBundle {
    pub electric_coil: ElectricCoil,
}

#[derive(Bundle, Debug, Clone, Default)]
pub struct BatteryPoweredItemBundle {
    pub battery: BatteryPoweredItem,
    pub lockout: ElectricCoilChargeLockout,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ElectricCoilState {
    pub accepted_requests: u64,
    pub rejected_requests: u64,
    pub slot_swap_animation_events: u64,
    pub brief_lockout_events: u64,
    pub last_request_id: u64,
    pub last_item_id: &'static str,
}

impl Default for ElectricCoilState {
    fn default() -> Self {
        Self {
            accepted_requests: 0,
            rejected_requests: 0,
            slot_swap_animation_events: 0,
            brief_lockout_events: 0,
            last_request_id: 0,
            last_item_id: "",
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElectricCoilChargeRequestEvent {
    pub request_id: u64,
    pub item_entity: Entity,
    pub held_item_id: &'static str,
    pub previous_slot_item_id: &'static str,
    pub slot_swapped_this_tick: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElectricCoilChargedEvent {
    pub request_id: u64,
    pub item_entity: Entity,
    pub item_id: &'static str,
    pub charge_after: I16F16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElectricCoilRejectedEvent {
    pub request_id: u64,
    pub item_entity: Entity,
    pub item_id: &'static str,
    pub reason: ElectricCoilRejectReason,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElectricCoilSlotSwapAnimationEvent {
    pub request_id: u64,
    pub item_entity: Entity,
    pub previous_slot_item_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElectricCoilBriefChargeLockoutEvent {
    pub request_id: u64,
    pub item_entity: Entity,
    pub item_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectricCoilRejectReason {
    NotBatteryPowered,
    NotRechargeableAtElectricCoil,
    BriefSlotSwapLockout,
}

pub fn electric_coil_rechargeable_items() -> &'static [&'static str] {
    &ELECTRIC_COIL_RECHARGEABLE_ITEMS
}

pub fn electric_coil_can_charge_item(item_id: &str) -> bool {
    ELECTRIC_COIL_RECHARGEABLE_ITEMS.contains(&item_id)
}

pub fn electric_coil_full_charge() -> I16F16 {
    ELECTRIC_COIL_FULL_CHARGE
}

fn electric_coil_process_charge_requests(
    mut requests: EventReader<ElectricCoilChargeRequestEvent>,
    mut charged_events: EventWriter<ElectricCoilChargedEvent>,
    mut rejected_events: EventWriter<ElectricCoilRejectedEvent>,
    mut animation_events: EventWriter<ElectricCoilSlotSwapAnimationEvent>,
    mut lockout_events: EventWriter<ElectricCoilBriefChargeLockoutEvent>,
    mut items: Query<(&mut BatteryPoweredItem, &mut ElectricCoilChargeLockout)>,
    mut state: ResMut<ElectricCoilState>,
) {
    for request in requests.read() {
        state.last_request_id = request.request_id;

        let Ok((mut battery, mut lockout)) = items.get_mut(request.item_entity) else {
            state.rejected_requests = state.rejected_requests.wrapping_add(1);
            rejected_events.send(ElectricCoilRejectedEvent {
                request_id: request.request_id,
                item_entity: request.item_entity,
                item_id: request.held_item_id,
                reason: ElectricCoilRejectReason::NotBatteryPowered,
            });
            continue;
        };

        state.last_item_id = battery.item_id;

        if request.slot_swapped_this_tick
            && electric_coil_can_charge_item(request.previous_slot_item_id)
            && !electric_coil_can_charge_item(request.held_item_id)
        {
            state.slot_swap_animation_events = state.slot_swap_animation_events.wrapping_add(1);
            animation_events.send(ElectricCoilSlotSwapAnimationEvent {
                request_id: request.request_id,
                item_entity: request.item_entity,
                previous_slot_item_id: request.previous_slot_item_id,
            });
        }

        if request.slot_swapped_this_tick
            && !electric_coil_can_charge_item(request.previous_slot_item_id)
            && electric_coil_can_charge_item(request.held_item_id)
        {
            lockout.remaining_ticks = lockout.remaining_ticks.max(1);
            state.brief_lockout_events = state.brief_lockout_events.wrapping_add(1);
            lockout_events.send(ElectricCoilBriefChargeLockoutEvent {
                request_id: request.request_id,
                item_entity: request.item_entity,
                item_id: battery.item_id,
            });
        }

        if lockout.remaining_ticks > 0 {
            lockout.remaining_ticks -= 1;
            state.rejected_requests = state.rejected_requests.wrapping_add(1);
            rejected_events.send(ElectricCoilRejectedEvent {
                request_id: request.request_id,
                item_entity: request.item_entity,
                item_id: battery.item_id,
                reason: ElectricCoilRejectReason::BriefSlotSwapLockout,
            });
            continue;
        }

        if !battery.can_charge_at_electric_coil || !electric_coil_can_charge_item(battery.item_id) {
            state.rejected_requests = state.rejected_requests.wrapping_add(1);
            rejected_events.send(ElectricCoilRejectedEvent {
                request_id: request.request_id,
                item_entity: request.item_entity,
                item_id: battery.item_id,
                reason: ElectricCoilRejectReason::NotRechargeableAtElectricCoil,
            });
            continue;
        }

        battery.current_charge = battery.max_charge;
        state.accepted_requests = state.accepted_requests.wrapping_add(1);
        charged_events.send(ElectricCoilChargedEvent {
            request_id: request.request_id,
            item_entity: request.item_entity,
            item_id: battery.item_id,
            charge_after: battery.current_charge,
        });
    }
}

fn electric_coil_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<ElectricCoilState>,
    items: Query<(&BatteryPoweredItem, &ElectricCoilChargeLockout)>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ELECTRIC_COIL_SOURCE_REVISION as u64);
    checksum.accumulate(ELECTRIC_COIL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ELECTRIC_COIL_FULL_CHARGE.to_bits() as u64);

    for dependency in ELECTRIC_COIL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for item_id in ELECTRIC_COIL_RECHARGEABLE_ITEMS {
        accumulate_str(&mut checksum, 0x1100, item_id);
    }

    for rule in ELECTRIC_COIL_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for rule in ELECTRIC_COIL_STRATEGY_RULES {
        accumulate_str(&mut checksum, 0x2100, rule.condition);
        accumulate_str(&mut checksum, 0x2101, rule.outcome);
    }

    for rule in ELECTRIC_COIL_NOTE_RULES {
        accumulate_str(&mut checksum, 0x2200, rule.condition);
        accumulate_str(&mut checksum, 0x2201, rule.outcome);
    }

    checksum.accumulate(state.accepted_requests);
    checksum.accumulate(state.rejected_requests);
    checksum.accumulate(state.slot_swap_animation_events);
    checksum.accumulate(state.brief_lockout_events);
    checksum.accumulate(state.last_request_id);
    accumulate_str(&mut checksum, 0x3000, state.last_item_id);

    for (battery, lockout) in items.iter() {
        accumulate_str(&mut checksum, 0x4000, battery.item_id);
        checksum.accumulate(battery.current_charge.to_bits() as u64);
        checksum.accumulate(battery.max_charge.to_bits() as u64);
        checksum.accumulate(u64::from(battery.can_charge_at_electric_coil));
        checksum.accumulate(lockout.remaining_ticks as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}