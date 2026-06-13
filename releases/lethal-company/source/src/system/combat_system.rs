// Sources: vault/item_index_pages/items.md, vault/item_index_pages/weapon.md
use bevy::prelude::*;
use fixed::types::I32F32;
use std::collections::BTreeMap;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimTick,
};
use crate::system::game_state_machine::GameState;

pub const COMBAT_SYSTEM_ID: &str = "combat_system";
pub const COMBAT_SYSTEM_NAME: &str = "Combat System";
pub const COMBAT_SYSTEM_TYPE: &str = "system";
pub const COMBAT_SYSTEM_SUBTYPE: &str = "combat";

pub const ITEMS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Items";
pub const ITEMS_SOURCE_REVISION: u32 = 21248;
pub const ITEMS_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const WEAPON_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Weapon";
pub const WEAPON_SOURCE_REVISION: u32 = 21293;
pub const WEAPON_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const SHOVEL_WEIGHT_LBS: I32F32 = I32F32::lit("14");
pub const SHOVEL_COST: I32F32 = I32F32::lit("30");
pub const SHOVEL_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const SHOVEL_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");
pub const SHOVEL_CONDUCTIVE: bool = true;
pub const SHOVEL_TWO_HANDED: bool = false;

pub const STOP_SIGN_WEIGHT_LBS: I32F32 = I32F32::lit("28");
pub const STOP_SIGN_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const STOP_SIGN_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");
pub const STOP_SIGN_CONDUCTIVE: bool = true;
pub const STOP_SIGN_TWO_HANDED: bool = false;

pub const YIELD_SIGN_WEIGHT_LBS: I32F32 = I32F32::lit("42");
pub const YIELD_SIGN_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const YIELD_SIGN_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");
pub const YIELD_SIGN_CONDUCTIVE: bool = true;
pub const YIELD_SIGN_TWO_HANDED: bool = false;

pub const KITCHEN_KNIFE_WEIGHT_LBS: I32F32 = I32F32::lit("0");
pub const KITCHEN_KNIFE_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const KITCHEN_KNIFE_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("10");
pub const KITCHEN_KNIFE_CONDUCTIVE: bool = true;
pub const KITCHEN_KNIFE_TWO_HANDED: bool = false;

pub const DOUBLE_BARREL_WEIGHT_LBS: I32F32 = I32F32::lit("16");
pub const DOUBLE_BARREL_CONDUCTIVE: bool = false;
pub const DOUBLE_BARREL_TWO_HANDED: bool = false;

pub const STUN_GRENADE_WEIGHT_LBS: I32F32 = I32F32::lit("5");
pub const STUN_GRENADE_COST: I32F32 = I32F32::lit("30");
pub const STUN_GRENADE_COUNTDOWN_SECONDS: I32F32 = I32F32::lit("3");
pub const STUN_GRENADE_CONDUCTIVE: bool = false;
pub const STUN_GRENADE_TWO_HANDED: bool = false;

pub const ZAP_GUN_WEIGHT_LBS: I32F32 = I32F32::lit("11");
pub const ZAP_GUN_COST: I32F32 = I32F32::lit("400");
pub const ZAP_GUN_CONDUCTIVE: bool = true;
pub const ZAP_GUN_TWO_HANDED: bool = false;

pub const DIY_FLASHBANG_WEIGHT_LBS: I32F32 = I32F32::lit("5");
pub const DIY_FLASHBANG_USER_DAMAGE: I32F32 = I32F32::lit("20");
pub const DIY_FLASHBANG_CONDUCTIVE: bool = false;
pub const DIY_FLASHBANG_TWO_HANDED: bool = false;

pub const COMBAT_RULES: [CombatRule; 15] = [
    CombatRule {
        condition: "an item is a damage weapon",
        outcome: "it deals HP damage to entities or employees instead of applying stun",
    },
    CombatRule {
        condition: "an item is a stun weapon",
        outcome: "it disables nearby entities or employees after its activation or detonation sequence",
    },
    CombatRule {
        condition: "an employee is holding a weapon-class item",
        outcome: "entity_targeting threat level calculations can change for that holder",
    },
    CombatRule {
        condition: "the item is a shovel",
        outcome: "it weighs 14 lb, is conductive, is one-handed, costs 30, and deals 1 HP to entities or 20 HP to employees",
    },
    CombatRule {
        condition: "the item is a stop_sign",
        outcome: "it weighs 28 lb, is conductive, is one-handed, and behaves like a heavier shovel that still deals 1 HP to entities or 20 HP to employees",
    },
    CombatRule {
        condition: "the item is a yield_sign",
        outcome: "it weighs 42 lb, is conductive, is one-handed, and behaves like a heavier shovel that still deals 1 HP to entities or 20 HP to employees",
    },
    CombatRule {
        condition: "the item is a kitchen_knife",
        outcome: "it weighs 0 lb, is conductive, is one-handed, spawns with a butler, and deals 1 HP to entities or 10 HP to employees",
    },
    CombatRule {
        condition: "the item is a double_barrel",
        outcome: "it weighs 16 lb, is non-conductive, is one-handed, spawns with a nutcracker, and consumes shotgun_shells as ammunition",
    },
    CombatRule {
        condition: "the item is shotgun_shells",
        outcome: "two are dropped when a nutcracker dies and they function only as ammunition for the double_barrel",
    },
    CombatRule {
        condition: "the item is a stun_grenade",
        outcome: "it weighs 5 lb, is non-conductive, is one-handed, costs 30, and uses a 3 s countdown before detonation",
    },
    CombatRule {
        condition: "the item is a zap_gun",
        outcome: "it weighs 11 lb, is conductive, is one-handed, costs 400, targets a nearby entity or employee, and uses a rechargeable battery",
    },
    CombatRule {
        condition: "the item is a diy_flashbang",
        outcome: "it weighs 5 lb, is non-conductive, is one-handed, explodes instantly on use, deals 20 damage to the user, and stuns nearby entities or employees",
    },
    CombatRule {
        condition: "a used stun_grenade is thrown over a landmine",
        outcome: "it can trigger the mine safely from a distance",
    },
    CombatRule {
        condition: "an item is classified as a weapon",
        outcome: "holding it changes entity_targeting threat-level calculations and it can deal damage to entities and employees",
    },
    CombatRule {
        condition: "an item is neither scrap nor store",
        outcome: "it is treated as a special item and is not lost when the entire crew dies",
    },
];

pub struct CombatSystemPlugin;

impl Plugin for CombatSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatSystemState>()
            .add_event::<WeaponAttackRequestEvent>()
            .add_event::<CombatDamageAppliedEvent>()
            .add_event::<CombatCreatureKilledEvent>()
            .add_event::<CombatStunAppliedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    combat_system_resolve_weapon_attacks,
                    combat_system_apply_damage,
                    combat_system_checksum,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combatant {
    pub stable_id: u64,
    pub entity_id: &'static str,
    pub kind: CombatantKind,
    pub killable: bool,
    pub difficulty_tier: u8,
    pub exp_reward: I32F32,
}

impl Default for Combatant {
    fn default() -> Self {
        Self {
            stable_id: 0,
            entity_id: "",
            kind: CombatantKind::Creature,
            killable: true,
            difficulty_tier: 0,
            exp_reward: I32F32::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CombatantKind {
    #[default]
    Creature = 0,
    Employee = 1,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeldWeaponThreat {
    pub holder_stable_id: u64,
    pub weapon: CombatWeaponKind,
    pub threat_level_delta: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CombatWeaponKind {
    #[default]
    Shovel = 0,
    StopSign = 1,
    YieldSign = 2,
    KitchenKnife = 3,
    DoubleBarrel = 4,
    StunGrenade = 5,
    ZapGun = 6,
    DiyFlashbang = 7,
    ShotgunShells = 8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CombatAttackMode {
    #[default]
    Damage = 0,
    Stun = 1,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CombatSystemState {
    pub attacks_requested: u64,
    pub damage_requests_emitted: u64,
    pub stun_applications: u64,
    pub damage_applications: u64,
    pub kills: u64,
    pub last_attacker_stable_id: u64,
    pub last_target_stable_id: u64,
    pub last_weapon: CombatWeaponKind,
    pub last_damage: I32F32,
    pub last_killed_stable_id: u64,
    pub total_damage_by_target: BTreeMap<u64, I32F32>,
}

impl Default for CombatSystemState {
    fn default() -> Self {
        Self {
            attacks_requested: 0,
            damage_requests_emitted: 0,
            stun_applications: 0,
            damage_applications: 0,
            kills: 0,
            last_attacker_stable_id: 0,
            last_target_stable_id: 0,
            last_weapon: CombatWeaponKind::Shovel,
            last_damage: I32F32::ZERO,
            last_killed_stable_id: 0,
            total_damage_by_target: BTreeMap::new(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponAttackRequestEvent {
    pub attacker: Entity,
    pub attacker_stable_id: u64,
    pub weapon: CombatWeaponKind,
    pub mode: CombatAttackMode,
    pub target: Entity,
    pub target_kind: CombatantKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatDamageAppliedEvent {
    pub target: Entity,
    pub target_stable_id: u64,
    pub source: Entity,
    pub amount: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatCreatureKilledEvent {
    pub target: Entity,
    pub target_stable_id: u64,
    pub killer: Entity,
    pub difficulty_tier: u8,
    pub exp_reward: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatStunAppliedEvent {
    pub target: Entity,
    pub target_stable_id: u64,
    pub source: Entity,
    pub weapon: CombatWeaponKind,
}

pub fn combat_weapon_damage(
    weapon: CombatWeaponKind,
    target_kind: CombatantKind,
) -> I32F32 {
    match (weapon, target_kind) {
        (CombatWeaponKind::Shovel, CombatantKind::Creature) => SHOVEL_ENTITY_DAMAGE,
        (CombatWeaponKind::Shovel, CombatantKind::Employee) => SHOVEL_EMPLOYEE_DAMAGE,
        (CombatWeaponKind::StopSign, CombatantKind::Creature) => STOP_SIGN_ENTITY_DAMAGE,
        (CombatWeaponKind::StopSign, CombatantKind::Employee) => STOP_SIGN_EMPLOYEE_DAMAGE,
        (CombatWeaponKind::YieldSign, CombatantKind::Creature) => YIELD_SIGN_ENTITY_DAMAGE,
        (CombatWeaponKind::YieldSign, CombatantKind::Employee) => YIELD_SIGN_EMPLOYEE_DAMAGE,
        (CombatWeaponKind::KitchenKnife, CombatantKind::Creature) => KITCHEN_KNIFE_ENTITY_DAMAGE,
        (CombatWeaponKind::KitchenKnife, CombatantKind::Employee) => KITCHEN_KNIFE_EMPLOYEE_DAMAGE,
        (CombatWeaponKind::DiyFlashbang, _) => DIY_FLASHBANG_USER_DAMAGE,
        (CombatWeaponKind::DoubleBarrel, _) => I32F32::ZERO,
        (CombatWeaponKind::StunGrenade, _) => I32F32::ZERO,
        (CombatWeaponKind::ZapGun, _) => I32F32::ZERO,
        (CombatWeaponKind::ShotgunShells, _) => I32F32::ZERO,
    }
}

pub const fn combat_weapon_attack_mode(weapon: CombatWeaponKind) -> CombatAttackMode {
    match weapon {
        CombatWeaponKind::StunGrenade | CombatWeaponKind::ZapGun => CombatAttackMode::Stun,
        _ => CombatAttackMode::Damage,
    }
}

pub const fn combat_weapon_is_conductive(weapon: CombatWeaponKind) -> bool {
    match weapon {
        CombatWeaponKind::Shovel => SHOVEL_CONDUCTIVE,
        CombatWeaponKind::StopSign => STOP_SIGN_CONDUCTIVE,
        CombatWeaponKind::YieldSign => YIELD_SIGN_CONDUCTIVE,
        CombatWeaponKind::KitchenKnife => KITCHEN_KNIFE_CONDUCTIVE,
        CombatWeaponKind::DoubleBarrel => DOUBLE_BARREL_CONDUCTIVE,
        CombatWeaponKind::StunGrenade => STUN_GRENADE_CONDUCTIVE,
        CombatWeaponKind::ZapGun => ZAP_GUN_CONDUCTIVE,
        CombatWeaponKind::DiyFlashbang => DIY_FLASHBANG_CONDUCTIVE,
        CombatWeaponKind::ShotgunShells => false,
    }
}

fn combat_system_resolve_weapon_attacks(
    mut attack_events: EventReader<WeaponAttackRequestEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut stun_events: EventWriter<CombatStunAppliedEvent>,
    combatants: Query<&Combatant>,
    mut state: ResMut<CombatSystemState>,
) {
    for event in attack_events.read() {
        let Ok(target) = combatants.get(event.target) else {
            continue;
        };

        state.attacks_requested = state.attacks_requested.wrapping_add(1);
        state.last_attacker_stable_id = event.attacker_stable_id;
        state.last_target_stable_id = target.stable_id;
        state.last_weapon = event.weapon;

        let mode = if event.mode == CombatAttackMode::Stun {
            CombatAttackMode::Stun
        } else {
            combat_weapon_attack_mode(event.weapon)
        };

        if mode == CombatAttackMode::Stun {
            state.stun_applications = state.stun_applications.wrapping_add(1);
            stun_events.send(CombatStunAppliedEvent {
                target: event.target,
                target_stable_id: target.stable_id,
                source: event.attacker,
                weapon: event.weapon,
            });
            continue;
        }

        let damage = combat_weapon_damage(event.weapon, event.target_kind);
        if damage <= I32F32::ZERO {
            continue;
        }

        state.damage_requests_emitted = state.damage_requests_emitted.wrapping_add(1);
        state.last_damage = damage;

        damage_events.send(IncomingDamageEvent {
            target: event.target,
            raw_amount: damage,
            damage_type: DamageType::Standard,
            source: event.attacker,
        });
    }
}

fn combat_system_apply_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut applied_events: EventWriter<CombatDamageAppliedEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    mut combat_killed_events: EventWriter<CombatCreatureKilledEvent>,
    mut combatants: Query<(Entity, &Combatant, &mut Health)>,
    mut state: ResMut<CombatSystemState>,
) {
    for event in damage_events.read() {
        if event.raw_amount <= I32F32::ZERO {
            continue;
        }

        let Ok((target_entity, combatant, mut health)) = combatants.get_mut(event.target) else {
            continue;
        };

        if target_entity != event.target || event.damage_type != DamageType::Standard {
            continue;
        }

        let previous_health = health.current;
        health.current = (health.current - event.raw_amount).max(I32F32::ZERO);
        let applied_amount = previous_health - health.current;

        state.damage_applications = state.damage_applications.wrapping_add(1);
        state.last_target_stable_id = combatant.stable_id;
        state.last_damage = applied_amount;

        let total = state
            .total_damage_by_target
            .entry(combatant.stable_id)
            .or_insert(I32F32::ZERO);
        *total += applied_amount;

        applied_events.send(CombatDamageAppliedEvent {
            target: event.target,
            target_stable_id: combatant.stable_id,
            source: event.source,
            amount: applied_amount,
            remaining_health: health.current,
        });

        if !combatant.killable || previous_health <= I32F32::ZERO || health.current > I32F32::ZERO {
            continue;
        }

        state.kills = state.kills.wrapping_add(1);
        state.last_killed_stable_id = combatant.stable_id;

        killed_events.send(EntityKilledEvent {
            entity: event.target,
            killer: event.source,
            exp_reward: combatant.exp_reward,
            difficulty_tier: combatant.difficulty_tier,
        });

        combat_killed_events.send(CombatCreatureKilledEvent {
            target: event.target,
            target_stable_id: combatant.stable_id,
            killer: event.source,
            exp_reward: combatant.exp_reward,
            difficulty_tier: combatant.difficulty_tier,
        });
    }
}

fn combat_system_checksum(
    tick: Res<SimTick>,
    state: Res<CombatSystemState>,
    combatants: Query<(&Combatant, &Health, Option<&HeldWeaponThreat>)>,
    mut checksum: ResMut<SimChecksumState>,
) {
    checksum.accumulate(tick.0);
    accumulate_str(&mut checksum, 0x1000, COMBAT_SYSTEM_ID);
    accumulate_str(&mut checksum, 0x1001, COMBAT_SYSTEM_NAME);
    accumulate_str(&mut checksum, 0x1002, COMBAT_SYSTEM_TYPE);
    accumulate_str(&mut checksum, 0x1003, COMBAT_SYSTEM_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ITEMS_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, WEAPON_SOURCE_URL);

    checksum.accumulate(ITEMS_SOURCE_REVISION as u64);
    checksum.accumulate(ITEMS_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(WEAPON_SOURCE_REVISION as u64);
    checksum.accumulate(WEAPON_CONFIDENCE_BASIS_POINTS as u64);

    checksum.accumulate(SHOVEL_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(SHOVEL_COST.to_bits() as u64);
    checksum.accumulate(SHOVEL_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(SHOVEL_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(SHOVEL_CONDUCTIVE as u64);
    checksum.accumulate(SHOVEL_TWO_HANDED as u64);

    checksum.accumulate(STOP_SIGN_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(STOP_SIGN_CONDUCTIVE as u64);
    checksum.accumulate(STOP_SIGN_TWO_HANDED as u64);

    checksum.accumulate(YIELD_SIGN_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(YIELD_SIGN_CONDUCTIVE as u64);
    checksum.accumulate(YIELD_SIGN_TWO_HANDED as u64);

    checksum.accumulate(KITCHEN_KNIFE_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(KITCHEN_KNIFE_CONDUCTIVE as u64);
    checksum.accumulate(KITCHEN_KNIFE_TWO_HANDED as u64);

    checksum.accumulate(DOUBLE_BARREL_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_CONDUCTIVE as u64);
    checksum.accumulate(DOUBLE_BARREL_TWO_HANDED as u64);

    checksum.accumulate(STUN_GRENADE_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_COST.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_COUNTDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_CONDUCTIVE as u64);
    checksum.accumulate(STUN_GRENADE_TWO_HANDED as u64);

    checksum.accumulate(ZAP_GUN_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_COST.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_CONDUCTIVE as u64);
    checksum.accumulate(ZAP_GUN_TWO_HANDED as u64);

    checksum.accumulate(DIY_FLASHBANG_WEIGHT_LBS.to_bits() as u64);
    checksum.accumulate(DIY_FLASHBANG_USER_DAMAGE.to_bits() as u64);
    checksum.accumulate(DIY_FLASHBANG_CONDUCTIVE as u64);
    checksum.accumulate(DIY_FLASHBANG_TWO_HANDED as u64);

    for rule in COMBAT_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    checksum.accumulate(state.attacks_requested);
    checksum.accumulate(state.damage_requests_emitted);
    checksum.accumulate(state.stun_applications);
    checksum.accumulate(state.damage_applications);
    checksum.accumulate(state.kills);
    checksum.accumulate(state.last_attacker_stable_id);
    checksum.accumulate(state.last_target_stable_id);
    checksum.accumulate(combat_weapon_code(state.last_weapon));
    checksum.accumulate(state.last_damage.to_bits() as u64);
    checksum.accumulate(state.last_killed_stable_id);

    for (stable_id, total_damage) in &state.total_damage_by_target {
        checksum.accumulate(*stable_id);
        checksum.accumulate(total_damage.to_bits() as u64);
    }

    for (combatant, health, held_weapon) in &combatants {
        checksum.accumulate(combatant.stable_id);
        accumulate_str(&mut checksum, 0x3000, combatant.entity_id);
        checksum.accumulate(combatant_kind_code(combatant.kind));
        checksum.accumulate(combatant.killable as u64);
        checksum.accumulate(combatant.difficulty_tier as u64);
        checksum.accumulate(combatant.exp_reward.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        if let Some(threat) = held_weapon {
            checksum.accumulate(threat.holder_stable_id);
            checksum.accumulate(combat_weapon_code(threat.weapon));
            checksum.accumulate(threat.threat_level_delta as i64 as u64);
        }
    }
}

fn combatant_kind_code(kind: CombatantKind) -> u64 {
    match kind {
        CombatantKind::Creature => 1,
        CombatantKind::Employee => 2,
    }
}

fn combat_weapon_code(weapon: CombatWeaponKind) -> u64 {
    match weapon {
        CombatWeaponKind::Shovel => 1,
        CombatWeaponKind::StopSign => 2,
        CombatWeaponKind::YieldSign => 3,
        CombatWeaponKind::KitchenKnife => 4,
        CombatWeaponKind::DoubleBarrel => 5,
        CombatWeaponKind::StunGrenade => 6,
        CombatWeaponKind::ZapGun => 7,
        CombatWeaponKind::DiyFlashbang => 8,
        CombatWeaponKind::ShotgunShells => 9,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}