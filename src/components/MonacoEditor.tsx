import React, { useState, useEffect, useCallback, useRef } from 'react';
import Editor, { OnMount, BeforeMount } from '@monaco-editor/react';
import { FileCode, X, Circle, Save, Loader2 } from 'lucide-react';
import { readFileContent, writeFileContent } from '../api';

const EXT_TO_LANGUAGE: Record<string, string> = {
  ts: 'typescript',
  tsx: 'typescript',
  js: 'javascript',
  jsx: 'javascript',
  rs: 'rust',
  py: 'python',
  go: 'go',
  java: 'java',
  html: 'html',
  css: 'css',
  json: 'json',
  md: 'markdown',
  yaml: 'yaml',
  yml: 'yaml',
  sql: 'sql',
  sh: 'shell',
  bash: 'shell',
  toml: 'toml',
  xml: 'xml',
  svg: 'xml',
  graphql: 'graphql',
  c: 'c',
  cpp: 'cpp',
  h: 'c',
  scss: 'scss',
  less: 'less',
};

function getLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  return EXT_TO_LANGUAGE[ext] || 'plaintext';
}

function getFileName(filePath: string): string {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath;
}

interface MonacoEditorProps {
  filePath?: string;
  filePaths?: string[];
  activeFilePath?: string;
  onFileChange?: (filePath: string, content: string) => void;
  onFileSave?: (filePath: string) => void;
  onTabClose?: (filePath: string) => void;
  onTabSwitch?: (filePath: string) => void;
}

interface FileState {
  content: string;
  originalContent: string;
  language: string;
  isDirty: boolean;
}

const MonacoEditor: React.FC<MonacoEditorProps> = ({
  filePath,
  filePaths,
  activeFilePath,
  onFileChange,
  onFileSave,
  onTabClose,
  onTabSwitch,
}) => {
  const resolvedPaths = React.useMemo(() => {
    if (filePaths && filePaths.length > 0) return filePaths;
    if (filePath) return [filePath];
    return [];
  }, [filePath, filePaths]);

  const [currentFilePath, setCurrentFilePath] = useState<string | undefined>(
    activeFilePath || resolvedPaths[0]
  );
  const [fileStates, setFileStates] = useState<Record<string, FileState>>({});
  const [loadingPath, setLoadingPath] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveToast, setSaveToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const tabContainerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (activeFilePath !== undefined) {
      setCurrentFilePath(activeFilePath);
    }
  }, [activeFilePath]);

  useEffect(() => {
    if (!resolvedPaths.length) return;
    if (!resolvedPaths.includes(currentFilePath || '')) {
      setCurrentFilePath(resolvedPaths[0]);
    }
  }, [resolvedPaths]);

  const fileStatesRef = useRef(fileStates);
  fileStatesRef.current = fileStates;

  const loadFile = useCallback(async (path: string) => {
    if (fileStatesRef.current[path]) return;
    setLoadingPath(path);
    try {
      const res = await readFileContent(path);
      const content = res.content;
      setFileStates(prev => ({
        ...prev,
        [path]: {
          content,
          originalContent: content,
          language: getLanguage(path),
          isDirty: false,
        },
      }));
    } catch {
      setFileStates(prev => ({
        ...prev,
        [path]: {
          content: '',
          originalContent: '',
          language: getLanguage(path),
          isDirty: false,
        },
      }));
    } finally {
      setLoadingPath(null);
    }
  }, []);

  useEffect(() => {
    if (currentFilePath) {
      loadFile(currentFilePath);
    }
  }, [currentFilePath]);

  const handleEditorChange = useCallback((value: string | undefined) => {
    if (!currentFilePath) return;
    const content = value || '';
    setFileStates(prev => {
      const existing = prev[currentFilePath];
      if (!existing) return prev;
      const isDirty = content !== existing.originalContent;
      return {
        ...prev,
        [currentFilePath]: { ...existing, content, isDirty },
      };
    });
    onFileChange?.(currentFilePath, content);
  }, [currentFilePath, onFileChange]);

  const handleSave = useCallback(async () => {
    if (!currentFilePath || saving) return;
    const fileState = fileStates[currentFilePath];
    if (!fileState) return;

    setSaving(true);
    try {
      await writeFileContent(currentFilePath, fileState.content);
      setFileStates(prev => ({
        ...prev,
        [currentFilePath]: { ...fileState, originalContent: fileState.content, isDirty: false },
      }));
      setSaveToast({ message: 'Saved', type: 'success' });
      onFileSave?.(currentFilePath);
    } catch {
      setSaveToast({ message: 'Save failed', type: 'error' });
    } finally {
      setSaving(false);
    }
  }, [currentFilePath, fileStates, saving, onFileSave]);

  const handleSaveRef = useRef(handleSave);
  handleSaveRef.current = handleSave;

  useEffect(() => {
    if (!saveToast) return;
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    toastTimerRef.current = setTimeout(() => setSaveToast(null), 2500);
    return () => {
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, [saveToast]);

  const handleBeforeMount: BeforeMount = (monaco) => {
    monaco.editor.defineTheme('custom-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'variable', foreground: '9CDCFE' },
      ],
      colors: {
        'editor.background': '#1a1a2e',
        'editor.foreground': '#e0e0e8',
        'editor.lineHighlightBackground': '#252540',
        'editor.selectionBackground': '#3d3d6e',
        'editorCursor.foreground': '#c0c0d0',
        'editorLineNumber.foreground': '#5a5a7a',
        'editorLineNumber.activeForeground': '#a0a0c0',
        'editor.inactiveSelectionBackground': '#2d2d50',
        'editorWidget.background': '#1e1e32',
        'editorWidget.border': '#2d2d44',
        'input.background': '#252540',
        'input.foreground': '#e0e0e8',
        'input.border': '#2d2d44',
        'scrollbar.shadow': '#00000033',
        'scrollbarSlider.background': '#3d3d5e66',
        'scrollbarSlider.hoverBackground': '#4d4d7e66',
        'scrollbarSlider.activeBackground': '#5d5d8e66',
        'minimap.background': '#1a1a2e',
      },
    });
  };

  const handleEditorMount: OnMount = (editor) => {
    editorRef.current = editor;
    editor.addCommand(
      2048 | 49,
      () => handleSaveRef.current()
    );
  };

  useEffect(() => {
    if (!editorRef.current || !currentFilePath) return;
    const language = fileStates[currentFilePath]?.language || 'plaintext';
    const model = editorRef.current.getModel();
    if (model && model.getLanguageId() !== language) {
      const uri = model.uri;
      monacoRef.current?.editor.setModelLanguage(model, language);
    }
  }, [currentFilePath, fileStates]);

  const handleTabClick = (path: string) => {
    setCurrentFilePath(path);
    onTabSwitch?.(path);
  };

  const handleTabClose = (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    setFileStates(prev => {
      const next = { ...prev };
      delete next[path];
      return next;
    });
    onTabClose?.(path);
  };

  const activeFileState = currentFilePath ? fileStates[currentFilePath] : undefined;
  const isLoading = loadingPath === currentFilePath;

  const showEmptyState = !resolvedPaths.length || (!currentFilePath && resolvedPaths.length === 0);

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e]">
      {!showEmptyState && resolvedPaths.length > 0 && (
        <div className="flex items-center border-b border-[#2d2d44] bg-[#141425] overflow-hidden flex-shrink-0">
          <div
            ref={tabContainerRef}
            className="flex-1 flex items-center overflow-x-auto"
            style={{ scrollbarWidth: 'thin', msOverflowStyle: 'none' }}
          >
            {resolvedPaths.map((path) => {
              const isActive = path === currentFilePath;
              const fileState = fileStates[path];
              return (
                <div
                  key={path}
                  onClick={() => handleTabClick(path)}
                  className={`group flex items-center gap-1.5 px-3 py-2 text-[13px] cursor-pointer border-r border-[#2d2d44] transition-colors flex-shrink-0 max-w-[200px] ${
                    isActive
                      ? 'bg-[#1a1a2e] text-[#e0e0e8] border-t-2 border-t-[#569CD6]'
                      : 'text-[#7a7a9a] hover:bg-[#1a1a2e]/50 hover:text-[#c0c0d0]'
                  }`}
                >
                  {fileState?.isDirty && (
                    <Circle size={8} className="text-[#DCDCAA] flex-shrink-0" fill="#DCDCAA" />
                  )}
                  <span className="truncate text-[12px]">{getFileName(path)}</span>
                  <button
                    onClick={(e) => handleTabClose(e, path)}
                    className="p-0.5 rounded hover:bg-[#2d2d44] opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 text-[#7a7a9a] hover:text-[#e0e0e8]"
                  >
                    <X size={12} />
                  </button>
                </div>
              );
            })}
          </div>
          <div className="flex items-center gap-1 px-3 flex-shrink-0 border-l border-[#2d2d44]">
            {saving ? (
              <Loader2 size={14} className="text-[#7a7a9a] animate-spin" />
            ) : (
              <Save
                size={14}
                className={`cursor-pointer transition-colors ${
                  activeFileState?.isDirty
                    ? 'text-[#DCDCAA] hover:text-[#e0e0c0]'
                    : 'text-[#5a5a7a]'
                }`}
                onClick={handleSave}
              />
            )}
          </div>
        </div>
      )}

      <div className="flex-1 relative min-h-0">
        {showEmptyState ? (
          <div className="flex flex-col items-center justify-center h-full text-[#5a5a7a] gap-3">
            <FileCode size={48} className="opacity-40" />
            <p className="text-[14px]">Open a file to start editing</p>
          </div>
        ) : isLoading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 size={24} className="text-[#7a7a9a] animate-spin" />
          </div>
        ) : currentFilePath ? (
          <div className="h-full w-full">
            <Editor
              key={currentFilePath}
              language={activeFileState?.language || 'plaintext'}
              value={activeFileState?.content || ''}
              onChange={handleEditorChange}
              theme="custom-dark"
              beforeMount={handleBeforeMount}
              onMount={handleEditorMount}
              loading={
                <div className="flex items-center justify-center h-full">
                  <Loader2 size={24} className="text-[#7a7a9a] animate-spin" />
                </div>
              }
              options={{
                fontSize: 14,
                fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, 'Courier New', monospace",
                fontLigatures: true,
                minimap: { enabled: true, scale: 1, showSlider: 'mouseover' },
                scrollBeyondLastLine: false,
                lineNumbers: 'on',
                renderLineHighlight: 'all',
                bracketPairColorization: { enabled: true },
                automaticLayout: true,
                tabSize: 2,
                insertSpaces: true,
                wordWrap: 'off',
                padding: { top: 12 },
                smoothScrolling: true,
                cursorBlinking: 'smooth',
                cursorSmoothCaretAnimation: 'on',
                roundedSelection: true,
                guides: { indentation: true, bracketPairs: true },
                suggest: { showWords: true, showSnippets: true },
              }}
            />
          </div>
        ) : null}
      </div>

      {saveToast && (
        <div
          className={`absolute bottom-4 right-4 px-4 py-2 rounded-lg text-[13px] font-medium shadow-lg transition-all duration-300 z-50 ${
            saveToast.type === 'success'
              ? 'bg-[#4EC9B0]/20 text-[#4EC9B0] border border-[#4EC9B0]/30'
              : 'bg-[#F44747]/20 text-[#F44747] border border-[#F44747]/30'
          }`}
        >
          {saveToast.message}
        </div>
      )}
    </div>
  );
};

export default MonacoEditor;