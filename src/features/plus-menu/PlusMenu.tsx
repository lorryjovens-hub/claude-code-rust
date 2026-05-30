import { Paperclip, Github, ChevronDown, Plus, FileText, Check, ListCollapse } from 'lucide-react';
import { IconProjects, IconResearch, IconWebSearch } from '../../components/Icons';
import { Project } from '../../api';

interface PlusMenuProps {
  menuRef: React.RefObject<HTMLDivElement | null>;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  projectList: any[];
  activeId: string | null;
  currentProjectId: string | null;
  pendingProjectId: string | null;
  enabledSkills: any[];
  researchMode: boolean;
  currentProviderSupportsWebSearch: boolean;
  showProjectsSubmenu: boolean;
  showSkillsSubmenu: boolean;
  isExistingChat: boolean;
  compactDisabled: boolean;
  onClose: () => void;
  onAddAttachment: () => void;
  onAddFromGithub: () => void;
  onAttachToProject: (p: Project) => void;
  onCreateProject: () => void;
  onSelectSkill: (skill: any) => void;
  onManageSkills: () => void;
  onToggleResearch: () => void;
  onWebSearch: () => void;
  onCompact: () => void;
  setShowProjectsSubmenu: (v: boolean) => void;
  setShowSkillsSubmenu: (v: boolean) => void;
  t: (key: string) => string;
}

export default function PlusMenu({
  menuRef, fileInputRef, projectList, activeId, currentProjectId, pendingProjectId,
  enabledSkills, researchMode, currentProviderSupportsWebSearch,
  showProjectsSubmenu, showSkillsSubmenu, isExistingChat, compactDisabled,
  onClose, onAddAttachment, onAddFromGithub, onAttachToProject,
  onCreateProject, onSelectSkill, onManageSkills, onToggleResearch,
  onWebSearch, onCompact, setShowProjectsSubmenu, setShowSkillsSubmenu, t,
}: PlusMenuProps) {
  const clearSubs = () => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); };

  return (
    <div ref={menuRef} className="absolute bottom-full left-0 mb-2 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50">
      <button
        onMouseEnter={clearSubs}
        onClick={onAddAttachment}
        className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
      >
        <Paperclip size={16} className="text-claude-textSecondary" />
        {t('chat.addAttachment')}
      </button>
      <button
        onMouseEnter={clearSubs}
        onClick={onAddFromGithub}
        className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
      >
        <Github size={16} className="text-claude-textSecondary" />
        Add from GitHub
      </button>

      {/* Add to project submenu */}
      <div className="relative" onMouseLeave={() => setShowProjectsSubmenu(false)}>
        <button
          onMouseEnter={() => { setShowProjectsSubmenu(true); setShowSkillsSubmenu(false); }}
          onClick={() => setShowProjectsSubmenu(!showProjectsSubmenu)}
          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
        >
          <div className="flex items-center gap-3">
            <IconProjects size={16} className="text-claude-textSecondary scale-[1.6] dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
            {t('projects.addToProject')}
          </div>
          <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
        </button>
        {showProjectsSubmenu && (
          <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
            {projectList.length > 0 ? projectList.map(p => {
              const isSelected = (activeId && currentProjectId === p.id) || (!activeId && pendingProjectId === p.id);
              return (
                <button
                  key={p.id}
                  onClick={() => onAttachToProject(p as any)}
                  className="w-full flex items-center justify-between gap-2 px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors text-left"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <IconProjects size={26} className="text-claude-textSecondary flex-shrink-0 dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
                    <span className="truncate">{p.name}</span>
                  </div>
                  {isSelected && <Check size={14} className="text-claude-textSecondary flex-shrink-0" />}
                </button>
              );
            }) : (
              <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">{t('projects.noProjectsYet')}</div>
            )}
            <div className="border-t border-claude-border mt-1 pt-1">
              <button
                onClick={onCreateProject}
                className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
              >
                <Plus size={14} />
                {t('projects.createNewProject')}
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Skills submenu */}
      <div className="relative" onMouseLeave={() => setShowSkillsSubmenu(false)}>
        <button
          onMouseEnter={() => { setShowSkillsSubmenu(true); setShowProjectsSubmenu(false); }}
          onClick={() => setShowSkillsSubmenu(!showSkillsSubmenu)}
          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
        >
          <div className="flex items-center gap-3">
            <FileText size={16} className="text-claude-textSecondary" />
            Skills
          </div>
          <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
        </button>
        {showSkillsSubmenu && (
          <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
            {enabledSkills.length > 0 ? enabledSkills.map(skill => (
              <button
                key={skill.id}
                onClick={() => onSelectSkill(skill)}
                className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors truncate"
              >
                {skill.name}
              </button>
            )) : (
              <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No skills enabled</div>
            )}
            <div className="border-t border-claude-border mt-1 pt-1">
              <button
                onClick={onManageSkills}
                className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
              >
                <FileText size={14} />
                Manage skills
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Compact conversation (existing chats only) */}
      {isExistingChat && (
        <button
          onMouseEnter={clearSubs}
          onClick={onCompact}
          disabled={compactDisabled}
          className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors disabled:opacity-40"
        >
          <ListCollapse size={16} className="text-claude-textSecondary" />
          Compact conversation
        </button>
      )}

      {/* Research toggle */}
      <div className="border-t border-claude-border mt-1 pt-1">
        <button
          onMouseEnter={clearSubs}
          onClick={onToggleResearch}
          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
        >
          <div className="flex items-center gap-3">
            <IconResearch size={16} className={researchMode ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
            <span className={researchMode ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>Research</span>
          </div>
          {researchMode && <Check size={14} className="text-[#2E7CF6]" />}
        </button>
      </div>

      {/* Web search indicator */}
      <div>
        <button
          onMouseEnter={clearSubs}
          onClick={onWebSearch}
          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
        >
          <div className="flex items-center gap-3">
            <IconWebSearch size={16} className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
            <span className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>{t('chat.webSearch')}</span>
          </div>
          {currentProviderSupportsWebSearch && <Check size={14} className="text-[#2E7CF6]" />}
        </button>
      </div>
    </div>
  );
}
