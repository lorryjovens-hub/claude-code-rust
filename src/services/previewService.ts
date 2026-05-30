import { config, ensureBridgePort, getApiUrl, getPreviewEventsUrl } from './config';

export interface PreviewContent {
  id: string;
  content: string;
  content_type: string;
  last_updated: number;
}

export interface PreviewSetRequest {
  content: string;
  content_type?: string;
}

const waitReady = ensureBridgePort();

export const previewService = {
  async getPreview(id: string): Promise<PreviewContent | null> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl(`/api/preview/${id}`));
      if (!response.ok) return null;
      return await response.json();
    } catch {
      return null;
    }
  },

  async setPreview(id: string, content: string, contentType: string = 'text/html'): Promise<boolean> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl(`/api/preview/${id}`), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ content, content_type: contentType }),
      });
      return response.ok;
    } catch {
      return false;
    }
  },

  async deletePreview(id: string): Promise<boolean> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl(`/api/preview/${id}`), {
        method: 'DELETE',
      });
      return response.ok;
    } catch {
      return false;
    }
  },

  async listPreviews(): Promise<PreviewContent[]> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl('/api/preview'));
      if (!response.ok) return [];
      return await response.json();
    } catch {
      return [];
    }
  },

  async streamPreviewUpdates(id: string, callback: (content: PreviewContent) => void): Promise<() => void> {
    await waitReady;
    const eventSource = new EventSource(getPreviewEventsUrl(id));

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        callback(data);
      } catch {
        console.error('Failed to parse preview event');
      }
    };

    eventSource.onerror = () => {
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  },
};

export interface OpenDesignMetadata {
  mode?: string;
  platform?: string;
  scenario?: string;
  surface?: string;
  category?: string | null;
  preview?: PreviewConfig & { type?: string };
  design_system?: {
    requires?: boolean;
    generates?: boolean;
    sections?: string[];
  };
  inputs?: Array<{
    name: string;
    type: string;
    required?: boolean;
    description?: string;
  }>;
  outputs?: {
    primary?: string;
    secondary?: string;
  };
  fidelity?: string;
  featured?: number;
  animations?: boolean;
  speaker_notes?: boolean;
  default_for?: string[];
  example_prompt?: string;
  upstream?: string;
}

export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  category: string;
  preview?: PreviewConfig;
  od_metadata?: OpenDesignMetadata;
  content?: string;
  when_to_use?: string;
  enabled?: boolean;
  source?: string;
  loaded_from?: string;
  source_dir?: string;
  is_example?: boolean;
  files?: Array<{
    name: string;
    type: string;
    children?: Array<{ name: string; type: string; children?: Array<{ name: string; type: string }> }>;
  }>;
  created_at?: string | null;
}

export interface PreviewConfig {
  reload_strategy?: string;
  debounce_ms?: number;
  type?: string;
  width?: number;
  height?: number;
}

export const skillService = {
  async listSkills(): Promise<SkillInfo[]> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl('/api/skills'));
      if (!response.ok) return [];
      return await response.json();
    } catch {
      return [];
    }
  },

  async getSkill(id: string): Promise<SkillInfo | null> {
    try {
      await waitReady;
      const response = await fetch(getApiUrl(`/api/skills/${id}`));
      if (!response.ok) return null;
      return await response.json();
    } catch {
      return null;
    }
  },

  async listDesignSkills(): Promise<SkillInfo[]> {
    try {
      const skills = await this.listSkills();
      return skills.filter(skill => skill.category?.toLowerCase() === 'design' ||
        skill.id.toLowerCase().includes('design') ||
        skill.name.toLowerCase().includes('design'));
    } catch {
      return [];
    }
  },

  async listDesignSkillsDetailed(): Promise<SkillInfo[]> {
    try {
      await waitReady;
      const url = getApiUrl('/api/skills/design');
      console.log('[previewService.listDesignSkillsDetailed] 📡 GET', url);
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 8000);
      const response = await fetch(url, { signal: controller.signal });
      clearTimeout(timer);
      console.log('[previewService.listDesignSkillsDetailed] 📡 响应状态:', response.status, response.ok);
      if (!response.ok) {
        console.warn('[previewService.listDesignSkillsDetailed] ⚠️ 非 2xx 响应');
        return [];
      }
      const data = await response.json();
      const count = data.skills?.length || 0;
      console.log(`[previewService.listDesignSkillsDetailed] 📦 解析完成，${count} 个技能`);
      if (count === 0) {
        console.warn('[previewService.listDesignSkillsDetailed] ⚠️ skills 数组为空');
      }
      return data.skills || [];
    } catch (err) {
      console.error('[previewService.listDesignSkillsDetailed] ❌ fetch 异常:', err);
      return [];
    }
  },

  async getDesignSkillDetail(id: string): Promise<SkillInfo | null> {
    try {
      await waitReady;
      const url = getApiUrl(`/api/skills/design/${id}`);
      console.log('[previewService.getDesignSkillDetail] 📡 GET', url);
      const response = await fetch(url);
      if (!response.ok) return null;
      const data = await response.json();
      console.log('[previewService.getDesignSkillDetail] 📦 解析完成:', data.name || data.error);
      return data;
    } catch {
      return null;
    }
  },

  async getDesignStats(): Promise<DesignStats> {
    try {
      await waitReady;
      const url = getApiUrl('/api/skills/design/stats');
      console.log('[previewService.getDesignStats] 📡 GET', url);
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 8000);
      const response = await fetch(url, { signal: controller.signal });
      clearTimeout(timer);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data = await response.json();
      console.log('[previewService.getDesignStats] 📦 统计加载完成:', data);
      return data;
    } catch (err) {
      console.error('[previewService.getDesignStats] ❌ fetch 异常:', err);
      return {
        total_design_skills: 0,
        featured_skills: 0,
        by_mode: {},
        by_scenario: {},
        by_fidelity: {},
        by_platform: {},
        today_usage: { today_messages: 0, today_conversations: 0, today_tokens: 0, date: '' },
        caveman: { total_segments: 0, tokens_saved: 0, total_tokens_processed: 0, avg_compression_ratio: 0 },
      };
    }
  },
};

export interface DesignStats {
  total_design_skills: number;
  featured_skills: number;
  by_mode: Record<string, number>;
  by_scenario: Record<string, number>;
  by_fidelity: Record<string, number>;
  by_platform: Record<string, number>;
  today_usage: {
    today_messages: number;
    today_conversations: number;
    today_tokens: number;
    date: string;
  };
  caveman: {
    total_segments: number;
    tokens_saved: number;
    total_tokens_processed: number;
    avg_compression_ratio: number;
  };
}
