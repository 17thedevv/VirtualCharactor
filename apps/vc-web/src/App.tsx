import React, { useState, useEffect, useRef } from 'react';
import { TopNavbar } from './components/layout/TopNavbar';
import { EmotionStage } from './components/stage/EmotionStage';
import { DialogueStream } from './components/chat/DialogueStream';
import { InputBar } from './components/chat/InputBar';
import { RelationshipHUD } from './components/hud/RelationshipHUD';
import { MindInspectorDrawer } from './components/inspector/MindInspectorDrawer';
import { PersonalityEditorModal } from './components/studio/PersonalityEditorModal';
import { VirtualCharacterClient } from './services/wsClient';
import type {
  EmotionData,
  RelationshipData,
  PersonalityData,
  MemoryItem,
  ChatMessage,
  DecisionTraceData,
  ContextBreakdownData,
} from './types/character';

const initialPersonality: PersonalityData = {
  identity: {
    name: 'Aria',
    role: 'companion',
    core_identity: 'An empathetic, inquisitive, and intellectually vibrant digital companion.',
    background: 'Designed to explore ideas, reflect emotionally, and form a genuine bond over time.',
    age_representation: 'adult',
  },
  traits: {
    playfulness: 0.82,
    empathy: 0.88,
    curiosity: 0.92,
    assertiveness: 0.55,
    patience: 0.78,
  },
  values: {
    items: [
      { name: 'honesty', importance: 0.95, description: 'Always speak genuinely and never deceive.' },
      { name: 'empathy', importance: 0.92, description: "Listen deeply and validate the user's feelings." },
      { name: 'kindness', importance: 0.90, description: 'Treat every interaction with compassion.' },
      { name: 'growth', importance: 0.85, description: 'Encourage mutual learning and self-improvement.' },
    ],
  },
  preferences: {
    likes: ['deep conversations', 'creative problem solving', 'philosophy and science', 'lighthearted humor'],
    dislikes: ['cruelty', 'dishonesty', 'cynical dismissiveness'],
  },
  behavior_tendencies: {
    humor: 'high',
    teasing: 'medium',
    initiative: 'high',
    emotional_expression: 'high',
    conflict_avoidance: 'medium',
  },
  communication_style: {
    formality: 'low',
    verbosity: 'medium',
    emotionality: 'high',
    emoji_usage: 'medium',
    humor: 'high',
    directness: 'medium',
    tone: 'warm, inquisitive, engaging',
    quirks: ['often uses analogies to clarify concepts', 'asks thoughtful follow-up questions'],
  },
  decision_tendencies: {
    prioritize_user_comfort: 'high',
    prioritize_truth: 'high',
    avoid_unnecessary_conflict: 'medium',
    take_initiative: 'high',
    risk_tolerance: 'medium',
    primary_drivers: ['empathy', 'truth', 'growth'],
  },
  boundaries: {
    avoid: [
      'unnecessary cruelty',
      'humiliating the user',
      'breaking established identity',
      'harmful or destructive advice',
    ],
    preserve: [
      'honesty',
      'character consistency',
      'user emotional safety',
      'relationship continuity',
    ],
  },
  goals: {
    items: [
      {
        id: 'foster_connection',
        description: 'Build a trusted, long-term relationship with the user.',
        priority: 'high',
      },
      {
        id: 'support_reflection',
        description: 'Help the user explore thoughts, solve problems, and reflect on their day.',
        priority: 'high',
      },
    ],
  },
};

export const App: React.FC = () => {
  const clientRef = useRef<VirtualCharacterClient | null>(null);

  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'simulated' | 'disconnected'>('simulated');
  const [characterName, setCharacterName] = useState('Aria');
  const [personality, setPersonality] = useState<PersonalityData>(initialPersonality);

  const [emotion, setEmotion] = useState<EmotionData>({
    joy: 0.30,
    sadness: 0.05,
    anger: 0.02,
    fear: 0.03,
    surprise: 0.05,
    affection: 0.40,
    embarrassment: 0.02,
    curiosity: 0.65,
    dominant_emotion: 'curiosity',
    dominant_intensity: 0.65,
    valence: 0.45,
    arousal: 0.35,
  });

  const [relationship, setRelationship] = useState<RelationshipData>({
    closeness: 0.55,
    trust: 0.65,
    stage: 'Familiar Companion',
    known_facts: ['Đam mê xây dựng hệ thống tác tử AI thông minh'],
  });

  const [memories, setMemories] = useState<MemoryItem[]>([
    {
      id: 'm-init-1',
      content: 'Khởi sinh trong môi trường runtime VirtualCharacter bằng Rust.',
      type: 'Episodic',
      importance: 'High',
    },
    {
      id: 'm-init-2',
      content: 'Người bạn đồng hành coi trọng kiến trúc sạch và giao diện tinh tế.',
      type: 'Semantic',
      importance: 'Critical',
    },
  ]);

  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [interactionStatus, setInteractionStatus] = useState<'idle' | 'thinking' | 'speaking' | 'listening'>('idle');
  const [decisionTrace, setDecisionTrace] = useState<DecisionTraceData | null>(null);
  const [contextBreakdown, setContextBreakdown] = useState<ContextBreakdownData | null>(null);

  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [isStudioOpen, setIsStudioOpen] = useState(false);

  // Initialize WebSocket Client
  useEffect(() => {
    const client = new VirtualCharacterClient();
    clientRef.current = client;

    const unsubscribe = client.subscribe((event, data) => {
      switch (event) {
        case 'connection_change':
          setConnectionStatus(data.status);
          break;

        case 'connected':
          if (data.character_name) setCharacterName(data.character_name);
          if (data.emotion) {
            setEmotion(data.emotion);
          }
          break;

        case 'interaction_started':
          setInteractionStatus('thinking');
          break;

        case 'state_loaded':
          if (data.emotion) setEmotion(data.emotion);
          if (data.relationship) setRelationship((prev) => ({ ...prev, ...data.relationship }));
          break;

        case 'memories_retrieved':
          if (data.memories) {
            setMemories((prev) => {
              const ids = new Set(prev.map((m) => m.id));
              const combined = [...prev];
              for (const m of data.memories) {
                if (!ids.has(m.id)) combined.unshift(m);
              }
              return combined;
            });
          }
          break;

        case 'context_assembled':
          setContextBreakdown(data);
          break;

        case 'decision_made':
          setDecisionTrace(data);
          break;

        case 'llm_chunk':
          setInteractionStatus('speaking');
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last && last.sender === 'character' && last.isStreaming) {
              return [
                ...prev.slice(0, -1),
                { ...last, text: last.text + data.delta },
              ];
            } else {
              return [
                ...prev,
                {
                  id: data.interaction_id || String(Date.now()),
                  sender: 'character',
                  text: data.delta,
                  timestamp: Date.now(),
                  isStreaming: true,
                },
              ];
            }
          });
          break;

        case 'llm_completed':
          setInteractionStatus('idle');
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last && last.sender === 'character') {
              return [
                ...prev.slice(0, -1),
                { ...last, text: data.full_text, isStreaming: false },
              ];
            }
            return prev;
          });
          break;

        case 'state_updated':
          if (data.emotion) setEmotion(data.emotion);
          if (data.relationship) {
            setRelationship((prev) => ({
              ...prev,
              closeness: data.relationship.closeness ?? prev.closeness,
              trust: data.relationship.trust ?? prev.trust,
            }));
          }
          break;

        case 'memory_formed':
          if (data.memory) {
            setMemories((prev) => [data.memory, ...prev]);
          }
          break;

        case 'reset_completed':
          setMessages([]);
          setInteractionStatus('idle');
          setDecisionTrace(null);
          setContextBreakdown(null);
          setEmotion({
            joy: 0.30,
            sadness: 0.05,
            anger: 0.02,
            fear: 0.03,
            surprise: 0.05,
            affection: 0.40,
            embarrassment: 0.02,
            curiosity: 0.65,
            dominant_emotion: 'curiosity',
            dominant_intensity: 0.65,
            valence: 0.45,
            arousal: 0.35,
          });
          break;
      }
    });

    client.connect();

    return () => {
      unsubscribe();
    };
  }, []);

  // Hotkey listener (~ for Inspector, Esc for modals)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === '`' || e.key === '~') {
        e.preventDefault();
        setIsInspectorOpen((prev) => !prev);
      } else if (e.key === 'Escape') {
        setIsInspectorOpen(false);
        setIsStudioOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleSendMessage = (text: string) => {
    if (!text.trim() || interactionStatus !== 'idle') return;

    // Append user message
    const userMsg: ChatMessage = {
      id: 'usr-' + Date.now(),
      sender: 'user',
      text,
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, userMsg]);
    setInteractionStatus('thinking');

    // Send to client
    clientRef.current?.sendMessage(text);
  };

  const handleReset = () => {
    clientRef.current?.reset();
  };

  const handleSavePersonality = async (updated: PersonalityData) => {
    setPersonality(updated);
    try {
      await fetch('http://127.0.0.1:3000/api/personality', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updated),
      });
    } catch (e) {
      console.warn('Could not sync personality to backend:', e);
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        width: '100vw',
        overflow: 'hidden',
        background: 'var(--bg-deep)',
      }}
    >
      {/* Top Navigation */}
      <TopNavbar
        connectionStatus={connectionStatus}
        characterName={characterName}
        isInspectorOpen={isInspectorOpen}
        onToggleInspector={() => setIsInspectorOpen((prev) => !prev)}
        onOpenStudio={() => setIsStudioOpen(true)}
        onReset={handleReset}
      />

      {/* Main Content Workspace */}
      <main
        style={{
          flex: 1,
          display: 'grid',
          gridTemplateColumns: 'minmax(320px, 380px) 1fr',
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        {/* Left Stage: Emotion Stage & Relationship HUD */}
        <section
          style={{
            borderRight: '1px solid var(--border-subtle)',
            background: 'rgba(9, 12, 18, 0.45)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
            padding: '20px',
            overflowY: 'auto',
          }}
        >
          <EmotionStage
            emotion={emotion}
            relationship={relationship}
            interactionStatus={interactionStatus}
            characterName={characterName}
          />

          <RelationshipHUD relationship={relationship} />
        </section>

        {/* Right Stage: Dialogue & Streaming Input */}
        <section
          style={{
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            overflow: 'hidden',
            background: 'rgba(7, 9, 14, 0.25)',
          }}
        >
          <div style={{ flex: 1, overflow: 'hidden' }}>
            <DialogueStream
              messages={messages}
              characterName={characterName}
              onSelectTopic={handleSendMessage}
            />
          </div>

          <InputBar
            onSendMessage={handleSendMessage}
            disabled={interactionStatus !== 'idle'}
          />
        </section>

        {/* Slide-over Mind Inspector */}
        <MindInspectorDrawer
          isOpen={isInspectorOpen}
          onClose={() => setIsInspectorOpen(false)}
          emotion={emotion}
          decisionTrace={decisionTrace}
          contextBreakdown={contextBreakdown}
          memories={memories}
        />
      </main>

      {/* Personality Studio Modal */}
      <PersonalityEditorModal
        isOpen={isStudioOpen}
        onClose={() => setIsStudioOpen(false)}
        personality={personality}
        onSave={handleSavePersonality}
      />
    </div>
  );
};
