import React, { useState, useEffect, useCallback } from 'react';
import { flushSync } from 'react-dom';
import { useNavigate } from 'react-router-dom';
import {
  ArrowLeft, Plus, Search, Trash2, Sparkles, LayoutGrid, FileText,
  ChevronRight, ChevronDown, Folder, File, MoreHorizontal, Info, Eye, Code,
  Settings, Check, MessageSquare, ClipboardList, Upload, Github, X, FolderPlus
} from 'lucide-react';
import MarkdownRenderer, { CodeBlock } from './MarkdownRenderer';
import { getSkills, getSkillDetail, getSkillFile, createSkill, updateSkill, deleteSkill, toggleSkill, getGithubStatus, getGithubAuthUrl, disconnectGithub } from '../api';
import searchIconImg from '../assets/icons/search-icon.png';
import skillsImg from '../assets/icons/skills.png';
import connectorsImg from '../assets/icons/connectors.png';
import customizeIconImg from '../assets/icons/customize-icon.png';
import customizeMainImg from '../assets/icons/customize-main.png';
import createSkillsImg from '../assets/icons/create-skills.png';

interface Skill {
  id: string;
  name: string;
  description: string;
  content?: string;
  is_example?: boolean;
  source_dir?: string;
  user_id?: string | null;
  enabled: boolean;
  created_at?: string;
  files?: any[];
}

// File structure for skill-creator matching the official anthropics/skills repo
const SKILL_CREATOR_FILES = [
  { name: 'SKILL.md', type: 'file' },
  {
    name: 'agents', type: 'folder', children: [
      { name: 'analyzer.md', type: 'file' },
      { name: 'comparator.md', type: 'file' },
      { name: 'grader.md', type: 'file' },
    ]
  },
  {
    name: 'assets', type: 'folder', children: [
      { name: 'eval_review.html', type: 'file' },
    ]
  },
  {
    name: 'eval-viewer', type: 'folder', children: [
      { name: 'generate_review.py', type: 'file' },
      { name: 'viewer.html', type: 'file' },
    ]
  },
  {
    name: 'references', type: 'folder', children: [
      { name: 'schemas.md', type: 'file' },
    ]
  },
  {
    name: 'scripts', type: 'folder', children: [
      { name: '__init__.py', type: 'file' },
      { name: 'aggregate_benchmark.py', type: 'file' },
      { name: 'generate_report.py', type: 'file' },
      { name: 'improve_description.py', type: 'file' },
      { name: 'package_skill.py', type: 'file' },
      { name: 'quick_validate.py', type: 'file' },
      { name: 'run_eval.py', type: 'file' },
      { name: 'run_loop.py', type: 'file' },
      { name: 'utils.py', type: 'file' },
    ]
  },
  { name: 'LICENSE.txt', type: 'file' },
];

interface FileTreeNodeProps {
  skill: Skill;
  isExpanded: boolean;
  onExpand: (e: React.MouseEvent) => void;
  selectedSkillId: string | null;
  detail: Skill | null;
  selectSkill: (id: string) => void;
  selectedFile: string;
  setSelectedFile: (name: string) => void;
  setSelectedSkillId: (id: string) => void;
  loadFileContent: (skillId: string, filePath: string) => void;
}

const FileTreeNode: React.FC<FileTreeNodeProps> = ({
  skill, isExpanded, onExpand,
  selectedSkillId, detail, selectSkill,
  selectedFile, setSelectedFile, setSelectedSkillId, loadFileContent
}) => {
  const isSelected = selectedSkillId === skill.id;
  const hasFiles = detail && detail.id === skill.id && detail.files && detail.files.length > 0;
  const isEnabled = skill.enabled;
  const [mockFolderState, setMockFolderState] = useState<Record<string, boolean>>({});

  const toggleMockFolder = (name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setMockFolderState(prev => ({ ...prev, [name]: !prev[name] }));
  };

  return (
    <div className="select-none overflow-hidden">
      <div
        onClick={(e) => {
          if (isSelected) {
            onExpand(e);
          } else {
            selectSkill(skill.id);
          }
        }}
        className={`flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer transition-colors ${isSelected ? 'bg-claude-hover' : 'hover:bg-claude-hover/50'}`}
      >
        <div className="flex items-center justify-center w-[24px] h-[24px] rounded-md border border-claude-border bg-black/5 dark:bg-[#30302E] flex-shrink-0">
          <img
            src={skillsImg}
            className={`w-[15.5px] h-[15.5px] ${isEnabled ? 'dark:invert opacity-90' : 'dark:invert opacity-30 invert-[0.3]'}`}
            alt="skill"
          />
        </div>

        <div className="flex-1 min-w-0">
          <span className={`truncate text-[14px] ${isEnabled ? 'text-claude-text font-medium' : 'text-[#262624] dark:text-[#6B6B68]'}`}>
            {skill.name}
          </span>
        </div>

        {isSelected && (
          <ChevronRight size={16} className={`text-claude-textSecondary transition-transform duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] ${isExpanded && hasFiles ? 'rotate-90' : ''}`} />
        )}
      </div>

      <div className={`grid transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] ${isExpanded && hasFiles ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0 pointer-events-none'}`}>
        <div className="overflow-hidden">
          <div className="pl-6 pt-0.5 pb-2 space-y-0.5">
            {(detail?.files || []).map((file: any) => (
              <div key={file.name}>
                <div
                  onClick={(e) => {
                    e.stopPropagation();
                    if (file.type === 'folder') toggleMockFolder(file.name, e);
                    else {
                      setSelectedFile(file.name);
                      setSelectedSkillId(skill.id);
                      loadFileContent(skill.id, file.name);
                    }
                  }}
                  className={`flex items-center gap-2 px-2 py-1.5 rounded-md cursor-pointer text-[13.5px] group ${selectedSkillId === skill.id && selectedFile === file.name && file.type !== 'folder' ? 'bg-claude-hover' : 'hover:bg-claude-hover/70'}`}
                >
                  <div className="w-[12px]" /> {/* Indent for chevron */}
                  {file.type === 'folder' ? (
                    <Folder size={15.5} className="text-claude-textSecondary fill-claude-textSecondary/10" />
                  ) : (
                    <File size={15.5} className="text-claude-textSecondary group-hover:text-claude-text transition-colors" />
                  )}
                  <span className={`truncate ${selectedSkillId === skill.id && selectedFile === file.name && file.type !== 'folder' ? 'text-claude-text font-medium' : 'text-claude-textSecondary group-hover:text-claude-text transition-colors'}`}>{file.name}</span>
                  {file.type === 'folder' && <ChevronRight size={14} className={`ml-auto text-claude-textSecondary transition-transform duration-200 ${mockFolderState[file.name] ? 'rotate-90' : ''}`} />}
                </div>
                {/* Folder children */}
                {file.type === 'folder' && (
                  <div className={`grid transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] ${mockFolderState[file.name] ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'}`}>
                    <div className="overflow-hidden">
                      {file.children && file.children.length > 0 ? (
                        <div className="pl-6 py-0.5 space-y-0.5">
                          {file.children.map((child: any) => (
                            <div
                              key={child.name}
                              onClick={(e) => {
                                e.stopPropagation();
                                // Build relative path: folder/child
                                const childPath = file.name + '/' + child.name;
                                setSelectedFile(child.name);
                                setSelectedSkillId(skill.id);
                                loadFileContent(skill.id, childPath);
                              }}
                              className={`flex items-center gap-2 px-2 py-1 rounded-md cursor-pointer text-[13px] group ${selectedSkillId === skill.id && selectedFile === child.name ? 'bg-claude-hover' : 'hover:bg-claude-hover/70'}`}
                            >
                              <File size={14} className="text-claude-textSecondary group-hover:text-claude-text transition-colors" />
                              <span className={`truncate ${selectedSkillId === skill.id && selectedFile === child.name ? 'text-claude-text font-medium' : 'text-claude-textSecondary group-hover:text-claude-text transition-colors'}`}>{child.name}</span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div className="pl-12 py-1 pb-2 text-[12px] text-claude-textSecondary/60 italic pointer-events-none">
                          Empty directory
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

const CustomizePage = ({ onCreateWithClaude }: { onCreateWithClaude?: () => void }) => {
  const navigate = useNavigate();
  const [tab, setTab] = useState<'overview' | 'skills' | 'connectors'>('overview');
  const [examples, setExamples] = useState<Skill[]>([]);
  const [mySkills, setMySkills] = useState<Skill[]>([]);
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);
  const [detail, setDetail] = useState<Skill | null>(null);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);

  // GitHub connector state
  const [githubConnected, setGithubConnected] = useState(false);
  const [githubUser, setGithubUser] = useState<{ login: string; avatar_url: string; name?: string } | null>(null);
  const [selectedConnector, setSelectedConnector] = useState<'github' | 'gdrive'>('github');

  useEffect(() => {
    getGithubStatus().then(data => {
      setGithubConnected(data.connected);
      if (data.user) setGithubUser(data.user);
    }).catch(() => { });
    // Poll for connection after OAuth redirect (user might have just authorized in browser)
    const poll = setInterval(() => {
      getGithubStatus().then(data => {
        if (data.connected && !githubConnected) {
          setGithubConnected(true);
          if (data.user) setGithubUser(data.user);
        }
      }).catch(() => { });
    }, 3000);
    return () => clearInterval(poll);
  }, [githubConnected]);

  const handleGithubConnect = async () => {
    try {
      const { url } = await getGithubAuthUrl();
      // Open in system browser (Electron) or new window (web)
      import('../utils/tauriAPI').then(m => {
        m.tauriAPI.openExternal(url);
      }).catch(() => window.open(url, '_blank'));
    } catch (e) { console.error('GitHub auth error:', e); }
  };

  const handleGithubDisconnect = async () => {
    await disconnectGithub();
    setGithubConnected(false);
    setGithubUser(null);
  };

  // Tree state
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['examples', 'myskills']));
  const [expandedSkills, setExpandedSkills] = useState<Set<string>>(new Set());
  const [selectedFile, setSelectedFile] = useState<string>('SKILL.md'); // For visual selection in tree
  const [fileContent, setFileContent] = useState<string>(''); // Content of selected non-SKILL.md file
  const [showSearchInput, setShowSearchInput] = useState(false);

  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [showUploadModal, setShowUploadModal] = useState(false);

  // Edit/Create state
  const [creating, setCreating] = useState(false);
  const [editName, setEditName] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editContent, setEditContent] = useState('');
  const [saving, setSaving] = useState(false);
  const [viewMode, setViewMode] = useState<'preview' | 'code'>('preview');

  useEffect(() => {
    if (selectedFile) {
      if (selectedFile.endsWith('.md') || selectedFile.endsWith('.mdx')) {
        setViewMode('preview');
      } else {
        setViewMode('code');
      }
    }
  }, [selectedFile]);

  const fetchList = useCallback(async () => {
    try {
      const data = await getSkills();
      const exList: Skill[] = data.examples || [];
      const myList: Skill[] = data.my_skills || [];

      // Sort examples: skill-creator first, then alphabetical
      exList.sort((a, b) => {
        if (a.source_dir === 'skill-creator') return -1;
        if (b.source_dir === 'skill-creator') return 1;
        return a.name.localeCompare(b.name);
      });

      setExamples(exList);
      setMySkills(myList);

      // Auto-select skill-creator if not selected
      if (!selectedSkillId && !creating) {
        const scIndex = exList.findIndex(s => s.source_dir === 'skill-creator');
        if (scIndex !== -1) {
          // Force enabled for skill-creator as per request
          exList[scIndex] = { ...exList[scIndex], enabled: true };
          selectSkill(exList[scIndex].id);
          // Default collapsed as per user request
          // setExpandedSkills(prev => new Set(prev).add(sc.id));
        }
      }
    } catch (e) { console.error(e); }
    setLoading(false);
  }, [creating, selectedSkillId]);

  useEffect(() => { fetchList(); }, [fetchList]);

  const loadFileContent = async (skillId: string, filePath: string) => {
    if (filePath === 'SKILL.md') { setFileContent(''); return; }
    try {
      const data = await getSkillFile(skillId, filePath);
      setFileContent(data.content || '');
    } catch (e) {
      setFileContent(`// Failed to load ${filePath}`);
    }
  };

  const selectSkill = async (id: string) => {
    setCreating(false);
    setSelectedSkillId(id);
    setSelectedFile('SKILL.md');
    setFileContent('');
    // Collapse first so animation can play from closed → open
    setExpandedSkills(prev => { const s = new Set(prev); s.delete(id); return s; });
    try {
      const d = await getSkillDetail(id);
      flushSync(() => {
        setDetail(d);
        if (!d.is_example) {
          setEditName(d.name);
          setEditDesc(d.description || '');
          setEditContent(d.content || '');
        }
      });

      // Force layout calculation so the browser registers the grid-rows-[0fr] state
      void document.body.offsetHeight;

      // Delay expanding so that the layout flush is definitively painted before changing to grid-rows-[1fr]
      setTimeout(() => {
        setExpandedSkills(prev => new Set([...prev, id]));
      }, 10);
    } catch (e) { console.error(e); }
  };

  const handleToggle = async (id: string, current: boolean, e?: React.MouseEvent) => {
    e?.stopPropagation();
    try {
      await toggleSkill(id, !current);
      setExamples(prev => prev.map(s => s.id === id ? { ...s, enabled: !current } : s));
      setMySkills(prev => prev.map(s => s.id === id ? { ...s, enabled: !current } : s));
      if (detail?.id === id) setDetail(prev => prev ? { ...prev, enabled: !current } : prev);
    } catch (e) { console.error(e); }
  };

  const toggleSection = (section: string) => {
    const newSet = new Set(expandedSections);
    if (newSet.has(section)) newSet.delete(section);
    else newSet.add(section);
    setExpandedSections(newSet);
  };

  const toggleSkillExpand = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const newSet = new Set(expandedSkills);
    if (newSet.has(id)) newSet.delete(id);
    else newSet.add(id);
    setExpandedSkills(newSet);
  };

  // ... (Create, Save, Delete handlers same as before, simplified for brevity)
  const handleCreate = async () => {
    if (!editName.trim()) return;
    setSaving(true);
    try {
      const s = await createSkill({ name: editName, description: editDesc, content: editContent });
      setMySkills(prev => [{ ...s, enabled: true }, ...prev]);
      setCreating(false);
      setSelectedSkillId(s.id);
      setDetail({ ...s, enabled: true });
    } catch (e) { console.error(e); }
    setSaving(false);
  };

  const handleSave = async () => {
    if (!detail) return;
    setSaving(true);
    try {
      await updateSkill(detail.id, { name: editName, description: editDesc, content: editContent });
      setDetail(prev => prev ? { ...prev, name: editName, description: editDesc, content: editContent } : prev);
      setMySkills(prev => prev.map(s => s.id === detail.id ? { ...s, name: editName, description: editDesc } : s));
    } catch (e) { console.error(e); }
    setSaving(false);
  };

  const handleDelete = async () => {
    if (!detail) return;
    if (!confirm('Are you sure you want to delete this skill?')) return;
    try {
      await deleteSkill(detail.id);
      setMySkills(prev => prev.filter(s => s.id !== detail.id));
      setSelectedSkillId(null);
      setDetail(null);
    } catch (e) { console.error(e); }
  };

  const startCreate = () => {
    setCreating(true);
    setSelectedSkillId(null);
    setDetail(null);
    setEditName('');
    setEditDesc('');
    setEditContent('');
  };

  // Filter skills
  const q = search.toLowerCase();
  const filteredExamples = examples.filter(s => s.name.toLowerCase().includes(q));
  const filteredMy = mySkills.filter(s => s.name.toLowerCase().includes(q));

  // --- Render Helpers ---

  const ToggleSwitch = ({ enabled, onToggle, size = 'md' }: { enabled: boolean; onToggle: (e: React.MouseEvent) => void, size?: 'sm' | 'md' }) => (
    <button onClick={onToggle}
      className={`relative inline-flex items-center rounded-full transition-colors duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] ${enabled ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-600'} ${size === 'sm' ? 'h-4 w-7' : 'h-5 w-9'}`}>
      <span className={`inline-block rounded-full bg-white shadow-sm transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] ${size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'} ${enabled ? (size === 'sm' ? 'translate-x-[14px]' : 'translate-x-[18px]') : 'translate-x-[2px]'}`} />
    </button>
  );


  return (
    <div className="flex h-full w-full bg-claude-bg text-claude-text font-sans">

      {/* 1. Left Navigation (Fixed width) */}
      <div className="w-[240px] border-r border-claude-border flex flex-col pt-4 pb-4 flex-shrink-0 bg-claude-bg">
        <div className="px-4 mb-6">
          <button onClick={() => {
            if (tab !== 'overview') setTab('overview');
            else navigate(-1);
          }}
            className="flex items-center gap-2 text-claude-text font-medium hover:text-claude-text/80 transition-colors">
            <ArrowLeft size={20} />
            <span className="text-lg font-semibold">Customize</span>
          </button>
        </div>
        <nav className="flex-1 px-2 space-y-1">
          <button onClick={() => setTab('skills')}
            className={`w-full flex items-center gap-3 px-3 py-2 text-[15px] font-medium rounded-lg transition-colors ${tab === 'skills' ? 'bg-claude-hover text-claude-text' : 'text-claude-text hover:bg-claude-hover'}`}>
            <img src={skillsImg} alt="" className="w-[22px] h-[22px] dark:invert" />
            Skills
          </button>
          <button onClick={() => setTab('connectors')}
            className={`w-full flex items-center gap-3 px-3 py-2 text-[15px] font-medium rounded-lg transition-colors ${tab === 'connectors' ? 'bg-claude-hover text-claude-text' : 'text-claude-text hover:bg-claude-hover'}`}>
            <img src={connectorsImg} alt="" className="w-[22px] h-[22px] dark:invert" />
            Connectors
          </button>
        </nav>
      </div>

      {/* 2. Middle Column: Skills List or Connectors List */}
      {tab === 'skills' ? (
        <div className="w-[300px] border-r border-claude-border flex flex-col flex-shrink-0 bg-claude-bg">
          {/* Header */}
          <div className="h-14 px-4 flex items-center justify-between border-b border-claude-border">
            <span className="font-semibold text-claude-text">Skills</span>
            <div className="flex items-center gap-2 relative">
              <button
                onClick={() => setShowSearchInput(!showSearchInput)}
                className="p-1.5 rounded-md hover:bg-claude-hover text-claude-textSecondary hover:text-claude-text transition-colors group"
              >
                <Search size={21} className="opacity-70 group-hover:opacity-100 transition-opacity" />
              </button>
              <div className="relative">
                <button
                  onClick={() => setShowPlusMenu(!showPlusMenu)}
                  className="p-1.5 rounded-md hover:bg-claude-hover text-claude-textSecondary hover:text-claude-text transition-colors"
                >
                  <Plus size={22} />
                </button>
                {showPlusMenu && (
                  <>
                    <div className="fixed inset-0 z-40" onClick={() => setShowPlusMenu(false)} />
                    <div className="absolute top-full right-0 mt-2 w-64 bg-white dark:bg-[#202020] rounded-[16px] shadow-[0_4px_24px_rgba(0,0,0,0.15)] border border-claude-border py-2 z-50">
                      <button className="w-full flex items-center gap-3.5 px-4 py-3 text-[14.5px] font-medium text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={() => { setShowPlusMenu(false); onCreateWithClaude?.(); }}>
                        <MessageSquare size={18} className="text-claude-textSecondary" />
                        Create with Claude
                      </button>
                      <button className="w-full flex items-center gap-3.5 px-4 py-3 text-[14.5px] font-medium text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={() => { setShowPlusMenu(false); startCreate(); }}>
                        <ClipboardList size={18} className="text-claude-textSecondary" />
                        Write skill instructions
                      </button>
                      <button className="w-full flex items-center gap-3.5 px-4 py-3 text-[14.5px] font-medium text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={() => { setShowPlusMenu(false); setShowUploadModal(true); }}>
                        <Upload size={18} className="text-claude-textSecondary" />
                        Upload a skill
                      </button>
                    </div>
                  </>
                )}
              </div>
            </div>
          </div>

          {/* Search Input (Conditional) */}
          {showSearchInput && (
            <div className="px-3 py-2 border-b border-claude-border">
              <input
                autoFocus
                value={search}
                onChange={e => setSearch(e.target.value)}
                placeholder="Filter skills..."
                className="w-full px-2 py-1.5 bg-claude-input rounded-md text-sm outline-none border border-transparent focus:border-blue-500"
              />
            </div>
          )}

          {/* List Content */}
          <div className="flex-1 overflow-y-auto py-2">
            {/* Examples Section */}
            {filteredExamples.length > 0 && (
              <div className="mb-2">
                <button
                  onClick={() => toggleSection('examples')}
                  className="w-full flex items-center gap-1 px-3 py-1.5 text-xs font-semibold text-claude-textSecondary hover:text-claude-text uppercase tracking-wider"
                >
                  <ChevronDown size={14} className={`transition-transform ${expandedSections.has('examples') ? '' : '-rotate-90'}`} />
                  Examples
                </button>

                {expandedSections.has('examples') && (
                  <div className="mt-0.5 px-2 space-y-0.5">
                    {filteredExamples.map(skill => (
                      <FileTreeNode
                        key={skill.id}
                        skill={skill}
                        isExpanded={expandedSkills.has(skill.id)}
                        onExpand={(e) => toggleSkillExpand(skill.id, e)}
                        selectedSkillId={selectedSkillId}
                        detail={detail}
                        selectSkill={selectSkill}
                        selectedFile={selectedFile}
                        setSelectedFile={setSelectedFile}
                        setSelectedSkillId={setSelectedSkillId}
                        loadFileContent={loadFileContent}
                      />
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* My Skills Section */}
            {filteredMy.length > 0 && (
              <div>
                <button
                  onClick={() => toggleSection('myskills')}
                  className="w-full flex items-center gap-1 px-3 py-1.5 text-xs font-semibold text-claude-textSecondary hover:text-claude-text uppercase tracking-wider"
                >
                  <ChevronDown size={14} className={`transition-transform ${expandedSections.has('myskills') ? '' : '-rotate-90'}`} />
                  My Skills
                </button>

                {expandedSections.has('myskills') && (
                  <div className="mt-0.5 px-2 space-y-0.5 overflow-hidden">
                    <div className={`grid transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] grid-rows-[1fr] opacity-100`}>
                      <div className="overflow-hidden">
                        {filteredMy.map(skill => (
                          <FileTreeNode
                            key={skill.id}
                            skill={skill}
                            isExpanded={expandedSkills.has(skill.id)}
                            onExpand={(e) => toggleSkillExpand(skill.id, e)}
                            selectedSkillId={selectedSkillId}
                            detail={detail}
                            selectSkill={selectSkill}
                            selectedFile={selectedFile}
                            setSelectedFile={setSelectedFile}
                            setSelectedSkillId={setSelectedSkillId}
                            loadFileContent={loadFileContent}
                          />
                        ))}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {!filteredExamples.length && !filteredMy.length && (
              <div className="p-4 text-center text-sm text-claude-textSecondary">
                No skills found
              </div>
            )}
          </div>
        </div>
      ) : tab === 'connectors' ? (
        <div className="w-[300px] border-r border-claude-border flex flex-col flex-shrink-0 bg-claude-bg">
          <div className="h-14 px-4 flex items-center justify-between border-b border-claude-border">
            <span className="font-semibold text-claude-text">Connectors</span>
          </div>
          <div className="flex-1 overflow-y-auto pt-4 px-2">
            {/* Connected section */}
            {githubConnected && (
              <div className="mb-2">
                <button className="w-full flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium text-claude-textSecondary/80 uppercase tracking-wider">
                  <ChevronDown size={14} className="stroke-[2px] opacity-70" />
                  Connected
                </button>
                <div className="mt-1 px-1 space-y-1">
                  <div
                    onClick={() => setSelectedConnector('github')}
                    className={`flex items-center gap-3 px-3 py-2.5 rounded-[10px] cursor-pointer transition-colors ${selectedConnector === 'github' ? 'bg-claude-hover' : 'hover:bg-claude-hover/50'}`}
                  >
                    <Github size={20} className="fill-current stroke-none opacity-80" />
                    <span className="truncate text-[14px] text-claude-text font-medium">GitHub Integration</span>
                    <div className="ml-auto w-2 h-2 rounded-full bg-green-500 flex-shrink-0" />
                  </div>
                </div>
              </div>
            )}
            {/* Not connected section */}
            {(!githubConnected || true) && (
              <div className="mb-2">
                <button className="w-full flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium text-claude-textSecondary/80 uppercase tracking-wider">
                  <ChevronDown size={14} className="stroke-[2px] opacity-70" />
                  {githubConnected ? 'Available' : 'Not connected'}
                </button>
                <div className="mt-1 px-1 space-y-1">
                  {!githubConnected && (
                    <div
                      onClick={() => setSelectedConnector('github')}
                      className={`flex items-center gap-3 px-3 py-2.5 rounded-[10px] cursor-pointer transition-colors ${selectedConnector === 'github' ? 'bg-claude-hover' : 'hover:bg-claude-hover/50'}`}
                    >
                      <Github size={20} className="fill-current stroke-none opacity-80" />
                      <span className={`truncate text-[14px] ${selectedConnector === 'github' ? 'text-claude-text font-medium' : 'text-claude-textSecondary'}`}>GitHub Integration</span>
                    </div>
                  )}
                  <div
                    onClick={() => setSelectedConnector('gdrive')}
                    className={`flex items-center gap-3 px-3 py-2.5 rounded-[10px] cursor-pointer transition-colors ${selectedConnector === 'gdrive' ? 'bg-claude-hover' : 'hover:bg-claude-hover/50'}`}
                  >
                    <div className="w-5 h-5 flex items-center justify-center">
                      <img src="https://upload.wikimedia.org/wikipedia/commons/1/12/Google_Drive_icon_%282020%29.svg" className="w-[18px] h-[18px]" alt="" />
                    </div>
                    <span className={`truncate text-[14px] ${selectedConnector === 'gdrive' ? 'text-claude-text font-medium' : 'text-claude-textSecondary'}`}>Google Drive</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : null}

      {/* 3. Right Column: Detail / Create / Overview */}
      <div className="flex-1 flex flex-col min-w-0">
        {tab === 'overview' ? (
          <div className="flex flex-col items-center justify-center h-full max-w-2xl mx-auto px-6 py-12">
            <div className="mb-6">
              <img src={customizeMainImg} alt="Customize" className="w-[140px] h-auto dark:invert opacity-90" />
            </div>
            <div className="text-center mb-12">
              <h2 className="text-xl font-medium text-claude-text mb-2">Customize and manage the context and tools you are giving Claude.</h2>
            </div>

            <div className="w-full space-y-4">
              <button
                onClick={() => setTab('connectors')}
                className="w-full flex items-center p-4 border border-claude-border bg-black/[0.02] dark:bg-white/[0.04] rounded-[24px] hover:bg-black/[0.05] dark:hover:bg-white/[0.07] transition-colors text-left group"
              >
                <div className="w-11 h-11 rounded-xl bg-claude-bg/50 border border-claude-border flex items-center justify-center mr-4 group-hover:border-claude-textSecondary/30 transition-colors">
                  <img src={connectorsImg} className="w-7 h-7 dark:invert opacity-70" alt="Connectors" />
                </div>
                <div>
                  <div className="font-medium text-claude-text text-[15.5px]">Connect your apps</div>
                  <div className="text-sm text-claude-textSecondary">Integrate with the tools you use to complete your tasks</div>
                </div>
                <div className="ml-auto pr-2">
                  <ArrowLeft size={16} className="rotate-180 text-claude-textSecondary" />
                </div>
              </button>

              <button
                onClick={() => {
                  setTab('skills');
                  startCreate();
                }}
                className="w-full flex items-center p-4 border border-claude-border bg-black/[0.02] dark:bg-white/[0.04] rounded-[24px] hover:bg-black/[0.05] dark:hover:bg-white/[0.07] transition-colors text-left group"
              >
                <div className="w-11 h-11 rounded-xl bg-claude-bg/50 border border-claude-border flex items-center justify-center mr-4 group-hover:border-claude-textSecondary/30 transition-colors">
                  <img src={createSkillsImg} className="w-7 h-7 dark:invert opacity-70" alt="Skills" />
                </div>
                <div>
                  <div className="font-medium text-claude-text text-[15.5px]">Create new skills</div>
                  <div className="text-sm text-claude-textSecondary">Teach Claude your processes, team norms, and expertise</div>
                </div>
                <div className="ml-auto pr-2">
                  <ArrowLeft size={16} className="rotate-180 text-claude-textSecondary" />
                </div>
              </button>
            </div>
          </div>
        ) : tab === 'connectors' ? (
          <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
            {selectedConnector === 'github' ? (
              githubConnected ? (
                <div className="h-full flex flex-col">
                  {/* Header */}
                  <div className="flex items-center justify-between px-8 py-5 border-b border-claude-border">
                    <div className="flex items-center gap-3">
                      <Github size={22} className="fill-current stroke-none text-claude-text" />
                      <span className="text-[17px] font-semibold text-claude-text">GitHub Integration</span>
                    </div>
                    <button onClick={handleGithubDisconnect} className="px-4 py-1.5 text-[13px] font-medium text-claude-textSecondary border border-claude-border rounded-lg hover:bg-claude-hover transition-colors">
                      Disconnect
                    </button>
                  </div>
                  {/* Content */}
                  <div className="px-8 py-6 overflow-y-auto">
                    <p className="text-[14px] text-claude-textSecondary mb-6 leading-relaxed">
                      已连接 GitHub 账号 <span className="text-claude-text font-medium">{githubUser?.login}</span>，Claude 可以访问你的仓库来辅助对话。
                    </p>
                    <ul className="space-y-4 text-[14px] text-claude-textSecondary leading-relaxed">
                      <li>
                        <span className="text-claude-text font-medium">对话</span> — 提问时可以直接引用仓库中的文件作为上下文。
                      </li>
                      <li>
                        <span className="text-claude-text font-medium">项目</span> — 将仓库文件同步到项目中，让 Claude 始终了解你的代码库。
                      </li>
                      <li>
                        <span className="text-claude-text font-medium">代码</span> — 浏览仓库分支、查看代码、追踪 Pull Request。
                      </li>
                      <li>
                        <span className="text-claude-text font-medium">更多</span> — 支持代码审查、仓库搜索等更多 GitHub 功能。
                      </li>
                    </ul>
                  </div>
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center h-full mt-[-10vh]">
                  <div className="w-[68px] h-[68px] bg-black/[0.03] dark:bg-white/[0.04] border border-claude-border rounded-[18px] flex items-center justify-center mb-6 shadow-sm">
                    <Github size={36} className="text-claude-text fill-current stroke-none" />
                  </div>
                  <p className="text-[14px] text-claude-textSecondary mb-6 font-normal">你还没有连接 GitHub。</p>
                  <button
                    onClick={handleGithubConnect}
                    className="px-6 py-2.5 text-[14px] font-medium text-claude-bg bg-claude-text hover:opacity-90 rounded-[10px] transition-colors shadow-sm"
                  >
                    连接
                  </button>
                </div>
              )
            ) : (
              <div className="flex flex-col items-center mt-[-10vh]">
                <div className="w-[68px] h-[68px] bg-black/[0.03] dark:bg-white/[0.04] border border-claude-border rounded-[18px] flex items-center justify-center mb-6 shadow-sm">
                  <img src="https://upload.wikimedia.org/wikipedia/commons/1/12/Google_Drive_icon_%282020%29.svg" className="w-9 h-9" alt="" />
                </div>
                <p className="text-[14px] text-claude-textSecondary mb-6 font-normal">Google Drive integration coming soon.</p>
              </div>
            )}
          </div>
        ) : creating ? (
          // Create Form
          <div className="max-w-3xl mx-auto w-full p-8 space-y-6 overflow-y-auto">
            <h2 className="text-2xl font-semibold text-claude-text">Create new skill</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-claude-textSecondary mb-1.5">Name</label>
                <input
                  value={editName} onChange={e => setEditName(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg border border-claude-border bg-transparent text-claude-text outline-none focus:border-blue-500 transition-colors"
                  placeholder="e.g. code-reviewer"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-claude-textSecondary mb-1.5">Description</label>
                <input
                  value={editDesc} onChange={e => setEditDesc(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg border border-claude-border bg-transparent text-claude-text outline-none focus:border-blue-500 transition-colors"
                  placeholder="Brief description of what this skill does"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-claude-textSecondary mb-1.5">Content</label>
                <textarea
                  value={editContent} onChange={e => setEditContent(e.target.value)} rows={15}
                  className="w-full px-3 py-2 rounded-lg border border-claude-border bg-transparent text-claude-text font-mono text-sm outline-none focus:border-blue-500 transition-colors resize-y"
                  placeholder="# Skill Title\n\nInstructions for Claude..."
                />
              </div>
              <div className="flex gap-3 pt-2">
                <button onClick={handleCreate} disabled={saving || !editName.trim()}
                  className="px-4 py-2 rounded-lg bg-blue-600 text-white font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors">
                  {saving ? 'Creating...' : 'Create Skill'}
                </button>
                <button onClick={() => setCreating(false)}
                  className="px-4 py-2 rounded-lg border border-claude-border text-claude-text hover:bg-claude-hover transition-colors">
                  Cancel
                </button>
              </div>
            </div>
          </div>
        ) : detail ? (
          // Detail View
          <div className="flex flex-col h-full overflow-hidden">
            {/* Header */}
            <div className="flex-shrink-0 px-8 py-6 border-b border-transparent">
              <div className="flex items-start justify-between mb-4">
                <h2 className="text-xl font-bold text-claude-text">{detail.name}</h2>
                <div className="flex items-center gap-4">
                  <ToggleSwitch
                    enabled={detail.enabled}
                    onToggle={(e) => handleToggle(detail.id, detail.enabled, e)}
                  />
                  <button className="text-claude-textSecondary hover:text-claude-text">
                    <MoreHorizontal size={20} />
                  </button>
                </div>
              </div>

              <div className="space-y-4">
                {detail.is_example && (
                  <div className="text-sm">
                    <span className="text-claude-textSecondary">Added by</span>
                    <div className="font-medium text-claude-text mt-0.5">Anthropic</div>
                  </div>
                )}

                <div>
                  <div className="flex items-center gap-1.5 text-sm text-claude-textSecondary mb-1">
                    Description
                    <Info size={14} />
                  </div>
                  <p className="text-sm text-claude-text leading-relaxed">
                    {detail.description || "No description provided."}
                  </p>
                </div>
              </div>
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto px-8 pb-8">
              <div className="border border-claude-border rounded-xl bg-white dark:bg-[#30302E] overflow-hidden shadow-sm">
                <div className="flex items-center justify-end px-4 py-2 border-b border-claude-border bg-claude-bg/50">
                  <button
                    onClick={() => setViewMode(viewMode === 'preview' ? 'code' : 'preview')}
                    className="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-claude-textSecondary hover:bg-claude-hover rounded-md transition-colors"
                  >
                    {viewMode === 'preview' ? <Code size={14} /> : <Eye size={14} />}
                    {viewMode === 'preview' ? 'Code' : 'Preview'}
                  </button>
                </div>

                <div className="p-6 min-h-[400px]">
                  {viewMode === 'preview' ? (
                    <div className="prose prose-sm max-w-none dark:prose-invert">
                      <MarkdownRenderer content={selectedFile === 'SKILL.md' ? (detail.content || '') : (fileContent || '')} />
                    </div>
                  ) : (
                    <div className="mx-auto w-full">
                      <CodeBlock
                        className="!border-transparent !bg-transparent !my-0"
                        language={
                          selectedFile.endsWith('.py') ? 'python' :
                            selectedFile.endsWith('.js') || selectedFile.endsWith('.jsx') ? 'javascript' :
                              selectedFile.endsWith('.ts') || selectedFile.endsWith('.tsx') ? 'typescript' :
                                selectedFile.endsWith('.css') ? 'css' :
                                  selectedFile.endsWith('.html') ? 'html' :
                                    selectedFile.endsWith('.json') ? 'json' :
                                      selectedFile.endsWith('.sh') ? 'bash' : 'text'
                        }
                        code={selectedFile === 'SKILL.md' ? (detail.content || '') : (fileContent || '')}
                      />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-claude-textSecondary">
            <div className="text-center">
              <Sparkles size={32} className="mx-auto mb-3 opacity-20" />
              <p>Select a skill to view details</p>
            </div>
          </div>
        )}
      </div>

      {showUploadModal && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
          <div className="bg-[#242424] w-[460px] rounded-[16px] flex flex-col shadow-[0_10px_40px_rgba(0,0,0,0.5)] relative border border-white/10 overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between px-6 pt-6 pb-4">
              <h3 className="text-[16px] font-semibold text-white tracking-wide">Upload skill</h3>
              <button
                onClick={() => setShowUploadModal(false)}
                className="text-white/50 hover:text-white/80 transition-colors"
              >
                <X size={18} strokeWidth={2} />
              </button>
            </div>

            {/* Dropzone */}
            <div className="mx-6 mb-6 p-8 border border-dashed border-white/20 rounded-xl flex flex-col items-center justify-center bg-white/[0.02] cursor-pointer hover:bg-white/[0.05] transition-colors group">
              <FolderPlus size={22} className="text-white/50 group-hover:text-white/70 transition-colors mb-3 stroke-[1.5]" />
              <div className="text-[13px] text-white/50 group-hover:text-white/70 transition-colors">
                Drag and drop or click to upload
              </div>
            </div>

            {/* Requirements Details */}
            <div className="px-6 pb-8">
              <div className="text-[12px] font-medium text-white/60 mb-2.5">File requirements</div>
              <ul className="list-disc pl-4 text-[12px] text-white/50 space-y-1.5 mb-5 marker:text-white/30">
                <li>.md file must contain skill name and description formatted in YAML</li>
                <li>.zip or .skill file must include a SKILL.md file</li>
              </ul>
              <div className="text-[12px] text-white/50">
                <a href="#" className="underline hover:text-white/70 underline-offset-2 transition-colors">Read more about creating skills</a>
                <span> or </span>
                <a href="#" className="underline hover:text-white/70 underline-offset-2 transition-colors">see an example</a>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default CustomizePage;
