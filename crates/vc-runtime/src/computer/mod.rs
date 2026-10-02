//! Controlled Computer Use Subsystem.
//!
//! Provides actions, tool permission policy, execution engines, and screen verification feedback loop.

pub mod action;
pub mod executor;
pub mod policy;
pub mod verifier;
pub mod windows;

pub use action::{ComputerAction, MouseButton};
pub use executor::{ActionResult, ComputerExecutor, MockComputerExecutor};
pub use policy::{ToolPermissionPolicy, ToolRiskLevel};
pub use verifier::{ScreenVerificationLoop, VerificationResult};
pub use windows::WindowsComputerExecutor;
