use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;
use vc_core::personality::Personality;

use crate::state::AppState;

#[derive(Serialize)]
pub struct CharacterOverview {
    pub id: String,
    pub name: String,
    pub personality: Personality,
    pub state: vc_core::state::CharacterState,
    pub relationship: vc_core::relationship::Relationship,
    pub memory_count: usize,
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "VirtualCharacter Gateway",
        "version": "0.1.0"
    }))
}

pub async fn get_character(State(state): State<AppState>) -> impl IntoResponse {
    let char = state.character.read().await;
    let personality = state.personality.read().await;
    let char_state = state.character_state.read().await;
    let relationship = state.relationship.read().await;
    let memories = state.memories.read().await;

    let overview = CharacterOverview {
        id: char.id.0.to_string(),
        name: char.name.clone(),
        personality: personality.clone(),
        state: char_state.clone(),
        relationship: relationship.clone(),
        memory_count: memories.len(),
    };

    Json(overview)
}

pub async fn update_personality(
    State(state): State<AppState>,
    Json(new_personality): Json<Personality>,
) -> impl IntoResponse {
    let mut personality = state.personality.write().await;
    *personality = new_personality.clone();
    Json(json!({
        "status": "updated",
        "personality": *personality
    }))
}

pub async fn get_memories(State(state): State<AppState>) -> impl IntoResponse {
    let memories = state.memories.read().await;
    Json(memories.clone())
}

pub async fn reset_character(State(state): State<AppState>) -> impl IntoResponse {
    state.reset().await;
    Json(json!({
        "status": "reset_successful",
        "message": "Character state, personality and memories have been restored to default baseline."
    }))
}
