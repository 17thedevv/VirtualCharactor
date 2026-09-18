import React, { useState } from 'react';
import type { PersonalityData, TraitScores, ValueItemData } from '../../types/character';
import { X, Sliders, Save, Plus, Shield, MessageSquare, Heart, Compass } from 'lucide-react';

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
  const [newValueName, setNewValueName] = useState('');
  const [newAvoid, setNewAvoid] = useState('');
  const [newPreserve, setNewPreserve] = useState('');

  if (!isOpen) return null;

  const handleTraitChange = (traitKey: keyof TraitScores, val: number) => {
    setDraft({
      ...draft,
      traits: {
        ...draft.traits,
        [traitKey]: val,
      },
    });
  };

  const handleValueImportanceChange = (index: number, val: number) => {
    const updated = [...draft.values.items];
    updated[index] = { ...updated[index], importance: val };
    setDraft({
      ...draft,
      values: { items: updated },
    });
  };

  const handleAddValue = () => {
    if (!newValueName.trim()) return;
    const newItem: ValueItemData = {
      name: newValueName.trim(),
      importance: 0.8,
      description: 'Custom added value',
    };
    setDraft({
      ...draft,
      values: { items: [...draft.values.items, newItem] },
    });
    setNewValueName('');
  };

  const handleRemoveValue = (index: number) => {
    setDraft({
      ...draft,
      values: { items: draft.values.items.filter((_, i) => i !== index) },
    });
  };

  const handleAddAvoid = () => {
    if (!newAvoid.trim()) return;
    setDraft({
      ...draft,
      boundaries: {
        ...draft.boundaries,
        avoid: [...draft.boundaries.avoid, newAvoid.trim()],
      },
    });
    setNewAvoid('');
  };

  const handleRemoveAvoid = (index: number) => {
    setDraft({
      ...draft,
      boundaries: {
        ...draft.boundaries,
        avoid: draft.boundaries.avoid.filter((_, i) => i !== index),
      },
    });
  };

  const handleAddPreserve = () => {
    if (!newPreserve.trim()) return;
    setDraft({
      ...draft,
      boundaries: {
        ...draft.boundaries,
        preserve: [...draft.boundaries.preserve, newPreserve.trim()],
      },
    });
    setNewPreserve('');
  };

  const handleRemovePreserve = (index: number) => {
    setDraft({
      ...draft,
      boundaries: {
        ...draft.boundaries,
        preserve: draft.boundaries.preserve.filter((_, i) => i !== index),
      },
    });
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
        backdropFilter: 'blur(12px)',
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
          width: '680px',
          maxWidth: '100%',
          maxHeight: '92vh',
          display: 'flex',
          flexDirection: 'column',
          background: 'rgba(12, 16, 26, 0.96)',
          overflow: 'hidden',
          boxShadow: '0 24px 60px rgba(0, 0, 0, 0.6), 0 0 40px rgba(0, 242, 254, 0.08)',
          border: '1px solid rgba(0, 242, 254, 0.25)',
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
            background: 'rgba(255, 255, 255, 0.02)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <Sliders size={20} color="var(--accent-cyan)" />
            <h3 style={{ fontSize: '1.05rem', fontWeight: 700, color: 'var(--text-primary)' }}>
              Personality Studio & Dimensional Tuning
            </h3>
          </div>
          <button onClick={onClose} className="btn-icon">
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div
          style={{
            padding: '24px',
            overflowY: 'auto',
            display: 'flex',
            flexDirection: 'column',
            gap: '22px',
          }}
        >
          {/* Identity Section */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Compass size={15} color="var(--accent-cyan)" />
              <span style={{ fontSize: '0.82rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
                Định Danh Cốt Lõi (Identity)
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={{ fontSize: '0.76rem', color: 'var(--text-muted)', display: 'block', marginBottom: '4px' }}>
                  Tên nhân vật
                </label>
                <input
                  type="text"
                  value={draft.identity.name || ''}
                  onChange={(e) => setDraft({ ...draft, identity: { ...draft.identity, name: e.target.value } })}
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 255, 255, 0.04)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-primary)',
                    fontSize: '0.85rem',
                  }}
                />
              </div>

              <div>
                <label style={{ fontSize: '0.76rem', color: 'var(--text-muted)', display: 'block', marginBottom: '4px' }}>
                  Vai trò (Role)
                </label>
                <input
                  type="text"
                  value={draft.identity.role || ''}
                  onChange={(e) => setDraft({ ...draft, identity: { ...draft.identity, role: e.target.value } })}
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 255, 255, 0.04)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-primary)',
                    fontSize: '0.85rem',
                  }}
                />
              </div>
            </div>

            <div>
              <label style={{ fontSize: '0.76rem', color: 'var(--text-muted)', display: 'block', marginBottom: '4px' }}>
                Bản sắc cốt lõi (Core Identity)
              </label>
              <textarea
                rows={2}
                value={draft.identity.core_identity}
                onChange={(e) => setDraft({ ...draft, identity: { ...draft.identity, core_identity: e.target.value } })}
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  borderRadius: 'var(--radius-sm)',
                  background: 'rgba(255, 255, 255, 0.04)',
                  border: '1px solid var(--border-subtle)',
                  color: 'var(--text-primary)',
                  fontSize: '0.85rem',
                  resize: 'none',
                }}
              />
            </div>
          </div>

          {/* Traits Vector Sliders */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Sliders size={15} color="var(--accent-cyan)" />
                <span style={{ fontSize: '0.82rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
                  Bộ Chiều Tính Cách (Trait Vectors [0.0 – 1.0])
                </span>
              </div>
              <span style={{ fontSize: '0.72rem', color: 'var(--accent-cyan)', fontFamily: 'var(--font-mono)' }}>
                Baseline Behavior
              </span>
            </div>

            <div
              className="glass-card"
              style={{
                padding: '16px',
                display: 'flex',
                flexDirection: 'column',
                gap: '12px',
                background: 'rgba(0, 0, 0, 0.25)',
              }}
            >
              {[
                { key: 'playfulness', label: 'Hóm Hỉnh / Vui Tươi (Playfulness)', val: draft.traits.playfulness },
                { key: 'empathy', label: 'Thấu Cảm / Lắng Nghe (Empathy)', val: draft.traits.empathy },
                { key: 'curiosity', label: 'Tò Mò Tri Thức (Curiosity)', val: draft.traits.curiosity },
                { key: 'assertiveness', label: 'Quyết Đoán / Tự Chủ (Assertiveness)', val: draft.traits.assertiveness },
                { key: 'patience', label: 'Kiên Nhẫn / Điềm Tĩnh (Patience)', val: draft.traits.patience },
              ].map((trait) => (
                <div key={trait.key} style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.78rem' }}>
                    <span style={{ color: 'var(--text-primary)' }}>{trait.label}</span>
                    <span style={{ color: 'var(--accent-cyan)', fontWeight: 600, fontFamily: 'var(--font-mono)' }}>
                      {(trait.val * 100).toFixed(0)}%
                    </span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={trait.val}
                    onChange={(e) => handleTraitChange(trait.key as keyof TraitScores, parseFloat(e.target.value))}
                    style={{
                      width: '100%',
                      accentColor: 'var(--accent-cyan)',
                      cursor: 'pointer',
                    }}
                  />
                </div>
              ))}
            </div>
          </div>

          {/* Core Values */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Heart size={15} color="var(--accent-amber)" />
              <span style={{ fontSize: '0.82rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
                Hệ Giá Trị Cốt Lõi (Core Values & Importance)
              </span>
            </div>

            <div
              className="glass-card"
              style={{
                padding: '16px',
                display: 'flex',
                flexDirection: 'column',
                gap: '12px',
                background: 'rgba(0, 0, 0, 0.25)',
              }}
            >
              {draft.values.items.map((v, idx) => (
                <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <span style={{ width: '110px', fontSize: '0.8rem', fontWeight: 600, color: 'var(--accent-amber)' }}>
                    {v.name}
                  </span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={v.importance}
                    onChange={(e) => handleValueImportanceChange(idx, parseFloat(e.target.value))}
                    style={{
                      flex: 1,
                      accentColor: 'var(--accent-amber)',
                      cursor: 'pointer',
                    }}
                  />
                  <span style={{ width: '40px', fontSize: '0.76rem', color: 'var(--text-secondary)', fontFamily: 'var(--font-mono)', textAlign: 'right' }}>
                    {(v.importance * 100).toFixed(0)}%
                  </span>
                  <button onClick={() => handleRemoveValue(idx)} className="btn-icon" style={{ padding: '4px' }}>
                    <X size={14} />
                  </button>
                </div>
              ))}

              <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
                <input
                  type="text"
                  placeholder="Thêm giá trị mới (vd: authenticity)..."
                  value={newValueName}
                  onChange={(e) => setNewValueName(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddValue()}
                  style={{
                    flex: 1,
                    padding: '6px 12px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 255, 255, 0.04)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-primary)',
                    fontSize: '0.8rem',
                  }}
                />
                <button
                  onClick={handleAddValue}
                  style={{
                    padding: '6px 14px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 159, 67, 0.15)',
                    border: '1px solid rgba(255, 159, 67, 0.35)',
                    color: 'var(--accent-amber)',
                    fontSize: '0.8rem',
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

          {/* Communication Style */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <MessageSquare size={15} color="var(--accent-magenta)" />
              <span style={{ fontSize: '0.82rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
                Phong Cách Giao Tiếp (Communication Style)
              </span>
            </div>

            <div>
              <label style={{ fontSize: '0.76rem', color: 'var(--text-muted)', display: 'block', marginBottom: '4px' }}>
                Giọng điệu (Tone)
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
                  padding: '8px 12px',
                  borderRadius: 'var(--radius-sm)',
                  background: 'rgba(255, 255, 255, 0.04)',
                  border: '1px solid var(--border-subtle)',
                  color: 'var(--text-primary)',
                  fontSize: '0.85rem',
                }}
              />
            </div>
          </div>

          {/* Boundaries */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Shield size={15} color="var(--accent-cyan)" />
              <span style={{ fontSize: '0.82rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
                Ranh Giới Hành Vi (Boundaries)
              </span>
            </div>

            {/* Avoid */}
            <div>
              <label style={{ fontSize: '0.76rem', color: '#ff6b6b', display: 'block', marginBottom: '6px' }}>
                Hành vi kiêng kỵ (Avoid)
              </label>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', marginBottom: '8px' }}>
                {draft.boundaries.avoid.map((item, idx) => (
                  <span
                    key={idx}
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '3px 10px',
                      borderRadius: 'var(--radius-full)',
                      background: 'rgba(255, 107, 107, 0.12)',
                      border: '1px solid rgba(255, 107, 107, 0.3)',
                      color: '#ff8787',
                      fontSize: '0.76rem',
                    }}
                  >
                    {item}
                    <button onClick={() => handleRemoveAvoid(idx)} style={{ color: 'inherit', display: 'flex' }}>
                      <X size={12} />
                    </button>
                  </span>
                ))}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="text"
                  placeholder="Thêm điều kiêng kỵ..."
                  value={newAvoid}
                  onChange={(e) => setNewAvoid(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddAvoid()}
                  style={{
                    flex: 1,
                    padding: '6px 12px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 255, 255, 0.04)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-primary)',
                    fontSize: '0.8rem',
                  }}
                />
                <button onClick={handleAddAvoid} className="btn-secondary" style={{ padding: '6px 12px', fontSize: '0.8rem' }}>
                  Thêm
                </button>
              </div>
            </div>

            {/* Preserve */}
            <div>
              <label style={{ fontSize: '0.76rem', color: 'var(--accent-cyan)', display: 'block', marginBottom: '6px' }}>
                Nguyên tắc gìn giữ (Preserve)
              </label>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', marginBottom: '8px' }}>
                {draft.boundaries.preserve.map((item, idx) => (
                  <span
                    key={idx}
                    className="badge badge-cyan"
                    style={{ display: 'inline-flex', alignItems: 'center', gap: '6px', fontSize: '0.76rem' }}
                  >
                    {item}
                    <button onClick={() => handleRemovePreserve(idx)} style={{ color: 'inherit', display: 'flex' }}>
                      <X size={12} />
                    </button>
                  </span>
                ))}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="text"
                  placeholder="Thêm nguyên tắc gìn giữ..."
                  value={newPreserve}
                  onChange={(e) => setNewPreserve(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddPreserve()}
                  style={{
                    flex: 1,
                    padding: '6px 12px',
                    borderRadius: 'var(--radius-sm)',
                    background: 'rgba(255, 255, 255, 0.04)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-primary)',
                    fontSize: '0.8rem',
                  }}
                />
                <button onClick={handleAddPreserve} className="btn-secondary" style={{ padding: '6px 12px', fontSize: '0.8rem' }}>
                  Thêm
                </button>
              </div>
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
            background: 'rgba(0, 0, 0, 0.25)',
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
            <span>Áp Dụng Bản Sắc Mới</span>
          </button>
        </div>
      </div>
    </div>
  );
};
