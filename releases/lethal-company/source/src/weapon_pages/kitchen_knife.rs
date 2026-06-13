// Sources: vault/weapon_pages/kitchen_knife.md, vault/item_index_pages/items.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
};

pub const KITCHEN_KNIFE_ID: &str = "kitchen_knife";
pub const KITCHEN_KNIFE_NAME: &str = "Kitchen knife";
pub const KITCHEN_KNIFE_TYPE: &str = "weapon_pages";
pub const KITCHEN_KNIFE_SUBTYPE: &str = "special_scrap_weapon";
pub const KITCHEN_KNIFE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Kitchen_Knife";
pub const KITCHEN_KNIFE_SOURCE_REVISION: u32 = 21222;
pub const KITCHEN_KNIFE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const KITCHEN_KNIFE_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const KITCHEN_KNIFE_EFFECTS: &str =
    "Can attack targets with a low cooldown and leaves blood on hit.";
pub const KITCHEN_KNIFE_WEIGHT: I32F32 = I32F32::lit("0");
pub const KITCHEN_KNIFE_CONDUCTIVE: bool = true;
pub const KITCHEN_KNIFE_SELL_VALUE: I32F32 = I32F32::lit("35");
pub const KITCHEN_KNIFE_TWO_HANDED: bool = false;
pub const KITCHEN_KNIFE_DAMAGE_ENTITY: I32F32 = I32F32::lit("1");
pub const KITCHEN_KNIFE_DAMAGE_EMPLOYEE: I32F32 = I32F32::lit("10");
pub const KITCHEN_KNIFE_COOLDOWN_SECONDS: I32F32 = I32F32::lit("0.375");

pub const KITCHEN_KNIFE_SPAWN_CHANCES: [KitchenKnifeSpawnChance; 3] = [
    KitchenKnifeSpawnChance {
        moon: "dine",
        chance: I32F32::lit("16.35"),
    },
    KitchenKnifeSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("7.22"),
    },
    KitchenKnifeSpawnChance {
        moon: "rend",
        chance: I32F32::lit("3.75"),
    },
];

pub const KITCHEN_KNIFE_BEHAVIORAL_MECHANICS: [KitchenKnifeBehaviorRule; 6] = [
    KitchenKnifeBehaviorRule {
        condition: "dine",
        outcome: "the spawn chance is 16.35% when a butler spawn roll occurs",
    },
    KitchenKnifeBehaviorRule {
        condition: "artifice",
        outcome: "the spawn chance is 7.22% when a butler spawn roll occurs",
    },
    KitchenKnifeBehaviorRule {
        condition: "rend",
        outcome: "the spawn chance is 3.75% when a butler spawn roll occurs",
    },
    KitchenKnifeBehaviorRule {
        condition: "the knife hits a target classified as an entity",
        outcome: "it deals 1 damage per hit and has a 0.375-second cooldown",
    },
    KitchenKnifeBehaviorRule {
        condition: "the knife hits a target classified as an employee",
        outcome: "it deals 10 damage per hit and has a 0.375-second cooldown",
    },
    KitchenKnifeBehaviorRule {
        condition: "the weapon is used as a melee item",
        outcome: "weight is 0, conductive is true, and two_handed is false",
    },
];

pub struct KitchenKnifePlugin;

impl Plugin for KitchenKnifePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnKitchenKnifeEvent>()
            .add_event::<KitchenKnifeSwingEvent>()
            .add_event::<KitchenKnifeHitEvent>()
            .add_event::<KitchenKnifeBloodOnHitEvent>()
            .add_event::<KitchenKnifeSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_kitchen_knife,
                    kitchen_knife_pickup_item_bar_bridge,
                    kitchen_knife_tick_cooldowns,
                    kitchen_knife_use_from_item_bar,
                    kitchen_knife_swing,
                    kitchen_knife_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KitchenKnife {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeWeapon {
    pub weight: I32F32,
    pub conductive: bool,
    pub sell_value: I32F32,
    pub two_handed: bool,
    pub entity_damage: I32F32,
    pub employee_damage: I32F32,
}

impl Default for KitchenKnifeWeapon {
    fn default() -> Self {
        Self {
            weight: KITCHEN_KNIFE_WEIGHT,
            conductive: KITCHEN_KNIFE_CONDUCTIVE,
            sell_value: KITCHEN_KNIFE_SELL_VALUE,
            two_handed: KITCHEN_KNIFE_TWO_HANDED,
            entity_damage: KITCHEN_KNIFE_DAMAGE_ENTITY,
            employee_damage: KITCHEN_KNIFE_DAMAGE_EMPLOYEE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KitchenKnifeHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KitchenKnifeAttackState {
    pub cooldown_ticks_remaining: u32,
    pub swings: u64,
    pub hits: u64,
    pub blood_marks: u64,
    pub last_used_tick: u64,
}

#[derive(Bundle)]
pub struct KitchenKnifeBundle {
    pub name: Name,
    pub kitchen_knife: KitchenKnife,
    pub weapon: KitchenKnifeWeapon,
    pub position: SimPosition,
    pub held_by: KitchenKnifeHeldBy,
    pub attack_state: KitchenKnifeAttackState,
}

impl KitchenKnifeBundle {
    pub fn new(event: SpawnKitchenKnifeEvent) -> Self {
        Self {
            name: Name::new(KITCHEN_KNIFE_NAME),
            kitchen_knife: KitchenKnife {
                stable_id: event.stable_id,
            },
            weapon: KitchenKnifeWeapon::default(),
            position: event.position,
            held_by: KitchenKnifeHeldBy::default(),
            attack_state: KitchenKnifeAttackState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnKitchenKnifeEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeSwingEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: KitchenKnifeTargetKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeHitEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Entity,
    pub target_kind: KitchenKnifeTargetKind,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeBloodOnHitEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub target: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitchenKnifeSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum KitchenKnifeTargetKind {
    #[default]
    Entity = 0,
    Employee = 1,
}

pub fn kitchen_knife_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    KITCHEN_KNIFE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

pub fn kitchen_knife_damage_for_target(target_kind: KitchenKnifeTargetKind) -> I32F32 {
    match target_kind {
        KitchenKnifeTargetKind::Entity => KITCHEN_KNIFE_DAMAGE_ENTITY,
        KitchenKnifeTargetKind::Employee => KITCHEN_KNIFE_DAMAGE_EMPLOYEE,
    }
}

fn spawn_kitchen_knife(mut commands: Commands, mut events: EventReader<SpawnKitchenKnifeEvent>) {
    for event in events.read() {
        commands.spawn(KitchenKnifeBundle::new(*event));
    }
}

fn kitchen_knife_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    knives: Query<(&KitchenKnife, &KitchenKnifeHeldBy), Changed<KitchenKnifeHeldBy>>,
) {
    for (knife, held_by) in &knives {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: KITCHEN_KNIFE_ID,
                two_handed: KITCHEN_KNIFE_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = knife.stable_id;
        }
    }
}

fn kitchen_knife_tick_cooldowns(mut knives: Query<&mut KitchenKnifeAttackState, With<KitchenKnife>>) {
    for mut attack_state in &mut knives {
        if attack_state.cooldown_ticks_remaining > 0 {
            attack_state.cooldown_ticks_remaining -= 1;
        }
    }
}

fn kitchen_knife_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut swing_events: EventWriter<KitchenKnifeSwingEvent>,
    knives: Query<(Entity, &KitchenKnifeHeldBy), With<KitchenKnife>>,
) {
    for event in item_events.read() {
        if event.item_id != KITCHEN_KNIFE_ID || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }

        for (weapon, held_by) in &knives {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            swing_events.send(KitchenKnifeSwingEvent {
                weapon,
                user: weapon,
                user_stable_id: event.employee_id,
                target: None,
                target_kind: KitchenKnifeTargetKind::Entity,
            });
        }
    }
}

fn kitchen_knife_swing(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut swing_events: EventReader<KitchenKnifeSwingEvent>,
    mut hit_events: EventWriter<KitchenKnifeHitEvent>,
    mut blood_events: EventWriter<KitchenKnifeBloodOnHitEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut knives: Query<(Entity, &KitchenKnife, &KitchenKnifeWeapon, &mut KitchenKnifeAttackState)>,
) {
    let cooldown_ticks = seconds_to_ticks(KITCHEN_KNIFE_COOLDOWN_SECONDS, sim_hz.0);

    for event in swing_events.read() {
        let Ok((weapon_entity, knife, weapon, mut attack_state)) = knives.get_mut(event.weapon)
        else {
            continue;
        };

        if attack_state.cooldown_ticks_remaining > 0 {
            continue;
        }

        attack_state.swings = attack_state.swings.wrapping_add(1);
        attack_state.cooldown_ticks_remaining = cooldown_ticks;
        attack_state.last_used_tick = tick.0;

        let Some(target) = event.target else {
            continue;
        };

        let damage = match event.target_kind {
            KitchenKnifeTargetKind::Entity => weapon.entity_damage,
            KitchenKnifeTargetKind::Employee => weapon.employee_damage,
        };

        damage_events.send(IncomingDamageEvent {
            target,
            raw_amount: damage,
            damage_type: DamageType::Standard,
            source: weapon_entity,
        });

        attack_state.hits = attack_state.hits.wrapping_add(1);
        attack_state.blood_marks = attack_state.blood_marks.wrapping_add(1);

        hit_events.send(KitchenKnifeHitEvent {
            weapon: weapon_entity,
            weapon_stable_id: knife.stable_id,
            user: event.user,
            user_stable_id: event.user_stable_id,
            target,
            target_kind: event.target_kind,
            damage,
        });

        blood_events.send(KitchenKnifeBloodOnHitEvent {
            weapon: weapon_entity,
            weapon_stable_id: knife.stable_id,
            target,
            damage,
        });
    }
}

fn kitchen_knife_checksum(
    mut checksum: ResMut<SimChecksumState>,
    knives: Query<(
        &KitchenKnife,
        &KitchenKnifeWeapon,
        &SimPosition,
        &KitchenKnifeHeldBy,
        &KitchenKnifeAttackState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, KITCHEN_KNIFE_ID);
    accumulate_str(&mut checksum, 0x1001, KITCHEN_KNIFE_NAME);
    accumulate_str(&mut checksum, 0x1002, KITCHEN_KNIFE_TYPE);
    accumulate_str(&mut checksum, 0x1003, KITCHEN_KNIFE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, KITCHEN_KNIFE_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, KITCHEN_KNIFE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, KITCHEN_KNIFE_EXTRACTED_AT);

    checksum.accumulate(KITCHEN_KNIFE_SOURCE_REVISION as u64);
    checksum.accumulate(KITCHEN_KNIFE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(KITCHEN_KNIFE_WEIGHT.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_CONDUCTIVE as u64);
    checksum.accumulate(KITCHEN_KNIFE_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_TWO_HANDED as u64);
    checksum.accumulate(KITCHEN_KNIFE_DAMAGE_ENTITY.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_DAMAGE_EMPLOYEE.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_COOLDOWN_SECONDS.to_bits() as u64);

    for spawn_chance in KITCHEN_KNIFE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x2000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in KITCHEN_KNIFE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (knife, weapon, position, held_by, attack_state) in &knives {
        checksum.accumulate(knife.stable_id);
        checksum.accumulate(weapon.weight.to_bits() as u64);
        checksum.accumulate(weapon.conductive as u64);
        checksum.accumulate(weapon.sell_value.to_bits() as u64);
        checksum.accumulate(weapon.two_handed as u64);
        checksum.accumulate(weapon.entity_damage.to_bits() as u64);
        checksum.accumulate(weapon.employee_damage.to_bits() as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(attack_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(attack_state.swings);
        checksum.accumulate(attack_state.hits);
        checksum.accumulate(attack_state.blood_marks);
        checksum.accumulate(attack_state.last_used_tick);
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