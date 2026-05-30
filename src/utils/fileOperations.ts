import { open } from '@tauri-apps/plugin-shell';
import { readFileContent } from '../api';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const MIME_TYPES: Record<string, string> = {
  '.txt': 'text/plain',
  '.html': 'text/html',
  '.htm': 'text/html',
  '.css': 'text/css',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.cjs': 'text/javascript',
  '.ts': 'text/typescript',
  '.tsx': 'text/typescript',
  '.jsx': 'text/javascript',
  '.json': 'application/json',
  '.xml': 'application/xml',
  '.svg': 'image/svg+xml',
  '.csv': 'text/csv',
  '.md': 'text/markdown',
  '.yaml': 'text/yaml',
  '.yml': 'text/yaml',
  '.toml': 'application/toml',
  '.ini': 'text/plain',
  '.cfg': 'text/plain',
  '.log': 'text/plain',
  '.sh': 'text/x-shellscript',
  '.bash': 'text/x-shellscript',
  '.zsh': 'text/x-shellscript',
  '.fish': 'text/x-shellscript',
  '.ps1': 'text/plain',
  '.bat': 'text/plain',
  '.cmd': 'text/plain',
  '.py': 'text/x-python',
  '.rb': 'text/x-ruby',
  '.php': 'text/x-php',
  '.java': 'text/x-java',
  '.c': 'text/x-c',
  '.h': 'text/x-c',
  '.cpp': 'text/x-c++',
  '.cc': 'text/x-c++',
  '.cxx': 'text/x-c++',
  '.hpp': 'text/x-c++',
  '.cs': 'text/x-csharp',
  '.go': 'text/x-go',
  '.rs': 'text/x-rust',
  '.swift': 'text/x-swift',
  '.kt': 'text/x-kotlin',
  '.kts': 'text/x-kotlin',
  '.scala': 'text/x-scala',
  '.r': 'text/x-r',
  '.sql': 'text/x-sql',
  '.graphql': 'application/graphql',
  '.gql': 'application/graphql',
  '.vue': 'text/x-vue',
  '.svelte': 'text/x-svelte',
  '.astro': 'text/plain',
  '.dockerfile': 'text/plain',
  '.gitignore': 'text/plain',
  '.env': 'text/plain',
  '.lock': 'text/plain',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.webp': 'image/webp',
  '.bmp': 'image/bmp',
  '.ico': 'image/x-icon',
  '.tiff': 'image/tiff',
  '.tif': 'image/tiff',
  '.avif': 'image/avif',
  '.pdf': 'application/pdf',
  '.doc': 'application/msword',
  '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  '.xls': 'application/vnd.ms-excel',
  '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  '.ppt': 'application/vnd.ms-powerpoint',
  '.pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
  '.zip': 'application/zip',
  '.tar': 'application/x-tar',
  '.gz': 'application/gzip',
  '.tgz': 'application/gzip',
  '.bz2': 'application/x-bzip2',
  '.xz': 'application/x-xz',
  '.7z': 'application/x-7z-compressed',
  '.rar': 'application/vnd.rar',
  '.mp3': 'audio/mpeg',
  '.wav': 'audio/wav',
  '.ogg': 'audio/ogg',
  '.flac': 'audio/flac',
  '.aac': 'audio/aac',
  '.wma': 'audio/x-ms-wma',
  '.m4a': 'audio/mp4',
  '.mp4': 'video/mp4',
  '.webm': 'video/webm',
  '.avi': 'video/x-msvideo',
  '.mov': 'video/quicktime',
  '.mkv': 'video/x-matroska',
  '.wmv': 'video/x-ms-wmv',
  '.flv': 'video/x-flv',
  '.ttf': 'font/ttf',
  '.otf': 'font/otf',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.eot': 'application/vnd.ms-fontobject',
  '.wasm': 'application/wasm',
  '.apk': 'application/vnd.android.package-archive',
  '.ipa': 'application/octet-stream',
  '.exe': 'application/vnd.microsoft.portable-executable',
  '.dmg': 'application/x-apple-diskimage',
  '.deb': 'application/vnd.debian.binary-package',
  '.rpm': 'application/x-rpm',
  '.iso': 'application/x-iso9660-image',
  '.bin': 'application/octet-stream',
  '.dat': 'application/octet-stream',
};

const TEXT_EXTENSIONS = new Set([
  '.txt', '.html', '.htm', '.css', '.js', '.mjs', '.cjs', '.ts', '.tsx', '.jsx',
  '.json', '.xml', '.svg', '.csv', '.md', '.markdown', '.yaml', '.yml', '.toml',
  '.ini', '.cfg', '.log', '.sh', '.bash', '.zsh', '.fish', '.ps1', '.bat', '.cmd',
  '.py', '.rb', '.php', '.java', '.c', '.h', '.cpp', '.cc', '.cxx', '.hpp',
  '.cs', '.go', '.rs', '.swift', '.kt', '.kts', '.scala', '.r', '.sql',
  '.graphql', '.gql', '.vue', '.svelte', '.astro', '.dockerfile', '.gitignore',
  '.env', '.lock', '.editorconfig', '.prettierrc', '.eslintrc', '.babelrc',
]);

const IMAGE_EXTENSIONS = new Set([
  '.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.ico', '.tiff', '.tif',
  '.avif', '.svg', '.heic', '.heif', '.raw', '.cr2', '.nef',
]);

function getExtension(filePath: string): string {
  const lastDot = filePath.lastIndexOf('.');
  if (lastDot === -1) return '';
  return filePath.slice(lastDot).toLowerCase();
}

function getFileName(filePath: string): string {
  const lastSlash = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'));
  return lastSlash === -1 ? filePath : filePath.slice(lastSlash + 1);
}

/**
 * Get MIME type string based on file extension.
 */
export function getFileMimeType(filePath: string): string {
  const ext = getExtension(filePath);
  return MIME_TYPES[ext] || 'application/octet-stream';
}

/**
 * Check if file extension indicates a text file.
 */
export function isTextFile(filePath: string): boolean {
  const ext = getExtension(filePath);
  return TEXT_EXTENSIONS.has(ext);
}

/**
 * Check if file extension indicates an image file.
 */
export function isImageFile(filePath: string): boolean {
  const ext = getExtension(filePath);
  return IMAGE_EXTENSIONS.has(ext);
}

/**
 * Format bytes into a human-readable string (KB, MB, GB).
 */
export function formatFileSize(bytes: number): string {
  if (bytes <= 0) return '0 B';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * Download a file.
 * If content is provided, creates a blob and triggers download directly.
 * If only path is provided, reads file content first, then downloads.
 */
export async function downloadFile(filePath: string, content?: string): Promise<void> {
  try {
    let fileContent: string;
    if (content !== undefined) {
      fileContent = content;
    } else {
      const result = await readFileContent(filePath);
      fileContent = result.content;
    }

    const fileName = getFileName(filePath);
    const mimeType = getFileMimeType(filePath);
    const blob = new Blob([fileContent], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = fileName;
    anchor.style.display = 'none';
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  } catch (err) {
    console.error('Failed to download file:', filePath, err);
    throw err;
  }
}

/**
 * Copy file path to clipboard.
 * Returns true if the copy operation succeeded.
 */
export async function copyFilePath(filePath: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(filePath);
    return true;
  } catch (err) {
    console.warn('Failed to copy file path to clipboard, trying fallback', err);
    try {
      const textArea = document.createElement('textarea');
      textArea.value = filePath;
      textArea.style.position = 'fixed';
      textArea.style.left = '-9999px';
      textArea.style.top = '0';
      document.body.appendChild(textArea);
      textArea.focus();
      textArea.select();
      const successful = document.execCommand('copy');
      document.body.removeChild(textArea);
      return successful;
    } catch (fallbackErr) {
      console.error('Fallback clipboard copy also failed', fallbackErr);
      return false;
    }
  }
}

/**
 * Open file in system default application.
 * Tries Tauri's shell open first, falls back to creating a file:// anchor.
 */
export async function openInSystemApp(filePath: string): Promise<void> {
  if (isTauri) {
    try {
      await open(`file://${filePath}`);
      return;
    } catch (err) {
      console.warn('Tauri shell open failed, falling back to file:// anchor', err);
    }
  }

  try {
    const anchor = document.createElement('a');
    anchor.href = `file://${filePath}`;
    anchor.target = '_blank';
    anchor.style.display = 'none';
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
  } catch (err) {
    console.error('Failed to open file in system app:', filePath, err);
    throw err;
  }
}