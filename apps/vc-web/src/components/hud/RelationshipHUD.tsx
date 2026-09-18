import React from 'react';
import type { RelationshipData } from '../../types/character';
import { Heart, BookOpen, ShieldCheck, AlertCircle } from 'lucide-react';

interface RelationshipHUDProps {
  relationship: RelationshipData;
}

export const RelationshipHUD: React.FC<RelationshipHUDProps> = ({ relationship }) => {
  const familiarity = relationship.familiarity ?? 0.5;
  const affection = relationship.affection ?? 0.55;
  const tension = relationship.tension ?? 0.0;

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
      {/* Header with Stage Badge */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Heart size={16} color="var(--accent-rose)" fill="var(--accent-rose)" style={{ opacity: 0.85 }} />
          <span style={{ fontSize: '0.86rem', fontWeight: 600, color: 'var(--text-primary)' }}>
            Mối Quan Hệ & Gắn Kết
          </span>
        </div>
        <span className="badge badge-amber" style={{ fontSize: '0.72rem', letterSpacing: '0.02em' }}>
          {relationship.stage}
        </span>
      </div>

      {/* Grid of Key Meters */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
        {/* Trust Bar */}
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.74rem', marginBottom: '4px' }}>
            <span style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', gap: '4px' }}>
              <ShieldCheck size={12} color="var(--accent-emerald)" />
              <span>Độ tin cậy (Trust)</span>
            </span>
            <span style={{ color: 'var(--accent-emerald)', fontWeight: 600 }}>
              {Math.round(relationship.trust * 100)}%
            </span>
          </div>
          <div
            style={{
              height: '5px',
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

        {/* Closeness Bar */}
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.74rem', marginBottom: '4px' }}>
            <span style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', gap: '4px' }}>
              <Heart size={12} color="var(--accent-rose)" />
              <span>Mức độ gần gũi (Closeness)</span>
            </span>
            <span style={{ color: 'var(--accent-cyan)', fontWeight: 600 }}>
              {Math.round(relationship.closeness * 100)}%
            </span>
          </div>
          <div
            style={{
              height: '5px',
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

        {/* Secondary Metrics: Familiarity & Affection */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', marginTop: '2px' }}>
          {/* Familiarity */}
          <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px 10px', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7rem', color: 'var(--text-muted)', marginBottom: '3px' }}>
              <span>Quen thuộc</span>
              <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{Math.round(familiarity * 100)}%</span>
            </div>
            <div style={{ height: '3px', background: 'rgba(255, 255, 255, 0.06)', borderRadius: '2px', overflow: 'hidden' }}>
              <div style={{ width: `${Math.round(familiarity * 100)}%`, height: '100%', background: '#a29bfe' }} />
            </div>
          </div>

          {/* Affection */}
          <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px 10px', borderRadius: 'var(--radius-sm)', border: '1px solid var(--border-subtle)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7rem', color: 'var(--text-muted)', marginBottom: '3px' }}>
              <span>Thiện cảm</span>
              <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{Math.round(affection * 100)}%</span>
            </div>
            <div style={{ height: '3px', background: 'rgba(255, 255, 255, 0.06)', borderRadius: '2px', overflow: 'hidden' }}>
              <div style={{ width: `${Math.round(affection * 100)}%`, height: '100%', background: '#ff758c' }} />
            </div>
          </div>
        </div>

        {/* Tension alert if present */}
        {tension > 0.05 && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              borderRadius: 'var(--radius-sm)',
              background: 'rgba(255, 107, 107, 0.08)',
              border: '1px solid rgba(255, 107, 107, 0.25)',
              fontSize: '0.72rem',
              color: '#ff6b6b',
            }}
          >
            <AlertCircle size={13} />
            <span>Mức độ phòng thủ / cẩn trọng: {Math.round(tension * 100)}%</span>
          </div>
        )}
      </div>

      {/* Known facts section */}
      {relationship.known_facts.length > 0 && (
        <div
          style={{
            padding: '10px 12px',
            background: 'rgba(0, 0, 0, 0.25)',
            borderRadius: 'var(--radius-sm)',
            border: '1px solid var(--border-subtle)',
            fontSize: '0.74rem',
            marginTop: '2px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--text-muted)', marginBottom: '5px' }}>
            <BookOpen size={12} />
            <span>Ký ức riêng về bạn ({relationship.known_facts.length}):</span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
            {relationship.known_facts.slice(0, 2).map((fact, idx) => (
              <p key={idx} style={{ color: 'var(--text-secondary)', lineHeight: 1.4, fontStyle: 'italic', margin: 0 }}>
                • "{fact}"
              </p>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
