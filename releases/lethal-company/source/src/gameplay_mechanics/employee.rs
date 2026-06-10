// Sources: vault/gameplay_mechanics/employee.md, vault/gameplay_mechanics/global_game_variables.md, vault/item_index_pages/scrap.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimTick, UnitStats,
};

pub const EMPLOYEE_ID: &str = "employee";
pub const EMPLOYEE_NAME: &str = "Employee";
pub const EMPLOYEE_TYPE: &str = "gameplay_mechanics";
pub const EMPLOYEE_SUBTYPE: &str = "playable_character";
pub const EMPLOYEE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Employee";
pub const EMPLOYEE_SOURCE_REVISION: u32 = 21472;
pub const EMPLOYEE_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const EMPLOYEE_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const EMPLOYEE_HEALTH: I32F32 = I32F32::lit("100");
pub const EMPLOYEE_CRITICAL_INJURY_THRESHOLD: I32F32 = I32F32::lit("20");
pub const EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH: I32F32 = I32F32::lit("20");
pub const EMPLOYEE_CRITICAL_INJURY_REGEN_RATE: I32F32 = I32F32::lit("1");
pub const EMPLOYEE_CRITICAL_LIMP_THRESHOLD: I32F32 = I32F32::lit("10");
pub const EMPLOYEE_CRITICAL_DAMAGE_FLOOR: I32F32 = I32F32::lit("5");
pub const EMPLOYEE_CRITICAL_DAMAGE_CAP: I32F32 = I32F32::lit("50");
pub const EMPLOYEE_STAMINA_MIN: I32F32 = I32F32::lit("0");
pub const EMPLOYEE_STAMINA_MAX: I32F32 = I32F32::lit("1");
pub const EMPLOYEE_STAMINA_EXHAUSTED_THRESHOLD: I32F32 = I32F32::lit("0.1");
pub const EMPLOYEE_STAMINA_JUMP_RESUME_THRESHOLD: I32F32 = I32F32::lit("0.2");
pub const EMPLOYEE_STAMINA_SPRINT_RESUME_THRESHOLD: I32F32 = I32F32::lit("0.3");
pub const EMPLOYEE_JUMP_STAMINA_COST: I32F32 = I32F32::lit("0.08");
pub const EMPLOYEE_SPRINT_STAMINA_DRAIN_RATE: I32F32 = I32F32::lit("0.2");
pub const EMPLOYEE_WALKING_STAMINA_REGEN_RATE: I32F32 = I32F32::lit("0.07");
pub const EMPLOYEE_STANDING_STAMINA_REGEN_RATE: I32F32 = I32F32::lit("0.11");
pub const EMPLOYEE_STAMINA_REFUND_DIVISOR: I32F32 = I32F32::lit("125");

pub const EMPLOYEE_DEPENDS_ON: [&str; 5] = [
    "lethal_company",
    "moons",
    "scrap",
    "the_company",
    "profit_quota",
];

pub const EMPLOYEE_RULES: [EmployeeRule; 6] = [
    EmployeeRule {
        condition: "health is tracked",
        outcome: "Health is 100 total",
    },
    EmployeeRule {
        condition: "health is below 20",
        outcome: "critical injury starts and ends once health reaches 20",
    },
    EmployeeRule {
        condition: "a lethal hit below 50 damage lands while not critically injured",
        outcome: "health is set to 5",
    },
    EmployeeRule {
        condition: "stamina is tracked",
        outcome: "it ranges from 0 to 1 with exhaustion below 0.1, jump recovery at 0.2, and sprint recovery at 0.3",
    },
    EmployeeRule {
        condition: "the scanner is used",
        outcome: "entrances, the ship, logs, scrap, and entities are highlighted and valuables contribute to the scan total",
    },
    EmployeeRule {
        condition: "an employee dies",
        outcome: "a player body appears and scanning it reveals the cause of death; abandoned employees show as MISSING",
    },
];

pub const EMPLOYEE_MODIFIERS: [EmployeeRule; 2] = [
    EmployeeRule {
        condition: "TZP-Inhalant is active",
        outcome: "stamina depletion is reduced and stamina regeneration is increased",
    },
    EmployeeRule {
        condition: "walking while stunned by the zap gun",
        outcome: "stamina depletes at half the normal walking rate",
    },
];

pub const EMPLOYEE_STRATEGY: [EmployeeRule; 3] = [
    EmployeeRule {
        condition: "sprinting should remain available",
        outcome: "stay at or above 0.3 stamina",
    },
    EmployeeRule {
        condition: "jumping should remain available",
        outcome: "stay at or above 0.2 stamina",
    },
    EmployeeRule {
        condition: "critical injury should be avoided",
        outcome: "stay at or above 20 health",
    },
];

pub const EMPLOYEE_NOTES: [EmployeeRule; 4] = [
    EmployeeRule {
        condition: "the stamina indicator is read",
        outcome: "it only shows part of the true value",
    },
    EmployeeRule {
        condition: "limping leaves blood trails",
        outcome: "the trails are cosmetic",
    },
    EmployeeRule {
        condition: "damage over 19 is received",
        outcome: "stamina refunds by damageAmount / 125",
    },
    EmployeeRule {
        condition: "the first entity scan in a run occurs",
        outcome: "the terminal dialog about new creature data triggers",
    },
];

pub const EMPLOYEE_BEHAVIORAL_MECHANICS: [EmployeeRule; 13] = [
    EmployeeRule {
        condition: "health drops below 20",
        outcome: "the employee becomes critically injured, a HUD overlay appears, and health regenerates at 1 health per second until 20 health",
    },
    EmployeeRule {
        condition: "health drops below 10 while critically injured",
        outcome: "the employee limps, cannot sprint, and leaves a blood trail",
    },
    EmployeeRule {
        condition: "incoming damage would be lethal and the damage amount is less than 50 and the employee is not currently critically injured",
        outcome: "health is reduced to 5 and critical injury starts",
    },
    EmployeeRule {
        condition: "stamina falls below 0.1",
        outcome: "the employee becomes exhausted and cannot sprint or jump",
    },
    EmployeeRule {
        condition: "stamina reaches at least 0.2",
        outcome: "jumping becomes available again",
    },
    EmployeeRule {
        condition: "stamina reaches at least 0.3",
        outcome: "sprinting becomes available again",
    },
    EmployeeRule {
        condition: "walking",
        outcome: "stamina regenerates at approximately 0.07 per second",
    },
    EmployeeRule {
        condition: "standing still",
        outcome: "stamina regenerates at approximately 0.11 per second",
    },
    EmployeeRule {
        condition: "damage greater than 19 is received",
        outcome: "stamina refunds by damageAmount / 125",
    },
    EmployeeRule {
        condition: "scanning",
        outcome: "entrances, the ship, logs, resources, and hazards can be highlighted, and highlighted valuables contribute to a total value readout",
    },
    EmployeeRule {
        condition: "first entity scan in a run",
        outcome: "all employees receive the terminal dialog about new creature data",
    },
    EmployeeRule {
        condition: "the employee dies",
        outcome: "a player body appears and the death is classified for later scanning",
    },
    EmployeeRule {
        condition: "the employee is abandoned by the ship while still alive",
        outcome: "the end screen shows MISSING instead of DECEASED",
    },
];

pub struct EmployeePlugin;

impl Plugin for EmployeePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmployeeRunState>()
            .add_event::<SpawnEmployeeEvent>()
            .add_event::<EmployeeMovementEvent>()
            .add_event::<EmployeeScanEvent>()
            .add_event::<EmployeeScanResolvedEvent>()
            .add_event::<EmployeeFirstEntityScanEvent>()
            .add_event::<EmployeeDeathEvent>()
            .add_event::<EmployeeBodySpawnedEvent>()
            .add_event::<EmployeeAbandonedEvent>()
            .add_event::<EmployeeEndStatusEvent>()
            .add_systems(
                FixedUpdate,
                (
                    employee_spawn,
                    employee_apply_movement,
                    employee_apply_damage,
                    employee_regenerate_critical_health,
                    employee_resolve_scan,
                    employee_mark_abandoned,
                    employee_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmployeeRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Employee {
    pub stable_id: u64,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeVitals {
    pub critically_injured: bool,
    pub limping: bool,
    pub dead: bool,
    pub abandoned: bool,
    pub death_cause_id: &'static str,
}

impl Default for EmployeeVitals {
    fn default() -> Self {
        Self {
            critically_injured: false,
            limping: false,
            dead: false,
            abandoned: false,
            death_cause_id: "",
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeStamina {
    pub current: I32F32,
    pub exhausted: bool,
    pub can_jump: bool,
    pub can_sprint: bool,
    pub activity: EmployeeActivity,
    pub tzp_inhalant_active: bool,
    pub zap_gun_stunned: bool,
}

impl Default for EmployeeStamina {
    fn default() -> Self {
        Self {
            current: EMPLOYEE_STAMINA_MAX,
            exhausted: false,
            can_jump: true,
            can_sprint: true,
            activity: EmployeeActivity::Standing,
            tzp_inhalant_active: false,
            zap_gun_stunned: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeActivity {
    Standing,
    Walking,
    Sprinting,
    Jumping,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeScannerTarget {
    pub stable_target_id: u64,
    pub category: EmployeeScanCategory,
    pub value: I32F32,
    pub highlighted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeScanCategory {
    Entrance,
    Ship,
    Log,
    Scrap,
    Entity,
    Hazard,
}

#[derive(Bundle, Debug, Clone)]
pub struct EmployeeBundle {
    pub employee: Employee,
    pub health: Health,
    pub stats: UnitStats,
    pub vitals: EmployeeVitals,
    pub stamina: EmployeeStamina,
}

impl Default for EmployeeBundle {
    fn default() -> Self {
        Self {
            employee: Employee::default(),
            health: Health::full(EMPLOYEE_HEALTH),
            stats: UnitStats {
                move_speed: I32F32::ZERO,
                attack_range: I32F32::ZERO,
                attack_damage: I32F32::ZERO,
                attack_speed: I32F32::ZERO,
                watch_range: I32F32::ZERO,
            },
            vitals: EmployeeVitals::default(),
            stamina: EmployeeStamina::default(),
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EmployeeRunState {
    pub spawned_employees: u64,
    pub deaths: u64,
    pub abandoned_alive: u64,
    pub scans_completed: u64,
    pub highlighted_targets: u64,
    pub scanned_valuable_total: I32F32,
    pub first_entity_scan_seen: bool,
    pub terminal_new_creature_dialogs: u64,
}

impl Default for EmployeeRunState {
    fn default() -> Self {
        Self {
            spawned_employees: 0,
            deaths: 0,
            abandoned_alive: 0,
            scans_completed: 0,
            highlighted_targets: 0,
            scanned_valuable_total: I32F32::ZERO,
            first_entity_scan_seen: false,
            terminal_new_creature_dialogs: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnEmployeeEvent {
    pub stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeMovementEvent {
    pub employee: Entity,
    pub activity: EmployeeActivity,
    pub tzp_inhalant_active: bool,
    pub zap_gun_stunned: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeScanEvent {
    pub employee: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeScanResolvedEvent {
    pub employee: Entity,
    pub highlighted_targets: u64,
    pub valuable_total: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeFirstEntityScanEvent {
    pub employee: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeDeathEvent {
    pub employee: Entity,
    pub stable_id: u64,
    pub cause_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeBodySpawnedEvent {
    pub employee: Entity,
    pub stable_id: u64,
    pub cause_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeAbandonedEvent {
    pub employee: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeEndStatusEvent {
    pub employee: Entity,
    pub stable_id: u64,
    pub status: EmployeeEndStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeEndStatus {
    Alive,
    Missing,
    Deceased,
}

pub fn employee_full_health() -> Health {
    Health::full(EMPLOYEE_HEALTH)
}

pub fn employee_stamina_refund(damage_amount: I32F32) -> I32F32 {
    if damage_amount > I32F32::lit("19") {
        damage_amount / EMPLOYEE_STAMINA_REFUND_DIVISOR
    } else {
        I32F32::ZERO
    }
}

pub fn employee_is_critically_injured(health: Health) -> bool {
    health.current < EMPLOYEE_CRITICAL_INJURY_THRESHOLD
}

pub fn employee_is_limping(health: Health, vitals: EmployeeVitals) -> bool {
    vitals.critically_injured && health.current < EMPLOYEE_CRITICAL_LIMP_THRESHOLD
}

fn employee_spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnEmployeeEvent>,
    mut state: ResMut<EmployeeRunState>,
) {
    for event in events.read() {
        commands.spawn(EmployeeBundle {
            employee: Employee {
                stable_id: event.stable_id,
            },
            ..Default::default()
        });
        state.spawned_employees = state.spawned_employees.wrapping_add(1);
    }
}

fn employee_apply_movement(
    mut events: EventReader<EmployeeMovementEvent>,
    sim_hz: Res<SimHz>,
    mut query: Query<(&mut EmployeeStamina, &EmployeeVitals)>,
) {
    for event in events.read() {
        let Ok((mut stamina, vitals)) = query.get_mut(event.employee) else {
            continue;
        };

        if vitals.dead || vitals.abandoned {
            continue;
        }

        stamina.activity = event.activity;
        stamina.tzp_inhalant_active = event.tzp_inhalant_active;
        stamina.zap_gun_stunned = event.zap_gun_stunned;

        let mut delta = I32F32::ZERO;
        match event.activity {
            EmployeeActivity::Standing => {
                delta = EMPLOYEE_STANDING_STAMINA_REGEN_RATE / sim_hz.0;
            }
            EmployeeActivity::Walking => {
                delta = EMPLOYEE_WALKING_STAMINA_REGEN_RATE / sim_hz.0;
                if event.zap_gun_stunned {
                    delta = (I32F32::ZERO - EMPLOYEE_WALKING_STAMINA_REGEN_RATE)
                        / I32F32::lit("2")
                        / sim_hz.0;
                }
            }
            EmployeeActivity::Sprinting => {
                if stamina.can_sprint && !employee_is_movement_blocked_by_limp(*vitals) {
                    delta = (I32F32::ZERO - EMPLOYEE_SPRINT_STAMINA_DRAIN_RATE) / sim_hz.0;
                }
            }
            EmployeeActivity::Jumping => {
                if stamina.can_jump {
                    delta = I32F32::ZERO - EMPLOYEE_JUMP_STAMINA_COST;
                }
            }
        }

        if event.tzp_inhalant_active {
            if delta < I32F32::ZERO {
                delta = delta / I32F32::lit("2");
            } else {
                delta = delta * I32F32::lit("2");
            }
        }

        stamina.current = clamp_stamina(stamina.current + delta);
        refresh_stamina_gates(&mut stamina);
    }
}

fn employee_apply_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    mut death_events: EventWriter<EmployeeDeathEvent>,
    mut body_events: EventWriter<EmployeeBodySpawnedEvent>,
    mut query: Query<(&Employee, &mut Health, &mut EmployeeVitals, &mut EmployeeStamina)>,
    mut state: ResMut<EmployeeRunState>,
) {
    for event in damage_events.read() {
        let Ok((employee, mut health, mut vitals, mut stamina)) = query.get_mut(event.target) else {
            continue;
        };

        if vitals.dead || vitals.abandoned {
            continue;
        }

        let lethal = health.current - event.raw_amount <= I32F32::ZERO;
        if lethal
            && event.raw_amount < EMPLOYEE_CRITICAL_DAMAGE_CAP
            && !vitals.critically_injured
        {
            health.current = EMPLOYEE_CRITICAL_DAMAGE_FLOOR;
            vitals.critically_injured = true;
        } else {
            health.current = health.current - event.raw_amount;
        }

        stamina.current = clamp_stamina(stamina.current + employee_stamina_refund(event.raw_amount));
        refresh_stamina_gates(&mut stamina);

        if health.current < EMPLOYEE_CRITICAL_INJURY_THRESHOLD && health.current > I32F32::ZERO {
            vitals.critically_injured = true;
        }

        vitals.limping = employee_is_limping(*health, *vitals);

        if health.current <= I32F32::ZERO {
            health.current = I32F32::ZERO;
            vitals.dead = true;
            vitals.death_cause_id = "incoming_damage";
            state.deaths = state.deaths.wrapping_add(1);

            killed_events.send(EntityKilledEvent {
                entity: event.target,
                killer: event.source,
                exp_reward: I32F32::ZERO,
                difficulty_tier: 0,
            });
            death_events.send(EmployeeDeathEvent {
                employee: event.target,
                stable_id: employee.stable_id,
                cause_id: vitals.death_cause_id,
            });
            body_events.send(EmployeeBodySpawnedEvent {
                employee: event.target,
                stable_id: employee.stable_id,
                cause_id: vitals.death_cause_id,
            });
        }
    }
}

fn employee_regenerate_critical_health(
    sim_hz: Res<SimHz>,
    mut query: Query<(&mut Health, &mut EmployeeVitals)>,
) {
    for (mut health, mut vitals) in query.iter_mut() {
        if vitals.dead || vitals.abandoned || !vitals.critically_injured {
            continue;
        }

        if health.current >= EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH {
            health.current = EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH;
            vitals.critically_injured = false;
            vitals.limping = false;
            continue;
        }

        health.current = health.current + (EMPLOYEE_CRITICAL_INJURY_REGEN_RATE / sim_hz.0);
        if health.current >= EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH {
            health.current = EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH;
            vitals.critically_injured = false;
        }

        vitals.limping = employee_is_limping(*health, *vitals);
    }
}

fn employee_resolve_scan(
    mut scan_events: EventReader<EmployeeScanEvent>,
    mut resolved_events: EventWriter<EmployeeScanResolvedEvent>,
    mut first_entity_events: EventWriter<EmployeeFirstEntityScanEvent>,
    mut state: ResMut<EmployeeRunState>,
    mut targets: Query<&mut EmployeeScannerTarget>,
) {
    for event in scan_events.read() {
        let mut highlighted_targets = 0;
        let mut valuable_total = I32F32::ZERO;
        let mut entity_seen = false;

        for mut target in targets.iter_mut() {
            target.highlighted = true;
            highlighted_targets += 1;

            if target.category == EmployeeScanCategory::Scrap {
                valuable_total = valuable_total + target.value;
            }

            if target.category == EmployeeScanCategory::Entity {
                entity_seen = true;
            }
        }

        state.scans_completed = state.scans_completed.wrapping_add(1);
        state.highlighted_targets = state.highlighted_targets.wrapping_add(highlighted_targets);
        state.scanned_valuable_total = state.scanned_valuable_total + valuable_total;

        if entity_seen && !state.first_entity_scan_seen {
            state.first_entity_scan_seen = true;
            state.terminal_new_creature_dialogs =
                state.terminal_new_creature_dialogs.wrapping_add(1);
            first_entity_events.send(EmployeeFirstEntityScanEvent {
                employee: event.employee,
            });
        }

        resolved_events.send(EmployeeScanResolvedEvent {
            employee: event.employee,
            highlighted_targets,
            valuable_total,
        });
    }
}

fn employee_mark_abandoned(
    mut abandoned_events: EventReader<EmployeeAbandonedEvent>,
    mut status_events: EventWriter<EmployeeEndStatusEvent>,
    mut query: Query<(&Employee, &mut EmployeeVitals)>,
    mut state: ResMut<EmployeeRunState>,
) {
    for event in abandoned_events.read() {
        let Ok((employee, mut vitals)) = query.get_mut(event.employee) else {
            continue;
        };

        if vitals.dead {
            status_events.send(EmployeeEndStatusEvent {
                employee: event.employee,
                stable_id: employee.stable_id,
                status: EmployeeEndStatus::Deceased,
            });
            continue;
        }

        if !vitals.abandoned {
            state.abandoned_alive = state.abandoned_alive.wrapping_add(1);
        }

        vitals.abandoned = true;
        status_events.send(EmployeeEndStatusEvent {
            employee: event.employee,
            stable_id: employee.stable_id,
            status: EmployeeEndStatus::Missing,
        });
    }
}

fn employee_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<EmployeeRunState>,
    employee_query: Query<(&Employee, &Health, &UnitStats, &EmployeeVitals, &EmployeeStamina)>,
    scan_query: Query<&EmployeeScannerTarget>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(EMPLOYEE_SOURCE_REVISION as u64);
    checksum.accumulate(EMPLOYEE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EMPLOYEE_HEALTH.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_INJURY_THRESHOLD.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_INJURY_RECOVERY_HEALTH.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_INJURY_REGEN_RATE.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_LIMP_THRESHOLD.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_DAMAGE_FLOOR.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_CRITICAL_DAMAGE_CAP.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_MIN.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_MAX.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_EXHAUSTED_THRESHOLD.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_JUMP_RESUME_THRESHOLD.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_SPRINT_RESUME_THRESHOLD.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_JUMP_STAMINA_COST.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_SPRINT_STAMINA_DRAIN_RATE.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_WALKING_STAMINA_REGEN_RATE.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STANDING_STAMINA_REGEN_RATE.to_bits() as u64);
    checksum.accumulate(EMPLOYEE_STAMINA_REFUND_DIVISOR.to_bits() as u64);

    for dependency in EMPLOYEE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in EMPLOYEE_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for modifier in EMPLOYEE_MODIFIERS {
        accumulate_str(&mut checksum, 0x3000, modifier.condition);
        accumulate_str(&mut checksum, 0x3001, modifier.outcome);
    }

    for rule in EMPLOYEE_STRATEGY {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for note in EMPLOYEE_NOTES {
        accumulate_str(&mut checksum, 0x5000, note.condition);
        accumulate_str(&mut checksum, 0x5001, note.outcome);
    }

    for rule in EMPLOYEE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x6000, rule.condition);
        accumulate_str(&mut checksum, 0x6001, rule.outcome);
    }

    checksum.accumulate(state.spawned_employees);
    checksum.accumulate(state.deaths);
    checksum.accumulate(state.abandoned_alive);
    checksum.accumulate(state.scans_completed);
    checksum.accumulate(state.highlighted_targets);
    checksum.accumulate(state.scanned_valuable_total.to_bits() as u64);
    checksum.accumulate(state.first_entity_scan_seen as u64);
    checksum.accumulate(state.terminal_new_creature_dialogs);

    for (employee, health, stats, vitals, stamina) in employee_query.iter() {
        checksum.accumulate(employee.stable_id);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(vitals.critically_injured as u64);
        checksum.accumulate(vitals.limping as u64);
        checksum.accumulate(vitals.dead as u64);
        checksum.accumulate(vitals.abandoned as u64);
        accumulate_str(&mut checksum, 0x7000, vitals.death_cause_id);
        checksum.accumulate(stamina.current.to_bits() as u64);
        checksum.accumulate(stamina.exhausted as u64);
        checksum.accumulate(stamina.can_jump as u64);
        checksum.accumulate(stamina.can_sprint as u64);
        checksum.accumulate(employee_activity_bits(stamina.activity));
        checksum.accumulate(stamina.tzp_inhalant_active as u64);
        checksum.accumulate(stamina.zap_gun_stunned as u64);
    }

    for target in scan_query.iter() {
        checksum.accumulate(target.stable_target_id);
        checksum.accumulate(employee_scan_category_bits(target.category));
        checksum.accumulate(target.value.to_bits() as u64);
        checksum.accumulate(target.highlighted as u64);
    }
}

fn employee_is_movement_blocked_by_limp(vitals: EmployeeVitals) -> bool {
    vitals.critically_injured && vitals.limping
}

fn clamp_stamina(value: I32F32) -> I32F32 {
    if value < EMPLOYEE_STAMINA_MIN {
        EMPLOYEE_STAMINA_MIN
    } else if value > EMPLOYEE_STAMINA_MAX {
        EMPLOYEE_STAMINA_MAX
    } else {
        value
    }
}

fn refresh_stamina_gates(stamina: &mut EmployeeStamina) {
    if stamina.current < EMPLOYEE_STAMINA_EXHAUSTED_THRESHOLD {
        stamina.exhausted = true;
        stamina.can_jump = false;
        stamina.can_sprint = false;
    }

    if stamina.current >= EMPLOYEE_STAMINA_JUMP_RESUME_THRESHOLD {
        stamina.can_jump = true;
    }

    if stamina.current >= EMPLOYEE_STAMINA_SPRINT_RESUME_THRESHOLD {
        stamina.can_sprint = true;
        stamina.exhausted = false;
    }
}

fn employee_activity_bits(activity: EmployeeActivity) -> u64 {
    match activity {
        EmployeeActivity::Standing => 1,
        EmployeeActivity::Walking => 2,
        EmployeeActivity::Sprinting => 3,
        EmployeeActivity::Jumping => 4,
    }
}

fn employee_scan_category_bits(category: EmployeeScanCategory) -> u64 {
    match category {
        EmployeeScanCategory::Entrance => 1,
        EmployeeScanCategory::Ship => 2,
        EmployeeScanCategory::Log => 3,
        EmployeeScanCategory::Scrap => 4,
        EmployeeScanCategory::Entity => 5,
        EmployeeScanCategory::Hazard => 6,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}