// Sources: vault/item_index_pages/items.md, vault/version_pages/version_72.md
use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimTick};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudState>().add_systems(
            Update,
            (
                spawn_hud_once,
                refresh_hud_rows,
                hud_checksum,
            )
                .chain(),
        );
    }
}

#[derive(Resource, Clone, Debug)]
pub struct HudState {
    pub quota_earned: I32F32,
    pub quota_required: I32F32,
    pub scrap_inventory: Vec<HudInventoryLine>,
    pub equipped_weapons: Vec<HudEquipmentLine>,
    pub active_upgrades: Vec<HudUpgradeLine>,
    pub time_remaining_ticks: u32,
    pub ticks_per_second: u32,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            quota_earned: I32F32::from_num(0),
            quota_required: I32F32::from_num(0),
            scrap_inventory: Vec::new(),
            equipped_weapons: Vec::new(),
            active_upgrades: Vec::new(),
            time_remaining_ticks: 0,
            ticks_per_second: 20,
        }
    }
}

impl HudState {
    pub fn set_quota_progress(&mut self, earned: I32F32, required: I32F32) {
        self.quota_earned = earned;
        self.quota_required = required;
    }

    pub fn set_time_remaining(&mut self, ticks: u32, ticks_per_second: u32) {
        self.time_remaining_ticks = ticks;
        self.ticks_per_second = ticks_per_second.max(1);
    }

    pub fn replace_scrap_inventory(&mut self, mut inventory: Vec<HudInventoryLine>) {
        inventory.sort_by(|left, right| left.item_id.cmp(&right.item_id));
        self.scrap_inventory = inventory;
    }

    pub fn replace_equipped_weapons(&mut self, mut weapons: Vec<HudEquipmentLine>) {
        weapons.sort_by(|left, right| left.item_id.cmp(&right.item_id));
        self.equipped_weapons = weapons;
    }

    pub fn replace_active_upgrades(&mut self, mut upgrades: Vec<HudUpgradeLine>) {
        upgrades.sort_by(|left, right| left.upgrade_id.cmp(&right.upgrade_id));
        self.active_upgrades = upgrades;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HudInventoryLine {
    pub item_id: &'static str,
    pub display_name: &'static str,
    pub value: I32F32,
    pub weight_lbs: I32F32,
    pub count: u32,
    pub conductive: bool,
    pub two_handed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HudEquipmentLine {
    pub item_id: &'static str,
    pub display_name: &'static str,
    pub ammo_loaded: u32,
    pub ammo_reserve: u32,
    pub changes_threat_level: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HudUpgradeLine {
    pub upgrade_id: &'static str,
    pub display_name: &'static str,
    pub active: bool,
}

#[derive(Component, Default)]
struct HudRoot;

#[derive(Component, Default)]
struct HudQuotaText;

#[derive(Component, Default)]
struct HudScrapText;

#[derive(Component, Default)]
struct HudWeaponsText;

#[derive(Component, Default)]
struct HudUpgradesText;

#[derive(Component, Default)]
struct HudTimeText;

const HUD_PANEL_COLOR: Color = Color::srgba(0.02, 0.025, 0.03, 0.78);
const HUD_TEXT_COLOR: Color = Color::srgb(0.88, 0.93, 0.88);

fn spawn_hud_once(mut commands: Commands, root_query: Query<Entity, With<HudRoot>>) {
    if !root_query.is_empty() {
        return;
    }

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(420.0),
                max_width: Val::Percent(46.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..Default::default()
            },
            BackgroundColor(HUD_PANEL_COLOR),
        ))
        .with_children(|root| {
            spawn_hud_text(root, HudQuotaText, "Quota: 0 / 0");
            spawn_hud_text(root, HudScrapText, "Scrap: none");
            spawn_hud_text(root, HudWeaponsText, "Weapons: none equipped");
            spawn_hud_text(root, HudUpgradesText, "Upgrades: none active");
            spawn_hud_text(root, HudTimeText, "Time: 00:00");
        });
}

fn spawn_hud_text<T: Component + Default>(parent: &mut ChildBuilder, marker: T, value: &str) {
    parent.spawn((
        marker,
        Text::new(value),
        TextFont {
            font_size: 16.0,
            ..Default::default()
        },
        TextColor(HUD_TEXT_COLOR),
        Node {
            min_height: Val::Px(20.0),
            ..Default::default()
        },
    ));
}

fn refresh_hud_rows(
    hud: Res<HudState>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<HudQuotaText>>,
        Query<&mut Text, With<HudScrapText>>,
        Query<&mut Text, With<HudWeaponsText>>,
        Query<&mut Text, With<HudUpgradesText>>,
        Query<&mut Text, With<HudTimeText>>,
    )>,
) {
    if !hud.is_changed() {
        return;
    }

    for mut text in &mut text_queries.p0() {
        text.0 = format!(
            "Quota: {} / {}",
            fixed_to_display(hud.quota_earned),
            fixed_to_display(hud.quota_required)
        );
    }

    for mut text in &mut text_queries.p1() {
        text.0 = format!("Scrap: {}", scrap_summary(&hud.scrap_inventory));
    }

    for mut text in &mut text_queries.p2() {
        text.0 = format!("Weapons: {}", weapons_summary(&hud.equipped_weapons));
    }

    for mut text in &mut text_queries.p3() {
        text.0 = format!("Upgrades: {}", upgrades_summary(&hud.active_upgrades));
    }

    for mut text in &mut text_queries.p4() {
        text.0 = format!("Time: {}", time_summary(hud.time_remaining_ticks, hud.ticks_per_second));
    }
}

fn scrap_summary(lines: &[HudInventoryLine]) -> String {
    if lines.is_empty() {
        return "none".to_string();
    }

    let mut parts = Vec::with_capacity(lines.len());
    for line in lines {
        let hand_label = if line.two_handed { "two-handed" } else { "one-handed" };
        let conductive_label = if line.conductive { "conductive" } else { "non-conductive" };
        parts.push(format!(
            "{} x{} (value {}, {} lb, {}, {})",
            line.display_name,
            line.count,
            fixed_to_display(line.value),
            fixed_to_display(line.weight_lbs),
            conductive_label,
            hand_label
        ));
    }
    parts.join(" | ")
}

fn weapons_summary(lines: &[HudEquipmentLine]) -> String {
    if lines.is_empty() {
        return "none equipped".to_string();
    }

    let mut parts = Vec::with_capacity(lines.len());
    for line in lines {
        let threat_label = if line.changes_threat_level {
            "threat"
        } else {
            "no threat"
        };
        parts.push(format!(
            "{} ({}/{}, {})",
            line.display_name, line.ammo_loaded, line.ammo_reserve, threat_label
        ));
    }
    parts.join(" | ")
}

fn upgrades_summary(lines: &[HudUpgradeLine]) -> String {
    let mut parts = Vec::new();
    for line in lines {
        if line.active {
            parts.push(line.display_name);
        }
    }

    if parts.is_empty() {
        "none active".to_string()
    } else {
        parts.join(" | ")
    }
}

fn time_summary(ticks: u32, ticks_per_second: u32) -> String {
    let safe_hz = ticks_per_second.max(1);
    let total_seconds = ticks / safe_hz;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn fixed_to_display(value: I32F32) -> String {
    let raw = value.to_bits();
    let integer = raw >> 32;
    let fraction = ((raw & 0xffff_ffff) * 100) >> 32;

    if fraction == 0 {
        format!("{integer}")
    } else {
        format!("{integer}.{fraction:02}")
    }
}

fn hud_checksum(
    hud: Res<HudState>,
    tick: Res<SimTick>,
    mut checksum: ResMut<SimChecksumState>,
) {
    if !hud.is_changed() {
        return;
    }

    checksum.accumulate(tick.0 as u64);
    checksum.accumulate(hud.quota_earned.to_bits() as u64);
    checksum.accumulate(hud.quota_required.to_bits() as u64);
    checksum.accumulate(hud.time_remaining_ticks as u64);
    checksum.accumulate(hud.ticks_per_second as u64);

    for line in &hud.scrap_inventory {
        checksum.accumulate(stable_str_bits(line.item_id));
        checksum.accumulate(line.value.to_bits() as u64);
        checksum.accumulate(line.weight_lbs.to_bits() as u64);
        checksum.accumulate(line.count as u64);
        checksum.accumulate(line.conductive as u64);
        checksum.accumulate(line.two_handed as u64);
    }

    for line in &hud.equipped_weapons {
        checksum.accumulate(stable_str_bits(line.item_id));
        checksum.accumulate(line.ammo_loaded as u64);
        checksum.accumulate(line.ammo_reserve as u64);
        checksum.accumulate(line.changes_threat_level as u64);
    }

    for line in &hud.active_upgrades {
        checksum.accumulate(stable_str_bits(line.upgrade_id));
        checksum.accumulate(line.active as u64);
    }
}

fn stable_str_bits(value: &str) -> u64 {
    let mut out = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        out ^= *byte as u64;
        out = out.wrapping_mul(0x0000_0100_0000_01b3);
    }
    out
}

pub const SPECIAL_ITEM_DEFINITIONS: &[HudInventoryLine] = &[
    HudInventoryLine {
        item_id: "clipboard",
        display_name: "Clipboard",
        value: I32F32::ZERO,
        weight_lbs: I32F32::ZERO,
        count: 1,
        conductive: false,
        two_handed: false,
    },
    HudInventoryLine {
        item_id: "key",
        display_name: "Key",
        value: I32F32::from_bits(3_i64 << 32),
        weight_lbs: I32F32::ZERO,
        count: 1,
        conductive: true,
        two_handed: false,
    },
    HudInventoryLine {
        item_id: "shotgun_shells",
        display_name: "Shotgun Shells",
        value: I32F32::ZERO,
        weight_lbs: I32F32::ZERO,
        count: 1,
        conductive: false,
        two_handed: false,
    },
    HudInventoryLine {
        item_id: "sticky_note",
        display_name: "Sigurd's Sticky Note",
        value: I32F32::ZERO,
        weight_lbs: I32F32::ZERO,
        count: 1,
        conductive: false,
        two_handed: false,
    },
];