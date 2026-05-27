// Sources: vault/campaign_content/outpost.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, UnitStats};

const OUTPOST_HP: I32F32 = I32F32::lit("2000");
const OUTPOST_DEFENSES_LIFE: I32F32 = I32F32::lit("1000");
const ADVANCED_OUTPOST_DEFENSES_LIFE: I32F32 = I32F32::lit("3000");
const OUTPOST_WATCH_RANGE: I32F32 = I32F32::lit("12");
const OUTPOST_ATTACK_RANGE: I32F32 = I32F32::ZERO;
const OUTPOST_ATTACK_DAMAGE: I32F32 = I32F32::ZERO;
const OUTPOST_ATTACK_SPEED: I32F32 = I32F32::ZERO;
const OUTPOST_MOVE_SPEED: I32F32 = I32F32::ZERO;
const OUTPOST_BUILD_TIME_SECONDS: i32 = 0;
const OUTPOST_ENERGY_COST: i32 = 0;
const OUTPOST_WOOD_COST: i32 = 0;
const OUTPOST_STONE_COST: i32 = 0;
const OUTPOST_IRON_COST: i32 = 0;
const OUTPOST_OIL_COST: i32 = 0;
const OUTPOST_GOLD_COST: i32 = 0;
const ARMORED_ATTACK_STATION_BONUS_EMPIRE_POINTS: i32 = 200;

#[derive(Component, Default)]
pub struct Outpost;

#[derive(Component, Default)]
pub struct AdvancedOutpost;

#[derive(Component, Clone, Copy)]
pub struct OutpostAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct OutpostDefense {
    pub defenses_life: I32F32,
}

impl Default for OutpostDefense {
    fn default() -> Self {
        Self {
            defenses_life: OUTPOST_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct OutpostBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for OutpostBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: OUTPOST_BUILD_TIME_SECONDS,
            completed: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct OutpostCampaignState {
    pub is_swarm_mission_headquarters: bool,
}

#[derive(Component, Clone, Copy)]
pub struct OutpostEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for OutpostEconomy {
    fn default() -> Self {
        Self {
            energy_cost: OUTPOST_ENERGY_COST,
            wood_cost: OUTPOST_WOOD_COST,
            stone_cost: OUTPOST_STONE_COST,
            iron_cost: OUTPOST_IRON_COST,
            oil_cost: OUTPOST_OIL_COST,
            gold_cost: OUTPOST_GOLD_COST,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct OutpostPlacementClaims {
    pub claims: BTreeMap<Entity, OutpostAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceOutpostEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_swarm_mission: bool,
}

#[derive(Event, Clone, Copy)]
pub struct OutpostPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedEnteredMapEvent {
    pub infected_unit_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct AssignInfectedAttackOutpostEvent {
    pub infected_unit_id: i32,
    pub outpost_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct SelectArmoredAttackStationEvent {
    pub outpost_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct OutpostReplacedByAdvancedOutpostEvent {
    pub outpost_entity: Entity,
    pub bonus_empire_points: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageOutpostEvent {
    pub outpost_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct OutpostDestroyedEvent {
    pub outpost_entity: Entity,
}

fn place_outpost_system(
    mut commands: Commands,
    mut events: EventReader<PlaceOutpostEvent>,
    mut rejected: EventWriter<OutpostPlacementRejectedEvent>,
    mut claims: ResMut<OutpostPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = OutpostAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims.claims.values().any(|existing| existing.x == anchor.x && existing.y == anchor.y) {
            rejected.send(OutpostPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Outpost,
                anchor,
                Health::full(OUTPOST_HP),
                OutpostDefense::default(),
                UnitStats {
                    move_speed: OUTPOST_MOVE_SPEED,
                    attack_range: OUTPOST_ATTACK_RANGE,
                    attack_damage: OUTPOST_ATTACK_DAMAGE,
                    attack_speed: OUTPOST_ATTACK_SPEED,
                    watch_range: OUTPOST_WATCH_RANGE,
                },
                OutpostBuildState::default(),
                OutpostCampaignState {
                    is_swarm_mission_headquarters: ev.is_swarm_mission,
                },
                OutpostEconomy::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn infected_attack_outpost_system(
    mut infected_events: EventReader<InfectedEnteredMapEvent>,
    mut assign_events: EventWriter<AssignInfectedAttackOutpostEvent>,
    outposts: Query<(Entity, &OutpostCampaignState), Or<(With<Outpost>, With<AdvancedOutpost>)>>,
) {
    let mut headquarters: Option<Entity> = None;
    for (entity, state) in &outposts {
        if state.is_swarm_mission_headquarters {
            headquarters = Some(entity);
            break;
        }
    }

    let Some(target) = headquarters else {
        return;
    };

    for ev in infected_events.read() {
        assign_events.send(AssignInfectedAttackOutpostEvent {
            infected_unit_id: ev.infected_unit_id,
            outpost_entity: target,
        });
    }
}

fn replace_with_advanced_outpost_system(
    mut commands: Commands,
    mut events: EventReader<SelectArmoredAttackStationEvent>,
    mut replaced: EventWriter<OutpostReplacedByAdvancedOutpostEvent>,
    mut outposts: Query<(Entity, &mut OutpostDefense), With<Outpost>>,
) {
    for ev in events.read() {
        let Ok((entity, mut defense)) = outposts.get_mut(ev.outpost_entity) else {
            continue;
        };

        commands.entity(entity).remove::<Outpost>();
        commands.entity(entity).insert(AdvancedOutpost);

        defense.defenses_life = ADVANCED_OUTPOST_DEFENSES_LIFE;

        replaced.send(OutpostReplacedByAdvancedOutpostEvent {
            outpost_entity: entity,
            bonus_empire_points: ARMORED_ATTACK_STATION_BONUS_EMPIRE_POINTS,
        });
    }
}

fn damage_outpost_system(
    mut commands: Commands,
    mut events: EventReader<DamageOutpostEvent>,
    mut destroyed: EventWriter<OutpostDestroyedEvent>,
    mut outposts: Query<(Entity, &mut Health), Or<(With<Outpost>, With<AdvancedOutpost>)>>,
    mut claims: ResMut<OutpostPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut health)) = outposts.get_mut(ev.outpost_entity) else {
            continue;
        };

        if ev.damage <= I32F32::ZERO {
            continue;
        }

        if health.current > ev.damage {
            health.current -= ev.damage;
            continue;
        }

        health.current = I32F32::ZERO;
        claims.claims.remove(&entity);
        destroyed.send(OutpostDestroyedEvent { outpost_entity: entity });
        commands.entity(entity).despawn();
    }
}

fn outpost_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    outposts: Query<
        (
            Entity,
            &OutpostAnchor,
            &Health,
            &OutpostDefense,
            &UnitStats,
            &OutpostBuildState,
            &OutpostCampaignState,
            &OutpostEconomy,
            Option<&Outpost>,
            Option<&AdvancedOutpost>,
        ),
        Or<(With<Outpost>, With<AdvancedOutpost>)>,
    >,
    claims: Res<OutpostPlacementClaims>,
) {
    for (
        entity,
        anchor,
        health,
        defense,
        stats,
        build,
        campaign,
        economy,
        is_outpost,
        is_advanced,
    ) in &outposts
    {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(defense.defenses_life.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));
        checksum.accumulate(u64::from(campaign.is_swarm_mission_headquarters));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);

        checksum.accumulate(u64::from(is_outpost.is_some()));
        checksum.accumulate(u64::from(is_advanced.is_some()));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct OutpostPlugin;

impl Plugin for OutpostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutpostPlacementClaims>()
            .add_event::<PlaceOutpostEvent>()
            .add_event::<OutpostPlacementRejectedEvent>()
            .add_event::<InfectedEnteredMapEvent>()
            .add_event::<AssignInfectedAttackOutpostEvent>()
            .add_event::<SelectArmoredAttackStationEvent>()
            .add_event::<OutpostReplacedByAdvancedOutpostEvent>()
            .add_event::<DamageOutpostEvent>()
            .add_event::<OutpostDestroyedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_outpost_system,
                    infected_attack_outpost_system,
                    replace_with_advanced_outpost_system,
                    damage_outpost_system,
                    outpost_checksum_system,
                )
                    .chain(),
            );
    }
}