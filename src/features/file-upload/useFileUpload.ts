import { uploadFile, deleteAttachment, createConversation, warmEngine, materializeGithub } from '../../api';
import { PendingFile } from '../../components/FileUploadPreview';
import { draftsStore } from '../message-list/MessageList';

const ACCEPTED_TYPES = 'image/png,image/jpeg,image/jpg,image/gif,image/webp,video/mp4,video/webm,video/quicktime,video/x-msvideo,audio/mpeg,audio/wav,audio/ogg,audio/webm,application/pdf,.docx,.xlsx,.pptx,.odt,.rtf,.epub,.txt,.md,.csv,.json,.xml,.yaml,.yml,.js,.jsx,.ts,.tsx,.py,.java,.cpp,.c,.h,.cs,.go,.rs,.rb,.php,.swift,.kt,.scala,.html,.css,.scss,.less,.sql,.sh,.bash,.vue,.svelte,.lua,.r,.m,.pl,.ex,.exs,.mp4,.webm,.mov,.avi,.mp3,.wav,.ogg,.m4a';

export { ACCEPTED_TYPES };

interface UseFileUploadOpts {
  activeId: string | null;
  pendingFiles: PendingFile[];
  setPendingFiles: (updater: (prev: PendingFile[]) => PendingFile[]) => void;
  currentModelString: string;
  researchMode: boolean;
  isModelSelectable: (model: string) => boolean;
  resolveModelForNewChat: (preferred?: string | null) => string;
  onNewChat: () => void;
  navigate: (path: string, opts?: { replace?: boolean }) => void;
  inputTextRef: React.MutableRefObject<string>;
  textareaHeightRef: React.MutableRefObject<number>;
  inputBarBaseHeight: number;
}

export function useFileUpload(opts: UseFileUploadOpts) {
  const {
    activeId, pendingFiles, setPendingFiles,
    currentModelString, researchMode,
    isModelSelectable, resolveModelForNewChat,
    onNewChat, navigate,
    inputTextRef, textareaHeightRef, inputBarBaseHeight,
  } = opts;

  const handleFilesSelected = (files: FileList | File[], defaults?: Partial<PendingFile>) => {
    const fileArray = Array.from(files);
    const maxFiles = 20;
    const currentCount = pendingFiles.length;
    const allowed = fileArray.slice(0, maxFiles - currentCount);

    for (const file of allowed) {
      const id = Math.random().toString(36).slice(2);
      const isImage = file.type.startsWith('image/');
      const isVideo = file.type.startsWith('video/');
      const previewUrl = (isImage || isVideo) ? URL.createObjectURL(file) : undefined;

      const pending: PendingFile = {
        id,
        file,
        fileName: file.name,
        mimeType: file.type,
        size: file.size,
        progress: 0,
        status: 'uploading',
        previewUrl,
        ...(defaults || {}),
      };

      // Local files (images, videos, audio, text docs) are read client-side as base64
      // and marked as done immediately. Only non-media files uploaded to remote API.
      const isLocalMedia = file.type.startsWith('image/') || file.type.startsWith('video/') || file.type.startsWith('audio/');
      const textExtensions = /\.(txt|md|csv|json|xml|yaml|yml|js|jsx|ts|tsx|py|java|cpp|c|h|cs|go|rs|rb|php|swift|kt|scala|html|css|scss|less|sql|sh|bash|vue|svelte|lua|r|m|pl|ex|exs)$/i;
      const isLocalText = file.size < 5 * 1024 * 1024 && (file.type.startsWith('text/') || textExtensions.test(file.name));

      if (isLocalMedia || isLocalText) {
        // Immediately mark as done with progress 100 for local files
        // The base64 data will be populated once FileReader completes
        setPendingFiles(prev => [...prev, { ...pending, status: 'done' as const, progress: 100, fileId: id }]);
        
        // Read file as base64 for client-side embedding
        const reader = new FileReader();
        reader.onload = (e) => {
          const dataUrl = e.target?.result as string;
          if (dataUrl) {
            let lineCount: number | undefined;
            if (isLocalText && dataUrl) {
              // Decode base64 data URL to get text content for line count
              const base64 = dataUrl.split(',')[1] || '';
              try {
                const text = atob(base64);
                lineCount = text.split(/\r\n|\r|\n/).length;
              } catch {}
            }
            // Update with final file type and data URL
            setPendingFiles(prev => prev.map(f => f.id === id ? {
              ...f,
              fileType: file.type.startsWith('image/') ? 'image' as const : file.type.startsWith('video/') ? 'video' as const : file.type.startsWith('audio/') ? 'audio' as const : 'document' as const,
              previewUrl: isLocalMedia ? dataUrl : undefined,
              lineCount,
            } : f));
          }
        };
        reader.onerror = () => {
          setPendingFiles(prev => prev.map(f => f.id === id ? {
            ...f,
            status: 'error' as const,
            error: '文件读取失败',
          } : f));
        };
        reader.readAsDataURL(file);
      } else if (activeId) {
        // Add pending file and upload non-media files (PDF, docx, etc.) to remote API
        setPendingFiles(prev => [...prev, pending]);
        // Add timeout protection to prevent infinite loading
        const uploadTimeout = setTimeout(() => {
          setPendingFiles(prev => prev.map(f => f.id === id ? {
            ...f,
            status: 'error' as const,
            error: '上传超时，请重试',
          } : f));
        }, 30000); // 30 second timeout

        uploadFile(file, (percent) => {
          setPendingFiles(prev => prev.map(f => f.id === id ? { ...f, progress: percent } : f));
        }, activeId).then((result) => {
          clearTimeout(uploadTimeout);
          setPendingFiles(prev => prev.map(f => f.id === id ? {
            ...f,
            fileId: result.fileId,
            fileType: result.fileType,
            status: 'done' as const,
            progress: 100,
          } : f));
        }).catch((err) => {
          clearTimeout(uploadTimeout);
          setPendingFiles(prev => prev.map(f => f.id === id ? {
            ...f,
            status: 'error' as const,
            error: err.message,
          } : f));
        });
      } else {
        // No activeId but remote upload needed — mark as done anyway for local-only mode
        setPendingFiles(prev => [...prev, { ...pending, status: 'done' as const, progress: 100, fileId: id, fileType: 'document' as const }]);
      }
    }
  };

  const handleRemoveFile = (id: string) => {
    setPendingFiles(prev => {
      const file = prev.find(f => f.id === id);
      if (file?.previewUrl) URL.revokeObjectURL(file.previewUrl);
      if (file?.fileId) {
        deleteAttachment(file.fileId).catch(() => {});
      }
      return prev.filter(f => f.id !== id);
    });
  };

  const handleGithubAdd = async (payload: { repoFullName: string; ref?: string; selections: any[] }): Promise<void> => {
    let convId: string | null = activeId;
    let createdNewConv = false;
    if (!convId) {
      const modelForCreate = isModelSelectable(currentModelString)
        ? currentModelString
        : resolveModelForNewChat(currentModelString);
      let newConv = await createConversation(undefined, modelForCreate, { research_mode: researchMode });
      if (!newConv || !newConv.id) {
        const fallbackId = 'fallback-' + crypto.randomUUID();
        console.warn('[MainContent] Server returned no id, using fallback:', fallbackId, newConv);
        newConv = { ...newConv!, id: fallbackId };
      }
      convId = newConv.id;
      createdNewConv = true;
      if (convId) warmEngine(convId);
      onNewChat();
    }

    const result = await materializeGithub(convId!, payload.repoFullName, payload.ref || '', payload.selections);

    const githubCard: PendingFile = {
      id: Math.random().toString(36).slice(2),
      file: new File([], 'github-placeholder'),
      fileName: payload.repoFullName,
      mimeType: 'application/x-github',
      size: 0,
      progress: 100,
      status: 'done',
      source: 'github',
      ghRepo: payload.repoFullName,
      ghRef: payload.ref,
      lineCount: result.fileCount,
    };

    if (createdNewConv) {
      const text = inputTextRef.current || '';
      const height = textareaHeightRef.current || inputBarBaseHeight;
      draftsStore.set(convId!, { text, files: [githubCard], height });
      navigate(`/chat/${convId}`, { replace: true });
    } else {
      setPendingFiles(prev => [...prev, githubCard]);
    }
  };

  return { handleFilesSelected, handleRemoveFile, handleGithubAdd };
}
