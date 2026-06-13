// Sources: vault/weapon_pages/shovel.md, vault/weapon_pages/stop_sign.md, vault/gameplay_mechanics/cause_of_death.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
};

pub const SHOVEL_ID: &str = "shovel";
pub const SHOVEL_NAME: &str = "Shovel";
pub const SHOVEL_TYPE: &str = "weapon_pages";
pub const SHOVEL_SUBTYPE: &str = "Offense Gear";
pub const SHOVEL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Shovel";
pub const SHOVEL_SOURCE_REVISION: u32 = 21128;
pub const SHOVEL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const SHOVEL_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const SHOVEL_EFFECTS: &str = "Can be used to attack fellow employees and entities.";
pub const SHOVEL_BUY_VALUE: I32F32 = I32F32::lit("30");
pub const SHOVEL_SELL_VALUE: I32F32 = I32F32::lit("0");
pub const SHOVEL_WEIGHT: I32F32 = I32F32::lit("14");
pub const SHOVEL_CONDUCTIVE: bool = true;
pub const SHOVEL_TWO_HANDED: bool = false;
pub const SHOVEL_EMPLOYEE_DAMAGE: I32F32 = I32F32::lit("20");
pub const SHOVEL_ENTITY_DAMAGE: I32F32 = I32F32::lit("1");
pub const SHOVEL_COOLDOWN_SECONDS: I32F32 = I32F32::lit("1");
pub const SHOVEL_WINDUP_TICKS: u32 = 1;

pub const SHOVEL_DAMAGE_RULES: [ShovelDamageRule; 14] = [
    ShovelDamageRule {
        target_id: "employee",
        hits_min: 5,
        hits_max: 5,
        area: "-",
    },
    ShovelDamageRule {
        target_id: "baboon_hawk",
        hits_min: 4,
        hits_max: 4,
        area: "Outdoor",
    },
    ShovelDamageRule {
        target_id: "bracken",
        hits_min: 3,
        hits_max: 5,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "bunker_spider",
        hits_min: 5,
        hits_max: 5,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "butler",
        hits_min: 2,
        hits_max: 8,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "eyeless_dog",
        hits_min: 12,
        hits_max: 12,
        area: "Outdoor",
    },
    ShovelDamageRule {
        target_id: "forest_keeper",
        hits_min: 38,
        hits_max: 38,
        area: "Outdoor",
    },
    ShovelDamageRule {
        target_id: "hoarding_bug",
        hits_min: 2,
        hits_max: 3,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "manticoil",
        hits_min: 1,
        hits_max: 1,
        area: "Daytime",
    },
    ShovelDamageRule {
        target_id: "masked",
        hits_min: 4,
        hits_max: 4,
        area: "Indoor & Outdoor",
    },
    ShovelDamageRule {
        target_id: "nutcracker",
        hits_min: 5,
        hits_max: 5,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "snare_flea",
        hits_min: 2,
        hits_max: 3,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "thumper",
        hits_min: 4,
        hits_max: 4,
        area: "Indoor",
    },
    ShovelDamageRule {
        target_id: "tulip_snake",
        hits_min: 1,
        hits_max: 1,
        area: "Daytime, Outdoor & Indoor",
    },
];

pub const SHOVEL_BEHAVIORAL_MECHANICS: [ShovelBehaviorRule; 12] = [
    ShovelBehaviorRule {
        condition: "you swing the shovel",
        outcome: "it has a brief wind-up before the hit lands",
    },
    ShovelBehaviorRule {
        condition: "you hold LMB",
        outcome: "the shovel is raised without swinging until you release LMB",
    },
    ShovelBehaviorRule {
        condition: "you aim slightly toward the bottom-right before a swing",
        outcome: "the shovel's head hitbox is placed closer to the target and may connect more reliably",
    },
    ShovelBehaviorRule {
        condition: "you use the shovel against a snare_flea latched onto any employee's head",
        outcome: "the swing knocks it off",
    },
    ShovelBehaviorRule {
        condition: "you strike a bunker_spider web",
        outcome: "the web is destroyed and the spider is alerted to the attacker",
    },
    ShovelBehaviorRule {
        condition: "you hit a turret with the shovel",
        outcome: "the turret malfunctions and begins firing while spinning",
    },
    ShovelBehaviorRule {
        condition: "you hit a landmine with a melee weapon",
        outcome: "it explodes immediately",
    },
    ShovelBehaviorRule {
        condition: "you attack an employee",
        outcome: "the shovel deals 20 damage",
    },
    ShovelBehaviorRule {
        condition: "you attack an entity",
        outcome: "the shovel deals 1 damage per hit against that target's Shovel HP",
    },
    ShovelBehaviorRule {
        condition: "you compare it to the stop_sign or yield_sign",
        outcome: "the combat behavior is the same",
    },
    ShovelBehaviorRule {
        condition: "a creature is immune to shovel damage",
        outcome: "the shovel does not affect that target",
    },
    ShovelBehaviorRule {
        condition: "you pair the shovel with a co-worker using a zap_gun or stun_grenade",
        outcome: "the target is less able to retaliate while you attack",
    },
];

pub struct ShovelPlugin;

impl Plugin for ShovelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnShovelEvent>()
            .add_event::<ShovelRaisedEvent>()
            .add_event::<ShovelSwingReleasedEvent>()
            .add_event::<ShovelSwingLandedEvent>()
            .add_event::<ShovelHitSpecialTargetEvent>()
            .add_event::<ShovelSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_shovel,
                    shovel_pickup_item_bar_bridge,
                    shovel_tick_cooldowns,
                    shovel_use_from_item_bar,
                    shovel_release_swing,
                    shovel_land_swing,
                    shovel_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelDamageRule {
    pub target_id: &'static str,
    pub hits_min: u8,
    pub hits_max: u8,
    pub area: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shovel {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelWeapon {
    pub buy_value: I32F32,
    pub sell_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ShovelWeapon {
    fn default() -> Self {
        Self {
            buy_value: SHOVEL_BUY_VALUE,
            sell_value: SHOVEL_SELL_VALUE,
            weight: SHOVEL_WEIGHT,
            conductive: SHOVEL_CONDUCTIVE,
            two_handed: SHOVEL_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShovelHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelSwingState {
    pub raised: bool,
    pub windup_ticks_remaining: u32,
    pub cooldown_ticks_remaining: u32,
    pub swings_started: u64,
    pub swings_landed: u64,
    pub immune_hits: u64,
    pub special_hits: u64,
    pub last_used_tick: u64,
}

impl Default for ShovelSwingState {
    fn default() -> Self {
        Self {
            raised: false,
            windup_ticks_remaining: 0,
            cooldown_ticks_remaining: 0,
            swings_started: 0,
            swings_landed: 0,
            immune_hits: 0,
            special_hits: 0,
            last_used_tick: 0,
        }
    }
}

#[derive(Bundle)]
pub struct ShovelBundle {
    pub name: Name,
    pub shovel: Shovel,
    pub weapon: ShovelWeapon,
    pub position: SimPosition,
    pub held_by: ShovelHeldBy,
    pub swing_state: ShovelSwingState,
}

impl ShovelBundle {
    pub fn new(event: SpawnShovelEvent) -> Self {
        Self {
            name: Name::new(SHOVEL_NAME),
            shovel: Shovel {
                stable_id: event.stable_id,
            },
            weapon: ShovelWeapon::default(),
            position: event.position,
            held_by: ShovelHeldBy::default(),
            swing_state: ShovelSwingState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnShovelEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelRaisedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelSwingReleasedEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: ShovelTargetKind,
    pub special_target: ShovelSpecialTarget,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelSwingLandedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: ShovelTargetKind,
    pub damage: I32F32,
    pub special_target: ShovelSpecialTarget,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelHitSpecialTargetEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user_stable_id: u64,
    pub target: Entity,
    pub special_target: ShovelSpecialTarget,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShovelSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ShovelTargetKind {
    #[default]
    Entity = 0,
    Employee = 1,
    ImmuneCreature = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ShovelSpecialTarget {
    #[default]
    None = 0,
    SnareFleaLatchedToEmployee = 1,
    BunkerSpiderWeb = 2,
    Turret = 3,
    Landmine = 4,
}

pub fn shovel_damage_for_target(target_kind: ShovelTargetKind) -> I32F32 {
    match target_kind {
        ShovelTargetKind::Entity => SHOVEL_ENTITY_DAMAGE,
        ShovelTargetKind::Employee => SHOVEL_EMPLOYEE_DAMAGE,
        ShovelTargetKind::ImmuneCreature => I32F32::ZERO,
    }
}

pub fn shovel_hits_for_target(target_id: &str) -> Option<ShovelDamageRule> {
    SHOVEL_DAMAGE_RULES
        .iter()
        .find(|rule| rule.target_id == target_id)
        .copied()
}

fn spawn_shovel(mut commands: Commands, mut events: EventReader<SpawnShovelEvent>) {
    for event in events.read() {
        commands.spawn(ShovelBundle::new(*event));
    }
}

fn shovel_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&Shovel, &ShovelHeldBy), Changed<ShovelHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: SHOVEL_ID,
                two_handed: SHOVEL_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn shovel_tick_cooldowns(mut weapons: Query<&mut ShovelSwingState, With<Shovel>>) {
    for mut swing_state in &mut weapons {
        if swing_state.cooldown_ticks_remaining > 0 {
            swing_state.cooldown_ticks_remaining -= 1;
        }

        if swing_state.windup_ticks_remaining > 0 {
            swing_state.windup_ticks_remaining -= 1;
        }
    }
}

fn shovel_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut raised_events: EventWriter<ShovelRaisedEvent>,
    weapons: Query<(Entity, &Shovel, &ShovelHeldBy), With<Shovel>>,
) {
    for event in item_events.read() {
        if event.item_id != SHOVEL_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (weapon_entity, shovel, held_by) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            raised_events.send(ShovelRaisedEvent {
                weapon: weapon_entity,
                weapon_stable_id: shovel.stable_id,
                user: weapon_entity,
                user_stable_id: event.employee_id,
            });
        }
    }
}

fn shovel_release_swing(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut raised_events: EventReader<ShovelRaisedEvent>,
    mut release_events: EventReader<ShovelSwingReleasedEvent>,
    mut weapons: Query<(&Shovel, &mut ShovelSwingState)>,
) {
    let cooldown_ticks = seconds_to_ticks(SHOVEL_COOLDOWN_SECONDS, sim_hz.0);

    for event in raised_events.read() {
        let Ok((shovel, mut swing_state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if shovel.stable_id != event.weapon_stable_id || swing_state.cooldown_ticks_remaining > 0 {
            continue;
        }

        swing_state.raised = true;
    }

    for event in release_events.read() {
        let Ok((_shovel, mut swing_state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if swing_state.cooldown_ticks_remaining > 0 {
            continue;
        }

        swing_state.raised = false;
        swing_state.windup_ticks_remaining = SHOVEL_WINDUP_TICKS;
        swing_state.cooldown_ticks_remaining = cooldown_ticks;
        swing_state.swings_started = swing_state.swings_started.wrapping_add(1);
        swing_state.last_used_tick = tick.0;
    }
}

fn shovel_land_swing(
    mut release_events: EventReader<ShovelSwingReleasedEvent>,
    mut landed_events: EventWriter<ShovelSwingLandedEvent>,
    mut special_events: EventWriter<ShovelHitSpecialTargetEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut weapons: Query<(&Shovel, &mut ShovelSwingState)>,
) {
    for event in release_events.read() {
        let Ok((shovel, mut swing_state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if swing_state.windup_ticks_remaining > 0 {
            continue;
        }

        let damage = shovel_damage_for_target(event.target_kind);

        if event.target_kind == ShovelTargetKind::ImmuneCreature {
            swing_state.immune_hits = swing_state.immune_hits.wrapping_add(1);
        }

        if event.special_target != ShovelSpecialTarget::None {
            swing_state.special_hits = swing_state.special_hits.wrapping_add(1);
            if let Some(target) = event.target {
                special_events.send(ShovelHitSpecialTargetEvent {
                    weapon: event.weapon,
                    weapon_stable_id: shovel.stable_id,
                    user_stable_id: event.user_stable_id,
                    target,
                    special_target: event.special_target,
                });
            }
        }

        if let Some(target) = event.target {
            if damage > I32F32::ZERO {
                damage_events.send(IncomingDamageEvent {
                    target,
                    raw_amount: damage,
                    damage_type: DamageType::Standard,
                    source: event.weapon,
                });
            }
        }

        swing_state.swings_landed = swing_state.swings_landed.wrapping_add(1);

        landed_events.send(ShovelSwingLandedEvent {
            weapon: event.weapon,
            weapon_stable_id: shovel.stable_id,
            user: event.user,
            user_stable_id: event.user_stable_id,
            target: event.target,
            target_kind: event.target_kind,
            damage,
            special_target: event.special_target,
        });
    }
}

fn shovel_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &Shovel,
        &ShovelWeapon,
        &SimPosition,
        &ShovelHeldBy,
        &ShovelSwingState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, SHOVEL_ID);
    accumulate_str(&mut checksum, 0x1001, SHOVEL_NAME);
    accumulate_str(&mut checksum, 0x1002, SHOVEL_TYPE);
    accumulate_str(&mut checksum, 0x1003, SHOVEL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, SHOVEL_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, SHOVEL_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, SHOVEL_EXTRACTED_AT);

    checksum.accumulate(SHOVEL_SOURCE_REVISION as u64);
    checksum.accumulate(SHOVEL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(SHOVEL_BUY_VALUE.to_bits() as u64);
    checksum.accumulate(SHOVEL_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(SHOVEL_WEIGHT.to_bits() as u64);
    checksum.accumulate(SHOVEL_CONDUCTIVE as u64);
    checksum.accumulate(SHOVEL_TWO_HANDED as u64);
    checksum.accumulate(SHOVEL_EMPLOYEE_DAMAGE.to_bits() as u64);
    checksum.accumulate(SHOVEL_ENTITY_DAMAGE.to_bits() as u64);
    checksum.accumulate(SHOVEL_COOLDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(SHOVEL_WINDUP_TICKS as u64);

    for rule in SHOVEL_DAMAGE_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.target_id);
        checksum.accumulate(rule.hits_min as u64);
        checksum.accumulate(rule.hits_max as u64);
        accumulate_str(&mut checksum, 0x2001, rule.area);
    }

    for rule in SHOVEL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (shovel, weapon, position, held_by, swing_state) in &weapons {
        checksum.accumulate(shovel.stable_id);
        checksum.accumulate(weapon.buy_value.to_bits() as u64);
        checksum.accumulate(weapon.sell_value.to_bits() as u64);
        checksum.accumulate(weapon.weight.to_bits() as u64);
        checksum.accumulate(weapon.conductive as u64);
        checksum.accumulate(weapon.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(swing_state.raised as u64);
        checksum.accumulate(swing_state.windup_ticks_remaining as u64);
        checksum.accumulate(swing_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(swing_state.swings_started);
        checksum.accumulate(swing_state.swings_landed);
        checksum.accumulate(swing_state.immune_hits);
        checksum.accumulate(swing_state.special_hits);
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