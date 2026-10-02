import React, { useState, useMemo } from 'react';
import {
  X,
  History,
  Search,
  Globe,
  GraduationCap,
  MessageSquare,
  Brain,
  Calendar,
  Clock,
  Trash2,
  Download,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  Sparkles,
  ArrowRight,
} from 'lucide-react';
import type {
  PersistentSearchRecord,
  MemoryItem,
  ChatMessage,
  WebSearchRecord,
} from '../../types/character';

interface HistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  searches: (PersistentSearchRecord | WebSearchRecord)[];
  memories: MemoryItem[];
  messages: ChatMessage[];
  onSelectTopic?: (topic: string) => void;
  onClearHistory?: () => Promise<void>;
}

type TabType = 'all' | 'searches' | 'knowledge' | 'dialogues';

interface UnifiedTimelineItem {
  id: string;
  type: 'search' | 'knowledge' | 'dialogue';
  title: string;
  subtitle?: string;
  details?: string[];
  timestamp: number;
  source?: string;
  extraBadge?: string;
  extraData?: any;
}

export const HistoryModal: React.FC<HistoryModalProps> = ({
  isOpen,
  onClose,
  searches,
  memories,
  messages,
  onSelectTopic,
  onClearHistory,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedItems, setExpandedItems] = useState<Record<string, boolean>>({});
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [isClearing, setIsClearing] = useState(false);

  // Toggle item expansion
  const toggleExpand = (id: string) => {
    setExpandedItems((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  // Copy helper
  const handleCopy = (id: string, text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1800);
  };

  // Export JSON helper
  const handleExportJSON = () => {
    const exportData = {
      exported_at: new Date().toISOString(),
      searches,
      memories,
      messages: messages.map((m) => ({
        id: m.id,
        sender: m.sender,
        text: m.text,
        timestamp: m.timestamp,
      })),
    };
    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `aria-history-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Build unified timeline
  const timelineItems: UnifiedTimelineItem[] = useMemo(() => {
    const list: UnifiedTimelineItem[] = [];

    // 1. Searches
    searches.forEach((s, idx) => {
      const ts = 'created_at' in s ? (s.created_at < 1e11 ? s.created_at * 1000 : s.created_at) : s.timestamp;
      list.push({
        id: `search-${idx}-${ts}`,
        type: 'search',
        title: s.query,
        subtitle: `Tra cứu tự động qua ${s.source} với ${s.snippets?.length || 0} kết quả trích xuất`,
        details: s.snippets || [],
        timestamp: ts,
        source: s.source,
        extraBadge: 'Web Search',
        extraData: s,
      });
    });

    // 2. Knowledge & Learned Facts (Semantic memories & Critical facts)
    memories.forEach((m, idx) => {
      // Estimate timestamp based on order if not present
      const ts = Date.now() - (memories.length - idx) * 60000;
      const isCritical = m.importance === 'Critical' || m.importance === 'High';
      const isSemantic = m.type === 'Semantic';
      list.push({
        id: `mem-${m.id || idx}`,
        type: 'knowledge',
        title: m.content,
        subtitle: `Bộ nhớ ${m.type} • Mức ưu tiên: ${m.importance}`,
        details: [
          `ID: ${m.id}`,
          `Loại hình: ${m.type} Memory`,
          `Mức độ quan trọng: ${m.importance}`,
          'Đã nạp vào bộ lưu trữ SQLite và đồng bộ vào Context của Aria',
        ],
        timestamp: ts,
        source: isSemantic ? 'User Teaching' : 'Core Memory',
        extraBadge: isCritical ? 'Kiến Thức Cốt Lõi' : 'Trí Nhớ Đã Học',
        extraData: m,
      });
    });

    // 3. User Questions & Dialogues
    for (let i = 0; i < messages.length; i++) {
      const msg = messages[i];
      if (msg.sender === 'user') {
        const nextMsg = messages[i + 1];
        const ariaReply = nextMsg && nextMsg.sender === 'character' ? nextMsg.text : undefined;
        list.push({
          id: `dlg-${msg.id}`,
          type: 'dialogue',
          title: msg.text,
          subtitle: ariaReply ? `Aria: "${ariaReply.slice(0, 100)}${ariaReply.length > 100 ? '...' : ''}"` : 'Câu hỏi từ người dùng',
          details: ariaReply ? [`Hỏi: ${msg.text}`, `Aria đáp: ${ariaReply}`] : [msg.text],
          timestamp: msg.timestamp,
          source: 'Hội Thoại',
          extraBadge: 'Hỏi & Đáp',
          extraData: { user: msg.text, reply: ariaReply },
        });
      }
    }

    // Sort descending by timestamp
    return list.sort((a, b) => b.timestamp - a.timestamp);
  }, [searches, memories, messages]);

  // Filter items by tab and search query
  const filteredItems = useMemo(() => {
    return timelineItems.filter((item) => {
      // Tab filter
      if (activeTab === 'searches' && item.type !== 'search') return false;
      if (activeTab === 'knowledge' && item.type !== 'knowledge') return false;
      if (activeTab === 'dialogues' && item.type !== 'dialogue') return false;

      // Query filter
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase().trim();
      const matchTitle = item.title.toLowerCase().includes(q);
      const matchSub = item.subtitle?.toLowerCase().includes(q);
      const matchDetails = item.details?.some((d) => d.toLowerCase().includes(q));
      const matchSource = item.source?.toLowerCase().includes(q);
      return matchTitle || matchSub || matchDetails || matchSource;
    });
  }, [timelineItems, activeTab, searchQuery]);

  if (!isOpen) return null;

  // Format date helper
  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    const today = new Date();
    if (d.toDateString() === today.toDateString()) {
      return 'Hôm nay';
    }
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    if (d.toDateString() === yesterday.toDateString()) {
      return 'Hôm qua';
    }
    return d.toLocaleDateString('vi-VN', {
      weekday: 'long',
      day: 'numeric',
      month: 'long',
      year: 'numeric',
    });
  };

  // Group items by Date
  const groupedItems = filteredItems.reduce<Record<string, UnifiedTimelineItem[]>>((acc, item) => {
    const dateKey = formatDate(item.timestamp);
    if (!acc[dateKey]) acc[dateKey] = [];
    acc[dateKey].push(item);
    return acc;
  }, {});

  const totalSearches = searches.length;
  const totalLearned = memories.filter((m) => m.type === 'Semantic' || m.importance === 'Critical').length;
  const totalDialogues = messages.filter((m) => m.sender === 'user').length;
  const totalMemories = memories.length;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(3, 5, 9, 0.85)',
        backdropFilter: 'blur(16px)',
        zIndex: 70,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '24px',
        animation: 'fadeIn 0.2s ease-out',
      }}
      onClick={onClose}
    >
      <div
        className="glass-panel"
        style={{
          width: '940px',
          maxWidth: '100%',
          height: '88vh',
          maxHeight: '900px',
          display: 'flex',
          flexDirection: 'column',
          background: 'rgba(10, 14, 23, 0.96)',
          borderRadius: 'var(--radius-lg)',
          overflow: 'hidden',
          boxShadow: '0 24px 60px rgba(0, 0, 0, 0.7), 0 0 50px rgba(0, 242, 254, 0.12)',
          border: '1px solid rgba(0, 242, 254, 0.28)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Top Header - Chrome History Style */}
        <div
          style={{
            padding: '18px 24px',
            borderBottom: '1px solid var(--border-subtle)',
            background: 'rgba(255, 255, 255, 0.02)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '16px',
          }}
        >
          {/* Brand & Chrome-like URL */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <div
              style={{
                width: '38px',
                height: '38px',
                borderRadius: 'var(--radius-sm)',
                background: 'linear-gradient(135deg, var(--accent-cyan), var(--accent-blue))',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 0 16px rgba(0, 242, 254, 0.4)',
              }}
            >
              <History size={20} color="#040812" />
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <h2
                  style={{
                    fontSize: '1.15rem',
                    fontWeight: 700,
                    color: 'var(--text-primary)',
                    margin: 0,
                    letterSpacing: '-0.01em',
                  }}
                >
                  Nhật Ký & Lịch Sử Hoạt Động
                </h2>
                <span
                  style={{
                    padding: '2px 8px',
                    borderRadius: 'var(--radius-full)',
                    background: 'rgba(0, 242, 254, 0.12)',
                    border: '1px solid rgba(0, 242, 254, 0.3)',
                    color: 'var(--accent-cyan)',
                    fontSize: '0.68rem',
                    fontFamily: 'var(--font-mono)',
                  }}
                >
                  chrome://history-aria
                </span>
              </div>
              <p style={{ fontSize: '0.78rem', color: 'var(--text-secondary)', margin: '2px 0 0' }}>
                Khám phá những gì Aria đã tra cứu trên web, học hỏi từ bạn và trả lời trong hội thoại
              </p>
            </div>
          </div>

          {/* Quick Action buttons */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <button
              onClick={handleExportJSON}
              className="btn-icon"
              title="Xuất file JSON toàn bộ lịch sử"
              style={{
                width: 'auto',
                padding: '0 12px',
                gap: '6px',
                fontSize: '0.78rem',
              }}
            >
              <Download size={15} />
              <span>Xuất JSON</span>
            </button>

            {onClearHistory && (
              <button
                onClick={async () => {
                  if (window.confirm('Bạn có chắc muốn xóa toàn bộ lịch sử tra cứu và hội thoại?')) {
                    setIsClearing(true);
                    try {
                      await onClearHistory();
                    } finally {
                      setIsClearing(false);
                    }
                  }
                }}
                disabled={isClearing}
                className="btn-icon"
                title="Xóa toàn bộ lịch sử"
                style={{
                  width: 'auto',
                  padding: '0 12px',
                  gap: '6px',
                  fontSize: '0.78rem',
                  color: 'var(--accent-rose)',
                  borderColor: 'rgba(255, 65, 108, 0.3)',
                }}
              >
                <Trash2 size={15} />
                <span>{isClearing ? 'Đang xóa...' : 'Xóa Lịch Sử'}</span>
              </button>
            )}

            <button
              onClick={onClose}
              className="btn-icon"
              title="Đóng (Esc)"
              style={{ width: '36px', height: '36px' }}
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* Chrome-Style Search Bar */}
        <div
          style={{
            padding: '14px 24px',
            background: 'rgba(16, 22, 36, 0.65)',
            borderBottom: '1px solid var(--border-subtle)',
            display: 'flex',
            alignItems: 'center',
            gap: '14px',
          }}
        >
          <div
            style={{
              flex: 1,
              position: 'relative',
              display: 'flex',
              alignItems: 'center',
            }}
          >
            <Search
              size={18}
              color="var(--accent-cyan)"
              style={{ position: 'absolute', left: '14px', pointerEvents: 'none' }}
            />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Tìm kiếm trong lịch sử (tên ca sĩ, bài hát, khái niệm, câu hỏi, kiến thức đã học...)"
              autoFocus
              style={{
                width: '100%',
                padding: '10px 40px 10px 42px',
                borderRadius: 'var(--radius-sm)',
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(0, 242, 254, 0.25)',
                color: 'var(--text-primary)',
                fontSize: '0.88rem',
                outline: 'none',
                boxShadow: 'inset 0 2px 4px rgba(0, 0, 0, 0.2)',
              }}
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                style={{
                  position: 'absolute',
                  right: '12px',
                  background: 'none',
                  border: 'none',
                  color: 'var(--text-muted)',
                  cursor: 'pointer',
                  padding: '4px',
                }}
              >
                <X size={16} />
              </button>
            )}
          </div>

          <div
            style={{
              fontSize: '0.78rem',
              color: 'var(--text-secondary)',
              whiteSpace: 'nowrap',
              fontFamily: 'var(--font-mono)',
            }}
          >
            {filteredItems.length} kết quả
          </div>
        </div>

        {/* Quick Stats Metrics Row */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(4, 1fr)',
            gap: '12px',
            padding: '14px 24px',
            background: 'rgba(7, 10, 16, 0.5)',
            borderBottom: '1px solid var(--border-subtle)',
          }}
        >
          {/* Stat 1: Web Searches */}
          <div
            onClick={() => setActiveTab('searches')}
            style={{
              padding: '10px 14px',
              borderRadius: 'var(--radius-sm)',
              background: activeTab === 'searches' ? 'rgba(0, 242, 254, 0.12)' : 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${activeTab === 'searches' ? 'rgba(0, 242, 254, 0.4)' : 'var(--border-subtle)'}`,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              transition: 'var(--transition-fast)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <Globe size={18} color="var(--accent-cyan)" />
              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Tra Cứu Web
                </div>
                <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                  {totalSearches}
                </div>
              </div>
            </div>
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>DuckDuckGo</span>
          </div>

          {/* Stat 2: Learned Knowledge */}
          <div
            onClick={() => setActiveTab('knowledge')}
            style={{
              padding: '10px 14px',
              borderRadius: 'var(--radius-sm)',
              background: activeTab === 'knowledge' ? 'rgba(5, 214, 158, 0.12)' : 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${activeTab === 'knowledge' ? 'rgba(5, 214, 158, 0.4)' : 'var(--border-subtle)'}`,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              transition: 'var(--transition-fast)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <GraduationCap size={18} color="var(--accent-emerald)" />
              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Đã Học Hỏi
                </div>
                <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                  {totalLearned}
                </div>
              </div>
            </div>
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>Sự thật mới</span>
          </div>

          {/* Stat 3: Dialogue Turns */}
          <div
            onClick={() => setActiveTab('dialogues')}
            style={{
              padding: '10px 14px',
              borderRadius: 'var(--radius-sm)',
              background: activeTab === 'dialogues' ? 'rgba(138, 43, 226, 0.12)' : 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${activeTab === 'dialogues' ? 'rgba(138, 43, 226, 0.4)' : 'var(--border-subtle)'}`,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              transition: 'var(--transition-fast)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <MessageSquare size={18} color="var(--accent-violet)" />
              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Hỏi & Đáp
                </div>
                <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                  {totalDialogues}
                </div>
              </div>
            </div>
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>Tương tác</span>
          </div>

          {/* Stat 4: SQLite Long-term Memories */}
          <div
            onClick={() => setActiveTab('all')}
            style={{
              padding: '10px 14px',
              borderRadius: 'var(--radius-sm)',
              background: activeTab === 'all' ? 'rgba(255, 159, 67, 0.12)' : 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${activeTab === 'all' ? 'rgba(255, 159, 67, 0.4)' : 'var(--border-subtle)'}`,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              transition: 'var(--transition-fast)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <Brain size={18} color="var(--accent-amber)" />
              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Tổng Ký Ức
                </div>
                <div style={{ fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                  {totalMemories}
                </div>
              </div>
            </div>
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>SQLite Local</span>
          </div>
        </div>

        {/* Tab Navigation Navigation Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '10px 24px',
            background: 'rgba(12, 16, 26, 0.75)',
            borderBottom: '1px solid var(--border-subtle)',
          }}
        >
          <button
            onClick={() => setActiveTab('all')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 14px',
              borderRadius: 'var(--radius-full)',
              background: activeTab === 'all' ? 'var(--accent-cyan)' : 'transparent',
              color: activeTab === 'all' ? '#040812' : 'var(--text-secondary)',
              fontWeight: 600,
              fontSize: '0.8rem',
              border: 'none',
              cursor: 'pointer',
              transition: 'var(--transition-fast)',
            }}
          >
            <Sparkles size={14} />
            <span>Tất Cả ({timelineItems.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('searches')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 14px',
              borderRadius: 'var(--radius-full)',
              background: activeTab === 'searches' ? 'var(--accent-cyan)' : 'transparent',
              color: activeTab === 'searches' ? '#040812' : 'var(--text-secondary)',
              fontWeight: 600,
              fontSize: '0.8rem',
              border: 'none',
              cursor: 'pointer',
              transition: 'var(--transition-fast)',
            }}
          >
            <Globe size={14} />
            <span>Tra Cứu Web ({totalSearches})</span>
          </button>

          <button
            onClick={() => setActiveTab('knowledge')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 14px',
              borderRadius: 'var(--radius-full)',
              background: activeTab === 'knowledge' ? 'var(--accent-cyan)' : 'transparent',
              color: activeTab === 'knowledge' ? '#040812' : 'var(--text-secondary)',
              fontWeight: 600,
              fontSize: '0.8rem',
              border: 'none',
              cursor: 'pointer',
              transition: 'var(--transition-fast)',
            }}
          >
            <GraduationCap size={14} />
            <span>Kiến Thức Đã Học ({totalMemories})</span>
          </button>

          <button
            onClick={() => setActiveTab('dialogues')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 14px',
              borderRadius: 'var(--radius-full)',
              background: activeTab === 'dialogues' ? 'var(--accent-cyan)' : 'transparent',
              color: activeTab === 'dialogues' ? '#040812' : 'var(--text-secondary)',
              fontWeight: 600,
              fontSize: '0.8rem',
              border: 'none',
              cursor: 'pointer',
              transition: 'var(--transition-fast)',
            }}
          >
            <MessageSquare size={14} />
            <span>Hỏi & Đáp ({totalDialogues})</span>
          </button>
        </div>

        {/* Main Content Timeline Feed */}
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
          {Object.keys(groupedItems).length === 0 ? (
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                height: '280px',
                color: 'var(--text-muted)',
                gap: '12px',
              }}
            >
              <Search size={40} strokeWidth={1.5} color="var(--text-faint)" />
              <div style={{ fontSize: '0.95rem', fontWeight: 600, color: 'var(--text-secondary)' }}>
                {searchQuery ? `Không tìm thấy kết quả nào cho "${searchQuery}"` : 'Chưa có hoạt động nào trong danh mục này'}
              </div>
              <p style={{ fontSize: '0.8rem', maxWidth: '420px', textAlign: 'center' }}>
                {searchQuery
                  ? 'Hãy thử kiểm tra lại chính tả hoặc tìm bằng từ khóa ngắn gọn hơn (ví dụ: "MCK", "thời tiết", "học").'
                  : 'Hãy trò chuyện, đặt câu hỏi hoặc dạy cho Aria điều gì đó mới để xem dòng thời gian hoạt động ở đây!'}
              </p>
            </div>
          ) : (
            Object.entries(groupedItems).map(([dateLabel, items]) => (
              <div key={dateLabel} style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                {/* Date Header */}
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    fontSize: '0.78rem',
                    fontWeight: 700,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                    color: 'var(--text-muted)',
                    paddingBottom: '4px',
                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                  }}
                >
                  <Calendar size={13} color="var(--accent-cyan)" />
                  <span>{dateLabel}</span>
                  <span style={{ fontSize: '0.7rem', color: 'var(--text-faint)' }}>({items.length} mục)</span>
                </div>

                {/* Items List */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {items.map((item) => {
                    const isExpanded = !!expandedItems[item.id];
                    const isSearch = item.type === 'search';
                    const isKnowledge = item.type === 'knowledge';
                    const isDialogue = item.type === 'dialogue';

                    const badgeColor = isSearch
                      ? 'var(--accent-cyan)'
                      : isKnowledge
                      ? 'var(--accent-emerald)'
                      : 'var(--accent-violet)';

                    const badgeBg = isSearch
                      ? 'rgba(0, 242, 254, 0.12)'
                      : isKnowledge
                      ? 'rgba(5, 214, 158, 0.12)'
                      : 'rgba(138, 43, 226, 0.12)';

                    return (
                      <div
                        key={item.id}
                        style={{
                          borderRadius: 'var(--radius-sm)',
                          background: isExpanded ? 'rgba(255, 255, 255, 0.05)' : 'rgba(255, 255, 255, 0.02)',
                          border: `1px solid ${isExpanded ? 'rgba(0, 242, 254, 0.3)' : 'var(--border-subtle)'}`,
                          padding: '12px 16px',
                          display: 'flex',
                          flexDirection: 'column',
                          gap: '8px',
                          transition: 'var(--transition-fast)',
                        }}
                      >
                        {/* Row Summary */}
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            gap: '12px',
                          }}
                        >
                          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flex: 1, minWidth: 0 }}>
                            {/* Time */}
                            <div
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '4px',
                                fontSize: '0.74rem',
                                color: 'var(--text-muted)',
                                fontFamily: 'var(--font-mono)',
                                whiteSpace: 'nowrap',
                              }}
                            >
                              <Clock size={12} />
                              <span>{formatTime(item.timestamp)}</span>
                            </div>

                            {/* Badge */}
                            <div
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '5px',
                                padding: '3px 8px',
                                borderRadius: 'var(--radius-full)',
                                background: badgeBg,
                                color: badgeColor,
                                fontSize: '0.7rem',
                                fontWeight: 700,
                                whiteSpace: 'nowrap',
                              }}
                            >
                              {isSearch && <Globe size={11} />}
                              {isKnowledge && <GraduationCap size={11} />}
                              {isDialogue && <MessageSquare size={11} />}
                              <span>{item.extraBadge}</span>
                            </div>

                            {/* Title & Subtitle */}
                            <div style={{ flex: 1, minWidth: 0 }}>
                              <div
                                style={{
                                  fontSize: '0.88rem',
                                  fontWeight: 600,
                                  color: 'var(--text-primary)',
                                  whiteSpace: 'nowrap',
                                  overflow: 'hidden',
                                  textOverflow: 'ellipsis',
                                }}
                              >
                                {item.title}
                              </div>
                              {item.subtitle && !isExpanded && (
                                <div
                                  style={{
                                    fontSize: '0.74rem',
                                    color: 'var(--text-secondary)',
                                    whiteSpace: 'nowrap',
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    marginTop: '1px',
                                  }}
                                >
                                  {item.subtitle}
                                </div>
                              )}
                            </div>
                          </div>

                          {/* Quick Actions */}
                          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                            {/* Ask again button */}
                            {onSelectTopic && (
                              <button
                                onClick={() => {
                                  onSelectTopic(item.title);
                                  onClose();
                                }}
                                className="btn-icon"
                                title="Trò chuyện lại về chủ đề này"
                                style={{ width: '28px', height: '28px' }}
                              >
                                <ArrowRight size={13} />
                              </button>
                            )}

                            {/* Copy button */}
                            <button
                              onClick={() => handleCopy(item.id, item.title)}
                              className="btn-icon"
                              title="Sao chép nội dung"
                              style={{ width: '28px', height: '28px' }}
                            >
                              {copiedId === item.id ? <Check size={13} color="var(--accent-emerald)" /> : <Copy size={13} />}
                            </button>

                            {/* Expand toggle */}
                            {(item.details && item.details.length > 0) && (
                              <button
                                onClick={() => toggleExpand(item.id)}
                                className="btn-icon"
                                title={isExpanded ? 'Thu gọn' : 'Xem chi tiết'}
                                style={{ width: '28px', height: '28px' }}
                              >
                                {isExpanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                              </button>
                            )}
                          </div>
                        </div>

                        {/* Collapsible Details */}
                        {isExpanded && item.details && (
                          <div
                            style={{
                              marginTop: '4px',
                              padding: '12px 14px',
                              borderRadius: 'var(--radius-xs)',
                              background: 'rgba(0, 0, 0, 0.35)',
                              border: '1px solid rgba(255, 255, 255, 0.06)',
                              fontSize: '0.8rem',
                              color: 'var(--text-secondary)',
                              display: 'flex',
                              flexDirection: 'column',
                              gap: '6px',
                              animation: 'fadeIn 0.15s ease-out',
                            }}
                          >
                            {isSearch && (
                              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '4px' }}>
                                <span style={{ color: 'var(--text-muted)', fontSize: '0.72rem' }}>Nguồn dữ liệu:</span>
                                <span className="badge badge-cyan" style={{ fontSize: '0.68rem', padding: '1px 6px' }}>
                                  {item.source || 'DuckDuckGo Lite'}
                                </span>
                              </div>
                            )}

                            {item.details.map((snippet, sIdx) => (
                              <div
                                key={sIdx}
                                style={{
                                  display: 'flex',
                                  alignItems: 'flex-start',
                                  gap: '8px',
                                  lineHeight: 1.45,
                                }}
                              >
                                <span style={{ color: badgeColor, marginTop: '2px' }}>•</span>
                                <span style={{ flex: 1 }}>{snippet}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Footer info */}
        <div
          style={{
            padding: '12px 24px',
            borderTop: '1px solid var(--border-subtle)',
            background: 'rgba(7, 9, 14, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            fontSize: '0.75rem',
            color: 'var(--text-muted)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span>Phím tắt:</span>
            <kbd style={{ padding: '2px 6px', background: 'rgba(255,255,255,0.08)', borderRadius: '4px' }}>
              Ctrl + H
            </kbd>
            <span>để mở/đóng lịch sử</span>
            <span>•</span>
            <kbd style={{ padding: '2px 6px', background: 'rgba(255,255,255,0.08)', borderRadius: '4px' }}>
              Esc
            </kbd>
            <span>để thoát</span>
          </div>

          <div style={{ color: 'var(--text-secondary)' }}>
            Dữ liệu được lưu trữ cục bộ 100% trong SQLite (<code style={{ color: 'var(--accent-cyan)' }}>virtual_character.db</code>)
          </div>
        </div>
      </div>
    </div>
  );
};
