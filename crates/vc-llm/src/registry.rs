use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Discrete functional capabilities supported by AI models in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    /// Text comprehension and conversation generation.
    TextGeneration,
    /// Image, visual screenshot, and GUI element perception.
    VisionUnderstanding,
    /// Dense vector representation for semantic text search.
    EmbeddingGeneration,
    /// Structured tool/function invocation.
    ToolCalling,
}

/// Resource and capability profile for an AI model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Model identifier (e.g., "qwen2.5:3b", "qwen2.5vl:3b", "gemini-1.5-flash").
    pub name: String,
    /// Provider backend ("ollama", "gemini", "mock", etc.).
    pub provider: String,
    /// Estimated GPU VRAM consumption in megabytes.
    pub vram_mb_required: u32,
    /// Estimated system RAM consumption in megabytes.
    pub ram_mb_required: u32,
    /// Set of capabilities enabled for this model.
    pub capabilities: Vec<ModelCapability>,
    /// Maximum context window in tokens.
    pub context_window: usize,
}

impl ModelProfile {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        vram_mb_required: u32,
        ram_mb_required: u32,
        context_window: usize,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            vram_mb_required,
            ram_mb_required,
            capabilities: Vec::new(),
            context_window,
        }
    }

    pub fn with_capability(mut self, cap: ModelCapability) -> Self {
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
        self
    }

    pub fn with_capabilities(mut self, caps: impl IntoIterator<Item = ModelCapability>) -> Self {
        for cap in caps {
            if !self.capabilities.contains(&cap) {
                self.capabilities.push(cap);
            }
        }
        self
    }

    pub fn has_capability(&self, cap: ModelCapability) -> bool {
        self.capabilities.contains(&cap)
    }

    // --- Standard Preset Profiles ---

    /// Default local chat model for personal hardware (RTX 3050 4GB).
    pub fn preset_qwen2_5_3b() -> Self {
        Self::new("qwen2.5:3b", "ollama", 2000, 500, 32768)
            .with_capability(ModelCapability::TextGeneration)
            .with_capability(ModelCapability::ToolCalling)
    }

    /// Primary local vision model for screen sensing and GUI interaction.
    pub fn preset_qwen2_5_vl_3b() -> Self {
        Self::new("qwen2.5vl:3b", "ollama", 3200, 800, 32768)
            .with_capability(ModelCapability::TextGeneration)
            .with_capability(ModelCapability::VisionUnderstanding)
            .with_capability(ModelCapability::ToolCalling)
    }

    /// Ultra-lightweight vision model for quick scene verification.
    pub fn preset_moondream() -> Self {
        Self::new("moondream", "ollama", 1400, 400, 8192)
            .with_capability(ModelCapability::TextGeneration)
            .with_capability(ModelCapability::VisionUnderstanding)
    }

    /// Cloud fallback model with large context and vision support (0 MB local VRAM).
    pub fn preset_gemini_1_5_flash() -> Self {
        Self::new("gemini-1.5-flash", "gemini", 0, 50, 1048576)
            .with_capability(ModelCapability::TextGeneration)
            .with_capability(ModelCapability::VisionUnderstanding)
            .with_capability(ModelCapability::ToolCalling)
    }
}

/// Abstract contract for managing and routing model capabilities.
pub trait ModelRegistry: Send + Sync {
    /// Return the profile of the active chat model.
    fn get_active_chat_model(&self) -> ModelProfile;

    /// Return the profile of the active vision model, if configured.
    fn get_active_vision_model(&self) -> Option<ModelProfile>;

    /// List all registered model profiles.
    fn list_available_models(&self) -> Vec<ModelProfile>;

    /// Check if a specific model supports a requested capability.
    fn supports_capability(&self, model: &str, cap: ModelCapability) -> bool;

    /// Retrieve a model profile by name.
    fn get_model(&self, model_name: &str) -> Option<ModelProfile>;
}

/// Thread-safe default implementation of `ModelRegistry`.
pub struct DefaultModelRegistry {
    active_chat_model: RwLock<String>,
    active_vision_model: RwLock<Option<String>>,
    models: RwLock<HashMap<String, ModelProfile>>,
}

impl DefaultModelRegistry {
    /// Create a new registry pre-populated with standard local & cloud profiles.
    pub fn new() -> Self {
        let mut models = HashMap::new();

        let qwen_chat = ModelProfile::preset_qwen2_5_3b();
        let qwen_vl = ModelProfile::preset_qwen2_5_vl_3b();
        let moondream = ModelProfile::preset_moondream();
        let gemini = ModelProfile::preset_gemini_1_5_flash();

        models.insert(qwen_chat.name.clone(), qwen_chat);
        models.insert(qwen_vl.name.clone(), qwen_vl);
        models.insert(moondream.name.clone(), moondream);
        models.insert(gemini.name.clone(), gemini);

        Self {
            active_chat_model: RwLock::new("qwen2.5:3b".to_string()),
            active_vision_model: RwLock::new(Some("qwen2.5vl:3b".to_string())),
            models: RwLock::new(models),
        }
    }

    /// Register or update a model profile.
    pub fn register_model(&self, profile: ModelProfile) {
        let mut models = self.models.write().unwrap();
        models.insert(profile.name.clone(), profile);
    }

    /// Switch the active chat model.
    pub fn set_active_chat_model(&self, model_name: &str) -> Result<(), String> {
        let models = self.models.read().unwrap();
        if let Some(profile) = models.get(model_name) {
            if !profile.has_capability(ModelCapability::TextGeneration) {
                return Err(format!(
                    "Model '{}' does not support TextGeneration",
                    model_name
                ));
            }
            drop(models);
            *self.active_chat_model.write().unwrap() = model_name.to_string();
            Ok(())
        } else {
            Err(format!("Model '{}' not found in registry", model_name))
        }
    }

    /// Switch or unset the active vision model.
    pub fn set_active_vision_model(&self, model_name: Option<&str>) -> Result<(), String> {
        match model_name {
            Some(name) => {
                let models = self.models.read().unwrap();
                if let Some(profile) = models.get(name) {
                    if !profile.has_capability(ModelCapability::VisionUnderstanding) {
                        return Err(format!(
                            "Model '{}' does not support VisionUnderstanding",
                            name
                        ));
                    }
                    drop(models);
                    *self.active_vision_model.write().unwrap() = Some(name.to_string());
                    Ok(())
                } else {
                    Err(format!("Model '{}' not found in registry", name))
                }
            }
            None => {
                *self.active_vision_model.write().unwrap() = None;
                Ok(())
            }
        }
    }
}

impl Default for DefaultModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry for DefaultModelRegistry {
    fn get_active_chat_model(&self) -> ModelProfile {
        let active_name = self.active_chat_model.read().unwrap().clone();
        self.get_model(&active_name)
            .unwrap_or_else(ModelProfile::preset_qwen2_5_3b)
    }

    fn get_active_vision_model(&self) -> Option<ModelProfile> {
        let active_name = self.active_vision_model.read().unwrap().clone();
        active_name.and_then(|name| self.get_model(&name))
    }

    fn list_available_models(&self) -> Vec<ModelProfile> {
        let models = self.models.read().unwrap();
        models.values().cloned().collect()
    }

    fn supports_capability(&self, model: &str, cap: ModelCapability) -> bool {
        self.get_model(model)
            .map(|p| p.has_capability(cap))
            .unwrap_or(false)
    }

    fn get_model(&self, model_name: &str) -> Option<ModelProfile> {
        let models = self.models.read().unwrap();
        models.get(model_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_profile_creation_and_capabilities() {
        let profile = ModelProfile::new("custom-model", "ollama", 1500, 300, 4096)
            .with_capability(ModelCapability::TextGeneration)
            .with_capability(ModelCapability::ToolCalling);

        assert_eq!(profile.name, "custom-model");
        assert_eq!(profile.provider, "ollama");
        assert_eq!(profile.vram_mb_required, 1500);
        assert_eq!(profile.context_window, 4096);
        assert!(profile.has_capability(ModelCapability::TextGeneration));
        assert!(profile.has_capability(ModelCapability::ToolCalling));
        assert!(!profile.has_capability(ModelCapability::VisionUnderstanding));
    }

    #[test]
    fn test_default_registry_preset_models() {
        let registry = DefaultModelRegistry::new();

        let chat = registry.get_active_chat_model();
        assert_eq!(chat.name, "qwen2.5:3b");
        assert!(chat.has_capability(ModelCapability::TextGeneration));

        let vision = registry
            .get_active_vision_model()
            .expect("Vision model should be active");
        assert_eq!(vision.name, "qwen2.5vl:3b");
        assert!(vision.has_capability(ModelCapability::VisionUnderstanding));

        assert!(registry.supports_capability("qwen2.5:3b", ModelCapability::TextGeneration));
        assert!(registry.supports_capability("qwen2.5vl:3b", ModelCapability::VisionUnderstanding));
        assert!(
            registry.supports_capability("gemini-1.5-flash", ModelCapability::VisionUnderstanding)
        );
        assert!(!registry.supports_capability("qwen2.5:3b", ModelCapability::VisionUnderstanding));
    }

    #[test]
    fn test_registry_switching_models() {
        let registry = DefaultModelRegistry::new();

        // Switch chat model to gemini
        registry
            .set_active_chat_model("gemini-1.5-flash")
            .expect("Switch to gemini failed");
        assert_eq!(registry.get_active_chat_model().name, "gemini-1.5-flash");

        // Try switching to unknown model
        assert!(registry
            .set_active_chat_model("non-existent-model")
            .is_err());

        // Switch vision model to moondream
        registry
            .set_active_vision_model(Some("moondream"))
            .expect("Switch to moondream failed");
        assert_eq!(
            registry.get_active_vision_model().unwrap().name,
            "moondream"
        );

        // Unset vision model
        registry
            .set_active_vision_model(None)
            .expect("Unset vision model failed");
        assert!(registry.get_active_vision_model().is_none());
    }

    #[test]
    fn test_register_custom_model() {
        let registry = DefaultModelRegistry::new();
        let custom = ModelProfile::new("my-custom-llm", "local", 1200, 200, 2048)
            .with_capability(ModelCapability::TextGeneration);

        registry.register_model(custom);
        assert!(registry.supports_capability("my-custom-llm", ModelCapability::TextGeneration));
        assert_eq!(
            registry
                .get_model("my-custom-llm")
                .unwrap()
                .vram_mb_required,
            1200
        );
    }
}
