// Sources: vault/weapon_pages/stop_sign.md, vault/item_index_pages/weapon.md, vault/gameplay_mechanics/cause_of_death.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
};

pub const STOP_SIGN_ID: &str = "stop_sign";
pub const STOP_SIGN_NAME: &str = "Stop sign";
pub const STOP_SIGN_TYPE: &str = "weapon_pages";
pub const STOP_SIGN_SUBTYPE: &str = "weapon";
pub const STOP_SIGN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Stop_Sign";
pub const STOP_SIGN_SOURCE_REVISION: u32 = 21219;
pub const STOP_SIGN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const STOP_SIGN_CONFIDENCE_BASIS_POINTS: u16 = 81;

pub const STOP_SIGN_EFFECTS: &str =
    "Can be used to attack employees and entities; ~1 second cooldown; 1 damage to entities; 20 damage to employees.";
pub const STOP_SIGN_WEIGHT: I32F32 = I32F32::lit("28");
pub const STOP_SIGN_CONDUCTIVE: bool = true;
pub const STOP_SIGN_MIN_VALUE: I32F32 = I32F32::lit("20");
pub const STOP_SIGN_MAX_VALUE: I32F32 = I32F32::lit("51");
pub const STOP_SIGN_TWO_HANDED: bool = false;
pub const STOP_SIGN_COOLDOWN_SECONDS: I32F32 = I32F32::lit("1");
pub const STOP_SIGN_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const STOP_SIGN_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");

pub const STOP_SIGN_BEHAVIORAL_MECHANICS: [StopSignBehaviorRule; 3] = [
    StopSignBehaviorRule {
        condition: "the player uses LMB",
        outcome: "the Stop sign swings downward as a melee attack with a ~1 second cooldown",
    },
    StopSignBehaviorRule {
        condition: "the attack hits an entity",
        outcome: "it deals 1 damage",
    },
    StopSignBehaviorRule {
        condition: "the attack hits an employee",
        outcome: "it deals 20 damage",
    },
];

pub struct StopSignPlugin;

impl Plugin for StopSignPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnStopSignEvent>()
            .add_event::<StopSignSwingEvent>()
            .add_event::<StopSignHitEvent>()
            .add_event::<StopSignSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_stop_sign,
                    stop_sign_pickup_item_bar_bridge,
                    stop_sign_tick_cooldowns,
                    stop_sign_use_from_item_bar,
                    stop_sign_swing,
                    stop_sign_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSignBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StopSign {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSignWeapon {
    pub weight: I32F32,
    pub conductive: bool,
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub two_handed: bool,
}

impl Default for StopSignWeapon {
    fn default() -> Self {
        Self {
            weight: STOP_SIGN_WEIGHT,
            conductive: STOP_SIGN_CONDUCTIVE,
            min_value: STOP_SIGN_MIN_VALUE,
            max_value: STOP_SIGN_MAX_VALUE,
            two_handed: STOP_SIGN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StopSignHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StopSignSwingState {
    pub cooldown_ticks_remaining: u32,
    pub swings: u64,
    pub entity_hits: u64,
    pub employee_hits: u64,
    pub last_used_tick: u64,
}

#[derive(Bundle)]
pub struct StopSignBundle {
    pub name: Name,
    pub stop_sign: StopSign,
    pub weapon: StopSignWeapon,
    pub position: SimPosition,
    pub held_by: StopSignHeldBy,
    pub swing_state: StopSignSwingState,
}

impl StopSignBundle {
    pub fn new(event: SpawnStopSignEvent) -> Self {
        Self {
            name: Name::new(STOP_SIGN_NAME),
            stop_sign: StopSign {
                stable_id: event.stable_id,
            },
            weapon: StopSignWeapon::default(),
            position: event.position,
            held_by: StopSignHeldBy::default(),
            swing_state: StopSignSwingState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnStopSignEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSignSwingEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: StopSignTargetKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSignHitEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Entity,
    pub target_kind: StopSignTargetKind,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopSignSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StopSignTargetKind {
    #[default]
    Entity = 0,
    Employee = 1,
}

pub fn stop_sign_damage_for_target(target_kind: StopSignTargetKind) -> I32F32 {
    match target_kind {
        StopSignTargetKind::Entity => STOP_SIGN_ENTITY_DAMAGE,
        StopSignTargetKind::Employee => STOP_SIGN_EMPLOYEE_DAMAGE,
    }
}

pub fn stop_sign_value_in_range(value: I32F32) -> bool {
    value >= STOP_SIGN_MIN_VALUE && value <= STOP_SIGN_MAX_VALUE
}

fn spawn_stop_sign(mut commands: Commands, mut events: EventReader<SpawnStopSignEvent>) {
    for event in events.read() {
        commands.spawn(StopSignBundle::new(*event));
    }
}

fn stop_sign_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&StopSign, &StopSignHeldBy), Changed<StopSignHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: STOP_SIGN_ID,
                two_handed: STOP_SIGN_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn stop_sign_tick_cooldowns(mut weapons: Query<&mut StopSignSwingState, With<StopSign>>) {
    for mut swing_state in &mut weapons {
        if swing_state.cooldown_ticks_remaining > 0 {
            swing_state.cooldown_ticks_remaining -= 1;
        }
    }
}

fn stop_sign_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut swing_events: EventWriter<StopSignSwingEvent>,
    weapons: Query<(Entity, &StopSignHeldBy), With<StopSign>>,
) {
    for event in item_events.read() {
        if event.item_id != STOP_SIGN_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (weapon, held_by) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            swing_events.send(StopSignSwingEvent {
                weapon,
                user: weapon,
                user_stable_id: event.employee_id,
                target: None,
                target_kind: StopSignTargetKind::Entity,
            });
        }
    }
}

fn stop_sign_swing(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut swing_events: EventReader<StopSignSwingEvent>,
    mut hit_events: EventWriter<StopSignHitEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut weapons: Query<(Entity, &StopSign, &mut StopSignSwingState)>,
) {
    let cooldown_ticks = seconds_to_ticks(STOP_SIGN_COOLDOWN_SECONDS, sim_hz.0);

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

        let damage = stop_sign_damage_for_target(event.target_kind);

        match event.target_kind {
            StopSignTargetKind::Entity => {
                swing_state.entity_hits = swing_state.entity_hits.wrapping_add(1);
            }
            StopSignTargetKind::Employee => {
                swing_state.employee_hits = swing_state.employee_hits.wrapping_add(1);
            }
        }

        damage_events.send(IncomingDamageEvent {
            target,
            raw_amount: damage,
            damage_type: DamageType::Standard,
            source: weapon_entity,
        });

        hit_events.send(StopSignHitEvent {
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

fn stop_sign_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &StopSign,
        &StopSignWeapon,
        &SimPosition,
        &StopSignHeldBy,
        &StopSignSwingState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, STOP_SIGN_ID);
    accumulate_str(&mut checksum, 0x1001, STOP_SIGN_NAME);
    accumulate_str(&mut checksum, 0x1002, STOP_SIGN_TYPE);
    accumulate_str(&mut checksum, 0x1003, STOP_SIGN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, STOP_SIGN_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, STOP_SIGN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, STOP_SIGN_EXTRACTED_AT);

    checksum.accumulate(STOP_SIGN_SOURCE_REVISION as u64);
    checksum.accumulate(STOP_SIGN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(STOP_SIGN_WEIGHT.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_CONDUCTIVE as u64);
    checksum.accumulate(STOP_SIGN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_TWO_HANDED as u64);
    checksum.accumulate(STOP_SIGN_COOLDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_EMPLOYEE_DAMAGE.to_bits() as u64);

    for rule in STOP_SIGN_BEHAVIORAL_MECHANICS {
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
        checksum.accumulate(swing_state.entity_hits);
        checksum.accumulate(swing_state.employee_hits);
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