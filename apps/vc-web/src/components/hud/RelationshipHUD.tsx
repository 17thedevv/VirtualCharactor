import React from 'react';
import type { RelationshipData } from '../../types/character';
import { Heart, BookOpen } from 'lucide-react';

interface RelationshipHUDProps {
  relationship: RelationshipData;
}

export const RelationshipHUD: React.FC<RelationshipHUDProps> = ({ relationship }) => {
  return (
    <div
      className="glass-card"
      style={{
        padding: '16px 20px',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Heart size={16} color="var(--accent-rose)" fill="var(--accent-rose)" style={{ opacity: 0.85 }} />
          <span style={{ fontSize: '0.86rem', fontWeight: 600, color: 'var(--text-primary)' }}>
            Mối Quan Hệ & Gắn Kết
          </span>
        </div>
        <span className="badge badge-amber" style={{ fontSize: '0.72rem' }}>
          {relationship.stage}
        </span>
      </div>

      {/* Closeness Bar */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.76rem', marginBottom: '4px' }}>
          <span style={{ color: 'var(--text-secondary)' }}>Mức độ gắn kết (Closeness)</span>
          <span style={{ color: 'var(--accent-cyan)', fontWeight: 600 }}>
            {Math.round(relationship.closeness * 100)}%
          </span>
        </div>
        <div
          style={{
            height: '6px',
            background: 'rgba(255, 255, 255, 0.06)',
            borderRadius: 'var(--radius-full)',
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              height: '100%',
              width: `${Math.round(relationship.closeness * 100)}%`,
              background: 'linear-gradient(90deg, var(--accent-cyan), var(--accent-blue))',
              borderRadius: 'var(--radius-full)',
              transition: 'width 0.8s cubic-bezier(0.16, 1, 0.3, 1)',
              boxShadow: '0 0 10px rgba(0, 242, 254, 0.4)',
            }}
          />
        </div>
      </div>

      {/* Trust Bar */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.76rem', marginBottom: '4px' }}>
          <span style={{ color: 'var(--text-secondary)' }}>Độ tin cậy tích lũy (Trust)</span>
          <span style={{ color: 'var(--accent-emerald)', fontWeight: 600 }}>
            {Math.round(relationship.trust * 100)}%
          </span>
        </div>
        <div
          style={{
            height: '6px',
            background: 'rgba(255, 255, 255, 0.06)',
            borderRadius: 'var(--radius-full)',
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              height: '100%',
              width: `${Math.round(relationship.trust * 100)}%`,
              background: 'linear-gradient(90deg, var(--accent-emerald), #38ef7d)',
              borderRadius: 'var(--radius-full)',
              transition: 'width 0.8s cubic-bezier(0.16, 1, 0.3, 1)',
              boxShadow: '0 0 10px rgba(5, 214, 158, 0.4)',
            }}
          />
        </div>
      </div>

      {/* Known facts pill */}
      {relationship.known_facts.length > 0 && (
        <div
          style={{
            padding: '10px 12px',
            background: 'rgba(0, 0, 0, 0.25)',
            borderRadius: 'var(--radius-sm)',
            border: '1px solid var(--border-subtle)',
            fontSize: '0.76rem',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--text-muted)', marginBottom: '4px' }}>
            <BookOpen size={12} />
            <span>Ký ức về bạn ({relationship.known_facts.length}):</span>
          </div>
          <p style={{ color: 'var(--text-secondary)', lineHeight: 1.4, fontStyle: 'italic' }}>
            "{relationship.known_facts[0]}"
          </p>
        </div>
      )}
    </div>
  );
};
