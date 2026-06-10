// Foundation lib.rs written by phase2/scaffold.py.
//
// Holds only the foundation modules. Per-kind `pub mod <kind>;` declarations
// are appended by phase2/loop_driver.py (via the `module_registration.crate_root`
// mechanism) as each new kind's first leaf module lands. Do not maintain those
// by hand — the loop owns the crate root.

pub mod sim;
pub mod app_plugins;
