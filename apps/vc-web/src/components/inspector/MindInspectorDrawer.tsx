import React, { useRef, useEffect } from 'react';
import type {
  EmotionData,
  DecisionTraceData,
  ContextBreakdownData,
  MemoryItem,
} from '../../types/character';
import { Brain, X, GitCommit, Database, Layers, Activity } from 'lucide-react';

interface MindInspectorDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  emotion: EmotionData;
  decisionTrace: DecisionTraceData | null;
  contextBreakdown: ContextBreakdownData | null;
  memories: MemoryItem[];
}

export const MindInspectorDrawer: React.FC<MindInspectorDrawerProps> = ({
  isOpen,
  onClose,
  emotion,
  decisionTrace,
  contextBreakdown,
  memories,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Render 2D Emotion Radar (Valence x Arousal)
  useEffect(() => {
    if (!isOpen || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);

    // Grid center
    const cx = width / 2;
    const cy = height / 2;

    // Draw grid background
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
    ctx.lineWidth = 1;

    // Axes
    ctx.beginPath();
    ctx.moveTo(0, cy);
    ctx.lineTo(width, cy);
    ctx.moveTo(cx, 0);
    ctx.lineTo(cx, height);
    ctx.stroke();

    // Concentric circles
    ctx.beginPath();
    ctx.arc(cx, cy, width * 0.22, 0, Math.PI * 2);
    ctx.arc(cx, cy, width * 0.42, 0, Math.PI * 2);
    ctx.stroke();

    // Labels for Quadrants
    ctx.fillStyle = 'rgba(255, 255, 255, 0.35)';
    ctx.font = '10px "Plus Jakarta Sans", sans-serif';
    ctx.textAlign = 'right';
    ctx.fillText('Hân hoan (+V, +A)', width - 10, 16);
    ctx.textAlign = 'left';
    ctx.fillText('Kích động (-V, +A)', 10, 16);
    ctx.textAlign = 'right';
    ctx.fillText('Thanh thản (+V, -A)', width - 10, height - 10);
    ctx.textAlign = 'left';
    ctx.fillText('Trầm tư (-V, -A)', 10, height - 10);

    // Map Valence (-1 to 1) -> X, Arousal (0 to 1) -> Y (inverted coordinate)
    const targetX = cx + emotion.valence * (width * 0.42);
    const targetY = cy - (emotion.arousal * 2 - 1) * (height * 0.42);

    // Target point glow
    const glowGradient = ctx.createRadialGradient(targetX, targetY, 0, targetX, targetY, 20);
    glowGradient.addColorStop(0, 'rgba(0, 242, 254, 0.9)');
    glowGradient.addColorStop(0.5, 'rgba(0, 242, 254, 0.35)');
    glowGradient.addColorStop(1, 'transparent');

    ctx.fillStyle = glowGradient;
    ctx.beginPath();
    ctx.arc(targetX, targetY, 20, 0, Math.PI * 2);
    ctx.fill();

    // Core point
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(targetX, targetY, 4.5, 0, Math.PI * 2);
    ctx.fill();

    // Coordinate text
    ctx.fillStyle = '#00f2fe';
    ctx.font = 'bold 10px "JetBrains Mono", monospace';
    ctx.textAlign = targetX > cx ? 'right' : 'left';
    ctx.fillText(
      `V: ${emotion.valence.toFixed(2)} | A: ${emotion.arousal.toFixed(2)}`,
      targetX > cx ? targetX - 10 : targetX + 10,
      targetY - 8
    );
  }, [isOpen, emotion]);

  if (!isOpen) return null;

  return (
    <aside
      className="animate-fade-in"
      style={{
        position: 'fixed',
        top: 0,
        right: 0,
        bottom: 0,
        width: '420px',
        maxWidth: '92vw',
        background: 'rgba(9, 12, 18, 0.92)',
        backdropFilter: 'blur(24px)',
        borderLeft: '1px solid var(--border-medium)',
        zIndex: 50,
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '-8px 0 36px rgba(0,0,0,0.7)',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '18px 24px',
          borderBottom: '1px solid var(--border-subtle)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <div
            style={{
              width: '32px',
              height: '32px',
              borderRadius: 'var(--radius-sm)',
              background: 'rgba(0, 242, 254, 0.12)',
              color: 'var(--accent-cyan)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Brain size={18} />
          </div>
          <div>
            <h3 style={{ fontSize: '0.96rem', fontWeight: 700, color: 'var(--text-primary)' }}>
              Mind Visualizer
            </h3>
            <span style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
              Telemetry nhận thức thời gian thực
            </span>
          </div>
        </div>

        <button onClick={onClose} className="btn-icon" title="Đóng Mind Inspector">
          <X size={18} />
        </button>
      </div>

      {/* Content Scrollable */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: '20px 24px',
          display: 'flex',
          flexDirection: 'column',
          gap: '24px',
        }}
      >
        {/* Section 1: 2D Emotion Circumplex Radar */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
            <Activity size={15} color="var(--accent-cyan)" />
            <span style={{ fontSize: '0.84rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              Radar Cảm Xúc (Valence-Arousal Space)
            </span>
          </div>

          <div
            className="glass-card"
            style={{
              padding: '12px',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              background: 'rgba(0, 0, 0, 0.35)',
            }}
          >
            <canvas
              ref={canvasRef}
              width={340}
              height={220}
              style={{ width: '100%', height: 'auto', borderRadius: 'var(--radius-sm)' }}
            />
            <div
              style={{
                width: '100%',
                display: 'flex',
                justifyContent: 'space-between',
                marginTop: '10px',
                fontSize: '0.74rem',
                color: 'var(--text-secondary)',
                padding: '0 4px',
              }}
            >
              <span>Cảm xúc chủ đạo: <strong style={{ color: 'var(--accent-cyan)' }}>{emotion.dominant_emotion || 'curiosity'}</strong></span>
              <span>Cường độ: <strong style={{ color: 'var(--text-primary)' }}>{Math.round((emotion.dominant_intensity ?? 0.5) * 100)}%</strong></span>
            </div>

            {/* 8-Axis Spectrum Breakdown */}
            <div style={{ width: '100%', marginTop: '12px', display: 'flex', flexDirection: 'column', gap: '6px', borderTop: '1px solid rgba(255,255,255,0.06)', paddingTop: '10px' }}>
              {[
                { key: 'joy', label: 'Joy (Hân hoan)', val: emotion.joy, color: '#ff9f43' },
                { key: 'affection', label: 'Affection (Yêu mến)', val: emotion.affection, color: '#ff758c' },
                { key: 'curiosity', label: 'Curiosity (Tò mò)', val: emotion.curiosity, color: '#05d69e' },
                { key: 'surprise', label: 'Surprise (Bất ngờ)', val: emotion.surprise, color: '#f9ca24' },
                { key: 'embarrassment', label: 'Embarrassment (Bối rối)', val: emotion.embarrassment, color: '#e056fd' },
                { key: 'sadness', label: 'Sadness (U sầu)', val: emotion.sadness, color: '#9d4edd' },
                { key: 'fear', label: 'Fear (Bất an)', val: emotion.fear, color: '#70a1ff' },
                { key: 'anger', label: 'Anger (Căng thẳng)', val: emotion.anger, color: '#ff416c' },
              ].map(item => (
                <div key={item.key} style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.7rem' }}>
                  <span style={{ width: '130px', color: 'var(--text-muted)' }}>{item.label}</span>
                  <div style={{ flex: 1, height: '4px', background: 'rgba(255,255,255,0.08)', borderRadius: '2px', overflow: 'hidden' }}>
                    <div style={{ width: `${Math.round((item.val ?? 0) * 100)}%`, height: '100%', background: item.color, transition: 'width 0.3s ease' }} />
                  </div>
                  <span style={{ width: '32px', textAlign: 'right', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
                    {Math.round((item.val ?? 0) * 100)}%
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Section 2: Decision Engine Trace */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
            <GitCommit size={15} color="var(--accent-amber)" />
            <span style={{ fontSize: '0.84rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              Lập Luận Ra Quyết Định (Decision Trace)
            </span>
          </div>

          <div className="glass-card" style={{ padding: '14px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {decisionTrace ? (
              <>
                <div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                    <span style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
                      HÀNH ĐỘNG ĐÃ CHỌN (SELECTED ACTION)
                    </span>
                    {decisionTrace.confidence !== undefined && (
                      <span style={{ fontSize: '0.7rem', color: 'var(--accent-amber)', fontFamily: 'var(--font-mono)' }}>
                        Tin cậy: {Math.round(decisionTrace.confidence * 100)}%
                      </span>
                    )}
                  </div>
                  <div
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '4px 12px',
                      borderRadius: 'var(--radius-full)',
                      background: 'rgba(255, 159, 67, 0.15)',
                      border: '1px solid rgba(255, 159, 67, 0.35)',
                      color: 'var(--accent-amber)',
                      fontFamily: 'var(--font-mono)',
                      fontSize: '0.82rem',
                      fontWeight: 600,
                    }}
                  >
                    <span>{decisionTrace.selected_action}</span>
                  </div>
                  {decisionTrace.action_description && (
                    <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '4px' }}>
                      {decisionTrace.action_description}
                    </div>
                  )}
                </div>

                {decisionTrace.policy && (
                  <div style={{ padding: '8px 10px', background: 'rgba(0, 0, 0, 0.2)', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
                    <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', marginBottom: '4px' }}>
                      CHÍNH SÁCH HÀNH VI (BEHAVIOR POLICY)
                    </div>
                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', fontSize: '0.72rem' }}>
                      <span style={{ background: 'rgba(5, 214, 158, 0.12)', color: 'var(--accent-cyan)', padding: '2px 6px', borderRadius: '4px' }}>
                        Tone: {decisionTrace.policy.tone}
                      </span>
                      <span style={{ background: 'rgba(255, 255, 255, 0.05)', color: 'var(--text-secondary)', padding: '2px 6px', borderRadius: '4px' }}>
                        Verbosity: {Math.round(decisionTrace.policy.verbosity * 100)}%
                      </span>
                      <span style={{ background: 'rgba(255, 255, 255, 0.05)', color: 'var(--text-secondary)', padding: '2px 6px', borderRadius: '4px' }}>
                        Initiative: {Math.round(decisionTrace.policy.initiative * 100)}%
                      </span>
                    </div>
                  </div>
                )}

                <div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', marginBottom: '4px' }}>
                    ĐỘC THOẠI NỘI TÂM (INNER REASONING)
                  </div>
                  <p
                    style={{
                      fontSize: '0.8rem',
                      color: 'var(--text-secondary)',
                      lineHeight: 1.55,
                      fontStyle: 'italic',
                      background: 'rgba(0, 0, 0, 0.25)',
                      padding: '8px 10px',
                      borderRadius: 'var(--radius-sm)',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    "{decisionTrace.reasoning}"
                  </p>
                </div>

                {/* Candidates table */}
                <div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', marginBottom: '6px' }}>
                    ỨNG VIÊN ĐÁNH GIÁ (CANDIDATES EVALUATED)
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                    {decisionTrace.candidates.map((cand, idx) => (
                      <div
                        key={idx}
                        style={{
                          display: 'flex',
                          flexDirection: 'column',
                          gap: '2px',
                          fontSize: '0.76rem',
                          padding: '6px 8px',
                          borderRadius: 'var(--radius-xs)',
                          background:
                            cand.action === decisionTrace.selected_action
                              ? 'rgba(255, 159, 67, 0.08)'
                              : 'rgba(255, 255, 255, 0.02)',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                          <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-primary)', fontWeight: cand.action === decisionTrace.selected_action ? 600 : 400 }}>
                            {cand.action}
                          </span>
                          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                            <div
                              style={{
                                width: '60px',
                                height: '5px',
                                background: 'rgba(255, 255, 255, 0.08)',
                                borderRadius: 'var(--radius-full)',
                                overflow: 'hidden',
                              }}
                            >
                              <div
                                style={{
                                  width: `${Math.round((cand.score ?? cand.confidence) * 100)}%`,
                                  height: '100%',
                                  background: 'var(--accent-amber)',
                                }}
                              />
                            </div>
                            <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-muted)', fontSize: '0.7rem' }}>
                              {Math.round((cand.score ?? cand.confidence) * 100)}%
                            </span>
                          </div>
                        </div>
                        {cand.rationale && (
                          <div style={{ fontSize: '0.68rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
                            {cand.rationale}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </>
            ) : (
              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                Chưa có dữ liệu lượt quyết định gần nhất.
              </span>
            )}
          </div>
        </div>

        {/* Section 3: Context Breakdown */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
            <Layers size={15} color="var(--accent-violet)" />
            <span style={{ fontSize: '0.84rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              Phân Rã Context Window (Token Budget)
            </span>
          </div>

          <div className="glass-card" style={{ padding: '14px', display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {contextBreakdown ? (
              <>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.78rem' }}>
                  <span style={{ color: 'var(--text-secondary)' }}>Đã sử dụng</span>
                  <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--accent-cyan)', fontWeight: 600 }}>
                    {contextBreakdown.tokens_used} / {contextBreakdown.token_budget} tokens
                  </span>
                </div>

                {/* Proportional bar */}
                <div
                  style={{
                    height: '8px',
                    borderRadius: 'var(--radius-full)',
                    background: 'rgba(255, 255, 255, 0.06)',
                    display: 'flex',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    title="Personality"
                    style={{
                      width: `${(contextBreakdown.breakdown.personality_tokens / contextBreakdown.token_budget) * 100}%`,
                      background: 'var(--accent-cyan)',
                    }}
                  />
                  <div
                    title="Memories"
                    style={{
                      width: `${(contextBreakdown.breakdown.memory_tokens / contextBreakdown.token_budget) * 100}%`,
                      background: 'var(--accent-emerald)',
                    }}
                  />
                  <div
                    title="State"
                    style={{
                      width: `${(contextBreakdown.breakdown.state_tokens / contextBreakdown.token_budget) * 100}%`,
                      background: 'var(--accent-amber)',
                    }}
                  />
                  <div
                    title="User Prompt"
                    style={{
                      width: `${(contextBreakdown.breakdown.user_input_tokens / contextBreakdown.token_budget) * 100}%`,
                      background: 'var(--emotion-melancholic)',
                    }}
                  />
                </div>

                {/* Legend */}
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', fontSize: '0.72rem', marginTop: '4px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--accent-cyan)' }} />
                    <span style={{ color: 'var(--text-secondary)' }}>Tính cách ({contextBreakdown.breakdown.personality_tokens}t)</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--accent-emerald)' }} />
                    <span style={{ color: 'var(--text-secondary)' }}>Ký ức ({contextBreakdown.breakdown.memory_tokens}t)</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--accent-amber)' }} />
                    <span style={{ color: 'var(--text-secondary)' }}>Trạng thái ({contextBreakdown.breakdown.state_tokens}t)</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--emotion-melancholic)' }} />
                    <span style={{ color: 'var(--text-secondary)' }}>Tin nhắn ({contextBreakdown.breakdown.user_input_tokens}t)</span>
                  </div>
                </div>
              </>
            ) : (
              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                Đang chờ tương tác để phân tích ngữ cảnh.
              </span>
            )}
          </div>
        </div>

        {/* Section 4: Memory Browser */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
            <Database size={15} color="var(--accent-emerald)" />
            <span style={{ fontSize: '0.84rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              Dòng Chảy Ký Ức Hoạt Động ({memories.length})
            </span>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {memories.map((mem) => (
              <div
                key={mem.id}
                className="glass-card"
                style={{
                  padding: '10px 12px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '4px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span
                    className={`badge ${mem.type === 'Semantic' ? 'badge-cyan' : 'badge-emerald'}`}
                    style={{ fontSize: '0.65rem', padding: '1px 6px' }}
                  >
                    {mem.type}
                  </span>
                  <span style={{ fontSize: '0.68rem', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
                    {mem.importance}
                  </span>
                </div>
                <p style={{ fontSize: '0.78rem', color: 'var(--text-secondary)', lineHeight: 1.4 }}>
                  {mem.content}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </aside>
  );
};
