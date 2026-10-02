use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use vc_llm::ModelProfile;

/// Target execution mode for GPU resource multiplexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Chat generation active (e.g. Qwen2.5:3B, ~2000 MB VRAM).
    Chat,
    /// Screen and image visual perception active (e.g. Qwen2.5-VL:3B, ~3200 MB VRAM).
    Vision,
    /// Low-power idle state between turns (e.g. ~500 MB baseline).
    Idle,
}

/// Errors raised when hardware safety boundaries or VRAM thresholds are breached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// Requested operation exceeds the strict VRAM budget (e.g. >3500 MB on 4GB RTX 3050).
    VramBudgetExceeded {
        required_mb: u32,
        current_mb: u32,
        limit_mb: u32,
    },
    /// An operation attempted to run concurrently while GPU is locked in an incompatible mode.
    ModeConflict {
        current: ExecutionMode,
        requested: ExecutionMode,
    },
    /// General system resource constraint.
    General(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VramBudgetExceeded {
                required_mb,
                current_mb,
                limit_mb,
            } => {
                write!(
                    f,
                    "VRAM budget exceeded: requires {} MB, current {} MB, hard ceiling is {} MB",
                    required_mb, current_mb, limit_mb
                )
            }
            Self::ModeConflict { current, requested } => {
                write!(
                    f,
                    "Resource conflict: GPU currently locked in {:?}, requested {:?}",
                    current, requested
                )
            }
            Self::General(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Real-time hardware utilization snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Estimated or measured GPU VRAM usage in megabytes.
    pub vram_used_mb: u32,
    /// Hard safety limit for VRAM before rejecting tasks (e.g. 3500 MB for 4GB RTX 3050).
    pub vram_limit_mb: u32,
    /// Estimated or measured system RAM usage in megabytes.
    pub system_ram_mb: u32,
    /// Current GPU execution mode.
    pub active_mode: ExecutionMode,
    /// Warning flag if VRAM utilization is above 85%.
    pub is_near_capacity: bool,
}

/// Dynamic resource orchestrator enforcing the strict 4GB VRAM ceiling.
///
/// Designed under Skill 22 (Runtime Engineering) and Skill 25 (Security/Resource Guardrails)
/// to multiplex AI models on personal hardware (Intel i5-12500H + NVIDIA RTX 3050 4GB).
pub struct ResourceManager {
    /// Baseline VRAM allocated by Windows OS / Desktop Window Manager (typically ~1000 MB).
    os_baseline_vram_mb: u32,
    /// Strict upper limit for total VRAM (default: 3500 MB on 4096 MB hardware).
    max_vram_mb: u32,
    /// Current execution mode and state.
    active_mode: RwLock<ExecutionMode>,
    /// Currently loaded model profile in VRAM (if any).
    active_vram_model: RwLock<Option<ModelProfile>>,
}

impl ResourceManager {
    /// Create a resource manager tuned for RTX 3050 4GB Laptop GPU.
    pub fn new_rtx3050_profile() -> Self {
        Self {
            os_baseline_vram_mb: 1000,
            max_vram_mb: 3500, // 3500 MB allows 596 MB buffer for display & spikes
            active_mode: RwLock::new(ExecutionMode::Chat),
            active_vram_model: RwLock::new(None),
        }
    }

    /// Create a customized resource manager with explicit limits.
    pub fn with_limits(os_baseline_vram_mb: u32, max_vram_mb: u32) -> Self {
        Self {
            os_baseline_vram_mb,
            max_vram_mb,
            active_mode: RwLock::new(ExecutionMode::Idle),
            active_vram_model: RwLock::new(None),
        }
    }

    /// Return the current hardware resource snapshot.
    pub fn snapshot(&self) -> ResourceSnapshot {
        let mode = *self.active_mode.read().unwrap();
        let model_guard = self.active_vram_model.read().unwrap();
        let model_vram = model_guard
            .as_ref()
            .map(|m| m.vram_mb_required)
            .unwrap_or(0);
        let total_vram = self.os_baseline_vram_mb + model_vram;

        let is_near_capacity = total_vram as f32 > (self.max_vram_mb as f32 * 0.85);

        ResourceSnapshot {
            vram_used_mb: total_vram,
            vram_limit_mb: self.max_vram_mb,
            system_ram_mb: 0, // Offloaded tasks (Whisper/Piper/FastEmbed) live in 32GB RAM
            active_mode: mode,
            is_near_capacity,
        }
    }

    /// Check if loading a model profile would exceed the strict safety ceiling.
    pub fn can_allocate(&self, profile: &ModelProfile) -> bool {
        let total_projected = self.os_baseline_vram_mb + profile.vram_mb_required;
        total_projected <= self.max_vram_mb
    }

    /// Switch active GPU model under Model Multiplexing.
    ///
    /// Swapping between Chat LLM and Vision LLM prevents simultaneous VRAM loading.
    pub fn switch_active_model(
        &self,
        profile: ModelProfile,
        target_mode: ExecutionMode,
    ) -> Result<(), ResourceError> {
        let projected_vram = self.os_baseline_vram_mb + profile.vram_mb_required;
        if projected_vram > self.max_vram_mb {
            return Err(ResourceError::VramBudgetExceeded {
                required_mb: projected_vram,
                current_mb: self.snapshot().vram_used_mb,
                limit_mb: self.max_vram_mb,
            });
        }

        *self.active_mode.write().unwrap() = target_mode;
        *self.active_vram_model.write().unwrap() = Some(profile);
        Ok(())
    }

    /// Unload active GPU model back to Idle baseline.
    pub fn release_gpu_to_idle(&self) {
        *self.active_mode.write().unwrap() = ExecutionMode::Idle;
        *self.active_vram_model.write().unwrap() = None;
    }

    /// Get current active mode.
    pub fn current_mode(&self) -> ExecutionMode {
        *self.active_mode.read().unwrap()
    }

    /// Set execution mode directly.
    pub fn set_mode(&self, mode: ExecutionMode) -> Result<(), ResourceError> {
        *self.active_mode.write().unwrap() = mode;
        Ok(())
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new_rtx3050_profile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtx3050_profile_defaults() {
        let manager = ResourceManager::new_rtx3050_profile();
        let snap = manager.snapshot();

        assert_eq!(snap.vram_limit_mb, 3500);
        assert_eq!(snap.vram_used_mb, 1000); // OS baseline only
        assert!(!snap.is_near_capacity);
    }

    #[test]
    fn test_can_allocate_chat_model() {
        let manager = ResourceManager::new_rtx3050_profile();
        let qwen_chat = ModelProfile::preset_qwen2_5_3b(); // 2000 MB VRAM

        assert!(manager.can_allocate(&qwen_chat));
        assert!(manager
            .switch_active_model(qwen_chat, ExecutionMode::Chat)
            .is_ok());

        let snap = manager.snapshot();
        assert_eq!(snap.vram_used_mb, 3000); // 1000 OS + 2000 model
        assert!(snap.is_near_capacity); // 3000 > 3500 * 0.85 (2975)
    }

    #[test]
    fn test_vram_budget_exceeded_for_oversized_model() {
        let manager = ResourceManager::new_rtx3050_profile();
        // Model requiring 3000 MB + 1000 MB OS = 4000 MB > 3500 MB limit
        let huge_model = ModelProfile::new("huge-model", "local", 3000, 1000, 8192);

        assert!(!manager.can_allocate(&huge_model));
        let res = manager.switch_active_model(huge_model, ExecutionMode::Chat);
        assert!(matches!(res, Err(ResourceError::VramBudgetExceeded { .. })));
    }

    #[test]
    fn test_gpu_model_multiplexing_switch() {
        let manager = ResourceManager::new_rtx3050_profile();
        let qwen_chat = ModelProfile::preset_qwen2_5_3b();
        let moondream = ModelProfile::preset_moondream(); // 1400 MB VRAM

        // 1. Load Chat
        manager
            .switch_active_model(qwen_chat, ExecutionMode::Chat)
            .unwrap();
        assert_eq!(manager.current_mode(), ExecutionMode::Chat);
        assert_eq!(manager.snapshot().vram_used_mb, 3000);

        // 2. Multiplex switch to Vision (unloads Chat, loads Moondream)
        manager
            .switch_active_model(moondream, ExecutionMode::Vision)
            .unwrap();
        assert_eq!(manager.current_mode(), ExecutionMode::Vision);
        assert_eq!(manager.snapshot().vram_used_mb, 2400); // 1000 + 1400

        // 3. Release to Idle
        manager.release_gpu_to_idle();
        assert_eq!(manager.current_mode(), ExecutionMode::Idle);
        assert_eq!(manager.snapshot().vram_used_mb, 1000);
    }
}
