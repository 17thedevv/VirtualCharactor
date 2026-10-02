import React, { useEffect, useRef } from 'react';
import type { ChatMessage } from '../../types/character';
import { Bot, User, Sparkles, Globe, Search, Volume2 } from 'lucide-react';

interface DialogueStreamProps {
  messages: ChatMessage[];
  characterName: string;
  onSelectTopic: (topic: string) => void;
  onPlayMessage?: (text: string) => void;
  currentSearch?: { query: string; message: string } | null;
}

export const DialogueStream: React.FC<DialogueStreamProps> = ({
  messages,
  characterName,
  onSelectTopic,
  onPlayMessage,
  currentSearch,
}) => {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const starterTopics = [
    'Chào Aria, hôm nay bạn cảm thấy thế nào?',
    'Bạn là ai và cơ chế suy nghĩ của bạn hoạt động ra sao?',
    'Mình vừa hoàn thành một kiến trúc phần mềm rất thú vị!',
    'Hôm nay công việc hơi áp lực, mình thấy hơi mệt...',
  ];

  // Helper to format text with *italic thoughts*
  const renderFormattedText = (text: string) => {
    const parts = text.split(/(\*[^*]+\*)/g);
    return parts.map((part, index) => {
      if (part.startsWith('*') && part.endsWith('*')) {
        return (
          <em
            key={index}
            style={{
              color: 'var(--accent-cyan)',
              opacity: 0.9,
              fontStyle: 'italic',
              margin: '0 2px',
            }}
          >
            {part.slice(1, -1)}
          </em>
        );
      }
      return <span key={index}>{part}</span>;
    });
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        overflowY: 'auto',
        padding: '24px 28px',
        gap: '20px',
      }}
    >
      {/* Welcome header if few messages */}
      {messages.length === 0 && (
        <div
          className="animate-fade-in"
          style={{
            margin: 'auto',
            textAlign: 'center',
            maxWidth: '520px',
            padding: '30px 20px',
          }}
        >
          <div
            style={{
              width: '48px',
              height: '48px',
              borderRadius: 'var(--radius-md)',
              background: 'rgba(0, 242, 254, 0.12)',
              border: '1px solid rgba(0, 242, 254, 0.25)',
              color: 'var(--accent-cyan)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto 16px',
            }}
          >
            <Sparkles size={24} />
          </div>
          <h3
            style={{
              fontFamily: 'var(--font-display)',
              fontSize: '1.25rem',
              fontWeight: 700,
              color: 'var(--text-primary)',
              marginBottom: '8px',
            }}
          >
            Bắt đầu tương tác với {characterName}
          </h3>
          <p
            style={{
              fontSize: '0.88rem',
              color: 'var(--text-secondary)',
              lineHeight: 1.6,
              marginBottom: '24px',
            }}
          >
            Mỗi cuộc trò chuyện sẽ tác động đến cảm xúc, tạo lập ký ức và bồi đắp mức độ tin cậy của nhân vật theo thời gian.
          </p>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Gợi ý mở đầu
            </span>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', justifyContent: 'center' }}>
              {starterTopics.map((topic, i) => (
                <button
                  key={i}
                  onClick={() => onSelectTopic(topic)}
                  style={{
                    padding: '8px 14px',
                    fontSize: '0.82rem',
                    borderRadius: 'var(--radius-full)',
                    background: 'rgba(255, 255, 255, 0.035)',
                    border: '1px solid var(--border-subtle)',
                    color: 'var(--text-secondary)',
                    transition: 'all var(--transition-fast)',
                    textAlign: 'left',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.borderColor = 'var(--accent-cyan)';
                    e.currentTarget.style.color = 'var(--text-primary)';
                    e.currentTarget.style.background = 'rgba(0, 242, 254, 0.08)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.borderColor = 'var(--border-subtle)';
                    e.currentTarget.style.color = 'var(--text-secondary)';
                    e.currentTarget.style.background = 'rgba(255, 255, 255, 0.035)';
                  }}
                >
                  {topic}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Message List */}
      {messages.map((msg) => {
        const isUser = msg.sender === 'user';
        return (
          <div
            key={msg.id}
            className="animate-fade-in"
            style={{
              display: 'flex',
              flexDirection: isUser ? 'row-reverse' : 'row',
              gap: '14px',
              maxWidth: '82%',
              alignSelf: isUser ? 'flex-end' : 'flex-start',
            }}
          >
            {/* Avatar icon */}
            <div
              style={{
                width: '34px',
                height: '34px',
                borderRadius: 'var(--radius-sm)',
                flexShrink: 0,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                background: isUser
                  ? 'rgba(255, 255, 255, 0.08)'
                  : 'linear-gradient(135deg, rgba(0, 242, 254, 0.2), rgba(79, 172, 254, 0.2))',
                border: isUser
                  ? '1px solid var(--border-subtle)'
                  : '1px solid rgba(0, 242, 254, 0.4)',
                color: isUser ? 'var(--text-secondary)' : 'var(--accent-cyan)',
              }}
            >
              {isUser ? <User size={16} /> : <Bot size={18} />}
            </div>

            {/* Bubble Content */}
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: '4px',
                alignItems: isUser ? 'flex-end' : 'flex-start',
              }}
            >
              <div
                style={{
                  fontSize: '0.75rem',
                  color: 'var(--text-muted)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                }}
              >
                <span>{isUser ? 'Bạn' : characterName}</span>
                <span>
                  {new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </span>
                {!isUser && onPlayMessage && !msg.isStreaming && (
                  <button
                    onClick={() => onPlayMessage(msg.text)}
                    title="Nghe giọng nói Aria đọc tin nhắn này"
                    style={{
                      background: 'rgba(0, 242, 254, 0.08)',
                      border: '1px solid rgba(0, 242, 254, 0.25)',
                      color: 'var(--accent-cyan)',
                      cursor: 'pointer',
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '4px',
                      padding: '2px 8px',
                      borderRadius: '12px',
                      fontSize: '0.72rem',
                      fontWeight: 500,
                      transition: 'all 0.2s ease',
                      marginLeft: '4px',
                    }}
                  >
                    <Volume2 size={12} />
                    <span>Nghe</span>
                  </button>
                )}
              </div>

              <div
                style={{
                  padding: '12px 18px',
                  borderRadius: isUser ? '18px 4px 18px 18px' : '4px 18px 18px 18px',
                  background: isUser
                    ? 'linear-gradient(135deg, #1e293b, #0f172a)'
                    : 'rgba(16, 22, 36, 0.85)',
                  border: isUser
                    ? '1px solid rgba(255, 255, 255, 0.12)'
                    : '1px solid rgba(0, 242, 254, 0.18)',
                  color: 'var(--text-primary)',
                  fontSize: '0.92rem',
                  lineHeight: 1.62,
                  boxShadow: '0 4px 20px rgba(0,0,0,0.35)',
                  backdropFilter: 'blur(10px)',
                  wordBreak: 'break-word',
                }}
              >
                {!isUser && msg.searchRecord && (
                  <div
                    style={{
                      marginBottom: '10px',
                      padding: '8px 12px',
                      borderRadius: '6px',
                      background: 'rgba(0, 242, 254, 0.08)',
                      border: '1px solid rgba(0, 242, 254, 0.25)',
                      fontSize: '0.78rem',
                      color: 'var(--accent-cyan)',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 600 }}>
                      <Globe size={13} />
                      <span>Đã tra cứu web: "{msg.searchRecord.query}" ({msg.searchRecord.source})</span>
                    </div>
                    {msg.searchRecord.snippets && msg.searchRecord.snippets.length > 0 && (
                      <div
                        style={{
                          marginTop: '4px',
                          color: 'var(--text-secondary)',
                          fontSize: '0.74rem',
                          fontStyle: 'italic',
                          borderLeft: '2px solid rgba(0, 242, 254, 0.4)',
                          paddingLeft: '8px',
                          lineHeight: 1.4,
                        }}
                      >
                        "{msg.searchRecord.snippets[0]}"
                      </div>
                    )}
                  </div>
                )}
                {renderFormattedText(msg.text)}
                {msg.isStreaming && <span className="cursor-caret" />}
              </div>
            </div>
          </div>
        );
      })}

      {currentSearch && (
        <div
          className="animate-pulse"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '8px 16px',
            borderRadius: '20px',
            background: 'rgba(0, 242, 254, 0.1)',
            border: '1px solid rgba(0, 242, 254, 0.35)',
            color: 'var(--accent-cyan)',
            fontSize: '0.82rem',
            width: 'fit-content',
            alignSelf: 'flex-start',
            marginLeft: '48px',
          }}
        >
          <Search size={14} className="animate-spin" />
          <span>{currentSearch.message || `Đang tra cứu về "${currentSearch.query}" trên web...`}</span>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
};
