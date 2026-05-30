import { extractTextContent } from './messageHelpers';

export type ExportFormat = 'txt' | 'md' | 'doc';

function formatContentForExport(content: string, role: string, format: ExportFormat, timestamp?: string): string {
  const roleLabel = role === 'user' ? 'User' : 'Claude';
  const header = format === 'md'
    ? `### ${roleLabel}${timestamp ? ` · ${timestamp}` : ''}\n\n`
    : `${roleLabel}${timestamp ? ` · ${timestamp}` : ''}\n\n`;

  return header + content + '\n\n---\n\n';
}

function sanitizeHtmlContent(content: string): string {
  return content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`([^`]+)`/g, '<code style="background:#f0f0f0;padding:2px 6px;border-radius:3px;font-family:monospace;">$1</code>')
    .replace(/```(\w*)\n([\s\S]*?)```/g, (_match: string, lang: string, code: string) => {
      return `<pre style="background:#f5f5f5;padding:12px;border-radius:8px;font-family:monospace;font-size:13px;overflow-x:auto;border:1px solid #e0e0e0;">${sanitizeHtmlContent(code.trim())}</pre>`;
    });
}

function buildWordHtml(content: string, title: string): string {
  const htmlContent = sanitizeHtmlContent(content);
  return `
<html xmlns:o="urn:schemas-microsoft-com:office:office"
      xmlns:w="urn:schemas-microsoft-com:office:word"
      xmlns="http://www.w3.org/TR/REC-html40">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <!--[if gte mso 9]><xml><w:WordDocument><w:View>Print</w:View><w:Zoom>100</w:Zoom></w:WordDocument></xml><![endif]-->
  <style>
    @page { margin: 2cm; }
    body { font-family: 'Segoe UI', 'Microsoft YaHei', sans-serif; font-size: 12pt; line-height: 1.8; color: #333; }
    h1 { font-size: 20pt; color: #1a1a1a; border-bottom: 2px solid #e0e0e0; padding-bottom: 8px; margin-bottom: 16px; }
    h2 { font-size: 16pt; color: #333; margin-top: 20px; }
    h3 { font-size: 13pt; color: #555; margin-top: 16px; margin-bottom: 8px; }
    .role-label { font-weight: bold; color: #1a7f37; margin-top: 16px; padding: 4px 0; border-bottom: 1px solid #eee; }
    .role-label.user { color: #0550ae; }
    .separator { border: none; border-top: 1px dashed #ccc; margin: 12px 0; }
    code { background: #f6f8fa; padding: 2px 6px; border-radius: 3px; font-family: 'Cascadia Code', Consolas, monospace; font-size: 10pt; }
    pre { background: #f6f8fa; padding: 12px; border-radius: 6px; font-family: 'Cascadia Code', Consolas, monospace; font-size: 10pt; overflow-x: auto; border: 1px solid #e0e0e0; }
    .timestamp { color: #888; font-size: 10pt; font-weight: normal; }
  </style>
</head>
<body>
  <h1>${title}</h1>
  ${htmlContent.split('\n---\n').map((block: string) => {
    const labelMatch = block.match(/^(User|Claude)( · .+)?\n<br>/);
    if (labelMatch) {
      const roleClass = labelMatch[1] === 'User' ? 'role-label user' : 'role-label';
      return `<div class="${roleClass}">${labelMatch[0]}</div>${block.slice(labelMatch[0].length)}<hr class="separator">`;
    }
    return block + '<hr class="separator">';
  }).join('\n')}
</body>
</html>`.trim();
}

function downloadBlob(content: string, filename: string, mimeType: string) {
  const bom = mimeType.includes('text/') ? '\uFEFF' : '';
  const blob = new Blob([bom + content], { type: `${mimeType};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function sanitizeFilename(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, '_').trim().slice(0, 120) || 'chat';
}

export function downloadMessageAsText(content: unknown, format: ExportFormat, role: string, timestamp?: string) {
  const text = extractTextContent(content);
  if (!text.trim()) return;

  const formatted = formatContentForExport(text, role, format, timestamp);
  const dateStr = new Date().toISOString().slice(0, 10);
  const rolePrefix = role === 'user' ? 'user' : 'claude';
  const sanitizedPreview = sanitizeFilename(text.slice(0, 30));

  if (format === 'txt') {
    downloadBlob(formatted, `${rolePrefix}-${sanitizedPreview}-${dateStr}.txt`, 'text/plain');
  } else if (format === 'md') {
    const header = `# ${role === 'user' ? 'User' : 'Claude'} Message\n\n`;
    const mdContent = formatted
      .replace(/^### /gm, '## ')
      .replace(/^---$/gm, '');
    downloadBlob(header + mdContent, `${rolePrefix}-${sanitizedPreview}-${dateStr}.md`, 'text/markdown');
  } else if (format === 'doc') {
    const title = `${role === 'user' ? 'User' : 'Claude'} Message - ${dateStr}`;
    const docContent = buildWordHtml(formatted, title);
    downloadBlob(docContent, `${rolePrefix}-${sanitizedPreview}-${dateStr}.doc`, 'application/msword');
  }
}

export function downloadConversationRange(
  messages: Array<{ role: string; content: unknown; created_at?: string }>,
  format: ExportFormat,
  title?: string,
) {
  if (!messages.length) return;

  const dateStr = new Date().toISOString().slice(0, 10);
  const filename = sanitizeFilename(title || 'conversation');

  let fullContent = '';
  for (const msg of messages) {
    const text = extractTextContent(msg.content);
    if (!text.trim()) continue;
    fullContent += formatContentForExport(text, msg.role || 'assistant', format, msg.created_at);
  }

  if (format === 'txt') {
    const header = `Conversation Export - ${dateStr}\n${'='.repeat(50)}\n\n`;
    downloadBlob(header + fullContent, `${filename}-${dateStr}.txt`, 'text/plain');
  } else if (format === 'md') {
    const header = `# Conversation Export\n\n*Exported: ${dateStr}*\n\n---\n\n`;
    const mdContent = fullContent
      .replace(/^### /gm, '## ')
      .replace(/^---$/gm, '');
    downloadBlob(header + mdContent, `${filename}-${dateStr}.md`, 'text/markdown');
  } else if (format === 'doc') {
    const docContent = buildWordHtml(fullContent, `Conversation Export - ${dateStr}`);
    downloadBlob(docContent, `${filename}-${dateStr}.doc`, 'application/msword');
  }
}