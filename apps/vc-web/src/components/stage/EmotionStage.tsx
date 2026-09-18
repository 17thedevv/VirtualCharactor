import React from 'react';
import type { EmotionData, RelationshipData } from '../../types/character';
import { Heart, Activity, Compass } from 'lucide-react';

interface EmotionStageProps {
  emotion: EmotionData;
  relationship: RelationshipData;
  interactionStatus: 'idle' | 'thinking' | 'speaking' | 'listening';
  characterName: string;
}

export const EmotionStage: React.FC<EmotionStageProps> = ({
  emotion,
  relationship,
  interactionStatus,
  characterName,
}) => {
  // Determine dominant hue based on emotion
  const getEmotionPalette = (name: string) => {
    switch (name.toLowerCase()) {
      case 'joy':
      case 'excited':
        return {
          primary: 'var(--emotion-joy)',
          glow: 'var(--emotion-joy-glow)',
          label: 'Hân hoan & Phấn khởi',
          gradient: 'radial-gradient(circle, #ff9f43 0%, #ff5252 60%, transparent 80%)',
        };
      case 'melancholic':
      case 'reflective':
        return {
          primary: 'var(--emotion-melancholic)',
          glow: 'var(--emotion-melancholic-glow)',
          label: 'Trầm tư & Sâu lắng',
          gradient: 'radial-gradient(circle, #9d4edd 0%, #3a0ca3 60%, transparent 80%)',
        };
      case 'agitated':
      case 'frustrated':
        return {
          primary: 'var(--emotion-agitated)',
          glow: 'var(--emotion-agitated-glow)',
          label: 'Kích động & Căng thẳng',
          gradient: 'radial-gradient(circle, #ff416c 0%, #ff4b2b 60%, transparent 80%)',
        };
      case 'calm':
      case 'serene':
        return {
          primary: 'var(--emotion-calm)',
          glow: 'var(--emotion-calm-glow)',
          label: 'Điềm tĩnh & Thư thái',
          gradient: 'radial-gradient(circle, #00f2fe 0%, #4facfe 60%, transparent 80%)',
        };
      case 'curious':
      case 'inspired':
      default:
        return {
          primary: 'var(--emotion-curious)',
          glow: 'var(--emotion-curious-glow)',
          label: 'Tò mò & Hứng khởi',
          gradient: 'radial-gradient(circle, #05d69e 0%, #00b4d8 60%, transparent 80%)',
        };
    }
  };

  const palette = getEmotionPalette(emotion.primary_emotion);
  const pulseSpeed = Math.max(1.8, 4.5 - emotion.intensity * 2.8); // Faster pulse when intensity is high

  const getStatusText = () => {
    switch (interactionStatus) {
      case 'thinking':
        return 'Đang lập luận quyết định...';
      case 'speaking':
        return 'Đang trò chuyện cùng bạn...';
      case 'listening':
        return 'Đang chăm chú lắng nghe...';
      default:
        return 'Sẵn sàng tương tác';
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '36px 20px',
        position: 'relative',
        userSelect: 'none',
      }}
    >
      {/* Dynamic Emotion Core / Orb */}
      <div
        style={{
          position: 'relative',
          width: '190px',
          height: '190px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          marginBottom: '22px',
        }}
      >
        {/* Outermost Pulsing Halo */}
        <div
          style={{
            position: 'absolute',
            inset: '-20px',
            borderRadius: '50%',
            background: palette.glow,
            filter: 'blur(35px)',
            opacity: 0.65 + emotion.intensity * 0.35,
            transition: 'background var(--transition-emotion), opacity var(--transition-emotion)',
            animation: `breathe ${pulseSpeed}s ease-in-out infinite`,
          }}
        />

        {/* Orbit Ring 1 */}
        <div
          style={{
            position: 'absolute',
            inset: '-6px',
            borderRadius: '50%',
            border: `1.5px dashed ${palette.primary}`,
            opacity: 0.45,
            animation: 'aura-spin 22s linear infinite',
            transition: 'border-color var(--transition-emotion)',
          }}
        />

        {/* Orbit Ring 2 */}
        <div
          style={{
            position: 'absolute',
            inset: '8px',
            borderRadius: '50%',
            border: `1px solid ${palette.primary}`,
            opacity: 0.3,
            animation: 'aura-spin 14s linear infinite reverse',
            transition: 'border-color var(--transition-emotion)',
          }}
        />

        {/* Inner Luminous Core */}
        <div
          style={{
            width: '130px',
            height: '130px',
            borderRadius: '50%',
            background: palette.gradient,
            boxShadow: `0 0 50px ${palette.glow}, inset 0 0 24px rgba(255,255,255,0.45)`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            position: 'relative',
            animation: `breathe ${pulseSpeed}s ease-in-out infinite`,
            transition: 'all var(--transition-emotion)',
          }}
        >
          {/* Inner Light Ripple */}
          <div
            style={{
              width: '85px',
              height: '85px',
              borderRadius: '50%',
              background: 'radial-gradient(circle, rgba(255,255,255,0.95) 0%, rgba(255,255,255,0.1) 75%, transparent 100%)',
              filter: 'blur(4px)',
            }}
          />
        </div>

        {/* Floating status icon */}
        <div
          style={{
            position: 'absolute',
            bottom: '2px',
            right: '12px',
            background: 'rgba(7, 9, 14, 0.85)',
            backdropFilter: 'blur(10px)',
            border: '1px solid var(--border-medium)',
            borderRadius: 'var(--radius-full)',
            padding: '6px',
            boxShadow: '0 4px 14px rgba(0,0,0,0.5)',
            color: palette.primary,
          }}
        >
          <Activity size={15} />
        </div>
      </div>

      {/* Character Identity & Mood Badge */}
      <h2
        style={{
          fontFamily: 'var(--font-display)',
          fontSize: '1.45rem',
          fontWeight: 700,
          color: 'var(--text-primary)',
          letterSpacing: '-0.01em',
          marginBottom: '6px',
        }}
      >
        {characterName}
      </h2>

      {/* Primary Emotion Pill */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '7px',
          padding: '4px 14px',
          borderRadius: 'var(--radius-full)',
          background: 'rgba(255, 255, 255, 0.04)',
          border: '1px solid var(--border-subtle)',
          fontSize: '0.8rem',
          color: 'var(--text-secondary)',
          marginBottom: '10px',
        }}
      >
        <span
          style={{
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: palette.primary,
            boxShadow: `0 0 10px ${palette.primary}`,
          }}
        />
        <span>{palette.label}</span>
        <span style={{ color: 'var(--text-muted)', fontSize: '0.72rem' }}>
          ({Math.round(emotion.intensity * 100)}%)
        </span>
      </div>

      {/* Activity / Cognitive status */}
      <p
        style={{
          fontSize: '0.82rem',
          color: 'var(--text-muted)',
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          marginBottom: '14px',
        }}
      >
        <Compass size={13} />
        <span>{getStatusText()}</span>
      </p>

      {/* Relationship Stage Mini Progress */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '6px 14px',
          borderRadius: 'var(--radius-sm)',
          background: 'rgba(255, 255, 255, 0.02)',
          border: '1px solid var(--border-subtle)',
          fontSize: '0.75rem',
          color: 'var(--text-secondary)',
        }}
      >
        <Heart size={13} color="var(--accent-rose)" fill="var(--accent-rose)" style={{ opacity: 0.85 }} />
        <span>Quan hệ: <strong style={{ color: 'var(--text-primary)' }}>{relationship.stage}</strong></span>
        <span style={{ color: 'var(--text-muted)' }}>• Gắn kết: {Math.round(relationship.closeness * 100)}%</span>
      </div>
    </div>
  );
};
