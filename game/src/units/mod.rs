use bevy::prelude::Component;

/// Marker for infected / enemy-faction entities.
/// Declared here so all unit modules share one canonical type.
#[derive(Component)]
pub struct Infected;
pub mod calliope;
pub mod mutant;
pub mod ranger;
pub mod sniper;
pub mod soldier;
pub mod lucifer;
pub mod thanatos;
pub mod titan;
pub mod caelus;
pub mod infected;
pub mod infected_aged;
pub mod infected_behemoth;
pub mod infected_chubby;
pub mod infected_colonists;
pub mod infected_decrepit;
pub mod infected_executive;
pub mod infected_fresh;
pub mod infected_harpy;
pub mod rabies_z;
pub mod v9;
pub mod quintus_crane;
