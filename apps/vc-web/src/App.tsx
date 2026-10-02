import React, { useState, useEffect, useRef } from 'react';
import { TopNavbar } from './components/layout/TopNavbar';
import { EmotionStage } from './components/stage/EmotionStage';
import { VrmAvatarStage } from './components/avatar/VrmAvatarStage';
import { DialogueStream } from './components/chat/DialogueStream';
import { InputBar } from './components/chat/InputBar';
import { RelationshipHUD } from './components/hud/RelationshipHUD';
import { MindInspectorDrawer } from './components/inspector/MindInspectorDrawer';
import { PersonalityEditorModal } from './components/studio/PersonalityEditorModal';
import { HistoryModal } from './components/history/HistoryModal';
import { VirtualCharacterClient } from './services/wsClient';
import { audioPlayer } from './services/audioPlayer';
import type {
  EmotionData,
  RelationshipData,
  PersonalityData,
  MemoryItem,
  ChatMessage,
  DecisionTraceData,
  ContextBreakdownData,
  WebSearchRecord,
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
    stage: 'Casual Companion',
    closeness: 0.55,
    trust: 0.65,
    familiarity: 0.60,
    affection: 0.62,
    tension: 0.02,
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
  const [searchHistory, setSearchHistory] = useState<WebSearchRecord[]>([]);
  const [currentSearch, setCurrentSearch] = useState<{ query: string; message: string } | null>(null);
  const activeSearchRef = useRef<WebSearchRecord | null>(null);

  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [isStudioOpen, setIsStudioOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [isVoiceMuted, setIsVoiceMuted] = useState(audioPlayer.isMuted());
  const [isAudioPlaying, setIsAudioPlaying] = useState(false);
  const [isTestVoiceLoading, setIsTestVoiceLoading] = useState(false);
  const [stageMode, setStageMode] = useState<'vrm' | 'orb'>('vrm');
  const [audioAmplitude, setAudioAmplitude] = useState(0);

  // Connect audioPlayer amplitude and playback callbacks
  useEffect(() => {
    audioPlayer.setCallbacks(
      (amp) => setAudioAmplitude(amp),
      (playing) => setIsAudioPlaying(playing)
    );

    // Unlock AudioContext on first user interaction anywhere in the page
    const handleFirstGesture = () => {
      audioPlayer.unlockAudioContext();
      window.removeEventListener('click', handleFirstGesture);
      window.removeEventListener('keydown', handleFirstGesture);
      window.removeEventListener('touchstart', handleFirstGesture);
    };
    window.addEventListener('click', handleFirstGesture);
    window.addEventListener('keydown', handleFirstGesture);
    window.addEventListener('touchstart', handleFirstGesture);

    return () => {
      window.removeEventListener('click', handleFirstGesture);
      window.removeEventListener('keydown', handleFirstGesture);
      window.removeEventListener('touchstart', handleFirstGesture);
    };
  }, []);

  // Load persistent search and dialogue history on mount
  useEffect(() => {
    fetch('http://127.0.0.1:3000/api/history/searches')
      .then((res) => res.json())
      .then((data: any[]) => {
        if (Array.isArray(data) && data.length > 0) {
          const recs: WebSearchRecord[] = data.map((d) => ({
            query: d.query,
            source: d.source,
            snippets: d.snippets || [],
            summary: d.summary,
            timestamp: d.created_at < 1e11 ? d.created_at * 1000 : d.created_at,
          }));
          setSearchHistory((prev) => {
            const existing = new Set(prev.map((p) => p.query + p.timestamp));
            const newOnes = recs.filter((r) => !existing.has(r.query + r.timestamp));
            return [...prev, ...newOnes];
          });
        }
      })
      .catch((e) => console.warn('Could not load persistent searches:', e));

    fetch('http://127.0.0.1:3000/api/history/dialogues')
      .then((res) => res.json())
      .then((data: any[]) => {
        if (Array.isArray(data) && data.length > 0) {
          setMessages((prev) => {
            if (prev.length > 0) return prev;
            return data.map((d) => ({
              id: d.id,
              sender: d.sender,
              text: d.text,
              timestamp: d.created_at < 1e11 ? d.created_at * 1000 : d.created_at,
            }));
          });
        }
      })
      .catch((e) => console.warn('Could not load persistent dialogues:', e));
  }, []);

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

        case 'tool_executing':
          setCurrentSearch({ query: data.query, message: data.message });
          break;

        case 'tool_executed': {
          const rec: WebSearchRecord = {
            query: data.query,
            source: data.source || 'DuckDuckGo',
            snippets: data.snippets || [],
            summary: data.summary,
            timestamp: Date.now(),
          };
          activeSearchRef.current = rec;
          setSearchHistory((prev) => [rec, ...prev]);
          setCurrentSearch(null);
          break;
        }

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
              const rec = activeSearchRef.current;
              return [
                ...prev,
                {
                  id: data.interaction_id || String(Date.now()),
                  sender: 'character',
                  text: data.delta,
                  timestamp: Date.now(),
                  isStreaming: true,
                  searchRecord: rec || undefined,
                },
              ];
            }
          });
          break;

        case 'llm_completed':
          setInteractionStatus('idle');
          activeSearchRef.current = null;
          setCurrentSearch(null);
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

        case 'audio_ready':
          if (data.audio_url) {
            audioPlayer.playUrl(data.audio_url);
          } else if (data.text) {
            audioPlayer.speak(data.text);
          }
          break;

        case 'state_updated':
          if (data.emotion) setEmotion(data.emotion);
          if (data.relationship) {
            setRelationship((prev) => ({
              ...prev,
              ...data.relationship,
            }));
          }
          break;

        case 'memory_formed':
          if (data.memory) {
            setMemories((prev) => [data.memory, ...prev]);
          }
          break;

        case 'reset_completed':
          audioPlayer.stop();
          setMessages([]);
          setInteractionStatus('idle');
          setDecisionTrace(null);
          setContextBreakdown(null);
          setSearchHistory([]);
          setCurrentSearch(null);
          activeSearchRef.current = null;
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
          setRelationship({
            stage: 'Casual Companion',
            closeness: 0.55,
            trust: 0.65,
            familiarity: 0.60,
            affection: 0.62,
            tension: 0.02,
            known_facts: ['Đam mê xây dựng hệ thống tác tử AI thông minh'],
          });
          break;
      }
    });

    client.connect();

    return () => {
      unsubscribe();
    };
  }, []);

  // Hotkey listener (~ for Inspector, Ctrl+H for History, Esc for modals)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'h') {
        e.preventDefault();
        setIsHistoryOpen((prev) => !prev);
      } else if (e.key === '`' || e.key === '~') {
        e.preventDefault();
        setIsInspectorOpen((prev) => !prev);
      } else if (e.key === 'Escape') {
        setIsInspectorOpen(false);
        setIsStudioOpen(false);
        setIsHistoryOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleClearHistory = async () => {
    try {
      await fetch('http://127.0.0.1:3000/api/history/clear', { method: 'POST' });
      setSearchHistory([]);
      setMessages([]);
    } catch (e) {
      console.warn('Could not clear history on backend:', e);
    }
  };

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
        isHistoryOpen={isHistoryOpen}
        onToggleHistory={() => setIsHistoryOpen((prev) => !prev)}
        isVoiceMuted={isVoiceMuted}
        onToggleVoice={() => {
          const next = !isVoiceMuted;
          audioPlayer.setMuted(next);
          setIsVoiceMuted(next);
        }}
        onTestVoice={async () => {
          if (isTestVoiceLoading || isAudioPlaying) return;
          setIsTestVoiceLoading(true);
          try {
            await audioPlayer.unlockAudioContext();
            await audioPlayer.speak('Ara ara~ Chào cưng nhé! Cáo tỷ tỷ Yae Miko đây. Hôm nay muốn tỷ tỷ cưng chiều điều gì nào?');
          } catch (e) {
            console.warn('Test voice error:', e);
          } finally {
            setIsTestVoiceLoading(false);
          }
        }}
        isAudioPlaying={isAudioPlaying}
        isAudioLoading={isTestVoiceLoading}
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
          {stageMode === 'vrm' ? (
            <VrmAvatarStage
              emotion={emotion}
              relationship={relationship}
              interactionStatus={interactionStatus}
              characterName={characterName}
              isAudioPlaying={isAudioPlaying}
              audioAmplitude={audioAmplitude}
              onSwitchToOrb={() => setStageMode('orb')}
            />
          ) : (
            <EmotionStage
              emotion={emotion}
              relationship={relationship}
              interactionStatus={interactionStatus}
              characterName={characterName}
              isAudioPlaying={isAudioPlaying}
              audioAmplitude={audioAmplitude}
              onSwitchToVrm={() => setStageMode('vrm')}
            />
          )}

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
              onPlayMessage={(text) => {
                audioPlayer.unlockAudioContext();
                audioPlayer.speak(text);
              }}
              currentSearch={currentSearch}
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
          searchHistory={searchHistory}
        />
      </main>

      {/* Personality Studio Modal */}
      <PersonalityEditorModal
        isOpen={isStudioOpen}
        onClose={() => setIsStudioOpen(false)}
        personality={personality}
        onSave={handleSavePersonality}
      />

      {/* Chrome-Style History Center Modal */}
      <HistoryModal
        isOpen={isHistoryOpen}
        onClose={() => setIsHistoryOpen(false)}
        searches={searchHistory}
        memories={memories}
        messages={messages}
        onSelectTopic={handleSendMessage}
        onClearHistory={handleClearHistory}
      />
    </div>
  );
};
