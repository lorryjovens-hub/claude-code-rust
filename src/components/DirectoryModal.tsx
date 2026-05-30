import React, { useState } from 'react';
import { Search, X, Filter, ChevronDown, FileText, Link2, Puzzle, ArrowRight, Download } from 'lucide-react';

interface DirectoryCategory {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  downloads: string;
  author?: string;
}

interface DirectoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate?: (type: 'skills' | 'connectors' | 'plugins', category?: string) => void;
}

const SKILL_CATEGORIES: DirectoryCategory[] = [
  {
    id: 'productivity',
    name: 'Productivity',
    description: 'Manage tasks, plan your day, and build up memory of important context about your work. Syncs with your calendar and tools.',
    icon: <FileText size={20} />,
    downloads: '604.1K',
    author: 'Anthropic',
  },
  {
    id: 'design',
    name: 'Design',
    description: 'Accelerate design workflows — critique, design system management, UX writing, accessibility audits, and more.',
    icon: <FileText size={20} />,
    downloads: '550.2K',
    author: 'Anthropic',
  },
  {
    id: 'marketing',
    name: 'Marketing',
    description: 'Create content, plan campaigns, and analyze performance across marketing channels. Maintain brand voice.',
    icon: <FileText size={20} />,
    downloads: '463.6K',
    author: 'Anthropic',
  },
  {
    id: 'data',
    name: 'Data',
    description: 'Write SQL, explore datasets, and generate insights faster. Build visualizations and dashboards, and turn raw data.',
    icon: <FileText size={20} />,
    downloads: '447.3K',
    author: 'Anthropic',
  },
  {
    id: 'engineering',
    name: 'Engineering',
    description: 'Streamline engineering workflows — standups, code review, architecture decisions, incident response, and more.',
    icon: <FileText size={20} />,
    downloads: '431.3K',
    author: 'Anthropic',
  },
  {
    id: 'finance',
    name: 'Finance',
    description: 'Streamline finance and accounting workflows, from journal entries and reconciliation to financial statements.',
    icon: <FileText size={20} />,
    downloads: '379.8K',
    author: 'Anthropic',
  },
  {
    id: 'product-management',
    name: 'Product management',
    description: 'Write feature specs, plan roadmaps, and synthesize user research faster. Keep stakeholders aligned and moving.',
    icon: <FileText size={20} />,
    downloads: '355.7K',
    author: 'Anthropic',
  },
  {
    id: 'operations',
    name: 'Operations',
    description: 'Optimize business operations — vendor management, process documentation, change management, capacity planning.',
    icon: <FileText size={20} />,
    downloads: '330.5K',
    author: 'Anthropic',
  },
];

const CONNECTOR_CATEGORIES: DirectoryCategory[] = [
  {
    id: 'slack',
    name: 'Slack',
    description: 'Connect Claude to your Slack workspace for seamless team communication and collaboration.',
    icon: <Link2 size={20} />,
    downloads: '892.4K',
    author: 'Anthropic',
  },
  {
    id: 'github',
    name: 'GitHub',
    description: 'Integrate with GitHub repositories for code review, PR management, and issue tracking.',
    icon: <Link2 size={20} />,
    downloads: '756.2K',
    author: 'Anthropic',
  },
  {
    id: 'google-drive',
    name: 'Google Drive',
    description: 'Access and search your Google Drive files directly through Claude.',
    icon: <Link2 size={20} />,
    downloads: '623.1K',
    author: 'Anthropic',
  },
  {
    id: 'notion',
    name: 'Notion',
    description: 'Connect to Notion workspaces for document creation, search, and knowledge management.',
    icon: <Link2 size={20} />,
    downloads: '534.8K',
    author: 'Anthropic',
  },
  {
    id: 'jira',
    name: 'Jira',
    description: 'Manage projects, track issues, and sync with your development workflow.',
    icon: <Link2 size={20} />,
    downloads: '445.6K',
    author: 'Anthropic',
  },
  {
    id: 'salesforce',
    name: 'Salesforce',
    description: 'Access CRM data, manage leads, and automate sales workflows.',
    icon: <Link2 size={20} />,
    downloads: '389.2K',
    author: 'Anthropic',
  },
];

const PLUGIN_CATEGORIES: DirectoryCategory[] = [
  {
    id: 'web-tools',
    name: 'Web Tools',
    description: 'Browser automation, web scraping, and online service integrations.',
    icon: <Puzzle size={20} />,
    downloads: '512.3K',
    author: 'Anthropic',
  },
  {
    id: 'code-tools',
    name: 'Code Tools',
    description: 'Enhanced coding capabilities, linting, formatting, and code generation.',
    icon: <Puzzle size={20} />,
    downloads: '478.9K',
    author: 'Anthropic',
  },
  {
    id: 'ai-models',
    name: 'AI Models',
    description: 'Connect to additional AI models and services for specialized tasks.',
    icon: <Puzzle size={20} />,
    downloads: '423.7K',
    author: 'Anthropic',
  },
  {
    id: 'file-tools',
    name: 'File Tools',
    description: 'File conversion, document processing, and media manipulation.',
    icon: <Puzzle size={20} />,
    downloads: '367.5K',
    author: 'Anthropic',
  },
  {
    id: 'communication',
    name: 'Communication',
    description: 'Email, messaging, and notification integrations.',
    icon: <Puzzle size={20} />,
    downloads: '334.1K',
    author: 'Anthropic',
  },
  {
    id: 'analytics',
    name: 'Analytics',
    description: 'Data analysis, reporting, and business intelligence tools.',
    icon: <Puzzle size={20} />,
    downloads: '298.6K',
    author: 'Anthropic',
  },
];

type TabType = 'skills' | 'connectors' | 'plugins';

const TAB_CONFIG: Record<TabType, { label: string; icon: React.ReactNode; categories: DirectoryCategory[] }> = {
  skills: {
    label: 'Skills',
    icon: <FileText size={16} />,
    categories: SKILL_CATEGORIES,
  },
  connectors: {
    label: 'Connectors',
    icon: <Link2 size={16} />,
    categories: CONNECTOR_CATEGORIES,
  },
  plugins: {
    label: 'Plugins',
    icon: <Puzzle size={16} />,
    categories: PLUGIN_CATEGORIES,
  },
};

const DirectoryModal: React.FC<DirectoryModalProps> = ({ isOpen, onClose, onNavigate }) => {
  const [activeTab, setActiveTab] = useState<TabType>('plugins');
  const [searchQuery, setSearchQuery] = useState('');

  if (!isOpen) return null;

  const currentConfig = TAB_CONFIG[activeTab];
  const filteredCategories = currentConfig.categories.filter(
    cat =>
      cat.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      cat.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="w-[720px] max-w-[90vw] max-h-[80vh] bg-claude-bg border border-claude-border rounded-2xl shadow-2xl flex flex-col overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-claude-border flex-shrink-0">
          <h2 className="font-[Spectral] text-[20px] text-claude-text" style={{ fontWeight: 500 }}>
            Directory
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex flex-1 min-h-0">
          {/* Left sidebar - tabs */}
          <div className="w-[160px] border-r border-claude-border flex-shrink-0 py-4">
            <nav className="flex flex-col gap-1 px-3">
              {(Object.keys(TAB_CONFIG) as TabType[]).map(tab => {
                const config = TAB_CONFIG[tab];
                const isActive = activeTab === tab;
                return (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-[14px] transition-colors text-left ${
                      isActive
                        ? 'bg-claude-hover text-claude-text font-medium'
                        : 'text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover/50'
                    }`}
                  >
                    {config.icon}
                    <span>{config.label}</span>
                  </button>
                );
              })}
            </nav>
          </div>

          {/* Right content */}
          <div className="flex-1 flex flex-col min-w-0">
            {/* Search bar */}
            <div className="px-6 pt-4 pb-3">
              <div className="relative">
                <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-claude-textSecondary" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  placeholder={`Search ${currentConfig.label.toLowerCase()}...`}
                  className="w-full pl-9 pr-4 py-2.5 bg-claude-input border border-claude-border rounded-xl text-[14px] text-claude-text placeholder:text-claude-textSecondary/50 outline-none focus:border-[#387ee0] transition-colors"
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery('')}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-claude-textSecondary hover:text-claude-text"
                  >
                    <X size={14} />
                  </button>
                )}
              </div>
            </div>

            {/* Banner for plugins tab */}
            {activeTab === 'plugins' && (
              <div className="mx-6 mb-3 px-4 py-2.5 bg-claude-hover/50 border border-claude-border rounded-lg text-[13px] text-claude-textSecondary">
                Plugins can be browsed, but are only available for use in the desktop app.{' '}
                <a href="#" className="text-[#4B9EFA] hover:underline">Download Claude for Desktop</a>
              </div>
            )}

            {/* Filter and sort bar */}
            <div className="flex items-center justify-between px-6 pb-3">
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-claude-textSecondary font-medium">Anthropic & Partners</span>
              </div>
              <div className="flex items-center gap-2">
                <button className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors">
                  <Filter size={12} />
                  Filter by
                  <ChevronDown size={12} />
                </button>
                <button className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors">
                  Sort by
                  <ChevronDown size={12} />
                </button>
              </div>
            </div>

            {/* Category cards grid */}
            <div className="flex-1 overflow-y-auto px-6 pb-6">
              <div className="grid grid-cols-2 gap-3">
                {filteredCategories.map(category => (
                  <button
                    key={category.id}
                    onClick={() => {
                      onNavigate?.(activeTab, category.id);
                      onClose();
                    }}
                    className="flex items-start gap-3 p-4 bg-claude-input border border-claude-border rounded-xl text-left hover:border-[#5a5a58] hover:shadow-md transition-all group"
                  >
                    <div className="w-9 h-9 rounded-lg bg-claude-bg flex items-center justify-center text-claude-textSecondary flex-shrink-0 group-hover:text-claude-text transition-colors">
                      {category.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="text-[14px] font-semibold text-claude-text truncate">{category.name}</h3>
                      </div>
                      {category.author && (
                        <div className="flex items-center gap-1.5 mb-1.5">
                          <span className="text-[11px] text-claude-textSecondary">{category.author}</span>
                          <span className="text-[11px] text-claude-textSecondary/40">·</span>
                          <span className="flex items-center gap-0.5 text-[11px] text-claude-textSecondary">
                            <Download size={9} />
                            {category.downloads}
                          </span>
                        </div>
                      )}
                      <p className="text-[12px] text-claude-textSecondary leading-snug line-clamp-2">
                        {category.description}
                      </p>
                    </div>
                    <ArrowRight size={14} className="text-claude-textSecondary opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 mt-1" />
                  </button>
                ))}
              </div>

              {filteredCategories.length === 0 && (
                <div className="flex flex-col items-center justify-center py-12 text-center">
                  <Search size={32} className="text-claude-textSecondary/30 mb-3" />
                  <p className="text-[14px] text-claude-textSecondary">No {currentConfig.label.toLowerCase()} found</p>
                  <p className="text-[12px] text-claude-textSecondary/60 mt-1">Try a different search term</p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DirectoryModal;
