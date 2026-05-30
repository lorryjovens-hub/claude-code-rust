import { invoke } from '@tauri-apps/api/core';
import { detectBridgePort } from '../api';
import { v4 as uuidv4 } from 'uuid';

// ━━━━━━━━━━━━━━━━━ Types (mirrors Rust remotion/mod.rs) ━━━━━━━━━━━━━━━━━

export interface CompositionInfo {
  id: string;
  duration_in_frames: number;
  fps: number;
  width: number;
  height: number;
}

export interface RemotionProject {
  name: string;
  path: string;
  compositions: CompositionInfo[];
  has_node_modules: boolean;
  created_at: string;
}

export interface RenderRequest {
  projectPath: string;
  compositionId: string;
  outputPath: string;
  fps?: number;
  frames?: number[];
}

export interface RenderResponse {
  success: boolean;
  output_file: string;
  duration_secs: number;
  error?: string;
}

// ━━━━━━━━━━━━━━━━━ Remotion Service API ━━━━━━━━━━━━━━━━━

export const remotionService = {
  /** Create a new Remotion project via `npx create-video@latest` */
  async createProject(name: string, targetDir: string, template?: string): Promise<RemotionProject> {
    return invoke('remotion_create_project', { name, targetDir, template });
  },

  /** Install npm dependencies for a Remotion project */
  async installDeps(projectPath: string): Promise<string> {
    return invoke('remotion_install_deps', { projectPath });
  },

  /** Start Remotion Studio dev server and open in browser */
  async startStudio(projectPath: string, port?: number): Promise<string> {
    return invoke('remotion_start_studio', { projectPath, port });
  },

  /** Render a composition to MP4/WebM */
  async render(request: RenderRequest): Promise<RenderResponse> {
    return invoke('remotion_render', { request });
  },

  /** List all compositions in a Remotion project */
  async listCompositions(projectPath: string): Promise<CompositionInfo[]> {
    return invoke('remotion_list_compositions', { projectPath });
  },

  /** Scan directory for Remotion projects */
  async scanProjects(scanDir: string): Promise<RemotionProject[]> {
    return invoke('remotion_scan_projects', { scanDir });
  },

  /** Open a Remotion project in VS Code */
  async openInEditor(projectPath: string): Promise<string> {
    return invoke('remotion_open_in_editor', { projectPath });
  },

  /** Render a single frame as an image */
  async still(projectPath: string, compositionId: string, outputPath: string, frame?: number): Promise<string> {
    return invoke('remotion_still', { projectPath, compositionId, outputPath, frame });
  },
};

// ━━━━━━━━━━━━━━━━━ AI Generation ━━━━━━━━━━━━━━━━━

export interface AIGenerateResult {
  success: boolean;
  code?: string;
  fullResponse?: string;
  error?: string;
}

const REMOTION_SYSTEM_PROMPT = `You are a Remotion video expert. When asked to create an animation/video, you MUST output a complete, working Remotion React component.

Respond in this exact format:

\`\`\`tsx
// === COMPOSITION ===
// This is the main entry — the <Composition> registration
import { Composition } from 'remotion';
import { MyVideo } from './MyVideo';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="MyVideo"
        component={MyVideo}
        durationInFrames={FPS * DURATION_SECONDS}
        fps={FPS}
        width={WIDTH}
        height={HEIGHT}
      />
    </>
  );
};
\`\`\`

\`\`\`tsx
// === COMPONENT ===
// This is the actual video component
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, spring } from 'remotion';
import { loadFont } from '@remotion/google-fonts/Inter';

const { fontFamily } = loadFont();

const FPS = 30;
const DURATION_SECONDS = 5;
const WIDTH = 1920;
const HEIGHT = 1080;

export const MyVideo: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const opacity = spring({ frame, fps, config: { damping: 10 } });

  return (
    <AbsoluteFill style={{ backgroundColor: '#1a1a2e', fontFamily }}>
      <h1 style={{ opacity, fontSize: 80, color: 'white', textAlign: 'center', marginTop: 300 }}>
        Hello Remotion!
      </h1>
    </AbsoluteFill>
  );
};
\`\`\`

Rules:
- Always output BOTH the Composition registration AND the video component
- The Composition MUST have id, component, durationInFrames, fps, width, height
- Use FPS, DURATION_SECONDS, WIDTH, HEIGHT as named constants at the top of the component file
- Use remotion built-in hooks: useCurrentFrame, useVideoConfig, interpolate, spring
- Use AbsoluteFill for full-screen layouts
- Make visually impressive animations with spring/interpolate
- Keep the component self-contained (no external data dependencies)
- Export the component as a named export`;

/** Call the bridge AI API to generate Remotion code from a natural language prompt */
export async function generateRemotionCode(
  userPrompt: string,
  model?: string,
  onStream?: (text: string) => void,
): Promise<AIGenerateResult> {
  try {
    const port = await detectBridgePort();
    const apiBase = `http://127.0.0.1:${port}/api`;
    const convId = 'remotion_' + uuidv4();
    const token = localStorage.getItem('bridge_api_key') || localStorage.getItem('auth_token') || '';

    const messages = [
      { role: 'system', content: REMOTION_SYSTEM_PROMPT },
      { role: 'user', content: userPrompt },
    ];

    const res = await fetch(`${apiBase}/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        conversation_id: convId,
        messages,
        model: model || 'claude-sonnet-4-6',
        user_mode: localStorage.getItem('user_mode') || 'clawparrot',
        env_token: localStorage.getItem('ANTHROPIC_API_KEY') || undefined,
        env_base_url: localStorage.getItem('ANTHROPIC_BASE_URL') || undefined,
      }),
    });

    if (!res.ok) {
      const errBody = await res.json().catch(() => ({ error: `HTTP ${res.status}` }));
      return { success: false, error: errBody.error || 'AI 请求失败' };
    }

    if (!res.body) {
      return { success: false, error: '无响应流' };
    }

    // Read SSE stream — bridge uses axum Event::default().data(json)
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    let fullText = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        // axum SSE: each line is "data: <json>\n\n"
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') continue;
          try {
            const parsed = JSON.parse(data);
            switch (parsed.type) {
              case 'content_block_delta': {
                // { type: "content_block_delta", delta: { type: "text_delta", text: "..." } }
                const delta = parsed.delta;
                if (delta?.type === 'text_delta' && delta.text) {
                  fullText += delta.text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'text': {
                // Alternative format
                const text = parsed.text || parsed.content || '';
                if (text) {
                  fullText += text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'message_stop': {
                // Complete text from the assistant at turn end
                if (parsed.full_text && parsed.full_text.length > fullText.length) {
                  fullText = parsed.full_text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'thinking': {
                // Include thinking in fullText for code extraction
                if (parsed.thinking) {
                  fullText += parsed.thinking;
                  onStream?.(fullText);
                }
                break;
              }
              case 'error': {
                const errMsg = parsed.error || '未知错误';
                throw new Error(errMsg);
              }
            }
          } catch {
            // Non-JSON SSE line — include as raw text
            if (data.trim() && !data.startsWith(':')) {
              fullText += data;
              onStream?.(fullText);
            }
          }
        }
      }
    }

    // Extract code blocks from the full response
    const code = extractCodeBlocks(fullText);

    if (!code) {
      return {
        success: false,
        error: 'AI 响应中未找到代码块。请重试。',
        fullResponse: fullText,
      };
    }

    return { success: true, code, fullResponse: fullText };
  } catch (e: unknown) {
    return { success: false, error: e instanceof Error ? e.message : 'AI 生成失败' };
  }
}

/** Extract TSX code blocks from AI response, combining Composition + Component */
function extractCodeBlocks(response: string): string | null {
  // Find all ```tsx blocks
  const tsxBlocks: string[] = [];
  const regex = /```tsx\n([\s\S]*?)```/g;
  let match;
  while ((match = regex.exec(response)) !== null) {
    tsxBlocks.push(match[1].trim());
  }

  if (tsxBlocks.length === 0) {
    // Try without language specifier
    const genericRegex = /```\n([\s\S]*?)```/g;
    while ((match = genericRegex.exec(response)) !== null) {
      const block = match[1].trim();
      if (block.includes('import') || block.includes('export') || block.includes('Composition')) {
        tsxBlocks.push(block);
      }
    }
  }

  if (tsxBlocks.length === 0) return null;

  // Combine all blocks into a single Root.tsx
  return tsxBlocks.join('\n\n');
}

/** Write the generated AI code to a project's source file */
export async function writeGeneratedCode(projectPath: string, code: string): Promise<boolean> {
  try {
    const fs = await import('@tauri-apps/plugin-fs');
    const srcDir = `${projectPath}\\src`;

    // Ensure src directory exists
    try {
      await fs.mkdir(srcDir, { recursive: true });
    } catch {}

    // Write the main component file
    await fs.writeTextFile(`${srcDir}\\Root.tsx`, code);
    return true;
  } catch (e) {
    console.error('[remotionService] Failed to write generated code:', e);
    return false;
  }
}

export const REMOTION_TEMPLATES = [
  { id: 'blank', name: '空白项目', description: '最小 Remotion 脚手架，从头开始', icon: 'square' },
  { id: 'hello-world', name: 'Hello World', description: '基础文字动画示例', icon: 'type' },
  { id: 'logo-reveal', name: 'Logo 揭示', description: '品牌 Logo 出场动画模板', icon: 'sparkles' },
  { id: 'react-three-fiber', name: '3D 场景', description: 'React Three Fiber 3D 动画', icon: 'box' },
];

export const RENDER_PRESETS = [
  { label: 'Instagram Reel', width: 1080, height: 1920, fps: 30 },
  { label: 'YouTube 1080p', width: 1920, height: 1080, fps: 30 },
  { label: 'YouTube 4K', width: 3840, height: 2160, fps: 60 },
  { label: 'TikTok', width: 1080, height: 1920, fps: 30 },
  { label: 'Twitter', width: 1280, height: 720, fps: 30 },
  { label: 'Custom', width: 0, height: 0, fps: 0 },
];

export const OUTPUT_FORMATS = [
  { id: 'mp4', label: 'MP4 (H.264)', ext: '.mp4' },
  { id: 'webm', label: 'WebM (VP9)', ext: '.webm' },
];
