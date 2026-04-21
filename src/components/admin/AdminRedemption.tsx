import React, { useEffect, useState, useCallback } from 'react';
import { getPlans, generateRedemptionCodes, getRedemptionCodes, disableRedemptionCodes } from '../../adminApi';
import { Plus, Ban, Copy, ChevronLeft, ChevronRight, RefreshCw, X, Check } from 'lucide-react';
import { copyToClipboard } from '../../utils/clipboard';

interface Code {
  id: number;
  code: string;
  plan_id: number;
  plan_name: string;
  status: string;
  batch_id: string;
  note: string | null;
  created_at: string;
  used_at: string | null;
  used_by: string | null;
  expires_at: string | null;
}

interface Stats { total: number; unused: number; used: number; expired: number; disabled: number; }

const STATUS_COLORS: Record<string, string> = {
  unused: 'bg-green-100 text-green-600',
  used: 'bg-blue-100 text-blue-600',
  expired: 'bg-gray-100 text-gray-500',
  disabled: 'bg-red-100 text-red-600',
};

const STATUS_LABELS: Record<string, string> = {
  unused: '未使用', used: '已使用', expired: '已过期', disabled: '已禁用',
};

export default function AdminRedemption() {
  const [codes, setCodes] = useState<Code[]>([]);
  const [stats, setStats] = useState<Stats>({ total: 0, unused: 0, used: 0, expired: 0, disabled: 0 });
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState('');
  const [batchFilter, setBatchFilter] = useState('');
  const [error, setError] = useState('');
  const [plans, setPlans] = useState<any[]>([]);
  const [showGenerate, setShowGenerate] = useState(false);
  const [genForm, setGenForm] = useState({ plan_id: 0, count: 10, expires_days: 90, note: '' });
  const [generatedCodes, setGeneratedCodes] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [copyFeedback, setCopyFeedback] = useState(false);
  const [quickLoadingPlanId, setQuickLoadingPlanId] = useState<number | null>(null);
  const [quickCopiedPlanId, setQuickCopiedPlanId] = useState<number | null>(null);
  const limit = 20;

  const load = useCallback(async () => {
    try {
      const params: any = { page, limit };
      if (statusFilter) params.status = statusFilter;
      if (batchFilter) params.batch_id = batchFilter;
      const data = await getRedemptionCodes(params);
      setCodes(data.codes);
      setStats(data.stats);
      setTotal(data.pagination.total);
    } catch (e: any) { setError(e.message); }
  }, [page, statusFilter, batchFilter]);

  useEffect(() => { load(); }, [load]);
  useEffect(() => {
    getPlans().then((p: any[]) => {
      const active = p.filter((x: any) => x.is_active);
      setPlans(active);
      if (active.length > 0 && !genForm.plan_id) setGenForm(f => ({ ...f, plan_id: active[0].id }));
    }).catch(() => { });
  }, []);

  const handleGenerate = async () => {
    if (!genForm.plan_id) { setError('请选择套餐'); return; }
    setLoading(true);
    try {
      const res = await generateRedemptionCodes(genForm);
      setGeneratedCodes(res.codes);
      await load();
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  };

  const handleDisable = async (code: string) => {
    if (!confirm(`确定禁用 ${code}？`)) return;
    try { await disableRedemptionCodes([code]); await load(); } catch (e: any) { setError(e.message); }
  };

  const copyAll = async () => {
    const success = await copyToClipboard(generatedCodes.join('\n'));
    if (success) {
      setCopyFeedback(true);
      setTimeout(() => setCopyFeedback(false), 2000);
    } else {
      setError('复制失败，请手动复制');
    }
  };

  const buildRedeemMessage = (code: string) => (
    `您的兑换码：${code}\n` +
    '请到 Claude Rust 注册一个账号，注册完成后在用户面板进行套餐的兑换。感谢您的支持~'
  );

  const handleQuickGenerateAndCopy = async (plan: any) => {
    if (!plan?.id) return;
    setError('');
    setQuickLoadingPlanId(plan.id);
    try {
      const res = await generateRedemptionCodes({
        plan_id: plan.id,
        count: 1,
        expires_days: genForm.expires_days,
        note: genForm.note || '后台快捷生成并复制',
      });
      const code = Array.isArray(res?.codes) ? res.codes[0] : '';
      if (!code) throw new Error('兑换码生成失败，请重试');
      setGeneratedCodes([code]);
      await load();
      const copied = await copyToClipboard(buildRedeemMessage(code));
      if (!copied) {
        throw new Error(`兑换码 ${code} 已生成，但复制失败，请手动复制`);
      }
      setQuickCopiedPlanId(plan.id);
      setTimeout(() => setQuickCopiedPlanId((prev) => (prev === plan.id ? null : prev)), 2000);
    } catch (e: any) {
      setError(e?.message || '生成失败，请重试');
    } finally {
      setQuickLoadingPlanId(null);
    }
  };

  const totalPages = Math.ceil(total / limit);

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-800">兑换码管理</h2>
        <div className="flex gap-2">
          <button onClick={load} className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-600 bg-white border border-gray-200 rounded-lg hover:bg-gray-50">
            <RefreshCw size={14} /> 刷新
          </button>
          <button onClick={() => { setShowGenerate(true); setGeneratedCodes([]); }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600">
            <Plus size={14} /> 生成兑换码
          </button>
        </div>
      </div>

      {error && <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">{error} <button onClick={() => setError('')} className="ml-2 underline">关闭</button></div>}

      {plans.length > 0 && (
        <div className="mb-4 p-4 bg-white border border-gray-200 rounded-xl">
          <div className="text-sm font-medium text-gray-800 mb-2">一键生成并复制兑换话术</div>
          <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-2">
            {plans.slice(0, 4).map((plan: any) => {
              const isLoading = quickLoadingPlanId === plan.id;
              const copied = quickCopiedPlanId === plan.id;
              return (
                <button
                  key={`quick-${plan.id}`}
                  onClick={() => handleQuickGenerateAndCopy(plan)}
                  disabled={isLoading}
                  className="px-3 py-2 text-sm rounded-lg border border-blue-200 text-blue-600 bg-blue-50 hover:bg-blue-100 disabled:opacity-60 disabled:cursor-not-allowed"
                >
                  {isLoading ? '生成中...' : copied ? '已复制到剪贴板' : `${plan.name}（一键生成）`}
                </button>
              );
            })}
          </div>
          <div className="mt-2 text-xs text-gray-500">点击后自动生成 1 条兑换码，并复制完整发送文案。</div>
        </div>
      )}

      <div className="flex gap-3 mb-4">
        {(['unused', 'used', 'expired', 'disabled'] as const).map(s => (
          <span key={s} className={`text-xs px-2.5 py-1 rounded-full ${STATUS_COLORS[s]}`}>
            {STATUS_LABELS[s]} {(stats as any)[s] || 0}
          </span>
        ))}
      </div>
      <div className="flex gap-2 mb-4">
        <select value={statusFilter} onChange={e => { setStatusFilter(e.target.value); setPage(1); }}
          className="px-3 py-2 border border-gray-200 rounded-lg text-sm bg-white">
          <option value="">全部状态</option>
          <option value="unused">未使用</option>
          <option value="used">已使用</option>
          <option value="expired">已过期</option>
          <option value="disabled">已禁用</option>
        </select>
        <input placeholder="批次 ID" value={batchFilter} onChange={e => setBatchFilter(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && load()}
          className="px-3 py-2 border border-gray-200 rounded-lg text-sm w-64" />
      </div>

      <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">兑换码</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">套餐</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">状态</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">批次</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">创建时间</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">使用/过期</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">使用者</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {codes.length === 0 && (
              <tr><td colSpan={8} className="px-4 py-8 text-center text-gray-400">无数据</td></tr>
            )}
            {codes.map(c => (
              <tr key={c.id} className="border-b border-gray-100">
                <td className="px-4 py-3 font-mono text-xs text-gray-700">{c.code}</td>
                <td className="px-4 py-3 text-gray-600">{c.plan_name || '-'}</td>
                <td className="px-4 py-3">
                  <span className={`text-xs px-2 py-0.5 rounded-full ${STATUS_COLORS[c.status] || 'bg-gray-100 text-gray-500'}`}>
                    {STATUS_LABELS[c.status] || c.status}
                  </span>
                </td>
                <td className="px-4 py-3 text-xs text-gray-400 max-w-[120px] truncate" title={c.batch_id}>{c.batch_id}</td>
                <td className="px-4 py-3 text-xs text-gray-400">{c.created_at?.slice(0, 16)}</td>
                <td className="px-4 py-3 text-xs text-gray-400">
                  {c.used_at ? `使用 ${c.used_at.slice(0, 16)}` : c.expires_at ? `过期 ${c.expires_at.slice(0, 16)}` : '-'}
                </td>
                <td className="px-4 py-3 text-xs text-gray-400">{c.used_by || '-'}</td>
                <td className="px-4 py-3">
                  {c.status === 'unused' && (
                    <button onClick={() => handleDisable(c.code)} className="p-1.5 text-gray-400 hover:text-red-500 rounded" title="禁用">
                      <Ban size={14} />
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {totalPages > 1 && (
        <div className="flex items-center justify-between mt-4">
          <span className="text-sm text-gray-500">共 {total} 条</span>
          <div className="flex items-center gap-2">
            <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page <= 1}
              className="p-1.5 text-gray-400 hover:text-gray-600 disabled:opacity-30 rounded border border-gray-200">
              <ChevronLeft size={16} />
            </button>
            <span className="text-sm text-gray-600">{page} / {totalPages}</span>
            <button onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page >= totalPages}
              className="p-1.5 text-gray-400 hover:text-gray-600 disabled:opacity-30 rounded border border-gray-200">
              <ChevronRight size={16} />
            </button>
          </div>
        </div>
      )}

      {showGenerate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-2xl shadow-xl w-[480px] p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-base font-semibold text-gray-800">生成兑换码</h3>
              <button onClick={() => setShowGenerate(false)} className="text-gray-400 hover:text-gray-600"><X size={18} /></button>
            </div>
            {generatedCodes.length > 0 ? (
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-green-600">已生成 {generatedCodes.length} 个兑换码</span>
                  <button onClick={copyAll} className="flex items-center gap-1 text-xs text-blue-500 hover:text-blue-600">
                    {copyFeedback ? <Check size={12} /> : <Copy size={12} />} {copyFeedback ? '已复制' : '复制全部'}
                  </button>
                </div>
                <div className="bg-gray-50 rounded-lg p-3 max-h-60 overflow-auto font-mono text-xs space-y-1">
                  {generatedCodes.map(c => <div key={c}>{c}</div>)}
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div>
                  <label className="block text-xs text-gray-500 mb-1">套餐 *</label>
                  <select value={genForm.plan_id} onChange={e => setGenForm({ ...genForm, plan_id: Number(e.target.value) })}
                    className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm bg-white">
                    {plans.map(p => <option key={p.id} value={p.id}>{p.name} (¥{(p.price / 100).toFixed(2)})</option>)}
                  </select>
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">数量 (1-100)</label>
                    <input type="number" min={1} max={100} value={genForm.count}
                      onChange={e => setGenForm({ ...genForm, count: Number(e.target.value) })}
                      className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
                  </div>
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">有效天数</label>
                    <input type="number" value={genForm.expires_days}
                      onChange={e => setGenForm({ ...genForm, expires_days: Number(e.target.value) })}
                      className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
                  </div>
                </div>
                <input placeholder="备注（可选）" value={genForm.note} onChange={e => setGenForm({ ...genForm, note: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
                <button onClick={handleGenerate} disabled={loading}
                  className="w-full py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50">
                  {loading ? '生成中...' : '生成'}
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
