import React, { useState } from 'react';
import type { PersonalityData } from '../../types/character';
import { X, Sliders, Save, Plus } from 'lucide-react';

interface PersonalityEditorModalProps {
  isOpen: boolean;
  onClose: () => void;
  personality: PersonalityData;
  onSave: (updated: PersonalityData) => void;
}

export const PersonalityEditorModal: React.FC<PersonalityEditorModalProps> = ({
  isOpen,
  onClose,
  personality,
  onSave,
}) => {
  const [draft, setDraft] = useState<PersonalityData>(personality);
  const [newTrait, setNewTrait] = useState('');
  const [newValue, setNewValue] = useState('');

  if (!isOpen) return null;

  const handleAddTrait = () => {
    if (!newTrait.trim()) return;
    setDraft({ ...draft, traits: [...draft.traits, newTrait.trim()] });
    setNewTrait('');
  };

  const handleRemoveTrait = (index: number) => {
    setDraft({ ...draft, traits: draft.traits.filter((_, i) => i !== index) });
  };

  const handleAddValue = () => {
    if (!newValue.trim()) return;
    setDraft({ ...draft, values: [...draft.values, newValue.trim()] });
    setNewValue('');
  };

  const handleRemoveValue = (index: number) => {
    setDraft({ ...draft, values: draft.values.filter((_, i) => i !== index) });
  };

  const handleSave = () => {
    onSave(draft);
    onClose();
  };

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(4, 6, 10, 0.75)',
        backdropFilter: 'blur(10px)',
        zIndex: 60,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '20px',
      }}
      onClick={onClose}
    >
      <div
        className="glass-panel"
        style={{
          width: '600px',
          maxWidth: '100%',
          maxHeight: '90vh',
          display: 'flex',
          flexDirection: 'column',
          background: 'rgba(12, 16, 26, 0.95)',
          overflow: 'hidden',
        }}
        onClick={(e) => e.stopPropagation()}
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
            <Sliders size={20} color="var(--accent-cyan)" />
            <h3 style={{ fontSize: '1.05rem', fontWeight: 700, color: 'var(--text-primary)' }}>
              Personality Studio & Tuning
            </h3>
          </div>
          <button onClick={onClose} className="btn-icon">
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div
          style={{
            padding: '20px 24px',
            overflowY: 'auto',
            display: 'flex',
            flexDirection: 'column',
            gap: '18px',
          }}
        >
          {/* Core Identity */}
          <div>
            <label style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', display: 'block', marginBottom: '6px' }}>
              Cốt Cách Định Danh (Core Identity)
            </label>
            <input
              type="text"
              value={draft.identity.core_identity}
              onChange={(e) =>
                setDraft({
                  ...draft,
                  identity: { ...draft.identity, core_identity: e.target.value },
                })
              }
              style={{
                width: '100%',
                padding: '10px 14px',
                borderRadius: 'var(--radius-sm)',
                background: 'rgba(255, 255, 255, 0.04)',
                border: '1px solid var(--border-subtle)',
                color: 'var(--text-primary)',
                fontFamily: 'inherit',
                fontSize: '0.88rem',
              }}
            />
          </div>

          {/* Tone */}
          <div>
            <label style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', display: 'block', marginBottom: '6px' }}>
              Giọng Điệu Giao Tiếp (Tone)
            </label>
            <input
              type="text"
              value={draft.communication_style.tone}
              onChange={(e) =>
                setDraft({
                  ...draft,
                  communication_style: { ...draft.communication_style, tone: e.target.value },
                })
              }
              style={{
                width: '100%',
                padding: '10px 14px',
                borderRadius: 'var(--radius-sm)',
                background: 'rgba(255, 255, 255, 0.04)',
                border: '1px solid var(--border-subtle)',
                color: 'var(--text-primary)',
                fontFamily: 'inherit',
                fontSize: '0.88rem',
              }}
            />
          </div>

          {/* Traits */}
          <div>
            <label style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', display: 'block', marginBottom: '6px' }}>
              Nét Tính Cách (Traits)
            </label>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '10px' }}>
              {draft.traits.map((t, idx) => (
                <span
                  key={idx}
                  className="badge badge-cyan"
                  style={{ display: 'inline-flex', alignItems: 'center', gap: '6px' }}
                >
                  {t}
                  <button
                    onClick={() => handleRemoveTrait(idx)}
                    style={{ color: 'inherit', display: 'flex', alignItems: 'center' }}
                  >
                    <X size={12} />
                  </button>
                </span>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                placeholder="Thêm trait mới (vd: Hài hước, Trầm tĩnh)..."
                value={newTrait}
                onChange={(e) => setNewTrait(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddTrait()}
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  borderRadius: 'var(--radius-sm)',
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid var(--border-subtle)',
                  color: 'var(--text-primary)',
                  fontSize: '0.82rem',
                }}
              />
              <button
                onClick={handleAddTrait}
                style={{
                  padding: '8px 14px',
                  background: 'rgba(0, 242, 254, 0.15)',
                  border: '1px solid rgba(0, 242, 254, 0.35)',
                  color: 'var(--accent-cyan)',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: '0.82rem',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                }}
              >
                <Plus size={14} /> Thêm
              </button>
            </div>
          </div>

          {/* Values */}
          <div>
            <label style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', display: 'block', marginBottom: '6px' }}>
              Hệ Giá Trị Cốt Lõi (Values)
            </label>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '10px' }}>
              {draft.values.map((v, idx) => (
                <span
                  key={idx}
                  className="badge badge-amber"
                  style={{ display: 'inline-flex', alignItems: 'center', gap: '6px' }}
                >
                  {v}
                  <button
                    onClick={() => handleRemoveValue(idx)}
                    style={{ color: 'inherit', display: 'flex', alignItems: 'center' }}
                  >
                    <X size={12} />
                  </button>
                </span>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                placeholder="Thêm giá trị mới (vd: Chân thực, Thấu hiểu)..."
                value={newValue}
                onChange={(e) => setNewValue(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddValue()}
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  borderRadius: 'var(--radius-sm)',
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid var(--border-subtle)',
                  color: 'var(--text-primary)',
                  fontSize: '0.82rem',
                }}
              />
              <button
                onClick={handleAddValue}
                style={{
                  padding: '8px 14px',
                  background: 'rgba(255, 159, 67, 0.15)',
                  border: '1px solid rgba(255, 159, 67, 0.35)',
                  color: 'var(--accent-amber)',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: '0.82rem',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                }}
              >
                <Plus size={14} /> Thêm
              </button>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'flex-end',
            gap: '12px',
            padding: '16px 24px',
            borderTop: '1px solid var(--border-subtle)',
            background: 'rgba(0, 0, 0, 0.2)',
          }}
        >
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              borderRadius: 'var(--radius-sm)',
              color: 'var(--text-secondary)',
              fontSize: '0.86rem',
            }}
          >
            Hủy bỏ
          </button>
          <button onClick={handleSave} className="btn-primary">
            <Save size={16} />
            <span>Áp Dụng Thay Đổi</span>
          </button>
        </div>
      </div>
    </div>
  );
};
