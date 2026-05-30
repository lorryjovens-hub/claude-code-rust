import React, { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { Play, GitBranch, ChevronUp, ChevronDown, Terminal, FolderOpen, X, Globe, FileCode } from 'lucide-react';
import ResizableLayout from './ResizableLayout';
import FileExplorer from './FileExplorer';
import MonacoEditor from './MonacoEditor';
import LivePreviewPanel from './LivePreviewPanel';
import TerminalPanel from './TerminalPanel';
import BuildPanel from './BuildPanel';
import WorkspacePanel from './WorkspacePanel';
import GitDiffView from './GitDiffView';
import GitCommitPanel from './GitCommitPanel';
import { readFileContent } from '../api';

type BottomTab = 'terminal' | 'build' | 'gitDiff' | 'gitCommit';
type PreviewFileType = 'html' | 'react' | 'jsx' | 'tsx' | undefined;

function getFileName(filePath: string): string {
  const parts = filePath.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || filePath;
}

function getFileType(filePath: string): PreviewFileType {
  const ext = filePath.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'html':
    case 'htm':
      return 'html';
    case 'jsx':
      return 'jsx';
    case 'tsx':
      return 'tsx';
    default:
      return undefined;
  }
}

function isPreviewable(filePath: string): boolean {
  const ext = filePath.split('.').pop()?.toLowerCase();
  return ['html', 'htm', 'jsx', 'tsx'].includes(ext || '');
}

const MainContent: React.FC = () => {
  const [workspacePath, setWorkspacePath] = useState<string>('');
  const [selectedFilePath, setSelectedFilePath] = useState<string | null>(null);
  const [openTabs, setOpenTabs] = useState<{ path: string; isDirty: boolean }[]>([]);
  const [activeTab, setActiveTab] = useState<string>('');
  const [previewContent, setPreviewContent] = useState<string>('');
  const [previewFilePath, setPreviewFilePath] = useState<string>('');
  const [previewFileType, setPreviewFileType] = useState<PreviewFileType>(undefined);
  const [bottomTab, setBottomTab] = useState<BottomTab>('terminal');
  const [bottomVisible, setBottomVisible] = useState(true);
  const [gitDiffContent, setGitDiffContent] = useState<string>('');
  const [rightCollapsed, setRightCollapsed] = useState(false);
  const tabsRef = useRef(openTabs);
  tabsRef.current = openTabs;

  const handleWorkspaceChange = useCallback((path: string) => {
    setWorkspacePath(path);
    setSelectedFilePath(null);
    setOpenTabs([]);
    setActiveTab('');
    setPreviewContent('');
    setPreviewFilePath('');
  }, []);

  const handleFileSelect = useCallback(async (filePath: string) => {
    setSelectedFilePath(filePath);

    setOpenTabs((prev) => {
      const exists = prev.find((t) => t.path === filePath);
      if (exists) {
        return prev;
      }
      return [...prev, { path: filePath, isDirty: false }];
    });
    setActiveTab(filePath);

    if (isPreviewable(filePath)) {
      try {
        const res = await readFileContent(filePath);
        setPreviewContent(res.content);
        setPreviewFilePath(filePath);
        setPreviewFileType(getFileType(filePath));
        setRightCollapsed(false);
      } catch {
        setPreviewContent('');
        setPreviewFilePath('');
      }
    }
  }, []);

  const handleTabClose = useCallback((filePath: string) => {
    setOpenTabs((prev) => {
      const next = prev.filter((t) => t.path !== filePath);
      if (activeTab === filePath) {
        if (next.length > 0) {
          const removedIndex = prev.findIndex((t) => t.path === filePath);
          const newIndex = Math.min(removedIndex, next.length - 1);
          setActiveTab(next[newIndex].path);
          setSelectedFilePath(next[newIndex].path);
        } else {
          setActiveTab('');
          setSelectedFilePath(null);
        }
      }
      if (previewFilePath === filePath) {
        setPreviewContent('');
        setPreviewFilePath('');
      }
      return next;
    });
  }, [activeTab, previewFilePath]);

  const handleTabSwitch = useCallback((filePath: string) => {
    setActiveTab(filePath);
    setSelectedFilePath(filePath);
  }, []);

  const handleFileChange = useCallback((filePath: string, content: string) => {
    setOpenTabs((prev) =>
      prev.map((t) => (t.path === filePath ? { ...t, isDirty: true } : t))
    );
  }, []);

  const handleFileSave = useCallback((filePath: string) => {
    setOpenTabs((prev) =>
      prev.map((t) => (t.path === filePath ? { ...t, isDirty: false } : t))
    );
  }, []);

  const handlePreviewRefresh = useCallback(() => {
    if (previewFilePath) {
      readFileContent(previewFilePath)
        .then((res) => setPreviewContent(res.content))
        .catch(() => {});
    }
  }, [previewFilePath]);

  const handleBottomTabChange = useCallback((tab: BottomTab) => {
    if (bottomTab === tab) {
      setBottomVisible((prev) => !prev);
    } else {
      setBottomTab(tab);
      setBottomVisible(true);
    }
  }, [bottomTab]);

  const tabPaths = useMemo(() => openTabs.map((t) => t.path), [openTabs]);

  const toolbar = (
    <div className="flex items-center justify-between px-4 py-2 bg-[#1a1a2e] border-b border-[#2d2d44] flex-shrink-0">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <FolderOpen size={16} className="text-amber-400" />
          <span className="text-[13px] font-medium text-[#e0e0e8] max-w-[240px] truncate">
            {workspacePath ? getFileName(workspacePath) : 'No workspace'}
          </span>
        </div>
        {workspacePath && (
          <span className="text-[11px] text-[#6b6b8a] truncate max-w-[300px]">
            {workspacePath}
          </span>
        )}
      </div>
      <div className="flex items-center gap-1">
        <button
          onClick={() => handleBottomTabChange('build')}
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-[12px] font-medium transition-colors ${
            bottomTab === 'build' && bottomVisible
              ? 'bg-[#d97706] text-[#1a1a2e]'
              : 'text-[#888] hover:text-[#d97706] hover:bg-[#252545]'
          }`}
          title="Build"
        >
          <Play size={14} fill={bottomTab === 'build' && bottomVisible ? 'currentColor' : 'none'} />
          Build
        </button>
        <div className="flex items-center">
          <button
            onClick={() => handleBottomTabChange('gitDiff')}
            className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-l-md text-[12px] font-medium transition-colors ${
              bottomTab === 'gitDiff' && bottomVisible
                ? 'bg-[#7ee787] text-[#1a1a2e]'
                : 'text-[#888] hover:text-[#7ee787] hover:bg-[#252545]'
            }`}
            title="Git Diff"
          >
            <GitBranch size={14} />
            Diff
          </button>
          <button
            onClick={() => handleBottomTabChange('gitCommit')}
            className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-r-md text-[12px] font-medium transition-colors border-l border-[#2d2d44] ${
              bottomTab === 'gitCommit' && bottomVisible
                ? 'bg-[#7ee787] text-[#1a1a2e]'
                : 'text-[#888] hover:text-[#7ee787] hover:bg-[#252545]'
            }`}
            title="Git Commit"
          >
            Commit
          </button>
        </div>
        <button
          onClick={() => handleBottomTabChange('terminal')}
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-[12px] font-medium transition-colors ${
            bottomTab === 'terminal' && bottomVisible
              ? 'bg-[#2d2d44] text-[#e0e0e8]'
              : 'text-[#888] hover:text-[#e0e0e8] hover:bg-[#252545]'
          }`}
          title={bottomVisible ? 'Hide terminal' : 'Show terminal'}
        >
          <Terminal size={14} />
          {bottomVisible ? <ChevronDown size={12} /> : <ChevronUp size={12} />}
        </button>
      </div>
    </div>
  );

  const leftPanel = (
    <div className="flex flex-col h-full">
      <div className="flex-shrink-0">
        <WorkspacePanel
          currentWorkspace={workspacePath}
          onWorkspaceChange={handleWorkspaceChange}
        />
      </div>
      <div className="flex-1 overflow-hidden">
        <FileExplorer
          onFileSelect={handleFileSelect}
          workspacePath={workspacePath}
        />
      </div>
    </div>
  );

  const centerPanel = (
    <MonacoEditor
      filePaths={tabPaths}
      activeFilePath={activeTab}
      onFileChange={handleFileChange}
      onFileSave={handleFileSave}
      onTabClose={handleTabClose}
      onTabSwitch={handleTabSwitch}
    />
  );

  const rightPanel = (
    <LivePreviewPanel
      content={previewContent}
      filePath={previewFilePath}
      fileType={previewFileType}
      onRefresh={handlePreviewRefresh}
    />
  );

  const bottomPanel = (
    <div className="flex flex-col h-full">
      <div className="flex items-center border-b border-[#2d2d44] bg-[#141425] flex-shrink-0">
        <button
          onClick={() => setBottomTab('terminal')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-[12px] transition-colors border-b-2 ${
            bottomTab === 'terminal'
              ? 'border-[#d97706] text-[#d97706]'
              : 'border-transparent text-[#888] hover:text-[#e0e0e8]'
          }`}
        >
          <Terminal size={13} />
          Terminal
        </button>
        <button
          onClick={() => setBottomTab('build')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-[12px] transition-colors border-b-2 ${
            bottomTab === 'build'
              ? 'border-[#d97706] text-[#d97706]'
              : 'border-transparent text-[#888] hover:text-[#e0e0e8]'
          }`}
        >
          <Play size={13} />
          Build
        </button>
        <button
          onClick={() => setBottomTab('gitDiff')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-[12px] transition-colors border-b-2 ${
            bottomTab === 'gitDiff'
              ? 'border-[#7ee787] text-[#7ee787]'
              : 'border-transparent text-[#888] hover:text-[#e0e0e8]'
          }`}
        >
          <GitBranch size={13} />
          Diff
        </button>
        <button
          onClick={() => setBottomTab('gitCommit')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-[12px] transition-colors border-b-2 ${
            bottomTab === 'gitCommit'
              ? 'border-[#7ee787] text-[#7ee787]'
              : 'border-transparent text-[#888] hover:text-[#e0e0e8]'
          }`}
        >
          Commit
        </button>
      </div>
      <div className="flex-1 overflow-hidden">
        {bottomTab === 'terminal' && (
          <TerminalPanel onClose={() => setBottomVisible(false)} />
        )}
        {bottomTab === 'build' && (
          <BuildPanel
            workspacePath={workspacePath}
            onBuildComplete={() => {}}
          />
        )}
        {bottomTab === 'gitDiff' && (
          <GitDiffView
            diffContent={gitDiffContent}
            workspacePath={workspacePath}
          />
        )}
        {bottomTab === 'gitCommit' && (
          <GitCommitPanel
            workspacePath={workspacePath}
            onCommit={() => {}}
            onRefresh={() => {}}
          />
        )}
      </div>
    </div>
  );

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {toolbar}
      <div className="flex-1 overflow-hidden">
        <ResizableLayout
          leftPanel={leftPanel}
          centerPanel={centerPanel}
          rightPanel={rightPanel}
          bottomPanel={bottomPanel}
          defaultLeftWidth={280}
          defaultRightWidth={400}
          defaultBottomHeight={250}
          leftCollapsed={false}
          rightCollapsed={rightCollapsed}
          bottomVisible={bottomVisible}
          onRightCollapsedChange={setRightCollapsed}
          onBottomVisibleChange={setBottomVisible}
        />
      </div>
    </div>
  );
};

export default MainContent;