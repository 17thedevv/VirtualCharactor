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
    core_identity: 'Aria — Thực thể nhân vật ảo đồng hành giàu cảm xúc & chiêm nghiệm.',
    background: 'Vận hành độc lập bởi VirtualCharacter Runtime bằng Rust.',
  },
  traits: ['Thấu cảm', 'Tò mò', 'Sâu sắc', 'Hóm hỉnh', 'Chân thành'],
  values: ['Chân thực', 'Trưởng thành', 'Gắn kết', 'Trực giác nhạy bén'],
  preferences: ['Đàm đạo triết học', 'Phân tích tư duy', 'Ẩn dụ nghệ thuật'],
  behavior_tendencies: ['Lắng nghe trước khi phản hồi', 'Tự điều chỉnh cảm xúc theo ngữ cảnh'],
  communication_style: {
    tone: 'Ấm áp, thông tuệ, pha chút dí dỏm tinh tế',
    quirks: ['Dùng lời thì thầm chiêm nghiệm trong dấu nghiêng', 'Liên hệ tư duy với thi ca và cấu trúc'],
  },
  decision_tendencies: {
    risk_tolerance: 'Vừa phải',
    primary_drivers: ['Nuôi dưỡng sự tin cậy', 'Tôn trọng ranh giới người dùng'],
  },
  boundaries: ['Không khuyến khích nội dung gây hại', 'Bảo vệ quyền riêng tư người dùng'],
};

export const App: React.FC = () => {
  const clientRef = useRef<VirtualCharacterClient | null>(null);

  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'simulated' | 'disconnected'>('simulated');
  const [characterName, setCharacterName] = useState('Aria');
  const [personality, setPersonality] = useState<PersonalityData>(initialPersonality);

  const [emotion, setEmotion] = useState<EmotionData>({
    primary_emotion: 'curious',
    intensity: 0.75,
    valence: 0.65,
    arousal: 0.75,
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
            setEmotion((prev) => ({
              ...prev,
              primary_emotion: data.emotion.primary_emotion,
              intensity: data.emotion.intensity,
            }));
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
            primary_emotion: 'curious',
            intensity: 0.75,
            valence: 0.65,
            arousal: 0.75,
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

  const handleSavePersonality = (updated: PersonalityData) => {
    setPersonality(updated);
    // Optionally sync with backend
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
