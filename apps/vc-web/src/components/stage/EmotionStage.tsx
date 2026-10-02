import React from 'react';
import type { EmotionData, RelationshipData } from '../../types/character';
import { Heart, Activity, Compass, Sparkles } from 'lucide-react';

interface EmotionStageProps {
  emotion: EmotionData;
  relationship: RelationshipData;
  interactionStatus: 'idle' | 'thinking' | 'speaking' | 'listening';
  characterName: string;
  isAudioPlaying?: boolean;
  audioAmplitude?: number;
  onSwitchToVrm?: () => void;
}

export const EmotionStage: React.FC<EmotionStageProps> = ({
  emotion,
  relationship,
  interactionStatus,
  characterName,
  isAudioPlaying,
  audioAmplitude = 0,
  onSwitchToVrm,
}) => {
  // Determine dominant hue based on emotion
  const getEmotionPalette = (name: string) => {
    switch (name.toLowerCase()) {
      case 'joy':
      case 'excited':
        return {
          primary: 'var(--emotion-joy)',
          glow: 'var(--emotion-joy-glow)',
          label: 'Hân hoan & Vui vẻ',
          gradient: 'radial-gradient(circle, #ff9f43 0%, #ff5252 60%, transparent 80%)',
        };
      case 'affection':
        return {
          primary: '#ff6b81',
          glow: 'rgba(255, 107, 129, 0.4)',
          label: 'Yêu mến & Gắn kết',
          gradient: 'radial-gradient(circle, #ff758c 0%, #ff7eb3 60%, transparent 80%)',
        };
      case 'sadness':
      case 'melancholic':
      case 'reflective':
        return {
          primary: 'var(--emotion-melancholic)',
          glow: 'var(--emotion-melancholic-glow)',
          label: 'Trầm tư & Sâu lắng',
          gradient: 'radial-gradient(circle, #9d4edd 0%, #3a0ca3 60%, transparent 80%)',
        };
      case 'anger':
      case 'agitated':
      case 'frustrated':
        return {
          primary: 'var(--emotion-agitated)',
          glow: 'var(--emotion-agitated-glow)',
          label: 'Căng thẳng & Tức giận',
          gradient: 'radial-gradient(circle, #ff416c 0%, #ff4b2b 60%, transparent 80%)',
        };
      case 'fear':
        return {
          primary: '#70a1ff',
          glow: 'rgba(112, 161, 255, 0.4)',
          label: 'Bất an & Thận trọng',
          gradient: 'radial-gradient(circle, #70a1ff 0%, #2f3542 60%, transparent 80%)',
        };
      case 'surprise':
        return {
          primary: '#f9ca24',
          glow: 'rgba(249, 202, 36, 0.45)',
          label: 'Kinh ngạc & Bất ngờ',
          gradient: 'radial-gradient(circle, #f9ca24 0%, #f0932b 60%, transparent 80%)',
        };
      case 'embarrassment':
        return {
          primary: '#e056fd',
          glow: 'rgba(224, 86, 253, 0.4)',
          label: 'Bối rối & Ngại ngùng',
          gradient: 'radial-gradient(circle, #e056fd 0%, #be2edd 60%, transparent 80%)',
        };
      case 'curious':
      case 'curiosity':
      default:
        return {
          primary: 'var(--emotion-curious)',
          glow: 'var(--emotion-curious-glow)',
          label: 'Tò mò & Khám phá',
          gradient: 'radial-gradient(circle, #05d69e 0%, #00b4d8 60%, transparent 80%)',
        };
    }
  };

  const dominantName = emotion.dominant_emotion || 'curiosity';
  const dominantIntensity = emotion.dominant_intensity ?? 0.5;
  const palette = getEmotionPalette(dominantName);
  const pulseSpeed = Math.max(1.8, 4.5 - dominantIntensity * 2.8); // Faster pulse when intensity is high

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
      {/* Switch to 3D VRM Avatar Button */}
      {onSwitchToVrm && (
        <button
          onClick={onSwitchToVrm}
          style={{
            position: 'absolute',
            top: '8px',
            right: '12px',
            display: 'inline-flex',
            alignItems: 'center',
            gap: '6px',
            padding: '4px 10px',
            borderRadius: '20px',
            background: 'rgba(255, 255, 255, 0.08)',
            border: '1px solid rgba(255, 255, 255, 0.15)',
            backdropFilter: 'blur(10px)',
            fontSize: '11px',
            color: 'rgba(255, 255, 255, 0.85)',
            cursor: 'pointer',
            transition: 'all 0.2s ease',
          }}
          title="Chuyển sang chế độ hiển thị 3D VRM Avatar"
        >
          <Sparkles size={12} color="#00d2d3" />
          <span>3D VRM Avatar</span>
        </button>
      )}

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
            opacity: 0.65 + dominantIntensity * 0.35,
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
            transform: isAudioPlaying ? `scale(${1 + audioAmplitude * 0.25})` : 'scale(1)',
            animation: isAudioPlaying ? 'none' : `breathe ${pulseSpeed}s ease-in-out infinite`,
            transition: 'transform 80ms ease-out, all var(--transition-emotion)',
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
          marginBottom: '6px',
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
          ({Math.round(dominantIntensity * 100)}%)
        </span>
      </div>

      {/* Real-time Voice Soundwave Equalizer when Aria is speaking */}
      {isAudioPlaying && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            height: '18px',
            marginBottom: '8px',
            padding: '2px 10px',
            borderRadius: 'var(--radius-full)',
            background: 'rgba(0, 0, 0, 0.3)',
            border: `1px solid ${palette.primary}44`,
          }}
        >
          {[0.5, 0.85, 1.0, 0.7, 0.9, 0.6].map((factor, i) => (
            <span
              key={i}
              style={{
                width: '3px',
                height: `${Math.max(4, Math.min(16, (audioAmplitude || 0.4) * factor * 20))}px`,
                backgroundColor: palette.primary,
                borderRadius: '2px',
                transition: 'height 60ms ease-out',
                boxShadow: `0 0 6px ${palette.primary}`,
              }}
            />
          ))}
          <span style={{ fontSize: '0.68rem', color: palette.primary, marginLeft: '4px', fontWeight: 600 }}>
            Đang nói...
          </span>
        </div>
      )}

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
