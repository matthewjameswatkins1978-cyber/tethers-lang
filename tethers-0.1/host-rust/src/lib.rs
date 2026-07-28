// Tethers Reference Host library - reusable foundations for J13.
//
// The library exposes CLI parsing, child-process supervision, engine
// session management, and shared outcome types.  The binary crate
// (main.rs) owns the legacy host internals and application coordinator.

pub mod child_process;
pub mod cli;
pub mod engine_stdio;
