import { detectBridgePort } from '../api';
import { v4 as uuidv4 } from 'uuid';

// ━━━━━━━━━━━━━━━━━ Types ━━━━━━━━━━━━━━━━━

export type DesignType = 'prototype' | 'slides' | 'infographic' | 'review';

export interface DesignGenerateResult {
  success: boolean;
  htmlCode?: string;
  fullResponse?: string;
  error?: string;
}

export interface DesignStyle {
  id: string;
  name: string;
  desc: string;
}

// ━━━━━━━━━━━━━━━━━ System prompts per design type ━━━━━━━━━━━━━━━━━

const BASE_RULES = `IMPORTANT: Output ONLY the complete HTML document inside a single \`\`\`html code block. The HTML must be self-contained (all CSS inline or in <style>, all JS inline or in <script>). No external dependencies except Google Fonts. Do NOT wrap in markdown — output the raw HTML directly.

Rules:
- Use modern CSS (grid, flexbox, custom properties, etc.)
- Make it responsive and visually polished
- Include smooth transitions and micro-interactions
- Use a coherent color palette
- Font: system fonts or Google Fonts via <link>
- The output MUST be a complete, standalone HTML document starting with <!DOCTYPE html>`;

const TYPE_PROMPTS: Record<DesignType, string> = {
  prototype: `You are an expert UI/UX designer. Create a high-fidelity interactive web/app prototype in HTML.

${BASE_RULES}

Additional prototype rules:
- Create multiple screens/views with navigation between them (tabs, buttons)
- Include realistic dummy data and content
- Make buttons, inputs, and interactive elements functional with inline JS
- Use shadows, gradients, and layered effects for depth
- Match modern design patterns (rounded corners, glass morphism, etc.)
- Include a navigation bar if appropriate
- Make it feel like a real production app, not a wireframe`,

  slides: `You are an expert presentation designer. Create beautiful HTML presentation slides.

${BASE_RULES}

Additional slide rules:
- Create 5-8 slides with clear visual hierarchy
- Each slide should have distinct visual character
- Use large typography, bold visuals, and ample whitespace
- Include slide navigation (prev/next buttons or keyboard arrows)
- Use animations for slide transitions and element reveals
- Make it feel cinematic with dynamic layouts
- Design for widescreen (16:9 ratio mindset)`,

  infographic: `You are an expert data visualization designer. Create a print-quality HTML infographic.

${BASE_RULES}

Additional infographic rules:
- Present data/stats visually with charts, graphs, and iconography
- Use CSS-only charts (bar, donut, line via gradients/transforms) or inline SVG
- Strong visual hierarchy with section dividers
- Typography-forward layout with varied font sizes and weights
- Include stat cards, timeline sections, and comparison grids
- Design for a scrolling vertical layout (A4/letter ratio)
- Make it information-dense but visually clean`,

  review: `You are an expert design critic. Create an interactive design review document in HTML.

${BASE_RULES}

Additional review rules:
- Structure the review with clear sections: Overview, Strengths, Areas for Improvement, Recommendations
- Use color-coded severity indicators (red/amber/green)
- Include visual comparison mockups or annotated diagrams using CSS/HTML
- Use tables, checklists, and scorecards for detailed analysis
- Professional, structured layout suitable for stakeholder presentation
- Include an executive summary section at the top`,
};

const STYLE_APPEND = `\n\nApply this visual style: {{STYLE_DESC}}.`;

// ━━━━━━━━━━━━━━━━━ Service ━━━━━━━━━━━━━━━━━

function buildSystemPrompt(type: DesignType, styleDesc?: string): string {
  let prompt = TYPE_PROMPTS[type];
  if (styleDesc) {
    prompt += STYLE_APPEND.replace('{{STYLE_DESC}}', styleDesc);
  }
  return prompt;
}

export async function generateDesignCode(
  userPrompt: string,
  designType: DesignType,
  styleDesc?: string,
  model?: string,
  onStream?: (text: string) => void,
): Promise<DesignGenerateResult> {
  try {
    const port = await detectBridgePort();
    const apiBase = `http://127.0.0.1:${port}/api`;
    const convId = 'design_' + uuidv4();
    const token = localStorage.getItem('bridge_api_key') || localStorage.getItem('auth_token') || '';

    const systemPrompt = buildSystemPrompt(designType, styleDesc);

    const messages = [
      { role: 'system', content: systemPrompt },
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
      return { success: false, error: errBody.error || 'AI request failed' };
    }

    if (!res.body) {
      return { success: false, error: 'No response stream' };
    }

    // Read SSE stream
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
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') continue;
          try {
            const parsed = JSON.parse(data);
            switch (parsed.type) {
              case 'content_block_delta': {
                const delta = parsed.delta;
                if (delta?.type === 'text_delta' && delta.text) {
                  fullText += delta.text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'text': {
                const text = parsed.text || parsed.content || '';
                if (text) {
                  fullText += text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'message_stop': {
                if (parsed.full_text && parsed.full_text.length > fullText.length) {
                  fullText = parsed.full_text;
                  onStream?.(fullText);
                }
                break;
              }
              case 'thinking': {
                if (parsed.thinking) {
                  fullText += parsed.thinking;
                  onStream?.(fullText);
                }
                break;
              }
              case 'error': {
                throw new Error(parsed.error || 'Unknown error');
              }
            }
          } catch (e) {
            if (e instanceof Error && e.message !== 'Unknown error') throw e;
            if (data.trim() && !data.startsWith(':')) {
              fullText += data;
              onStream?.(fullText);
            }
          }
        }
      }
    }

    // Extract HTML code
    const htmlCode = extractHtmlCode(fullText);

    if (!htmlCode) {
      return {
        success: false,
        error: 'No HTML code found in AI response. Please retry.',
        fullResponse: fullText,
      };
    }

    return { success: true, htmlCode, fullResponse: fullText };
  } catch (e: unknown) {
    return { success: false, error: e instanceof Error ? e.message : 'Design generation failed' };
  }
}

/** Extract complete HTML document from AI response */
function extractHtmlCode(response: string): string | null {
  // Find ```html blocks
  const htmlRegex = /```html\n([\s\S]*?)```/g;
  let match;
  while ((match = htmlRegex.exec(response)) !== null) {
    const code = match[1].trim();
    if (code.includes('<!DOCTYPE') || code.includes('<html') || code.includes('<div')) {
      return code;
    }
  }

  // Try without language specifier
  const genericRegex = /```\n([\s\S]*?)```/g;
  while ((match = genericRegex.exec(response)) !== null) {
    const code = match[1].trim();
    if (code.includes('<!DOCTYPE') || code.includes('<html')) {
      return code;
    }
  }

  // Try to find DOCTYPE block directly
  const doctypeMatch = response.match(/<!DOCTYPE html>[\s\S]*?<\/html>/i);
  if (doctypeMatch) return doctypeMatch[0];

  // Last resort: find a complete <html>...</html> block
  const htmlMatch = response.match(/<html[\s\S]*?<\/html>/i);
  if (htmlMatch) return '<!DOCTYPE html>\n' + htmlMatch[0];

  return null;
}

/** Extract HTML from streaming response and inject into canvas */
export function extractStreamingHtml(fullText: string): string | null {
  // During streaming, try to extract partial HTML for live preview
  // Look for content between ```html and ```
  const marker = '```html';
  const idx = fullText.indexOf(marker);
  if (idx === -1) return null;

  const contentStart = idx + marker.length;
  let content = fullText.slice(contentStart);

  // Remove trailing ``` if present
  const endIdx = content.lastIndexOf('```');
  if (endIdx !== -1) {
    content = content.slice(0, endIdx);
  }

  content = content.trim();
  if (!content) return null;

  // If complete HTML document, return as-is
  if (content.includes('<!DOCTYPE') || content.includes('<html')) {
    return content;
  }

  // If partial/code-level content, wrap in minimal document
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; min-height: 100vh; }
</style>
</head>
<body>
${content}
</body>
</html>`;
}

export const DESIGN_STYLES: DesignStyle[] = [
  { id: 'pentagram', name: 'Pentagram 信息建筑', desc: '精准网格、大胆排版、信息优先，黑白为主色调，强烈对比' },
  { id: 'field', name: 'Field.io 运动诗学', desc: '粒子系统、流体动画、数字美学，暗色背景+霓虹色' },
  { id: 'kenya-hara', name: 'Kenya Hara 东方极简', desc: '留白、素朴、自然、侘寂美学，米白/灰色调' },
  { id: 'sagmeister', name: 'Sagmeister 实验先锋', desc: '大胆色彩碰撞、实验性排版、打破常规布局' },
  { id: 'apple', name: 'Apple 精致科技', desc: '极致精致、微光效果、深空灰+银白渐变，高端感' },
];
