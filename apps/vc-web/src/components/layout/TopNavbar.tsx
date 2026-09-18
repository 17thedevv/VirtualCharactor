import React from 'react';
import { Sparkles, Brain, Sliders, RotateCcw } from 'lucide-react';

interface TopNavbarProps {
  connectionStatus: 'connected' | 'simulated' | 'disconnected';
  characterName: string;
  isInspectorOpen: boolean;
  onToggleInspector: () => void;
  onOpenStudio: () => void;
  onReset: () => void;
}

export const TopNavbar: React.FC<TopNavbarProps> = ({
  connectionStatus,
  characterName,
  isInspectorOpen,
  onToggleInspector,
  onOpenStudio,
  onReset,
}) => {
  return (
    <header
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '14px 24px',
        borderBottom: '1px solid var(--border-subtle)',
        background: 'rgba(7, 9, 14, 0.75)',
        backdropFilter: 'blur(16px)',
        zIndex: 40,
      }}
    >
      {/* Brand & Character Title */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
        <div
          style={{
            width: '38px',
            height: '38px',
            borderRadius: 'var(--radius-sm)',
            background: 'linear-gradient(135deg, var(--accent-cyan), var(--accent-blue))',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 0 16px rgba(0, 242, 254, 0.35)',
          }}
        >
          <Sparkles size={20} color="#040812" />
        </div>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '1.05rem', letterSpacing: '0.02em' }}>
              VirtualCharacter
            </span>
            <span className="badge badge-cyan" style={{ fontSize: '0.68rem', padding: '2px 8px' }}>
              Runtime Core
            </span>
          </div>
          <div style={{ fontSize: '0.78rem', color: 'var(--text-secondary)' }}>
            Đang tương tác cùng <strong style={{ color: 'var(--text-primary)' }}>{characterName}</strong>
          </div>
        </div>
      </div>

      {/* Connection & Actions */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
        {/* Connection status badge */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '5px 12px',
            borderRadius: 'var(--radius-full)',
            background: 'rgba(255, 255, 255, 0.04)',
            border: '1px solid var(--border-subtle)',
            fontSize: '0.78rem',
            color: 'var(--text-secondary)',
          }}
        >
          <span
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor:
                connectionStatus === 'connected'
                  ? 'var(--accent-emerald)'
                  : connectionStatus === 'simulated'
                  ? 'var(--accent-amber)'
                  : 'var(--accent-rose)',
              boxShadow:
                connectionStatus === 'connected'
                  ? '0 0 10px var(--accent-emerald)'
                  : connectionStatus === 'simulated'
                  ? '0 0 10px var(--accent-amber)'
                  : 'none',
            }}
          />
          {connectionStatus === 'connected'
            ? 'Axum WS Connected'
            : connectionStatus === 'simulated'
            ? 'Local Sim Mode'
            : 'Reconnecting...'}
        </div>

        {/* Mind Visualizer Toggle */}
        <button
          onClick={onToggleInspector}
          className={`btn-icon ${isInspectorOpen ? 'active' : ''}`}
          title="Bật/tắt Mind Visualizer (Phím tắt ~)"
          style={{
            width: 'auto',
            padding: '0 14px',
            gap: '6px',
            fontSize: '0.82rem',
            fontWeight: 600,
          }}
        >
          <Brain size={16} />
          <span>Mind Inspector</span>
          <kbd
            style={{
              padding: '1px 5px',
              background: 'rgba(255,255,255,0.08)',
              borderRadius: '4px',
              fontSize: '0.7rem',
              color: 'var(--text-muted)',
            }}
          >
            ~
          </kbd>
        </button>

        {/* Personality Studio */}
        <button
          onClick={onOpenStudio}
          className="btn-icon"
          title="Tùy chỉnh tính cách nhân vật (Personality Studio)"
        >
          <Sliders size={18} />
        </button>

        {/* Reset State */}
        <button
          onClick={onReset}
          className="btn-icon"
          title="Khôi phục trạng thái mặc định"
        >
          <RotateCcw size={17} />
        </button>
      </div>
    </header>
  );
};
