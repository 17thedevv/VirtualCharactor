import React, { useState, useRef, useEffect } from 'react';
import { Send, CornerDownLeft } from 'lucide-react';

interface InputBarProps {
  onSendMessage: (text: string) => void;
  disabled: boolean;
}

export const InputBar: React.FC<InputBarProps> = ({ onSendMessage, disabled }) => {
  const [input, setInput] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed || disabled) return;
    onSendMessage(trimmed);
    setInput('');
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    // Auto adjust height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 140)}px`;
    }
  };

  useEffect(() => {
    if (!disabled) {
      textareaRef.current?.focus();
    }
  }, [disabled]);

  const canSend = input.trim().length > 0 && !disabled;

  return (
    <div
      style={{
        padding: '16px 24px 20px',
        background: 'rgba(7, 9, 14, 0.8)',
        backdropFilter: 'blur(16px)',
        borderTop: '1px solid var(--border-subtle)',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'flex-end',
          gap: '12px',
          background: 'rgba(16, 22, 36, 0.7)',
          border: canSend ? '1px solid rgba(0, 242, 254, 0.35)' : '1px solid var(--border-subtle)',
          borderRadius: 'var(--radius-md)',
          padding: '10px 14px',
          transition: 'all var(--transition-smooth)',
          boxShadow: canSend ? '0 0 20px rgba(0, 242, 254, 0.1)' : 'none',
        }}
      >
        <textarea
          ref={textareaRef}
          rows={1}
          value={input}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={disabled ? 'Aria đang trả lời...' : 'Gửi lời nhắn hoặc chia sẻ tâm sự cùng Aria... (Enter để gửi)'}
          disabled={disabled}
          style={{
            flex: 1,
            background: 'transparent',
            border: 'none',
            outline: 'none',
            color: 'var(--text-primary)',
            fontFamily: 'var(--font-sans)',
            fontSize: '0.92rem',
            lineHeight: 1.5,
            resize: 'none',
            maxHeight: '140px',
            padding: '4px 0',
          }}
        />

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <button
            onClick={handleSend}
            disabled={!canSend}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '38px',
              height: '38px',
              borderRadius: 'var(--radius-sm)',
              background: canSend
                ? 'linear-gradient(135deg, var(--accent-cyan), var(--accent-blue))'
                : 'rgba(255, 255, 255, 0.04)',
              color: canSend ? '#040812' : 'var(--text-muted)',
              cursor: canSend ? 'pointer' : 'not-allowed',
              transition: 'all var(--transition-smooth)',
              boxShadow: canSend ? '0 0 16px rgba(0, 242, 254, 0.35)' : 'none',
            }}
          >
            <Send size={16} />
          </button>
        </div>
      </div>

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginTop: '8px',
          fontSize: '0.72rem',
          color: 'var(--text-muted)',
          padding: '0 4px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <CornerDownLeft size={11} />
          <span>Nhấn <strong>Enter</strong> để gửi, <strong>Shift+Enter</strong> để xuống dòng</span>
        </div>
        <span>{input.length} ký tự</span>
      </div>
    </div>
  );
};
