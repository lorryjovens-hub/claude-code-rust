import { DocumentInfo } from '../components/DocumentCard';
import { DocumentDraftInfo } from '../components/DocumentCreationProcess';
import type { PendingFile } from '../components/FileUploadPreview';

export interface ContentBlock {
  type: string;
  text?: string;
  [key: string]: unknown;
}

export interface ImageContentBlock extends ContentBlock {
  type: 'image';
  source: {
    type: 'base64';
    media_type: string;
    data: string;
  };
}

export interface TextContentBlock extends ContentBlock {
  type: 'text';
  text: string;
}

/**
 * Convert text + attached files into Anthropic multi-content blocks.
 * Images become base64 image blocks; other files remain as text references.
 */
export async function buildContentBlocks(
  text: string,
  files: PendingFile[]
): Promise<(TextContentBlock | ImageContentBlock)[]> {
  const blocks: (TextContentBlock | ImageContentBlock)[] = [];

  for (const file of files) {
    if (file.fileType === 'image' && file.file) {
      try {
        const base64 = await fileToBase64(file.file);
        blocks.push({
          type: 'image',
          source: {
            type: 'base64',
            media_type: file.mimeType || 'image/png',
            data: base64,
          },
        });
      } catch (e) {
        console.error('Failed to encode image:', e);
        blocks.push({ type: 'text', text: `[Image: ${file.fileName} (failed to encode)]` });
      }
    } else {
      blocks.push({ type: 'text', text: `[Attachment: ${file.fileName}]` });
    }
  }

  if (text.trim()) {
    blocks.push({ type: 'text', text });
  }

  return blocks;
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Strip the data:image/...;base64, prefix
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

interface MessageLike {
  id?: string;
  role?: string;
  content?: string;
  thinking?: string;
  thinkingSummary?: string;
  citations?: Array<{ url: string; title: string; cited_text?: string }>;
  searchLogs?: unknown[];
  isThinking?: boolean;
  document?: { id?: string; title?: string; url?: string; format?: string; content?: string; filename?: string } | null;
  documents?: Array<{ id?: string; title?: string; url?: string; format?: string; content?: string; filename?: string }>;
  documentDrafts?: DocumentDraftInfo[];
  toolCalls?: Array<{ name: string; input: Record<string, unknown>; output?: string; error?: string }>;
  model?: string;
  usage?: { input_tokens: number; output_tokens: number };
  files?: Array<{ id: string; name: string; url: string }>;
  created_at?: string;
}

interface GenerationState {
  text?: string;
  thinking?: string;
  thinkingSummary?: string;
  citations?: Array<{ url: string; title: string; cited_text?: string }>;
  searchLogs?: unknown[];
  document?: DocumentInfo | null;
  documents?: DocumentInfo[];
  documentDrafts?: DocumentDraftInfo[];
  isThinking?: boolean;
}

export function extractTextContent(content: unknown): string {
  if (!content) return '';
  if (typeof content !== 'string') return String(content);
  if (content.startsWith('[')) {
    try {
      const parsed: unknown = JSON.parse(content);
      if (Array.isArray(parsed)) {
        return (parsed as ContentBlock[])
          .filter((block) => block && block.type === 'text' && block.text)
          .map((block) => block.text)
          .join('\n');
      }
    } catch {
      // Not valid JSON, treat as plain text
    }
  }
  return content;
}

export function formatMessageTime(dateStr: string): string {
  if (!dateStr) return '';
  let timeStr = dateStr;
  if (timeStr.includes(' ') && !timeStr.includes('T')) {
    timeStr = timeStr.replace(' ', 'T');
  }
  if (!/Z$|[+-]\d{2}:?\d{2}$/.test(timeStr)) {
    timeStr += 'Z';
  }
  const date = new Date(timeStr);
  if (isNaN(date.getTime())) return '';
  const now = new Date();
  const isToday = date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (isToday) {
    return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }
  const isSameYear = date.getFullYear() === now.getFullYear();
  if (isSameYear) {
    return `${date.getMonth() + 1}月${date.getDate()}日`;
  }
  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}

export function withAuthToken(url: string) {
  if (!url || url.startsWith('data:') || /[?&]token=/.test(url)) return url;
  if (typeof window === 'undefined') return url;
  const token = localStorage.getItem('auth_token');
  if (!token) return url;
  return `${url}${url.includes('?') ? '&' : '?'}token=${encodeURIComponent(token)}`;
}

export function parseInlineArtifactDisplay(content: unknown): { cleanedContent: string; draft: DocumentDraftInfo | null } | null {
  if (typeof content !== 'string' || !content.includes('<cp_artifact')) return null;
  const openMatch = content.match(/<cp_artifact\s+([^>]*)>/i);
  if (!openMatch || openMatch.index === undefined) return null;
  const attrsRaw = openMatch[1] || '';
  const title = (attrsRaw.match(/title="([^"]*)"/i)?.[1] || '').trim() || 'Untitled document';
  const format = (attrsRaw.match(/format="([^"]*)"/i)?.[1] || 'markdown').trim() || 'markdown';
  const openTag = openMatch[0];
  const bodyStart = openMatch.index + openTag.length;
  const closeTag = '</cp_artifact>';
  const closeIdx = content.indexOf(closeTag, bodyStart);
  if (closeIdx === -1) {
    const preview = content.slice(bodyStart).replace(/^\n/, '');
    const cleanedContent = content.slice(0, openMatch.index).trim().replace(/\n{3,}/g, '\n\n');
    return {
      cleanedContent,
      draft: {
        draftId: `inline-${title}-${format}`,
        title,
        format,
        preview,
        previewAvailable: preview.length > 0,
        done: false,
      },
    };
  }
  const preview = content.slice(bodyStart, closeIdx).replace(/^\n/, '');
  const before = content.slice(0, openMatch.index);
  const after = content.slice(closeIdx + closeTag.length);
  const cleanedContent = `${before}${after}`.trim().replace(/\n{3,}/g, '\n\n');
  return {
    cleanedContent,
    draft: {
      draftId: `inline-${title}-${format}`,
      title,
      format,
      preview,
      previewAvailable: preview.length > 0,
      done: true,
    },
  };
}

export function normalizeDocumentDrafts(message: MessageLike): DocumentDraftInfo[] {
  const raw = Array.isArray(message?.documentDrafts) ? message.documentDrafts : [];
  const last = raw[raw.length - 1];
  if (!last || typeof last !== 'object') return [];
  const key = last.draftId || last.draft_id || last.title || 'draft';
  return [{
    draftId: key,
    title: last.title,
    format: last.format,
    preview: last.preview,
    previewAvailable: last.previewAvailable ?? last.preview_available,
    done: !!last.done,
  }];
}

export function mergeDocumentDraftIntoMessage(message: MessageLike, incomingDraft: DocumentDraftInfo) {
  if (!incomingDraft || typeof incomingDraft !== 'object') return message;
  const draftId = incomingDraft.draftId || incomingDraft.draft_id || incomingDraft.title;
  if (!draftId) return message;
  const current = normalizeDocumentDrafts(message)[0] || null;
  const nextDraft: DocumentDraftInfo = {
    draftId,
    title: incomingDraft.title,
    format: incomingDraft.format,
    preview: incomingDraft.preview ?? incomingDraft.document?.content,
    previewAvailable: incomingDraft.previewAvailable ?? incomingDraft.preview_available ?? !!incomingDraft.document?.content,
    done: !!incomingDraft.done,
  };
  const merged: DocumentDraftInfo = current
    ? {
      ...current,
      ...nextDraft,
      draftId: current.draftId || nextDraft.draftId,
      title: nextDraft.title || current.title,
      format: nextDraft.format || current.format,
      preview: nextDraft.preview ?? current.preview,
      previewAvailable: nextDraft.previewAvailable ?? current.previewAvailable,
      done: typeof incomingDraft.done === 'boolean' ? incomingDraft.done : current.done,
    }
    : nextDraft;
  return { ...message, documentDrafts: [merged] };
}

export function normalizeMessageDocuments(message: MessageLike): DocumentInfo[] {
  const raw = Array.isArray(message?.documents)
    ? message.documents
    : (message?.document ? [message.document] : []);
  const docs: DocumentInfo[] = [];
  const seen = new Set<string>();
  for (const doc of raw) {
    if (!doc || typeof doc !== 'object') continue;
    const key = doc.id || doc.url || doc.filename || `${doc.title || 'doc'}-${docs.length}`;
    if (seen.has(key)) continue;
    seen.add(key);
    docs.push(doc as DocumentInfo);
  }
  const previewExts = ['md', 'txt', 'html', 'json', 'xml', 'yaml', 'yml', 'csv'];
  if (Array.isArray(message?.toolCalls)) {
    const fileContents = new Map<string, string>();
    const fileOrder: string[] = [];
    for (const tc of message.toolCalls) {
      if (tc.name === 'Write' && tc.input?.file_path && tc.input?.content) {
        const fp = tc.input.file_path as string;
        fileContents.set(fp, tc.input.content as string);
        if (!fileOrder.includes(fp)) fileOrder.push(fp);
      }
    }
    for (const tc of message.toolCalls) {
      if ((tc.name === 'Edit' || tc.name === 'MultiEdit') && tc.input?.file_path && tc.input?.old_string != null && tc.input?.new_string != null) {
        const fp = tc.input.file_path as string;
        const current = fileContents.get(fp);
        if (current != null) {
          fileContents.set(fp, current.replaceAll(tc.input.old_string as string, tc.input.new_string as string));
        }
      }
    }
    for (const fp of fileOrder) {
      const fileName = fp.split(/[/\\]/).pop() || fp;
      const ext = fileName.split('.').pop()?.toLowerCase() || '';
      if (!previewExts.includes(ext)) continue;
      const key = `write-${fp}`;
      if (seen.has(key)) continue;
      seen.add(key);
      docs.push({
        id: key,
        title: fileName,
        filename: fileName,
        url: '',
        content: fileContents.get(fp) || '',
        format: ext === 'md' ? 'markdown' : 'text',
      });
    }
  }
  return docs;
}

export function mergeDocumentsIntoMessage(message: MessageLike, incomingDoc?: DocumentInfo | null, incomingDocs?: DocumentInfo[] | null) {
  const merged = [...normalizeMessageDocuments(message)];
  const queue = [
    ...(Array.isArray(incomingDocs) ? incomingDocs : []),
    ...(incomingDoc ? [incomingDoc] : []),
  ];
  for (const doc of queue) {
    if (!doc || typeof doc !== 'object') continue;
    const key = doc.id || doc.url || doc.filename || doc.title;
    if (!key) continue;
    const index = merged.findIndex(item => (item.id || item.url || item.filename || item.title) === key);
    if (index >= 0) merged[index] = doc;
    else merged.push(doc);
  }
  if (merged.length === 0) return message;
  return { ...message, document: merged[merged.length - 1], documents: merged };
}

export function sanitizeInlineArtifactMessage(message: MessageLike) {
  if (!message || message.role !== 'assistant') return message;
  const parsed = parseInlineArtifactDisplay(message.content);
  if (!parsed) return message;
  let next: MessageLike = { ...message, content: parsed.cleanedContent };
  if (parsed.draft && normalizeMessageDocuments(next).length === 0) {
    next = mergeDocumentDraftIntoMessage(next, parsed.draft);
  }
  return next;
}

export function applyGenerationState(message: MessageLike, state: GenerationState) {
  const base = {
    ...message,
    content: state.text || message.content,
    thinking: state.thinking || message.thinking,
    thinkingSummary: state.thinkingSummary || message.thinkingSummary,
    citations: state.citations?.length ? state.citations : message.citations,
    searchLogs: state.searchLogs?.length ? state.searchLogs : message.searchLogs,
    isThinking: !state.text && !!state.thinking,
  };
  const withDocuments = mergeDocumentsIntoMessage(base, state.document, state.documents);
  const drafts = Array.isArray(state?.documentDrafts) ? state.documentDrafts : [];
  const withDrafts = drafts.length === 0
    ? withDocuments
    : drafts.reduce((acc: MessageLike, draft: DocumentDraftInfo) => mergeDocumentDraftIntoMessage(acc, draft), withDocuments);
  return sanitizeInlineArtifactMessage(withDrafts);
}

export function extractMessageAttachments(msg: any) {
  const raw = Array.isArray(msg?.attachments)
    ? msg.attachments.filter((att: any) => att && ((typeof att.id === 'string' && att.id.trim()) || (typeof att.fileId === 'string' && att.fileId.trim())))
    : [];
  const attachments = raw.map((att: any) => ({
    file_name: att.file_name || att.fileName || 'file',
    file_type: att.file_type || att.fileType || 'document',
    mime_type: att.mime_type || att.mimeType || '',
    file_size: att.file_size || att.size || 0,
    ...att,
    id: att.id || att.fileId || '',
  }));
  const attachmentIds = attachments.map((att: any) => att.id);
  return {
    attachmentIds,
    attachmentsPayload: attachments.length > 0
      ? attachments.map((att: any) => ({ fileId: att.id, fileName: att.file_name, fileType: att.file_type, mimeType: att.mime_type, size: att.file_size }))
      : null,
    optimisticAttachments: attachments,
  };
}
