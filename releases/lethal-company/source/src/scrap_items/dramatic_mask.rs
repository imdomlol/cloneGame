// Sources: vault/scrap_items/dramatic_mask.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick};

pub const DRAMATIC_MASK_ID: &str = "dramatic_mask";
pub const DRAMATIC_MASK_NAME: &str = "Dramatic Mask";
pub const DRAMATIC_MASK_TYPE: &str = "scrap_items";
pub const DRAMATIC_MASK_SUBTYPE: &str = "mask";
pub const DRAMATIC_MASK_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Dramatic_Mask";
pub const DRAMATIC_MASK_SOURCE_REVISION: u32 = 21196;
pub const DRAMATIC_MASK_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const DRAMATIC_MASK_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const DRAMATIC_MASK_EFFECTS: &str = "Converts the wearer into a Masked if worn.";
pub const DRAMATIC_MASK_WEIGHT: I32F32 = I32F32::lit("11");
pub const DRAMATIC_MASK_CONDUCTIVE: bool = false;
pub const DRAMATIC_MASK_MIN_VALUE: I32F32 = I32F32::lit("28");
pub const DRAMATIC_MASK_MAX_VALUE: I32F32 = I32F32::lit("51");
pub const DRAMATIC_MASK_TWO_HANDED: bool = false;
pub const DRAMATIC_MASK_HOLD_TO_FACE_TICKS: u16 = 125;
pub const DRAMATIC_MASK_CONVERSION_CHANCE_PERCENT: u32 = 65;
pub const DRAMATIC_MASK_CONVERSION_ROLL_MODULUS: u32 = 100;
pub const DRAMATIC_MASK_CONVERSION_ROLL_SALT: u64 = 0x6472_616d_6174_6963;

pub const DRAMATIC_MASK_DEPENDS_ON: [&str; 1] = ["masked"];

pub const DRAMATIC_MASK_BEHAVIORAL_MECHANICS: [DramaticMaskBehaviorRule; 4] = [
    DramaticMaskBehaviorRule {
        condition: "the mask is held to the face for 5 seconds",
        outcome: "it has a 65% chance to convert the wearer into a masked",
    },
    DramaticMaskBehaviorRule {
        condition: "conversion succeeds",
        outcome: "the wearer dies as part of the transformation",
    },
    DramaticMaskBehaviorRule {
        condition: "the item is only carried and not worn",
        outcome: "it does not trigger possession or conversion",
    },
    DramaticMaskBehaviorRule {
        condition: "the wearer is teleported within 5 seconds after activation",
        outcome: "the conversion can be interrupted",
    },
];

pub struct DramaticMaskPlugin;

impl Plugin for DramaticMaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDramaticMaskEvent>()
            .add_event::<DramaticMaskActivationEvent>()
            .add_event::<DramaticMaskConversionRollEvent>()
            .add_event::<DramaticMaskConversionStartedEvent>()
            .add_event::<DramaticMaskConversionInterruptedEvent>()
            .add_event::<DramaticMaskConversionSucceededEvent>()
            .add_event::<DramaticMaskSoldEvent>()
            .add_event::<DramaticMaskWearerTeleportedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_dramatic_mask,
                    dramatic_mask_pickup_item_bar_bridge,
                    dramatic_mask_use_from_item_bar,
                    dramatic_mask_advance_conversion,
                    dramatic_mask_interrupt_teleported_wearers,
                    dramatic_mask_sell_bridge,
                    dramatic_mask_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DramaticMaskBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DramaticMask {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DramaticMaskScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for DramaticMaskScrap {
    fn default() -> Self {
        Self {
            min_value: DRAMATIC_MASK_MIN_VALUE,
            max_value: DRAMATIC_MASK_MAX_VALUE,
            weight: DRAMATIC_MASK_WEIGHT,
            conductive: DRAMATIC_MASK_CONDUCTIVE,
            two_handed: DRAMATIC_MASK_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DramaticMaskHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DramaticMaskWearState {
    pub held_to_face: bool,
    pub wearer_id: u64,
    pub activation_tick: u64,
    pub worn_ticks: u16,
    pub interrupted: bool,
    pub conversion_resolved: bool,
}

impl Default for DramaticMaskWearState {
    fn default() -> Self {
        Self {
            held_to_face: false,
            wearer_id: 0,
            activation_tick: 0,
            worn_ticks: 0,
            interrupted: false,
            conversion_resolved: false,
        }
    }
}

#[derive(Bundle)]
pub struct DramaticMaskBundle {
    pub name: Name,
    pub dramatic_mask: DramaticMask,
    pub scrap: DramaticMaskScrap,
    pub position: SimPosition,
    pub held_by: DramaticMaskHeldBy,
    pub wear_state: DramaticMaskWearState,
}

impl DramaticMaskBundle {
    pub fn new(event: SpawnDramaticMaskEvent) -> Self {
        Self {
            name: Name::new(DRAMATIC_MASK_NAME),
            dramatic_mask: DramaticMask {
                stable_id: event.stable_id,
            },
            scrap: DramaticMaskScrap::default(),
            position: event.position,
            held_by: DramaticMaskHeldBy::default(),
            wear_state: DramaticMaskWearState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnDramaticMaskEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskActivationEvent {
    pub mask_entity: Entity,
    pub mask_stable_id: u64,
    pub employee_id: u64,
    pub activation_tick: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskConversionRollEvent {
    pub mask_entity: Entity,
    pub mask_stable_id: u64,
    pub employee_id: u64,
    pub roll_percent: u32,
    pub succeeded: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskConversionStartedEvent {
    pub mask_entity: Entity,
    pub mask_stable_id: u64,
    pub employee_id: u64,
    pub target_entity_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskConversionInterruptedEvent {
    pub mask_entity: Entity,
    pub mask_stable_id: u64,
    pub employee_id: u64,
    pub interrupt_tick: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskConversionSucceededEvent {
    pub mask_entity: Entity,
    pub mask_stable_id: u64,
    pub employee_id: u64,
    pub target_entity_id: &'static str,
    pub wearer_dies: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskSoldEvent {
    pub mask_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DramaticMaskWearerTeleportedEvent {
    pub employee_id: u64,
    pub teleport_tick: u64,
}

pub fn dramatic_mask_value_range() -> (I32F32, I32F32) {
    (DRAMATIC_MASK_MIN_VALUE, DRAMATIC_MASK_MAX_VALUE)
}

fn spawn_dramatic_mask(mut commands: Commands, mut events: EventReader<SpawnDramaticMaskEvent>) {
    for event in events.read() {
        commands.spawn(DramaticMaskBundle::new(*event));
    }
}

fn dramatic_mask_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    masks: Query<(&DramaticMask, &DramaticMaskHeldBy), Changed<DramaticMaskHeldBy>>,
) {
    for (mask, held_by) in &masks {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: DRAMATIC_MASK_ID,
                two_handed: DRAMATIC_MASK_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = mask.stable_id;
        }
    }
}

fn dramatic_mask_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut activation_events: EventWriter<DramaticMaskActivationEvent>,
    mut start_events: EventWriter<DramaticMaskConversionStartedEvent>,
    mut masks: Query<(Entity, &DramaticMask, &DramaticMaskHeldBy, &mut DramaticMaskWearState)>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != DRAMATIC_MASK_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, mask, held_by, mut wear_state) in &mut masks {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            wear_state.held_to_face = true;
            wear_state.wearer_id = event.employee_id;
            wear_state.activation_tick = tick.0;
            wear_state.worn_ticks = 0;
            wear_state.interrupted = false;
            wear_state.conversion_resolved = false;

            activation_events.send(DramaticMaskActivationEvent {
                mask_entity: entity,
                mask_stable_id: mask.stable_id,
                employee_id: event.employee_id,
                activation_tick: tick.0,
            });

            start_events.send(DramaticMaskConversionStartedEvent {
                mask_entity: entity,
                mask_stable_id: mask.stable_id,
                employee_id: event.employee_id,
                target_entity_id: "masked",
            });
        }
    }
}

fn dramatic_mask_advance_conversion(
    mut roll_events: EventWriter<DramaticMaskConversionRollEvent>,
    mut succeeded_events: EventWriter<DramaticMaskConversionSucceededEvent>,
    mut masks: Query<(Entity, &DramaticMask, &mut DramaticMaskWearState)>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for (entity, mask, mut wear_state) in &mut masks {
        if !wear_state.held_to_face || wear_state.interrupted || wear_state.conversion_resolved {
            continue;
        }

        wear_state.worn_ticks = wear_state.worn_ticks.saturating_add(1);

        if wear_state.worn_ticks < DRAMATIC_MASK_HOLD_TO_FACE_TICKS {
            continue;
        }

        let salt = DRAMATIC_MASK_CONVERSION_ROLL_SALT ^ mask.stable_id ^ wear_state.wearer_id;
        let mut rng = tick_rng(seed.0, tick.0, salt);
        let roll_percent = rng.next_u32() % DRAMATIC_MASK_CONVERSION_ROLL_MODULUS;
        let succeeded = roll_percent < DRAMATIC_MASK_CONVERSION_CHANCE_PERCENT;

        wear_state.conversion_resolved = true;
        wear_state.held_to_face = false;

        roll_events.send(DramaticMaskConversionRollEvent {
            mask_entity: entity,
            mask_stable_id: mask.stable_id,
            employee_id: wear_state.wearer_id,
            roll_percent,
            succeeded,
        });

        if succeeded {
            succeeded_events.send(DramaticMaskConversionSucceededEvent {
                mask_entity: entity,
                mask_stable_id: mask.stable_id,
                employee_id: wear_state.wearer_id,
                target_entity_id: "masked",
                wearer_dies: true,
            });
        }
    }
}

fn dramatic_mask_interrupt_teleported_wearers(
    mut teleport_events: EventReader<DramaticMaskWearerTeleportedEvent>,
    mut interrupted_events: EventWriter<DramaticMaskConversionInterruptedEvent>,
    mut masks: Query<(Entity, &DramaticMask, &mut DramaticMaskWearState)>,
) {
    for teleport_event in teleport_events.read() {
        for (entity, mask, mut wear_state) in &mut masks {
            if !wear_state.held_to_face || wear_state.wearer_id != teleport_event.employee_id {
                continue;
            }

            let elapsed_ticks = teleport_event
                .teleport_tick
                .saturating_sub(wear_state.activation_tick);

            if elapsed_ticks > DRAMATIC_MASK_HOLD_TO_FACE_TICKS as u64 {
                continue;
            }

            wear_state.held_to_face = false;
            wear_state.interrupted = true;
            wear_state.conversion_resolved = true;

            interrupted_events.send(DramaticMaskConversionInterruptedEvent {
                mask_entity: entity,
                mask_stable_id: mask.stable_id,
                employee_id: teleport_event.employee_id,
                interrupt_tick: teleport_event.teleport_tick,
            });
        }
    }
}

fn dramatic_mask_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<DramaticMaskSoldEvent>,
    masks: Query<(&DramaticMask, &DramaticMaskScrap)>,
) {
    for event in sell_events.read() {
        for (mask, scrap) in &masks {
            if mask.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(DramaticMaskSoldEvent {
                mask_stable_id: mask.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn dramatic_mask_checksum(
    mut checksum: ResMut<SimChecksumState>,
    masks: Query<(
        &DramaticMask,
        &DramaticMaskScrap,
        &SimPosition,
        &DramaticMaskHeldBy,
        &DramaticMaskWearState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, DRAMATIC_MASK_ID);
    accumulate_str(&mut checksum, 0x1001, DRAMATIC_MASK_NAME);
    accumulate_str(&mut checksum, 0x1002, DRAMATIC_MASK_TYPE);
    accumulate_str(&mut checksum, 0x1003, DRAMATIC_MASK_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, DRAMATIC_MASK_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, DRAMATIC_MASK_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, DRAMATIC_MASK_EXTRACTED_AT);

    checksum.accumulate(DRAMATIC_MASK_SOURCE_REVISION as u64);
    checksum.accumulate(DRAMATIC_MASK_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(DRAMATIC_MASK_WEIGHT.to_bits() as u64);
    checksum.accumulate(DRAMATIC_MASK_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(DRAMATIC_MASK_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(DRAMATIC_MASK_CONDUCTIVE as u64);
    checksum.accumulate(DRAMATIC_MASK_TWO_HANDED as u64);
    checksum.accumulate(DRAMATIC_MASK_HOLD_TO_FACE_TICKS as u64);
    checksum.accumulate(DRAMATIC_MASK_CONVERSION_CHANCE_PERCENT as u64);
    checksum.accumulate(DRAMATIC_MASK_CONVERSION_ROLL_MODULUS as u64);

    for dependency in DRAMATIC_MASK_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in DRAMATIC_MASK_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (mask, scrap, position, held_by, wear_state) in &masks {
        checksum.accumulate(mask.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(wear_state.held_to_face as u64);
        checksum.accumulate(wear_state.wearer_id);
        checksum.accumulate(wear_state.activation_tick);
        checksum.accumulate(wear_state.worn_ticks as u64);
        checksum.accumulate(wear_state.interrupted as u64);
        checksum.accumulate(wear_state.conversion_resolved as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}