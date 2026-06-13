// Sources: vault/weapon_pages/yield_sign.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
};

pub const YIELD_SIGN_ID: &str = "yield_sign";
pub const YIELD_SIGN_NAME: &str = "Yield sign";
pub const YIELD_SIGN_TYPE: &str = "weapon_pages";
pub const YIELD_SIGN_SUBTYPE: &str = "scrap";
pub const YIELD_SIGN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Yield_Sign";
pub const YIELD_SIGN_SOURCE_REVISION: u32 = 21220;
pub const YIELD_SIGN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const YIELD_SIGN_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const YIELD_SIGN_EFFECTS: &str = "Can be used to attack employees and entities";
pub const YIELD_SIGN_WEIGHT: I32F32 = I32F32::lit("42");
pub const YIELD_SIGN_CONDUCTIVE: bool = true;
pub const YIELD_SIGN_MIN_VALUE: I32F32 = I32F32::lit("18");
pub const YIELD_SIGN_MAX_VALUE: I32F32 = I32F32::lit("35");
pub const YIELD_SIGN_TWO_HANDED: bool = false;
pub const YIELD_SIGN_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const YIELD_SIGN_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");
pub const YIELD_SIGN_COOLDOWN_SECONDS: I32F32 = I32F32::lit("1");

pub const YIELD_SIGN_BEHAVIORAL_MECHANICS: [YieldSignBehaviorRule; 7] = [
    YieldSignBehaviorRule {
        condition: "LMB is pressed",
        outcome: "swing the sign downward",
    },
    YieldSignBehaviorRule {
        condition: "the swing hits an employee",
        outcome: "it deals 20 damage",
    },
    YieldSignBehaviorRule {
        condition: "the swing hits an entity",
        outcome: "it deals 1 damage",
    },
    YieldSignBehaviorRule {
        condition: "swings are repeated",
        outcome: "the cooldown is about 1 second per attack",
    },
    YieldSignBehaviorRule {
        condition: "the item is held",
        outcome: "its weight is 42 and it is conductive",
    },
    YieldSignBehaviorRule {
        condition: "compared with a shovel",
        outcome: "it is not two-handed",
    },
    YieldSignBehaviorRule {
        condition: "read from the infobox",
        outcome: "its min value is 18 and its max value is 35",
    },
];

pub struct YieldSignPlugin;

impl Plugin for YieldSignPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnYieldSignEvent>()
            .add_event::<YieldSignSwingEvent>()
            .add_event::<YieldSignHitEvent>()
            .add_event::<YieldSignSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_yield_sign,
                    yield_sign_pickup_item_bar_bridge,
                    yield_sign_tick_cooldowns,
                    yield_sign_use_from_item_bar,
                    yield_sign_swing,
                    yield_sign_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YieldSignBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct YieldSign {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct YieldSignWeapon {
    pub weight: I32F32,
    pub conductive: bool,
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub two_handed: bool,
}

impl Default for YieldSignWeapon {
    fn default() -> Self {
        Self {
            weight: YIELD_SIGN_WEIGHT,
            conductive: YIELD_SIGN_CONDUCTIVE,
            min_value: YIELD_SIGN_MIN_VALUE,
            max_value: YIELD_SIGN_MAX_VALUE,
            two_handed: YIELD_SIGN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct YieldSignHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct YieldSignSwingState {
    pub cooldown_ticks_remaining: u32,
    pub swings: u64,
    pub hits: u64,
    pub last_used_tick: u64,
}

#[derive(Bundle)]
pub struct YieldSignBundle {
    pub name: Name,
    pub yield_sign: YieldSign,
    pub weapon: YieldSignWeapon,
    pub position: SimPosition,
    pub held_by: YieldSignHeldBy,
    pub swing_state: YieldSignSwingState,
}

impl YieldSignBundle {
    pub fn new(event: SpawnYieldSignEvent) -> Self {
        Self {
            name: Name::new(YIELD_SIGN_NAME),
            yield_sign: YieldSign {
                stable_id: event.stable_id,
            },
            weapon: YieldSignWeapon::default(),
            position: event.position,
            held_by: YieldSignHeldBy::default(),
            swing_state: YieldSignSwingState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnYieldSignEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct YieldSignSwingEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: YieldSignTargetKind,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct YieldSignHitEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Entity,
    pub target_kind: YieldSignTargetKind,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct YieldSignSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum YieldSignTargetKind {
    #[default]
    Entity = 0,
    Employee = 1,
}

pub fn yield_sign_damage_for_target(target_kind: YieldSignTargetKind) -> I32F32 {
    match target_kind {
        YieldSignTargetKind::Entity => YIELD_SIGN_ENTITY_DAMAGE,
        YieldSignTargetKind::Employee => YIELD_SIGN_EMPLOYEE_DAMAGE,
    }
}

fn spawn_yield_sign(mut commands: Commands, mut events: EventReader<SpawnYieldSignEvent>) {
    for event in events.read() {
        commands.spawn(YieldSignBundle::new(*event));
    }
}

fn yield_sign_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&YieldSign, &YieldSignHeldBy), Changed<YieldSignHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: YIELD_SIGN_ID,
                two_handed: YIELD_SIGN_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn yield_sign_tick_cooldowns(mut weapons: Query<&mut YieldSignSwingState, With<YieldSign>>) {
    for mut swing_state in &mut weapons {
        if swing_state.cooldown_ticks_remaining > 0 {
            swing_state.cooldown_ticks_remaining -= 1;
        }
    }
}

fn yield_sign_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut swing_events: EventWriter<YieldSignSwingEvent>,
    weapons: Query<(Entity, &YieldSignHeldBy, &SimPosition), With<YieldSign>>,
) {
    for event in item_events.read() {
        if event.item_id != YIELD_SIGN_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (weapon, held_by, position) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            swing_events.send(YieldSignSwingEvent {
                weapon,
                user: weapon,
                user_stable_id: event.employee_id,
                target: None,
                target_kind: YieldSignTargetKind::Entity,
                position: *position,
            });
        }
    }
}

fn yield_sign_swing(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut swing_events: EventReader<YieldSignSwingEvent>,
    mut hit_events: EventWriter<YieldSignHitEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut weapons: Query<(Entity, &YieldSign, &mut YieldSignSwingState)>,
) {
    let cooldown_ticks = seconds_to_ticks(YIELD_SIGN_COOLDOWN_SECONDS, sim_hz.0);

    for event in swing_events.read() {
        let Ok((weapon_entity, weapon, mut swing_state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if swing_state.cooldown_ticks_remaining > 0 {
            continue;
        }

        swing_state.cooldown_ticks_remaining = cooldown_ticks;
        swing_state.swings = swing_state.swings.wrapping_add(1);
        swing_state.last_used_tick = tick.0;

        let Some(target) = event.target else {
            continue;
        };

        let damage = yield_sign_damage_for_target(event.target_kind);
        swing_state.hits = swing_state.hits.wrapping_add(1);

        damage_events.send(IncomingDamageEvent {
            target,
            raw_amount: damage,
            damage_type: DamageType::Standard,
            source: weapon_entity,
        });

        hit_events.send(YieldSignHitEvent {
            weapon: weapon_entity,
            weapon_stable_id: weapon.stable_id,
            user: event.user,
            user_stable_id: event.user_stable_id,
            target,
            target_kind: event.target_kind,
            damage,
        });
    }
}

fn yield_sign_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &YieldSign,
        &YieldSignWeapon,
        &SimPosition,
        &YieldSignHeldBy,
        &YieldSignSwingState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, YIELD_SIGN_ID);
    accumulate_str(&mut checksum, 0x1001, YIELD_SIGN_NAME);
    accumulate_str(&mut checksum, 0x1002, YIELD_SIGN_TYPE);
    accumulate_str(&mut checksum, 0x1003, YIELD_SIGN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, YIELD_SIGN_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, YIELD_SIGN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, YIELD_SIGN_EXTRACTED_AT);

    checksum.accumulate(YIELD_SIGN_SOURCE_REVISION as u64);
    checksum.accumulate(YIELD_SIGN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(YIELD_SIGN_WEIGHT.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_CONDUCTIVE as u64);
    checksum.accumulate(YIELD_SIGN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_TWO_HANDED as u64);
    checksum.accumulate(YIELD_SIGN_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_COOLDOWN_SECONDS.to_bits() as u64);

    for rule in YIELD_SIGN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for (weapon, weapon_data, position, held_by, swing_state) in &weapons {
        checksum.accumulate(weapon.stable_id);
        checksum.accumulate(weapon_data.weight.to_bits() as u64);
        checksum.accumulate(weapon_data.conductive as u64);
        checksum.accumulate(weapon_data.min_value.to_bits() as u64);
        checksum.accumulate(weapon_data.max_value.to_bits() as u64);
        checksum.accumulate(weapon_data.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(swing_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(swing_state.swings);
        checksum.accumulate(swing_state.hits);
        checksum.accumulate(swing_state.last_used_tick);
    }
}

fn seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    let rounded_up = ticks.ceil();
    rounded_up.max(I32F32::from_num(1_u32)).to_num::<u32>()
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}