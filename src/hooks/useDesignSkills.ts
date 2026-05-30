import { useState, useEffect, useMemo, useCallback } from 'react';
import { skillService, type SkillInfo } from '../services/previewService';

const CACHE_KEY = 'design_skills_cache';
const CACHE_DURATION = 5 * 60 * 1000;

interface CacheEntry {
  skills: SkillInfo[];
  timestamp: number;
}

export interface DesignSkillsState {
  skills: SkillInfo[];
  filteredSkills: SkillInfo[];
  loading: boolean;
  error: string | null;
  modeFilter: string[];
  scenarioFilter: string[];
  searchQuery: string;
  selectedSkillId: string | null;
  availableModes: string[];
  availableScenarios: string[];
  setModeFilter: (modes: string[]) => void;
  setScenarioFilter: (scenarios: string[]) => void;
  setSearchQuery: (query: string) => void;
  setSelectedSkillId: (id: string | null) => void;
  toggleMode: (mode: string) => void;
  toggleScenario: (scenario: string) => void;
  refresh: () => void;
}

function loadCache(): CacheEntry | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const entry: CacheEntry = JSON.parse(raw);
    if (Date.now() - entry.timestamp > CACHE_DURATION) return null;
    return entry;
  } catch {
    return null;
  }
}

function saveCache(skills: SkillInfo[]): void {
  try {
    localStorage.setItem(
      CACHE_KEY,
      JSON.stringify({ skills, timestamp: Date.now() }),
    );
  } catch {
    //
  }
}

const USE_MOCK = true;

const MOCK_DESIGN_SKILLS: SkillInfo[] = [
  {
    id: 'mock_001',
    name: '移动端 AI 原型生成器',
    description: '快速生成 iOS/Android 高保真原型，支持多个屏幕、交互动画和真实点击跳转',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'mobile',
      scenario: 'design',
      fidelity: 'high',
      featured: 1,
      category: 'UI Generation',
      preview: { type: 'html', width: 375, height: 812 },
      design_system: { requires: true, generates: false, sections: ['colors', 'typography', 'components'] },
      inputs: [{ name: 'prompt', type: 'text', required: true }],
      outputs: { primary: 'HTML/CSS', secondary: 'Figma' },
      example_prompt: '做一个 AI 番茄钟 iOS 原型，4 个核心屏幕：专注、统计、设置、成就，要带真实点击跳转',
    },
  },
  {
    id: 'mock_002',
    name: '产品路演幻灯片',
    description: '自动生成精美的产品路演/融资 PPT 风格幻灯片，含动画和演讲者备注',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'deck',
      platform: 'desktop',
      scenario: 'marketing',
      fidelity: 'high',
      featured: 1,
      category: 'Presentation',
      preview: { type: 'slides', width: 1280, height: 720 },
      design_system: { requires: true, generates: true, sections: ['slides', 'branding'] },
      animations: true,
      speaker_notes: true,
      example_prompt: '生成一个 10 页的 SaaS 产品路演幻灯片，目标客户是企业 HR 部门',
    },
  },
  {
    id: 'mock_003',
    name: '品牌信息图生成器',
    description: '将数据和文案转化为精美的信息图，支持品牌色自动适配和导出',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'image',
      platform: 'desktop',
      scenario: 'marketing',
      fidelity: 'medium',
      featured: 0,
      category: 'Visual Content',
      preview: { type: 'image', width: 1200, height: 1600 },
      outputs: { primary: 'PNG', secondary: 'SVG' },
      example_prompt: '把这份 Q3 销售数据做成品牌信息图，用蓝绿色系，突出增长率',
    },
  },
  {
    id: 'mock_004',
    name: 'UI 设计评审工具',
    description: '自动化 UI 设计评审，检测可访问性、一致性、间距和排版问题',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'review',
      platform: 'desktop',
      scenario: 'engineering',
      fidelity: 'medium',
      featured: 0,
      category: 'Quality Assurance',
      preview: { type: 'html', width: 1440, height: 900 },
      example_prompt: '审查这个登录页面的 UI 设计，重点检查可访问性和响应式布局',
    },
  },
  {
    id: 'mock_005',
    name: '产品导览动画',
    description: '创建产品功能演示动画，支持页面转场、元素入场和交互高亮效果',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'video',
      platform: 'desktop',
      scenario: 'product',
      fidelity: 'high',
      featured: 0,
      category: 'Video Content',
      preview: { type: 'video', width: 1920, height: 1080 },
      animations: true,
      example_prompt: '为新上线的数据看板功能做一个 30 秒的导览动画',
    },
  },
  {
    id: 'mock_006',
    name: '设计系统生成器',
    description: '从零生成完整的设计系统，包含色彩、排版、组件库和交互规范',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'desktop',
      scenario: 'engineering',
      fidelity: 'high',
      featured: 1,
      category: 'Design System',
      preview: { type: 'html', width: 1440, height: 900 },
      design_system: { requires: false, generates: true, sections: ['colors', 'typography', 'components', 'tokens'] },
      outputs: { primary: 'CSS Variables', secondary: 'Figma Tokens' },
      example_prompt: '创建一个 SaaS 管理后台的设计系统，使用蓝色主色调，包含按钮、表单、表格等组件',
    },
  },
  {
    id: 'mock_007',
    name: '个人简历网站',
    description: '一键生成精美个人简历/作品集网站，支持多模板和深色模式',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'desktop',
      scenario: 'personal',
      fidelity: 'high',
      featured: 0,
      category: 'Website Builder',
      preview: { type: 'html', width: 1440, height: 900 },
      example_prompt: '做一个全栈工程师的在线简历，包含项目展示、技能雷达图和时间线',
    },
  },
  {
    id: 'mock_008',
    name: '财务报表看板',
    description: '生成财务数据可视化看板，支持实时数据绑定和多维度图表展示',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'desktop',
      scenario: 'finance',
      fidelity: 'medium',
      featured: 0,
      category: 'Dashboard',
      preview: { type: 'html', width: 1440, height: 900 },
      example_prompt: '生成 CFO 财务报表看板，包含收入趋势、支出明细、利润瀑布图',
    },
  },
  {
    id: 'mock_009',
    name: 'HR 入职流程页面',
    description: '创建企业 HR 入职流程的多步骤页面，包含表单验证和进度追踪',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'desktop',
      scenario: 'hr',
      fidelity: 'medium',
      featured: 0,
      category: 'HR Tools',
      preview: { type: 'html', width: 1440, height: 900 },
      inputs: [{ name: 'company_name', type: 'text', required: true }],
      example_prompt: '创建新员工入职流程的 5 步表单页面，包含信息填写、文档签署和欢迎页',
    },
  },
  {
    id: 'mock_010',
    name: '电商促销活动页',
    description: '快速生成电商促销活动落地页，支持倒计时、限时抢购和动效',
    category: 'design',
    enabled: true,
    source: 'user',
    loaded_from: 'user',
    is_example: false,
    od_metadata: {
      mode: 'prototype',
      platform: 'mobile',
      scenario: 'sale',
      fidelity: 'high',
      featured: 0,
      category: 'E-Commerce',
      preview: { type: 'html', width: 375, height: 812 },
      animations: true,
      example_prompt: '做一个双十一美妆品牌促销活动页，包含倒计时、秒杀商品和凑单推荐',
    },
  },
];

export function useDesignSkills(): DesignSkillsState {
  const [skills, setSkills] = useState<SkillInfo[]>(() => loadCache()?.skills || []);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [modeFilter, setModeFilter] = useState<string[]>([]);
  const [scenarioFilter, setScenarioFilter] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);

  const fetchSkills = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const cached = loadCache();
      if (cached) {
        console.log('[useDesignSkills] 📦 命中缓存，共', cached.skills.length, '个技能');
        setSkills(cached.skills);
        setLoading(false);
        return;
      }

      console.log('[useDesignSkills] 🔄 开始请求 /api/skills/design ...');
      const startTime = performance.now();
      const result = await skillService.listDesignSkillsDetailed();
      const elapsed = (performance.now() - startTime).toFixed(0);

      if (result && result.length > 0) {
        console.log(`[useDesignSkills] ✅ API 返回 ${result.length} 个技能 (${elapsed}ms)`);
        console.log('[useDesignSkills] 📋 技能详情:', result.map(s => ({
          id: s.id,
          name: s.name,
          mode: s.od_metadata?.mode,
          scenario: s.od_metadata?.scenario,
          fidelity: s.od_metadata?.fidelity,
          hasOdMetadata: !!s.od_metadata,
        })));
        setSkills(result);
        saveCache(result);
      } else {
        console.warn(`[useDesignSkills] ⚠️ API 返回空数组 (${elapsed}ms)，使用 MOCK 数据`);
        if (USE_MOCK) {
          console.log('[useDesignSkills] 🎭 加载 MOCK 数据，共', MOCK_DESIGN_SKILLS.length, '个技能');
          setSkills(MOCK_DESIGN_SKILLS);
        } else {
          setSkills([]);
        }
      }
    } catch (err) {
      console.error('[useDesignSkills] ❌ API 请求失败:', err instanceof Error ? err.message : err);
      if (USE_MOCK) {
        console.log('[useDesignSkills] 🎭 使用 MOCK 数据作为降级方案');
        setSkills(MOCK_DESIGN_SKILLS);
        setError(null);
      } else {
        setError(err instanceof Error ? err.message : 'Failed to load design skills');
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSkills();
  }, [fetchSkills]);

  const availableModes = useMemo(() => {
    const modes = new Set<string>();
    skills.forEach(s => {
      if (s.od_metadata?.mode) modes.add(s.od_metadata.mode);
    });
    return Array.from(modes).sort();
  }, [skills]);

  const availableScenarios = useMemo(() => {
    const scenarios = new Set<string>();
    skills.forEach(s => {
      if (s.od_metadata?.scenario) scenarios.add(s.od_metadata.scenario);
    });
    return Array.from(scenarios).sort();
  }, [skills]);

  const filteredSkills = useMemo(() => {
    let result = skills;

    if (modeFilter.length > 0) {
      result = result.filter(s => {
        const mode = s.od_metadata?.mode;
        return mode && modeFilter.includes(mode);
      });
    }

    if (scenarioFilter.length > 0) {
      result = result.filter(s => {
        const scenario = s.od_metadata?.scenario;
        return scenario && scenarioFilter.includes(scenario);
      });
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      result = result.filter(s =>
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.od_metadata?.example_prompt?.toLowerCase().includes(q),
      );
    }

    return result;
  }, [skills, modeFilter, scenarioFilter, searchQuery]);

  const toggleMode = useCallback((mode: string) => {
    setModeFilter(prev =>
      prev.includes(mode) ? prev.filter(m => m !== mode) : [...prev, mode],
    );
  }, []);

  const toggleScenario = useCallback((scenario: string) => {
    setScenarioFilter(prev =>
      prev.includes(scenario) ? prev.filter(s => s !== scenario) : [...prev, scenario],
    );
  }, []);

  return {
    skills,
    filteredSkills,
    loading,
    error,
    modeFilter,
    scenarioFilter,
    searchQuery,
    selectedSkillId,
    availableModes,
    availableScenarios,
    setModeFilter,
    setScenarioFilter,
    setSearchQuery,
    setSelectedSkillId,
    toggleMode,
    toggleScenario,
    refresh: fetchSkills,
  };
}
