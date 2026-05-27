//! Smoke test: every unit's spawned `Health.max` must equal its vault HP.
//!
//! Source of truth is each unit's vault note (`vault/units/<slug>.md`, `hp:`
//! field). This guards against silent constant drift between the vault and the
//! generated code — a regression where a codegen turn rounds, truncates, or
//! drops a stat would otherwise pass `cargo build` unnoticed.
//!
//! Every unit exposes a `<Unit>Bundle` whose `Default` builds
//! `Health::full(<UNIT>_HP)`, so `<Unit>Bundle::default().health.max` is the
//! HP a freshly spawned unit gets. The expected values below are transcribed
//! from the vault; update them only when the vault HP genuinely changes.

use clone_game::units::caelus::CaelusBundle;
use clone_game::units::calliope::CalliopeBundle;
use clone_game::units::lucifer::LuciferBundle;
use clone_game::units::mutant::MutantBundle;
use clone_game::units::ranger::RangerBundle;
use clone_game::units::sniper::SniperBundle;
use clone_game::units::soldier::SoldierBundle;
use clone_game::units::thanatos::ThanatosBundle;
use clone_game::units::titan::TitanBundle;
use fixed::types::I32F32;

#[test]
fn unit_health_max_matches_vault_hp() {
    assert_eq!(CaelusBundle::default().health.max, I32F32::lit("120"));
    assert_eq!(CalliopeBundle::default().health.max, I32F32::lit("60"));
    assert_eq!(LuciferBundle::default().health.max, I32F32::lit("500"));
    assert_eq!(MutantBundle::default().health.max, I32F32::lit("2000"));
    assert_eq!(RangerBundle::default().health.max, I32F32::lit("60"));
    assert_eq!(SniperBundle::default().health.max, I32F32::lit("150"));
    assert_eq!(SoldierBundle::default().health.max, I32F32::lit("120"));
    assert_eq!(ThanatosBundle::default().health.max, I32F32::lit("250"));
    assert_eq!(TitanBundle::default().health.max, I32F32::lit("800"));
}
