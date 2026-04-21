import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Search, Plus, ChevronDown, ArrowLeft, MoreVertical, Star, ArrowUp, FileText, Trash, Pencil, MessageSquare, X, Upload, Check, AudioLines, ChevronRight, Archive } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { Paperclip, ListCollapse } from 'lucide-react';
import { getProjects, createProject, getProject, updateProject, deleteProject, uploadProjectFile, deleteProjectFile, createProjectConversation, deleteConversation, getSkills, Project, ProjectFile } from '../api';
import ModelSelector, { SelectableModel } from './ModelSelector';
import { IconPlus } from './Icons';
import startProjectsImg from '../assets/icons/start-projects.png';

const ProjectsPage = () => {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [projectDescription, setProjectDescription] = useState('');
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentProject, setCurrentProject] = useState<any>(null);
  const [editingInstructions, setEditingInstructions] = useState(false);
  const [instructionsText, setInstructionsText] = useState('');
  const [uploading, setUploading] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const [editingName, setEditingName] = useState(false);
  const [editName, setEditName] = useState('');
  const [sortMenuOpen, setSortMenuOpen] = useState(false);
  const [sortBy, setSortBy] = useState<'activity' | 'edited' | 'created'>('activity');
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [projectToDelete, setProjectToDelete] = useState<Project | null>(null);
  const [projectToEdit, setProjectToEdit] = useState<Project | null>(null);
  const [editDetailsName, setEditDetailsName] = useState('');
  const [editDetailsDesc, setEditDetailsDesc] = useState('');
  const [message, setMessage] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [showSkillsSubmenu, setShowSkillsSubmenu] = useState(false);
  const [enabledSkills, setEnabledSkills] = useState<Array<{ id: string; name: string; description?: string }>>([]);
  const [selectedSkill, setSelectedSkill] = useState<{ name: string; slug: string; description?: string } | null>(null);
  const plusMenuRef = useRef<HTMLDivElement>(null);
  const plusBtnRef = useRef<HTMLButtonElement>(null);

  // Model selector state — load from self-hosted config or use defaults
  const isSelfHostedMode = localStorage.getItem('user_mode') === 'selfhosted';
  const selectorModels = useMemo<SelectableModel[]>(() => {
    if (isSelfHostedMode) {
      try {
        const chatModels = JSON.parse(localStorage.getItem('chat_models') || '[]');
        if (chatModels.length > 0) {
          const tierDescMap: Record<string, string> = {
            'opus': 'Most capable for ambitious work',
            'sonnet': 'Most efficient for everyday tasks',
            'haiku': 'Fastest for quick answers',
          };
          return chatModels.map((m: any) => ({
            id: m.id,
            name: m.name || m.id,
            enabled: 1,
            tier: m.tier || 'extra',
            description: m.tier && tierDescMap[m.tier] ? tierDescMap[m.tier] : undefined,
          }));
        }
      } catch (_) { }
    }
    return [
      { id: 'claude-opus-4-6', name: 'Opus 4.6', enabled: 1, description: 'Most capable for ambitious work' },
      { id: 'claude-sonnet-4-6', name: 'Sonnet 4.6', enabled: 1, description: 'Most efficient for everyday tasks' },
      { id: 'claude-haiku-4-5-20251001', name: 'Haiku 4.5', enabled: 1, description: 'Fastest for quick answers' },
    ];
  }, [isSelfHostedMode]);
  const [currentModelString, setCurrentModelString] = useState(localStorage.getItem('default_model') || 'claude-sonnet-4-6');
  const handleModelChange = (newModelString: string) => {
    setCurrentModelString(newModelString);
  };

  const handleChatSubmit = async () => {
    if (!message.trim() || !currentProject) return;
    try {
      const conv = await createProjectConversation(currentProject.id, message.slice(0, 50), currentModelString);
      navigate(`/chat/${conv.id}`, { state: { initialMessage: message, model: currentModelString } });
      setMessage('');
    } catch (err) {
      console.error(err);
    }
  };

  const loadProjects = useCallback(async () => {
    try {
      const data = await getProjects();
      setProjects(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error('Failed to load projects:', err);
      setProjects([]);
    }
    setLoading(false);
  }, []);

  useEffect(() => { loadProjects(); }, [loadProjects]);

  // Load skills when plus menu opens
  useEffect(() => {
    if (!showPlusMenu) { setShowSkillsSubmenu(false); return; }
    getSkills().then((data: any) => {
      const all = [...(data.examples || []), ...(data.my_skills || [])];
      setEnabledSkills(all.filter((s: any) => s.enabled).map((s: any) => ({ id: s.id, name: s.name, description: s.description })));
    }).catch(() => {});
  }, [showPlusMenu]);

  // Close plus menu on outside click
  useEffect(() => {
    if (!showPlusMenu) return;
    const handleClick = (e: MouseEvent) => {
      if (plusMenuRef.current && !plusMenuRef.current.contains(e.target as Node) &&
        plusBtnRef.current && !plusBtnRef.current.contains(e.target as Node)) {
        setShowPlusMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [showPlusMenu]);

  const loadProject = useCallback(async (id: string) => {
    try {
      const data = await getProject(id);
      setCurrentProject(data);
      setInstructionsText(data.instructions || '');
    } catch (_) { }
  }, []);

  const handleCreate = async () => {
    const name = projectName.trim() || 'Untitled Project';
    try {
      const project = await createProject(name, projectDescription.trim());
      setIsCreating(false);
      setProjectName('');
      setProjectDescription('');
      loadProject(project.id);
      loadProjects();
    } catch (_) { }
  };

  const handleDelete = async () => {
    if (!currentProject) return;
    if (!window.confirm(`确定要删除项目「${currentProject.name}」吗？所有关联的文件和对话也会被删除。`)) return;
    try {
      await deleteProject(currentProject.id);
      setCurrentProject(null);
      setShowMenu(false);
      loadProjects();
    } catch (_) { }
  };

  const handleDeleteProject = async (p: Project) => {
    try {
      await deleteProject(p.id);
      if (currentProject && currentProject.id === p.id) {
        setCurrentProject(null);
      }
      setProjectToDelete(null);
      loadProjects();
    } catch (_) { }
  };

  const handleSaveEditDetails = async () => {
    if (!projectToEdit) return;
    try {
      await updateProject(projectToEdit.id, {
        name: editDetailsName,
        description: editDetailsDesc
      });
      setProjectToEdit(null);
      loadProjects();
      if (currentProject && currentProject.id === projectToEdit.id) {
        loadProject(currentProject.id);
      }
    } catch (_) { }
  };

  const handleSaveInstructions = async () => {
    if (!currentProject) return;
    await updateProject(currentProject.id, { instructions: instructionsText });
    setEditingInstructions(false);
    loadProject(currentProject.id);
  };

  const handleFileUpload = async (files: FileList | File[]) => {
    if (!currentProject) return;
    setUploading(true);
    for (const file of Array.from(files)) {
      try {
        await uploadProjectFile(currentProject.id, file);
      } catch (_) { }
    }
    setUploading(false);
    loadProject(currentProject.id);
  };

  const handleDeleteFile = async (fileId: string) => {
    if (!currentProject) return;
    await deleteProjectFile(currentProject.id, fileId);
    loadProject(currentProject.id);
  };

  const handleNewChat = async () => {
    if (!currentProject) return;
    try {
      const conv = await createProjectConversation(currentProject.id);
      navigate(`/chat/${conv.id}`);
    } catch (_) { }
  };

  const handleDeleteConversation = async (convId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!currentProject) return;
    try {
      await deleteConversation(convId);
      loadProject(currentProject.id);
      loadProjects(); // refresh chat_count
    } catch (_) { }
  };

  const handleRenameSave = async () => {
    if (!currentProject || !editName.trim()) return;
    await updateProject(currentProject.id, { name: editName.trim() });
    setEditingName(false);
    loadProject(currentProject.id);
    loadProjects();
  };

  const filteredProjects = useMemo(() => {
    const filtered = projects.filter(p =>
      (p.name || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (p.description || '').toLowerCase().includes(searchQuery.toLowerCase())
    );
    return [...filtered].sort((a, b) => {
      if (sortBy === 'created') return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      // 'activity' and 'edited' both sort by updated_at
      return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
    });
  }, [projects, searchQuery, sortBy]);

  // ═══ Project Detail View ═══
  if (currentProject) {
    return (
      <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
        <div className="max-w-[800px] mx-auto px-8 py-12">
          <div className="mb-4">
            <button
              onClick={() => { setCurrentProject(null); loadProjects(); }}
              className="flex items-center gap-1.5 text-[14px] text-claude-textSecondary hover:text-claude-text transition-colors font-medium -ml-1"
            >
              <ArrowLeft size={16} />
              All projects
            </button>
          </div>

          <div className="flex items-start justify-between mb-8 gap-4">
            <div className="flex-1 min-w-0">
              {editingName ? (
                <div className="flex items-center gap-2">
                  <input
                    autoFocus
                    value={editName}
                    onChange={e => setEditName(e.target.value)}
                    onKeyDown={e => { if (e.key === 'Enter') handleRenameSave(); if (e.key === 'Escape') setEditingName(false); }}
                    className="font-[Spectral] text-[32px] text-claude-text bg-transparent border-b-2 border-claude-accent outline-none w-full"
                    style={{ fontWeight: 500 }}
                  />
                </div>
              ) : (
                <h1
                  className="font-[Spectral] text-[32px] text-claude-text leading-tight mb-2"
                  style={{ fontWeight: 500 }}
                >
                  {currentProject.name}
                </h1>
              )}
              {currentProject.description && (
                <p className="text-[15.5px] text-claude-textSecondary">{currentProject.description}</p>
              )}
            </div>
            <div className="flex items-center gap-1 text-claude-textSecondary mt-2 flex-shrink-0">
              <button className="p-1 hover:text-claude-text hover:bg-black/5 dark:hover:bg-white/5 rounded-md transition-colors"><MoreVertical size={18} /></button>
            </div>
          </div>

          <div className="space-y-4">
            {/* Chat Input Container — matches MainContent new chat input */}
            <div
              className="bg-claude-input border border-claude-border dark:border-[#3a3a38] shadow-[0_2px_8px_rgba(0,0,0,0.02)] hover:shadow-[0_2px_8px_rgba(0,0,0,0.08)] hover:border-[#CCC] dark:hover:border-[#5a5a58] focus-within:shadow-[0_2px_8px_rgba(0,0,0,0.08)] focus-within:border-[#CCC] dark:focus-within:border-[#5a5a58] transition-all duration-200 flex flex-col max-h-[60vh] font-sans rounded-2xl"
            >
              <div className="flex-1 overflow-y-auto min-h-0">
                <div className="relative">
                  {/* Skill overlay */}
                  {message.match(/^\/[a-zA-Z0-9_-]+/) && (
                    <div className="pl-5 pr-4 pt-5 pb-1 text-[16px] font-sans font-[350]" style={{ minHeight: '48px', position: 'absolute', top: 0, left: 0, right: 0, pointerEvents: 'none', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }} aria-hidden>
                      {(() => { const m = message.match(/^(\/[a-zA-Z0-9_-]+)([\s\S]*)$/); return m ? <><span className="text-[#4B9EFA]">{m[1]}</span><span className="text-claude-text">{m[2]}</span></> : null; })()}
                    </div>
                  )}
                  <textarea
                    ref={textareaRef}
                    className={`w-full pl-5 pr-4 pt-5 pb-1 placeholder:text-claude-textSecondary text-[16px] outline-none resize-none overflow-hidden bg-transparent font-sans font-[350] ${message.match(/^\/[a-zA-Z0-9_-]+/) ? 'text-transparent caret-claude-text' : 'text-claude-text'}`}
                    style={{ minHeight: '48px', borderRadius: '16px 16px 0 0' }}
                    placeholder={selectedSkill ? `Describe what you want ${selectedSkill.name} to do...` : "How can I help you today?"}
                    value={message}
                    onChange={(e) => {
                      setMessage(e.target.value);
                      e.target.style.height = 'auto';
                      e.target.style.height = Math.min(e.target.scrollHeight, 300) + 'px';
                      e.target.style.overflowY = e.target.scrollHeight > 300 ? 'auto' : 'hidden';
                    }}
                    onKeyDown={e => {
                      if (e.key === 'Backspace' && selectedSkill) {
                        const pos = (e.target as HTMLTextAreaElement).selectionStart;
                        const prefix = `/${selectedSkill.slug} `;
                        if (pos > 0 && pos <= prefix.length && message.startsWith(prefix.slice(0, pos))) {
                          e.preventDefault();
                          setMessage(message.slice(prefix.length));
                          setSelectedSkill(null);
                          return;
                        }
                      }
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleChatSubmit();
                      }
                    }}
                  />
                </div>
              </div>
              <div className="px-4 pb-3 pt-1 flex items-center justify-between flex-shrink-0">
                <div className="relative flex items-center">
                  <button
                    ref={plusBtnRef}
                    onClick={() => setShowPlusMenu(prev => !prev)}
                    className="p-2 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
                  >
                    <IconPlus size={20} />
                  </button>
                  {showPlusMenu && (
                    <div ref={plusMenuRef} className="absolute bottom-full left-0 mb-2 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50">
                      <button onClick={() => { setShowPlusMenu(false); fileInputRef.current?.click(); }} className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors">
                        <Paperclip size={16} className="text-claude-textSecondary" />
                        Add files or photos
                      </button>
                      <div className="relative">
                        <button onMouseEnter={() => setShowSkillsSubmenu(true)} onClick={() => setShowSkillsSubmenu(p => !p)} className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors">
                          <div className="flex items-center gap-3"><FileText size={16} className="text-claude-textSecondary" />Skills</div>
                          <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
                        </button>
                        {showSkillsSubmenu && (
                          <div className="absolute left-full bottom-0 ml-1 w-[200px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[300px] overflow-y-auto" onMouseLeave={() => setShowSkillsSubmenu(false)}>
                            {enabledSkills.length > 0 ? enabledSkills.map(skill => (
                              <button key={skill.id} onClick={() => {
                                setShowPlusMenu(false); setShowSkillsSubmenu(false);
                                const slug = skill.name.toLowerCase().replace(/\s+/g, '-');
                                setSelectedSkill({ name: skill.name, slug, description: skill.description });
                                setMessage(prev => prev ? `/${slug} ${prev}` : `/${slug} `);
                                textareaRef.current?.focus();
                              }} className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors truncate">{skill.name}</button>
                            )) : <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No skills enabled</div>}
                            <div className="border-t border-claude-border mt-1 pt-1">
                              <button onClick={() => { setShowPlusMenu(false); window.location.hash = '#/customize'; }} className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"><FileText size={14} />Manage skills</button>
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-3">
                  <ModelSelector
                    currentModelString={currentModelString}
                    models={selectorModels}
                    onModelChange={handleModelChange}
                    isNewChat={true}
                  />
                  <button
                    onClick={handleChatSubmit}
                    disabled={!message.trim()}
                    className="p-2 bg-[#C6613F] text-white rounded-lg hover:bg-[#D97757] transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                  >
                    <ArrowUp size={22} strokeWidth={2.5} />
                  </button>
                </div>
              </div>
            </div>

            {/* Conversation List / Banner */}
            {currentProject.conversations && currentProject.conversations.length > 0 ? (
              <div className="border border-claude-border rounded-[16px] overflow-hidden bg-transparent mt-2">
                <div className="px-5 py-3 text-[13px] font-medium text-claude-textSecondary border-b border-claude-border">
                  {currentProject.conversations.length} conversation{currentProject.conversations.length > 1 ? 's' : ''}
                </div>
                {currentProject.conversations.map((conv: any) => (
                  <div
                    key={conv.id}
                    onClick={() => navigate(`/chat/${conv.id}`)}
                    className="px-5 py-3 flex items-center gap-3 hover:bg-claude-hover cursor-pointer border-b border-claude-border last:border-b-0 transition-colors group"
                  >
                    <MessageSquare size={16} className="text-claude-textSecondary flex-shrink-0" />
                    <span className="text-[14px] text-claude-text truncate">{conv.title}</span>
                    <span className="text-[12px] text-claude-textSecondary ml-auto flex-shrink-0">
                      {new Date(conv.created_at).toLocaleDateString()}
                    </span>
                    <button
                      onClick={(e) => handleDeleteConversation(conv.id, e)}
                      className="p-1 text-claude-textSecondary hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
                      title="Delete conversation"
                    >
                      <Trash size={14} />
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="w-full border border-claude-border rounded-[16px] px-6 py-10 flex items-center justify-center bg-transparent mt-2">
                <span className="text-[14.5px] text-[#A1A1AA]">
                  Start a chat to keep conversations organized and re-use project knowledge.
                </span>
              </div>
            )}

            {/* Instructions and Files */}
            <div className="w-full border border-claude-border rounded-[16px] overflow-hidden bg-transparent mt-2">
              {/* Instructions Header */}
              <div
                className="p-5 border-b border-claude-border hover:bg-black/[0.015] dark:hover:bg-white/[0.015] transition-colors cursor-pointer group"
                onClick={() => { if (!editingInstructions) setEditingInstructions(true); }}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <h3 className="font-semibold text-claude-text mb-0.5" style={{ fontSize: '15.5px' }}>Instructions</h3>
                    {!editingInstructions && (
                      <p className="text-[13px] text-[#A1A1AA]">
                        {currentProject.instructions
                          ? currentProject.instructions.slice(0, 200) + (currentProject.instructions.length > 200 ? '...' : '')
                          : "Add instructions to tailor Claude's responses"}
                      </p>
                    )}
                  </div>
                  {!editingInstructions && (
                    <button className="text-[#A1A1AA] hover:text-claude-text transition-colors">
                      {currentProject.instructions ? <Pencil size={18} strokeWidth={1.5} /> : <Plus size={22} strokeWidth={1.5} />}
                    </button>
                  )}
                </div>
                {editingInstructions && (
                  <div
                    className="fixed inset-0 z-50 flex items-center justify-center bg-black/70"
                    onClick={() => { setEditingInstructions(false); setInstructionsText(currentProject.instructions || ''); }}
                  >
                    <div
                      className="w-full max-w-[800px] bg-white dark:bg-[#2A2928] border border-claude-border rounded-[20px] shadow-2xl p-7"
                      onClick={e => e.stopPropagation()}
                    >
                      <h2 className="text-[20px] font-bold text-claude-text mb-2">Set project instructions</h2>
                      <p className="text-[14px] text-[#A1A1AA] mb-5">
                        Provide Claude with relevant instructions and information for chats within {currentProject.name}. This will work alongside <span className="underline decoration-[#555] underline-offset-2 cursor-pointer hover:text-claude-text">user preferences</span> and the selected style in a chat.
                      </p>

                      <textarea
                        autoFocus
                        value={instructionsText}
                        onChange={e => setInstructionsText(e.target.value)}
                        placeholder="Break down large tasks and ask clarifying questions when needed."
                        className="w-full h-[400px] px-4 py-3 bg-claude-bg dark:bg-[#202020] border border-claude-border rounded-[12px] text-[15px] text-claude-text resize-none outline-none focus:border-[#3A7ADA] focus:ring-1 focus:ring-[#3A7ADA] transition-colors"
                      />

                      <div className="flex justify-end gap-3 mt-5">
                        <button
                          onClick={() => { setEditingInstructions(false); setInstructionsText(currentProject.instructions || ''); }}
                          className="px-4 py-2 text-[14px] font-medium text-claude-text hover:bg-white/5 border border-transparent hover:border-claude-border rounded-xl transition-all"
                        >
                          Cancel
                        </button>
                        <button
                          onClick={handleSaveInstructions}
                          className="px-4 py-2 text-[14px] font-medium bg-[#E6E6E6] text-[#222] rounded-xl hover:opacity-90 transition-opacity"
                        >
                          Save instructions
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>

              {/* Files */}
              <div className="p-5 pb-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-semibold text-claude-text" style={{ fontSize: '15.5px' }}>
                    Files {currentProject.files?.length > 0 && <span className="text-claude-textSecondary text-[13px] ml-1">({currentProject.files.length})</span>}
                  </h3>
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    className="text-[#A1A1AA] hover:text-claude-text transition-colors"
                  >
                    <Plus size={22} strokeWidth={1.5} />
                  </button>
                  <input
                    ref={fileInputRef}
                    type="file"
                    multiple
                    className="hidden"
                    onChange={e => { if (e.target.files) handleFileUpload(e.target.files); e.target.value = ''; }}
                  />
                </div>

                {uploading && (
                  <div className="text-[13px] text-claude-textSecondary animate-pulse mb-3">Uploading...</div>
                )}

                {currentProject.files && currentProject.files.length > 0 ? (
                  <div className="space-y-2">
                    {currentProject.files.map((f: ProjectFile) => (
                      <div key={f.id} className="flex items-center gap-3 px-3 py-2.5 rounded-[12px] bg-black/[0.02] dark:bg-white/[0.03] group border border-transparent hover:border-claude-border transition-all">
                        <FileText size={16} className="text-[#A1A1AA] flex-shrink-0" />
                        <div className="flex-1 min-w-0">
                          <div className="text-[13.5px] text-claude-text truncate font-medium">{f.file_name}</div>
                          <div className="text-[11.5px] text-[#A1A1AA]">
                            {f.file_size > 1024 * 1024 ? `${(f.file_size / 1024 / 1024).toFixed(1)} MB` : `${(f.file_size / 1024).toFixed(1)} KB`}
                          </div>
                        </div>
                        <button
                          onClick={() => handleDeleteFile(f.id)}
                          className="p-1 text-[#A1A1AA] hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                        >
                          <X size={16} />
                        </button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div
                    className="w-full bg-[#FAFAFA] dark:bg-[#191919] rounded-[16px] flex flex-col items-center justify-center py-8 border border-transparent dark:border-white/[0.04] cursor-pointer hover:bg-[#F3F3F3] dark:hover:bg-[#222222] transition-colors"
                    onClick={() => fileInputRef.current?.click()}
                    onDragOver={e => { e.preventDefault(); e.stopPropagation(); }}
                    onDrop={e => { e.preventDefault(); e.stopPropagation(); if (e.dataTransfer.files.length) handleFileUpload(e.dataTransfer.files); }}
                  >
                    <div className="flex items-center justify-center mb-3">
                      <div className="w-[84px] h-[48px] relative opacity-60 mix-blend-luminosity grayscale">
                        <div className="absolute right-[4px] bottom-0 w-[28px] h-[36px] bg-[#3B3B3B] border border-[#555] rounded-[4px] flex flex-col items-center py-1.5 px-1 gap-[3px] shadow-sm transform translate-x-2 translate-y-2 -rotate-12 z-0">
                          <div className="w-full h-[1.5px] bg-[#666] rounded-full mx-1"></div>
                          <div className="w-3/4 h-[1.5px] bg-[#666] rounded-full mx-1 self-start"></div>
                        </div>
                        <div className="absolute left-[4px] bottom-0 w-[28px] h-[36px] bg-[#3B3B3B] border border-[#555] rounded-[4px] flex flex-col items-center py-1.5 px-1 gap-[3px] shadow-sm transform -translate-x-2 translate-y-1 rotate-12 z-0">
                          <div className="w-full h-[1.5px] bg-[#666] rounded-full mx-1"></div>
                          <div className="w-full h-[1.5px] bg-[#666] rounded-full mx-1"></div>
                          <div className="w-1/2 h-[1.5px] bg-[#666] rounded-full mx-1 self-start"></div>
                        </div>
                        <div className="absolute left-1/2 bottom-0 -translate-x-1/2 w-[34px] h-[42px] bg-[#444] border border-[#666] rounded-[6px] shadow-md flex flex-col items-center py-2 px-1.5 gap-[4px] z-10">
                          <div className="w-[12px] h-[12px] bg-[#555] rounded-sm flex items-center justify-center self-end mb-0.5"><Plus size={8} className="text-white" /></div>
                          <div className="w-full h-[2px] bg-[#888] rounded-full mx-1"></div>
                          <div className="w-full h-[2px] bg-[#888] rounded-full mx-1"></div>
                          <div className="w-2/3 h-[2px] bg-[#888] rounded-full mx-1 self-start"></div>
                        </div>
                      </div>
                    </div>
                    <span className="text-[13px] text-[#A1A1AA] text-center max-w-[200px] leading-relaxed">
                      Add PDFs, documents, or other text to reference in this project.
                    </span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // ═══ Create View ═══
  if (isCreating) {
    return (
      <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
        <div className="max-w-[560px] mx-auto px-8 pt-12 pb-8">
          <h1 className="font-[Spectral] text-[32px] text-claude-text mb-6" style={{ fontWeight: 600 }}>
            Create a personal project
          </h1>

          <div className="bg-[#EFEEE7] dark:bg-[#2A2928] rounded-2xl p-6 mb-6 border border-transparent dark:border-white/5">
            <h3 className="font-semibold text-claude-text text-[15.5px] mb-2 text-[#403A35] dark:text-[#E3E0D8]">How to use projects</h3>
            <p className="text-[14.5px] leading-relaxed text-[#564E48] dark:text-[#A8A096] mb-3">
              Projects help organize your work and leverage knowledge across multiple conversations. Upload docs, code, and files to create themed collections that Claude can reference again and again.
            </p>
            <p className="text-[14.5px] leading-relaxed text-[#564E48] dark:text-[#A8A096]">
              Start by creating a memorable title and description to organize your project. You can always edit it later.
            </p>
          </div>

          <div className="space-y-5">
            <div>
              <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">What are you working on?</label>
              <input
                type="text"
                placeholder="Name your project"
                value={projectName}
                onChange={e => setProjectName(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter' && projectName.trim()) handleCreate(); }}
                className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px]"
              />
            </div>
            <div>
              <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">What are you trying to achieve?</label>
              <textarea
                placeholder="Describe your project, goals, subject, etc..."
                rows={3}
                value={projectDescription}
                onChange={e => setProjectDescription(e.target.value)}
                className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px] resize-none"
              />
            </div>
          </div>

          <div className="flex items-center justify-end gap-3 mt-6">
            <button
              onClick={() => { setIsCreating(false); setProjectName(''); setProjectDescription(''); }}
              className="px-5 py-2.5 text-[15px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-gray-300 dark:border-claude-border hover:bg-gray-50 dark:hover:bg-claude-hover rounded-xl transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleCreate}
              disabled={!projectName.trim()}
              className="px-5 py-2.5 text-[15px] font-medium text-claude-bg bg-black dark:bg-white dark:text-black hover:opacity-90 rounded-xl transition-opacity disabled:opacity-40"
            >
              Create project
            </button>
          </div>
        </div>
      </div>
    );
  }

  // ═══ Projects List View ═══
  return (
    <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
      <div className="max-w-[800px] mx-auto px-8 py-12">
        <div className="flex items-center justify-between mb-8">
          <h1 className="font-[Spectral] text-[32px] text-claude-text" style={{ fontWeight: 500 }}>Projects</h1>
          <button
            onClick={() => setIsCreating(true)}
            className="flex items-center gap-2 px-3.5 py-1.5 bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity font-medium"
            style={{ fontSize: '14px' }}
          >
            <Plus size={16} strokeWidth={2.5} />
            New project
          </button>
        </div>

        {projects.length > 0 && (
          <>
            <div className="relative mb-6">
              <div className="absolute inset-y-0 left-3 flex items-center pointer-events-none">
                <Search className="h-5 w-5 text-claude-textSecondary opacity-80" />
              </div>
              <input
                type="text"
                placeholder="Search projects..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-claude-textSecondary focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all text-[15px]"
              />
            </div>

            <div className="flex justify-end mb-6">
              <div className="flex items-center gap-3 text-[14.5px] text-[#A1A1AA] relative">
                <span>Sort by</span>
                <button
                  onClick={() => setSortMenuOpen(!sortMenuOpen)}
                  className={`flex items-center gap-2 text-claude-text border border-[#3A3A3A] hover:border-[#4A4A4A] dark:border-claude-border dark:hover:bg-claude-hover rounded-[10px] px-3.5 py-1.5 transition-colors ${sortMenuOpen ? 'bg-claude-hover' : ''}`}
                >
                  {sortBy === 'activity' ? 'Activity' : sortBy === 'edited' ? 'Last edited' : 'Date created'}
                  <ChevronDown size={14} className="text-claude-textSecondary" />
                </button>
                {sortMenuOpen && (
                  <>
                    <div className="fixed inset-0 z-40" onClick={() => setSortMenuOpen(false)} />
                    <div className="absolute top-full right-0 mt-1.5 w-[200px] bg-white dark:bg-[#2A2928] border border-gray-200 dark:border-claude-border rounded-[14px] shadow-lg py-1.5 z-50">
                      {[
                        { id: 'activity', label: 'Recent activity' },
                        { id: 'edited', label: 'Last edited' },
                        { id: 'created', label: 'Date created' },
                      ].map(opt => (
                        <button
                          key={opt.id}
                          onClick={() => {
                            setSortBy(opt.id as any);
                            setSortMenuOpen(false);
                          }}
                          className="w-full flex items-center justify-between px-4 py-2.5 text-[15px] text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
                        >
                          {opt.label}
                          {sortBy === opt.id && <Check size={16} className="text-claude-text opacity-80" />}
                        </button>
                      ))}
                    </div>
                  </>
                )}
              </div>
            </div>
          </>
        )}

        {loading ? (
          <div className="text-center text-claude-textSecondary text-[14px] mt-12">Loading...</div>
        ) : filteredProjects.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {filteredProjects.map(p => (
              <div
                key={p.id}
                onClick={() => loadProject(p.id)}
                className="flex flex-col p-5 border border-claude-border rounded-[12px] bg-transparent hover:bg-black/[0.02] dark:hover:bg-white/[0.02] cursor-pointer transition-colors group min-h-[170px]"
              >
                <div className="flex items-center justify-between mb-2.5 relative">
                  <div className="flex items-center gap-3">
                    <h3 className="text-[15.5px] font-medium text-claude-text truncate">{p.name}</h3>
                  </div>
                  <div className="relative" onClick={(e) => e.stopPropagation()}>
                    <button
                      onClick={(e) => { e.stopPropagation(); setActiveMenu(activeMenu === p.id ? null : p.id); }}
                      className={`p-1 text-[#A1A1AA] hover:text-claude-text hover:bg-black/5 dark:hover:bg-white/5 rounded-[6px] transition-all ${activeMenu === p.id ? 'opacity-100 bg-black/5 dark:bg-white/5' : 'opacity-0 group-hover:opacity-100'}`}
                    >
                      <MoreVertical size={18} />
                    </button>

                    {activeMenu === p.id && (
                      <>
                        <div className="fixed inset-0 z-40" onClick={(e) => { e.stopPropagation(); setActiveMenu(null); }} />
                        <div className="absolute top-full right-0 mt-1 w-[180px] bg-white dark:bg-[#30302E] rounded-[16px] shadow-[0_4px_24px_rgba(0,0,0,0.15)] border border-gray-200 dark:border-[#65645F] py-1.5 z-50">
                          <button className="w-full flex items-center gap-3 px-4 py-2.5 text-[14px] text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={(e) => { e.stopPropagation(); setActiveMenu(null); }}>
                            <Star size={16} className="text-claude-textSecondary" />
                            Star
                          </button>
                          <button className="w-full flex items-center gap-3 px-4 py-2.5 text-[14px] text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={(e) => { e.stopPropagation(); setActiveMenu(null); setProjectToEdit(p); setEditDetailsName(p.name); setEditDetailsDesc(p.description || ''); }}>
                            <Pencil size={16} className="text-claude-textSecondary" />
                            Edit details
                          </button>
                          <div className="my-1.5 border-t border-claude-border opacity-50" />
                          <button className="w-full flex items-center gap-3 px-4 py-2.5 text-[14px] text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left" onClick={(e) => { e.stopPropagation(); setActiveMenu(null); }}>
                            <Archive size={16} className="text-claude-textSecondary" />
                            Archive
                          </button>
                          <button className="w-full flex items-center gap-3 px-4 py-2.5 text-[14px] text-[#E05A5A] hover:bg-red-500/10 transition-colors text-left" onClick={(e) => { e.stopPropagation(); setActiveMenu(null); setProjectToDelete(p); }}>
                            <Trash size={16} className="text-[#E05A5A]" />
                            Delete
                          </button>
                        </div>
                      </>
                    )}
                  </div>
                </div>

                <p className="text-[14px] text-claude-textSecondary line-clamp-3 leading-relaxed flex-1">
                  {p.description || "No description provided."}
                </p>

                <div className="mt-4 pt-1 flex items-center gap-4 text-[12px] text-claude-textSecondary/80">
                  <span>Updated {new Date(p.updated_at).toLocaleDateString()}</span>
                  {(p.file_count ?? 0) > 0 && <span>• {p.file_count} files</span>}
                  {(p.chat_count ?? 0) > 0 && <span>• {p.chat_count} chats</span>}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center mt-12">
            <img src={startProjectsImg} alt="Start a project" className="w-[100px] h-auto mb-6 dark:invert opacity-90" />
            <h2 className="text-[17px] font-medium text-claude-text mb-3">Looking to start a project?</h2>
            <p className="text-[15px] text-claude-textSecondary text-center max-w-[400px] leading-relaxed mb-6">
              Upload materials, set custom instructions, and organize conversations in one space.
            </p>
            <button
              onClick={() => setIsCreating(true)}
              className="flex items-center gap-2 px-4 py-2 bg-transparent border border-claude-border hover:bg-claude-hover rounded-xl text-claude-text transition-colors text-[14.5px] font-medium"
            >
              <Plus size={18} strokeWidth={2.5} />
              New project
            </button>
          </div>
        )}
      </div>

      {projectToDelete && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
          <div className="bg-claude-input w-[460px] rounded-[16px] flex flex-col shadow-2xl relative border border-claude-border overflow-hidden">
            <div className="px-6 pt-6 pb-4 text-left">
              <h3 className="text-[19px] font-semibold text-claude-text mb-3">Delete project</h3>
              <p className="text-[15px] text-claude-textSecondary leading-relaxed pr-4">
                确定要删除项目「{projectToDelete.name}」吗？所有关联的文件和对话也会被删除。
              </p>
            </div>
            <div className="px-5 pb-5 pt-2 flex justify-end gap-3 mt-4">
              <button
                onClick={() => setProjectToDelete(null)}
                className="px-5 py-2 text-[14.5px] font-medium text-claude-text border border-claude-border hover:bg-claude-hover rounded-[8px] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => handleDeleteProject(projectToDelete)}
                className="px-5 py-2 text-[14.5px] font-medium text-white bg-[#E05A5A] hover:bg-[#E86B6B] rounded-[8px] transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

      {projectToEdit && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
          <div className="bg-claude-input w-[460px] rounded-[16px] flex flex-col shadow-2xl relative border border-claude-border overflow-hidden">
            <div className="px-6 pt-6 pb-4 text-left">
              <h3 className="text-[19px] font-semibold text-claude-text mb-5">Edit details</h3>

              <div className="space-y-4">
                <div>
                  <label className="block text-[14px] text-claude-textSecondary mb-2 font-medium">Name</label>
                  <input
                    type="text"
                    value={editDetailsName}
                    onChange={(e) => setEditDetailsName(e.target.value)}
                    className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-[8px] text-claude-text outline-none focus:border-[#3A7ADA] focus:ring-1 focus:ring-[#3A7ADA] transition-all text-[15px]"
                    autoFocus
                  />
                </div>
                <div>
                  <label className="block text-[14px] text-claude-textSecondary mb-2 font-medium">Description</label>
                  <textarea
                    value={editDetailsDesc}
                    onChange={(e) => setEditDetailsDesc(e.target.value)}
                    rows={4}
                    className="w-full px-3 py-2 bg-claude-bg border border-claude-border rounded-[8px] text-claude-text outline-none focus:border-[#3A7ADA] focus:ring-1 focus:ring-[#3A7ADA] transition-all resize-none text-[14.5px] leading-relaxed"
                  />
                </div>
              </div>
            </div>

            <div className="px-6 pb-6 pt-2 flex justify-end gap-3 mt-4">
              <button
                onClick={() => setProjectToEdit(null)}
                className="px-5 py-2.5 text-[14.5px] font-medium text-claude-text border border-claude-border hover:bg-claude-hover rounded-[8px] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveEditDetails}
                className="px-5 py-2.5 text-[14.5px] font-medium bg-claude-text text-claude-bg hover:opacity-90 rounded-[8px] transition-opacity"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ProjectsPage;
