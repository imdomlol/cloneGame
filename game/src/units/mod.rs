use bevy::prelude::Component;

/// Marker for infected / enemy-faction entities.
/// Declared here so all unit modules share one canonical type.
#[derive(Component)]
pub struct Infected;

pub mod soldier;
pub mod ranger;
pub mod caelus;
pub mod calliope;
pub mod mutant;
pub mod thanatos;
pub mod titan;
pub mod sniper;
pub mod lucifer;
