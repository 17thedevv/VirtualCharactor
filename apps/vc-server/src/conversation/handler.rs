//! Interaction orchestration handler (Context, Decision, Memory, Streaming & State update).

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use uuid::Uuid;

use vc_core::context::{ContextBudget, ContextBuilder};
use vc_core::decision::action::{Action, ActionType};
use vc_core::decision::context::DecisionContext;
use vc_core::decision::DecisionEngine;
use vc_core::memory::{Memory, MemoryImportance};
use vc_core::rag::traits::KnowledgeRepository;
use vc_core::rag::types::RagQuery;
use vc_core::relationship::RelationshipTransition;
use vc_core::state::EmotionEngine;
use vc_storage::{MemoryRepository, RelationshipRepository, StateRepository};

use crate::conversation::prompt::{
    build_companion_llm_request, build_emotion_json, summarize_snippet,
};
use crate::state::AppState;
use crate::streaming::{stream_brain_to_voice, StreamingOutcome};

fn chrono_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub async fn process_user_interaction(
    sender: &mut SplitSink<WebSocket, Message>,
    state: &AppState,
    user_input: String,
    actor_id: String,
    cancellation_flag: Arc<AtomicBool>,
) {
    let interaction_id = Uuid::new_v4().to_string();
    let start_time = chrono_now_secs();
    let char_id = state.character.read().await.id;

    // 1. Session Management & Interaction Started
    let session_id = {
        let mut mgr = state
            .runtime
            .session_manager
            .write()
            .expect("session lock poisoned");
        let session = mgr.get_or_create(char_id, &actor_id, start_time);
        session.touch(start_time);
        session.id.0.to_string()
    };

    // Transition Conversation FSM: Thinking
    {
        let mut fsm = state.conversation_fsm.write().await;
        let _ = fsm.on_transcript_final(start_time, &user_input);
    }

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "interaction_started",
                "interaction_id": interaction_id,
                "session_id": session_id,
                "actor_id": actor_id,
                "timestamp": start_time,
            })
            .to_string()
            .into(),
        ))
        .await;

    // Persist user dialogue record
    let user_dialogue = vc_storage::DialogueRecord {
        id: format!("usr-{}", start_time),
        character_id: char_id,
        actor_id: actor_id.clone(),
        sender: "user".to_string(),
        text: user_input.clone(),
        created_at: start_time,
    };
    let _ = state.storage.save_dialogue_record(&user_dialogue);

    // 2. State Inspection
    let char_state = state.character_state.read().await.clone();
    let rel = state.relationship.read().await.clone();
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "state_loaded",
                "interaction_id": interaction_id,
                "emotion": build_emotion_json(&char_state.emotion),
                "cognition": {
                    "attention": char_state.cognition.attention.value(),
                    "confusion": char_state.cognition.confusion.value(),
                    "curiosity": char_state.cognition.curiosity.value(),
                    "confidence": char_state.cognition.confidence.value(),
                    "focus": char_state.cognition.focus.value(),
                    "current_topic": char_state.cognition.current_topic,
                },
                "behavior": {
                    "playfulness": char_state.behavior.playfulness.value(),
                    "seriousness": char_state.behavior.seriousness.value(),
                    "verbosity": char_state.behavior.verbosity.value(),
                    "initiative": char_state.behavior.initiative.value(),
                    "current_activity": char_state.behavior.current_activity,
                },
                "relationship": {
                    "stage": rel.state.stage.as_str(),
                    "closeness": rel.state.closeness,
                    "trust": rel.state.trust,
                    "familiarity": rel.state.familiarity,
                    "affection": rel.state.affection,
                    "tension": rel.state.tension,
                    "known_facts": rel.state.known_facts,
                },
                "session": {
                    "session_id": session_id,
                    "turn_count": char_state.session.turn_count + 1,
                    "status": "Active"
                }
            })
            .to_string()
            .into(),
        ))
        .await;

    // 3. Teaching Detection
    let teaching_fact = vc_runtime::KnowledgeLearner::detect_teaching(&user_input);
    if let Some(ref taught) = teaching_fact {
        let importance = if taught.is_user_preference {
            MemoryImportance::High
        } else {
            MemoryImportance::Critical
        };
        let mem_content = format!("Người dùng đã dạy/chia sẻ: \"{}\"", taught.fact);
        let mut taught_mem =
            Memory::new_semantic(mem_content, importance, Some(actor_id.clone()), start_time);
        taught_mem.character_id = Some(char_id);

        {
            let mut memories_mut = state.memories.write().await;
            memories_mut.push(taught_mem.clone());
        }
        let _ = state.storage.save_memory(&taught_mem);

        {
            let mut rel_mut = state.relationship.write().await;
            rel_mut.add_known_fact(&taught.fact);
            let _ = state.storage.save_relationship(&rel_mut);
        }

        println!(
            "🎓 [KNOWLEDGE LEARNED] Saved semantic fact: '{}'",
            taught.fact
        );
        let _ = sender
            .send(Message::Text(
                json!({
                    "event": "memory_formed",
                    "interaction_id": interaction_id,
                    "memory": {
                        "id": taught_mem.id.0.to_string(),
                        "content": taught_mem.content,
                        "type": "Semantic",
                        "importance": format!("{:?}", importance)
                    }
                })
                .to_string()
                .into(),
            ))
            .await;
    }

    // 4. Web Search Tool
    let search_query = vc_runtime::WebSearchTool::should_search(&user_input);
    let web_search_result = if let Some(ref q) = search_query {
        println!("🌐 [WEB SEARCH] Triggered for query: '{}'", q);
        let _ = sender
            .send(Message::Text(
                json!({
                    "event": "tool_executing",
                    "interaction_id": interaction_id,
                    "tool": "web_search",
                    "query": q,
                    "message": format!("Đang tra cứu web về: {}...", q)
                })
                .to_string()
                .into(),
            ))
            .await;

        let res = vc_runtime::WebSearchTool::search(q);
        println!(
            "🌐 [WEB SEARCH] Found {} snippet(s) from {}",
            res.snippets.len(),
            res.source
        );

        let search_rec = vc_storage::SearchRecord {
            id: Uuid::new_v4().to_string(),
            character_id: char_id,
            query: q.clone(),
            source: res.source.clone(),
            snippets: res.snippets.clone(),
            summary: Some(res.summary()),
            created_at: start_time,
        };
        let _ = state.storage.save_search_record(&search_rec);

        let _ = sender
            .send(Message::Text(
                json!({
                    "event": "tool_executed",
                    "interaction_id": interaction_id,
                    "tool": "web_search",
                    "query": q,
                    "source": res.source,
                    "snippets": res.snippets,
                    "summary": res.summary()
                })
                .to_string()
                .into(),
            ))
            .await;

        Some(res)
    } else {
        None
    };

    // 5. Retrieve Memory with strict actor isolation
    let retrieved_memories = {
        let mut memories = state.memories.write().await;
        let query = vc_core::memory::MemoryQuery::new(3)
            .with_actor(&actor_id)
            .with_text(&user_input);
        vc_runtime::in_memory_store::InMemoryMemoryStore::retrieve_from_slice(
            &mut memories,
            &query,
            start_time,
        )
    };

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "memories_retrieved",
                "interaction_id": interaction_id,
                "count": retrieved_memories.len(),
                "memories": retrieved_memories.iter().map(|m| json!({
                    "id": m.id.0.to_string(),
                    "content": m.content,
                    "type": format!("{:?}", m.metadata.memory_type),
                    "importance": format!("{:?}", m.metadata.importance)
                })).collect::<Vec<_>>()
            })
            .to_string()
            .into(),
        ))
        .await;

    // 5b. RAG Hybrid Retrieval (Dense Vector + FTS5 Sparse Keywords)
    let mut rag_query = RagQuery::new(&user_input, 4)
        .with_character(char_id)
        .with_actor(&actor_id);
    if let Ok(vec) = state.embedder.embed_text(&user_input) {
        rag_query = rag_query.with_vector(vec);
    }
    let rag_chunks = state.storage.search_hybrid(&rag_query).unwrap_or_default();

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "rag_retrieved",
                "interaction_id": interaction_id,
                "count": rag_chunks.len(),
                "chunks": rag_chunks.iter().map(|r| json!({
                    "id": r.chunk.id.0.to_string(),
                    "source_type": r.chunk.source_type.as_str(),
                    "content": r.chunk.content,
                    "score": r.final_score,
                    "dense_score": r.dense_score,
                    "sparse_score": r.sparse_score,
                    "rrf_score": r.rrf_score,
                })).collect::<Vec<_>>()
            })
            .to_string()
            .into(),
        ))
        .await;

    // 6. Context Assembly
    let personality = state.personality.read().await.clone();
    let world = state.world_state.read().await.clone();
    let assembled_context = ContextBuilder::new()
        .with_system_directive("You are Aria, an independent living virtual companion.")
        .with_personality(personality.clone())
        .with_state(char_state.clone())
        .with_relationship(rel.clone())
        .with_memories(retrieved_memories.clone())
        .with_rag_chunks(&rag_chunks)
        .with_world(&world)
        .with_user_input(&user_input)
        .build(ContextBudget::standard_4k());

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "context_assembled",
                "interaction_id": interaction_id,
                "token_budget": assembled_context.budget.total_tokens,
                "tokens_used": assembled_context.total_tokens,
                "breakdown": {
                    "personality_tokens": assembled_context.breakdown.personality_tokens,
                    "memory_tokens": assembled_context.breakdown.memory_tokens,
                    "knowledge_tokens": assembled_context.breakdown.knowledge_tokens,
                    "state_tokens": assembled_context.breakdown.state_tokens,
                    "relationship_tokens": assembled_context.breakdown.relationship_tokens,
                    "user_input_tokens": assembled_context.breakdown.user_tokens,
                    "system_directive_tokens": assembled_context.breakdown.system_tokens,
                    "dropped_items_count": assembled_context.breakdown.dropped_items_count
                }
            })
            .to_string()
            .into(),
        ))
        .await;

    // 7. Decision Engine
    let decision_ctx = DecisionContext::new(
        &user_input,
        Some(actor_id.clone()),
        personality.clone(),
        char_state.clone(),
        Some(rel.clone()),
        retrieved_memories.clone(),
    )
    .with_world(world);

    let decision = state
        .decision_engine
        .make_decision(&decision_ctx)
        .unwrap_or_else(|_| {
            vc_core::decision::Decision::new(vc_core::decision::DecisionResult {
                selected_action: Action::simple(ActionType::WarmGreeting),
                confidence: 0.9,
                reasoning: "Fallback welcoming decision".into(),
                candidates: vec![],
                policy: None,
            })
        });

    let chosen_action_name = decision.result.selected_action.action_type.to_string();
    let chosen_action_desc = decision.result.selected_action.description.clone();
    let reasoning = decision.result.reasoning.clone();
    let candidates_json: Vec<serde_json::Value> = decision
        .result
        .candidates
        .iter()
        .map(|c| {
            json!({
                "action": c.action.action_type.to_string(),
                "description": c.action.description,
                "confidence": c.confidence,
                "score": c.score,
                "rationale": c.rationale
            })
        })
        .collect();

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "decision_made",
                "interaction_id": interaction_id,
                "selected_action": chosen_action_name,
                "action_description": chosen_action_desc,
                "confidence": decision.result.confidence,
                "reasoning": reasoning,
                "policy": decision.result.policy,
                "candidates": candidates_json
            })
            .to_string()
            .into(),
        ))
        .await;

    // 8. Build Prompt Request
    let recent_dialogues = state
        .storage
        .list_dialogue_records(char_id, 10)
        .unwrap_or_default();
    let llm_request = build_companion_llm_request(
        &personality,
        &char_state,
        &rel,
        &retrieved_memories,
        &recent_dialogues,
        &user_input,
        &decision.result.selected_action,
        &reasoning,
        decision.result.policy.as_ref(),
        web_search_result.as_ref(),
        teaching_fact.as_ref(),
        &rag_chunks,
    );

    // 9. Execute Real-Time Streaming Brain to Voice Pipeline
    let outcome = stream_brain_to_voice(
        state.runtime.llm_provider.clone(),
        llm_request,
        sender,
        state.conversation_fsm.clone(),
        state.tts_queue.clone(),
        &interaction_id,
        &personality.identity.name,
        cancellation_flag,
    )
    .await;

    // 10. Process Outcome and Update State & Memory
    match outcome {
        StreamingOutcome::Completed { full_text, .. } => {
            // Save complete dialogue record
            let char_dialogue = vc_storage::DialogueRecord {
                id: format!("char-{}", chrono_now_secs()),
                character_id: char_id,
                actor_id: actor_id.clone(),
                sender: "character".to_string(),
                text: full_text.clone(),
                created_at: chrono_now_secs(),
            };
            let _ = state.storage.save_dialogue_record(&char_dialogue);

            // Archive recent dialogues into episodic RAG memory chunks
            let recent_records = state
                .storage
                .list_dialogue_records(char_id, 10)
                .unwrap_or_default();
            let _ = state.conversation_archiver.archive_dialogues(
                char_id,
                &actor_id,
                &session_id,
                &recent_records,
            );

            // Emotion evaluation & decay
            let emotion_delta =
                state
                    .emotion_engine
                    .evaluate(&char_state.emotion, &user_input, &personality);
            let previous_dominant = char_state.emotion.dominant_emotion().0.name().to_string();

            {
                let mut char_state_mut = state.character_state.write().await;
                char_state_mut.emotion.apply_delta(&emotion_delta);
                char_state_mut.emotion.decay(5);
                char_state_mut.sync_behavior(&personality);
                char_state_mut.session.increment_turn();
                if char_state_mut.session.turn_count % 10 == 0 {
                    if let Ok(report) =
                        state.run_memory_consolidation(char_id, chrono_now_secs(), 3600)
                    {
                        if report.memories_decayed > 0 || report.memories_pruned > 0 {
                            println!(
                                "🧠 [CONSOLIDATION] Memory cycle: {} scanned, {} decayed, {} pruned",
                                report.memories_scanned, report.memories_decayed, report.memories_pruned
                            );
                        }
                    }
                }
                char_state_mut
                    .cognition
                    .update_topic(summarize_snippet(&user_input));
                char_state_mut.cognition.set_attention(0.85);
                char_state_mut.cognition.set_focus(0.80);

                let mut world_mut = state.world_state.write().await;
                world_mut.record_speech(chrono_now_secs());

                let mut rel_mut = state.relationship.write().await;
                let rel_transition = RelationshipTransition {
                    familiarity_delta: 0.03,
                    closeness_delta: 0.02,
                    trust_delta: 0.015,
                    affection_delta: 0.02,
                    tension_delta: -0.01,
                };
                rel_mut.record_interaction(&rel_transition, start_time);
            }

            let new_char_state = state.character_state.read().await.clone();
            let new_rel = state.relationship.read().await.clone();
            let _ = state.storage.save_state(char_id, &new_char_state);
            let _ = state.storage.save_relationship(&new_rel);
            let new_dominant = new_char_state
                .emotion
                .dominant_emotion()
                .0
                .name()
                .to_string();

            let _ = sender
                .send(Message::Text(
                    json!({
                        "event": "state_updated",
                        "interaction_id": interaction_id,
                        "emotion": build_emotion_json(&new_char_state.emotion),
                        "emotion_delta": {
                            "joy": emotion_delta.joy,
                            "sadness": emotion_delta.sadness,
                            "anger": emotion_delta.anger,
                            "fear": emotion_delta.fear,
                            "surprise": emotion_delta.surprise,
                            "affection": emotion_delta.affection,
                            "embarrassment": emotion_delta.embarrassment,
                            "curiosity": emotion_delta.curiosity,
                        },
                        "transition": {
                            "previous_dominant": previous_dominant,
                            "new_dominant": new_dominant,
                        },
                        "behavior": {
                            "playfulness": new_char_state.behavior.playfulness.value(),
                            "seriousness": new_char_state.behavior.seriousness.value(),
                            "verbosity": new_char_state.behavior.verbosity.value(),
                            "initiative": new_char_state.behavior.initiative.value(),
                        },
                        "relationship": {
                            "stage": new_rel.state.stage.as_str(),
                            "closeness": new_rel.state.closeness,
                            "trust": new_rel.state.trust,
                            "familiarity": new_rel.state.familiarity,
                            "affection": new_rel.state.affection,
                            "tension": new_rel.state.tension,
                            "known_facts": new_rel.state.known_facts,
                        },
                        "session": {
                            "session_id": session_id,
                            "turn_count": new_char_state.session.turn_count,
                            "status": "Active"
                        }
                    })
                    .to_string()
                    .into(),
                ))
                .await;

            // Form episodic memory
            if teaching_fact.is_none() {
                let new_memory_content =
                    format!("User discussed: \"{}\"", summarize_snippet(&user_input));
                let new_mem = Memory::new_episodic(
                    new_memory_content,
                    MemoryImportance::Medium,
                    Some(actor_id),
                    start_time,
                );
                let _ = state.storage.save_memory(&new_mem);
                {
                    let mut mems = state.memories.write().await;
                    mems.push(new_mem.clone());
                }

                let _ = sender
                    .send(Message::Text(
                        json!({
                            "event": "memory_formed",
                            "interaction_id": interaction_id,
                            "memory": {
                                "id": new_mem.id.0.to_string(),
                                "content": new_mem.content,
                                "importance": "Medium",
                                "type": "Episodic"
                            }
                        })
                        .to_string()
                        .into(),
                    ))
                    .await;
            }
        }
        StreamingOutcome::Interrupted { heard_text, .. } => {
            println!(
                "🛑 [INTERRUPTED] Recording trimmed dialogue history: '{}'",
                heard_text
            );
            let preserved = if heard_text.trim().is_empty() {
                "[Interrupted by user]".to_string()
            } else {
                format!("{} [Interrupted by user]", heard_text.trim())
            };

            let char_dialogue = vc_storage::DialogueRecord {
                id: format!("char-{}", chrono_now_secs()),
                character_id: char_id,
                actor_id: actor_id.clone(),
                sender: "character".to_string(),
                text: preserved,
                created_at: chrono_now_secs(),
            };
            let _ = state.storage.save_dialogue_record(&char_dialogue);
        }
        StreamingOutcome::Failed { error } => {
            eprintln!("❌ [STREAMING ERROR] {}", error);
            let _ = sender
                .send(Message::Text(
                    json!({
                        "event": "error",
                        "interaction_id": interaction_id,
                        "message": error
                    })
                    .to_string()
                    .into(),
                ))
                .await;
        }
    }
}
