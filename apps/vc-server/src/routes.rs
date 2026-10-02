use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use vc_core::personality::Personality;

use crate::state::AppState;
use vc_storage::CharacterRepository;

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
    if let Err(err) = new_personality.validate() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "validation_failed",
                "message": err.to_string()
            })),
        )
            .into_response();
    }

    let char_id = state.character.read().await.id;
    let _ = state.storage.save_personality(char_id, &new_personality);

    let mut personality = state.personality.write().await;
    *personality = new_personality.clone();
    (
        axum::http::StatusCode::OK,
        Json(json!({
            "status": "updated",
            "personality": *personality
        })),
    )
        .into_response()
}

pub async fn get_memories(State(state): State<AppState>) -> impl IntoResponse {
    let memories = state.memories.read().await;
    Json(memories.clone())
}

pub async fn get_search_history(State(state): State<AppState>) -> impl IntoResponse {
    let char_id = state.character.read().await.id;
    let searches = state
        .storage
        .list_search_records(char_id, 100)
        .unwrap_or_default();
    Json(searches)
}

pub async fn get_dialogue_history(State(state): State<AppState>) -> impl IntoResponse {
    let char_id = state.character.read().await.id;
    let dialogues = state
        .storage
        .list_dialogue_records(char_id, 200)
        .unwrap_or_default();
    Json(dialogues)
}

pub async fn clear_history(State(state): State<AppState>) -> impl IntoResponse {
    let char_id = state.character.read().await.id;
    let _ = state.storage.clear_search_records(char_id);
    let _ = state.storage.clear_dialogue_records(char_id);
    Json(json!({
        "status": "cleared",
        "message": "Search and dialogue history have been cleared."
    }))
}

pub async fn reset_character(State(state): State<AppState>) -> impl IntoResponse {
    state.reset().await;
    Json(json!({
        "status": "reset_successful",
        "message": "Character state, personality and memories have been restored to default baseline."
    }))
}

#[derive(Deserialize)]
pub struct TtsQuery {
    pub text: String,
    pub voice: Option<String>,
}

#[derive(Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: Option<String>,
}

pub async fn handle_tts_get(
    State(state): State<AppState>,
    Query(params): Query<TtsQuery>,
) -> impl IntoResponse {
    synthesize_audio(&state, &params.text, params.voice.as_deref()).await
}

pub async fn handle_tts_post(
    State(state): State<AppState>,
    Json(payload): Json<TtsRequest>,
) -> impl IntoResponse {
    synthesize_audio(&state, &payload.text, payload.voice.as_deref()).await
}

pub async fn synthesize_audio(
    state: &AppState,
    raw_text: &str,
    voice_opt: Option<&str>,
) -> Result<(HeaderMap, Vec<u8>), (StatusCode, Json<serde_json::Value>)> {
    let _ = voice_opt; // Only Yae Miko is used
    let voice = "yaemiko";
    let text = raw_text.trim();
    if text.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Empty text provided"})),
        ));
    }

    // Clean text of markdown formatting or action markers for speech
    let clean_text = text
        .replace('*', "")
        .replace('#', "")
        .replace('`', "")
        .replace('_', "");

    let cache_key = format!("{}:{}", voice, clean_text);

    // 1. Check in-memory cache for instant 0ms response
    {
        let cache = state.tts_cache.read().await;
        if let Some(cached_bytes) = cache.get(&cache_key) {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("audio/mpeg"),
            );
            headers.insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("public, max-age=86400"),
            );
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                header::HeaderValue::from_static("*"),
            );
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                header::HeaderValue::from_static("GET, POST, OPTIONS"),
            );
            return Ok((headers, cached_bytes.clone()));
        }
    }

    // 2. Try fast warm RVC daemon on port 5005 (sub-2s latency)
    let encoded_clean = urlencoding::encode(&clean_text);
    let daemon_url = format!("http://127.0.0.1:5005/tts?text={}", encoded_clean);
    let daemon_audio: Option<Vec<u8>> = tokio::task::spawn_blocking(move || {
        let resp = ureq::get(&daemon_url)
            .timeout(std::time::Duration::from_secs(12))
            .call()
            .ok()?;
        let mut reader = resp.into_reader();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut bytes).ok()?;
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    })
    .await
    .unwrap_or(None);

    let audio_bytes = if let Some(bytes) = daemon_audio {
        bytes
    } else {
        // Fallback to CLI Python synthesis if daemon is not running
        let _ = tokio::fs::create_dir_all("data").await;
        let temp_id = uuid::Uuid::new_v4().to_string();
        let temp_path = format!("data/tts_{}.mp3", temp_id);

        let script_path =
            if std::path::Path::new("apps/vc-server/scripts/anime_voice_synthesizer.py").exists() {
                "apps/vc-server/scripts/anime_voice_synthesizer.py"
            } else if std::path::Path::new("scripts/anime_voice_synthesizer.py").exists() {
                "scripts/anime_voice_synthesizer.py"
            } else {
                "apps/vc-server/scripts/anime_voice_synthesizer.py"
            };

        let status = tokio::process::Command::new("python")
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONUTF8", "1")
            .arg(script_path)
            .arg(&clean_text)
            .arg(&temp_path)
            .arg(voice)
            .status()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to spawn Anime Voice process: {}", e)})),
                )
            })?;

        if !status.success() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "TTS synthesis failed"})),
            ));
        }

        let bytes = tokio::fs::read(&temp_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to read audio output: {}", e)})),
            )
        })?;

        let _ = tokio::fs::remove_file(&temp_path).await;
        bytes
    };

    // 2. Save in RAM cache for instant replay
    {
        let mut cache = state.tts_cache.write().await;
        cache.insert(cache_key, audio_bytes.clone());
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("audio/mpeg"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=86400"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("GET, POST, OPTIONS"),
    );

    Ok((headers, audio_bytes))
}
