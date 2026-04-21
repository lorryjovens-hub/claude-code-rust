import React, { useState, useEffect, useRef } from 'react';
import { Search, X, MessageSquare } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface Chat {
  id: string;
  title: string;
  updated_at: string;
  created_at: string;
}

interface SearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  chats: Chat[];
}

const getTimeLabel = (dateStr: string) => {
  const date = new Date(dateStr);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const last7Days = new Date(today);
  last7Days.setDate(last7Days.getDate() - 7);

  if (date >= today) return 'Today';
  if (date >= yesterday) return 'Yesterday';
  if (date >= last7Days) return 'Past week';
  return 'Past month';
};

const SearchModal: React.FC<SearchModalProps> = ({ isOpen, onClose, chats }) => {
  const [query, setQuery] = useState('');
  const [filteredChats, setFilteredChats] = useState<Chat[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => {
        inputRef.current?.focus();
      }, 50);
      setFilteredChats(chats);
    } else {
      setQuery('');
    }
  }, [isOpen, chats]);

  useEffect(() => {
    if (!query.trim()) {
      setFilteredChats(chats);
      return;
    }
    const lowerQuery = query.toLowerCase();
    const filtered = chats.filter(chat =>
      (chat.title || 'Untitled conversation').toLowerCase().includes(lowerQuery)
    );
    setFilteredChats(filtered);
  }, [query, chats]);

  if (!isOpen) return null;

  const handleSelectChat = (id: string) => {
    navigate(`/chat/${id}`);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-start justify-center pt-[14vh] px-4" onClick={onClose}>
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/10 backdrop-blur-[1px] transition-opacity" />

      {/* Modal */}
      <div
        className="relative w-full max-w-[600px] bg-[#F9F9F8] dark:bg-[#2D2D2A] rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[60vh] animate-in fade-in zoom-in-95 duration-200 border border-gray-200/50 dark:border-gray-700/50"
        onClick={e => e.stopPropagation()}
        style={{ boxShadow: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)' }}
      >
        {/* Search Input Area */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-200/50 dark:border-gray-700/50 bg-white dark:bg-[#2D2D2A]">
          <Search className="w-5 h-5 text-gray-400 dark:text-claude-text/60" strokeWidth={2} />
          <input
            ref={inputRef}
            type="text"
            className="flex-1 bg-transparent border-none outline-none text-[15px] text-gray-800 dark:text-claude-text placeholder-gray-400 dark:placeholder-claude-text/60 font-normal"
            placeholder="Search chats and projects"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Escape') onClose();
            }}
          />
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-100 dark:hover:bg-white/10 rounded-md transition-colors text-gray-400 dark:text-claude-text/60 hover:text-gray-600 dark:hover:text-claude-text"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Results List */}
        <div className="overflow-y-auto flex-1 py-2 custom-scrollbar bg-[#F9F9F8] dark:bg-[#2D2D2A]">
          {filteredChats.length === 0 ? (
            <div className="px-6 py-12 text-center text-gray-500 text-sm">
              No results found
            </div>
          ) : (
            <div className="px-2 space-y-0.5">
              {filteredChats.map(chat => (
                <div
                  key={chat.id}
                  onClick={() => handleSelectChat(chat.id)}
                  className="flex items-center justify-between px-3 py-2.5 hover:bg-white dark:hover:bg-gray-700/50 hover:shadow-sm rounded-lg cursor-pointer group transition-all duration-200 border border-transparent hover:border-gray-100 dark:hover:border-gray-600/30"
                >
                  <div className="flex items-center gap-3 min-w-0 flex-1 pr-4">
                    <div className="w-5 h-5 flex items-center justify-center flex-shrink-0 text-gray-500 dark:text-claude-text/60">
                      <MessageSquare className="w-4 h-4" strokeWidth={2} />
                    </div>
                    <span className="text-[14px] text-gray-700 dark:text-claude-text truncate font-normal leading-none pt-0.5">
                      {chat.title || 'Untitled conversation'}
                    </span>
                  </div>
                  <span className="text-[12px] text-gray-400 dark:text-claude-text/60 flex-shrink-0 font-normal">
                    {getTimeLabel(chat.updated_at || chat.created_at)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SearchModal;
