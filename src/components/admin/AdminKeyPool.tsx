import React, { useEffect, useState, useCallback } from 'react';
import { getKeys, addKey, updateKey, deleteKey, toggleKey, getPoolStatus, getRecharges, addRecharge, deleteRecharge, getUpstreamRoutes, updateUpstreamRoutes } from '../../adminApi';
import { Plus, Trash2, Power, RefreshCw, Edit2, X, Check, DollarSign, ChevronDown, ChevronUp } from 'lucide-react';

interface ApiKey {
  id: number;
  api_key: string;
  base_url: string;
  relay_name: string | null;
  relay_url: string | null;
  max_concurrency: number;
  enabled: number;
  priority: number;
  weight: number;
  note: string | null;
  health_status: string;
  consecutive_errors: number;
  daily_tokens_input: number;
  daily_tokens_output: number;
  daily_request_count: number;
  last_request_at: string | null;
  last_error: string | null;
  created_at: string;
  input_rate: number;
  output_rate: number;
  group_multiplier: number;
  charge_rate: number;
}

interface PoolItem {
  id: number;
  current_concurrency: number;
  health_status: string;
}

type ModelGroup = 'opus' | 'sonnet' | 'haiku' | 'gpt';

interface UpstreamRouteItem {
  model_group: ModelGroup;
  base_url: string;
  preferred_key_id: number | null;
  updated_at?: string | null;
}

type UpstreamRouteMap = Record<ModelGroup, UpstreamRouteItem>;

const EMPTY_FORM = {
  api_key: '', base_url: '', relay_name: '', relay_url: '',
  max_concurrency: 3, priority: 0, weight: 1, note: '',
  input_rate: 0, output_rate: 0, group_multiplier: 1.0, charge_rate: 0,
};

const EMPTY_ROUTE_MAP: UpstreamRouteMap = {
  opus: { model_group: 'opus', base_url: '', preferred_key_id: null },
  sonnet: { model_group: 'sonnet', base_url: '', preferred_key_id: null },
  haiku: { model_group: 'haiku', base_url: '', preferred_key_id: null },
  gpt: { model_group: 'gpt', base_url: '', preferred_key_id: null },
};

function StatusDot({ status }: { status: string }) {
  const color = status === 'healthy' ? 'bg-green-500' : status === 'degraded' ? 'bg-yellow-500' : 'bg-red-500';
  return <span className={`inline-block w-2.5 h-2.5 rounded-full ${color}`} title={status} />;
}

export default function AdminKeyPool() {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [pool, setPool] = useState<PoolItem[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [editId, setEditId] = useState<number | null>(null);
  const [form, setForm] = useState(EMPTY_FORM);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [showRecharge, setShowRecharge] = useState(false);
  const [rechargeForm, setRechargeForm] = useState({ amount_cny: '', key_ids: [] as number[], remark: '' });
  const [recharges, setRecharges] = useState<any[]>([]);
  const [showRechargeList, setShowRechargeList] = useState(false);
  const [upstreamRoutes, setUpstreamRoutes] = useState<UpstreamRouteMap>(EMPTY_ROUTE_MAP);
  const [savingRoutes, setSavingRoutes] = useState(false);

  const load = useCallback(async () => {
    try {
      const [k, p, routeResp] = await Promise.all([getKeys(), getPoolStatus(), getUpstreamRoutes()]);
      setKeys(k);
      setPool(p);
      if (routeResp && routeResp.routes) {
        setUpstreamRoutes({
          opus: routeResp.routes.opus || EMPTY_ROUTE_MAP.opus,
          sonnet: routeResp.routes.sonnet || EMPTY_ROUTE_MAP.sonnet,
          haiku: routeResp.routes.haiku || EMPTY_ROUTE_MAP.haiku,
          gpt: routeResp.routes.gpt || EMPTY_ROUTE_MAP.gpt,
        });
      }
    } catch (e: any) { setError(e.message); }
  }, []);

  useEffect(() => { load(); const t = setInterval(load, 10000); return () => clearInterval(t); }, [load]);

  const handleAdd = async () => {
    if (!form.api_key || !form.base_url) { setError('请填写 API Key 和 Base URL'); return; }
    setLoading(true);
    try {
      await addKey(form);
      setForm(EMPTY_FORM);
      setShowAdd(false);
      await load();
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  };

  const handleUpdate = async () => {
    if (editId === null) return;
    setLoading(true);
    try {
      await updateKey(editId, form);
      setEditId(null);
      setForm(EMPTY_FORM);
      await load();
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  };

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除此密钥？')) return;
    try { await deleteKey(id); await load(); } catch (e: any) { setError(e.message); }
  };

  const handleToggle = async (id: number) => {
    try { await toggleKey(id); await load(); } catch (e: any) { setError(e.message); }
  };

  const startEdit = (k: ApiKey) => {
    setEditId(k.id);
    setShowAdd(false);
    setForm({
      api_key: '', base_url: k.base_url, relay_name: k.relay_name || '',
      relay_url: k.relay_url || '', max_concurrency: k.max_concurrency,
      priority: k.priority, weight: k.weight, note: k.note || '',
      input_rate: k.input_rate || 0, output_rate: k.output_rate || 0,
      group_multiplier: k.group_multiplier || 1.0, charge_rate: k.charge_rate || 0,
    });
  };

  const getConcurrency = (id: number) => {
    const p = pool.find(p => p.id === id);
    return p ? p.current_concurrency : 0;
  };

  const updateRoute = (group: ModelGroup, patch: Partial<UpstreamRouteItem>) => {
    setUpstreamRoutes(prev => ({
      ...prev,
      [group]: { ...prev[group], ...patch, model_group: group },
    }));
  };

  const handleRouteKeyChange = (group: ModelGroup, value: string) => {
    const keyId = value ? Number(value) : null;
    const key = keyId ? keys.find(k => k.id === keyId) : null;
    updateRoute(group, {
      preferred_key_id: keyId,
      base_url: key ? key.base_url : upstreamRoutes[group].base_url,
    });
  };

  const handleSaveRoutes = async () => {
    setSavingRoutes(true);
    try {
      await updateUpstreamRoutes(upstreamRoutes);
      await load();
    } catch (e: any) {
      setError(e.message);
    }
    setSavingRoutes(false);
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-800">密钥池管理</h2>
        <div className="flex gap-2">
          <button onClick={load} className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-600 bg-white border border-gray-200 rounded-lg hover:bg-gray-50">
            <RefreshCw size={14} /> 刷新
          </button>
          <button onClick={() => { setShowRecharge(true); setRechargeForm({ amount_cny: '', key_ids: [], remark: '' }); }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-emerald-500 rounded-lg hover:bg-emerald-600">
            <DollarSign size={14} /> 录入充值
          </button>
          <button onClick={() => { setShowAdd(true); setEditId(null); setForm(EMPTY_FORM); }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600">
            <Plus size={14} /> 添加密钥
          </button>
        </div>
      </div>

      {error && <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">{error} <button onClick={() => setError('')} className="ml-2 underline">关闭</button></div>}

      <div className="mb-6 bg-white border border-gray-200 rounded-xl p-5">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-medium text-gray-700">模型上游路由</h3>
          <button
            onClick={handleSaveRoutes}
            disabled={savingRoutes}
            className="px-3 py-1.5 text-xs text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50"
          >
            {savingRoutes ? '保存中...' : '保存路由'}
          </button>
        </div>
        <div className="text-xs text-gray-500 mb-3">
          可按模型族指定上游地址和优先密钥。优先密钥不可用时，会回退到同一上游地址下的其他可用密钥。
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3">
          {([
            { group: 'sonnet', label: 'Sonnet' },
            { group: 'haiku', label: 'Haiku' },
            { group: 'opus', label: 'Opus' },
            { group: 'gpt', label: 'GPT' },
          ] as Array<{ group: ModelGroup; label: string }>).map(({ group, label }) => (
            <div key={group} className="border border-gray-200 rounded-lg p-3">
              <div className="text-xs font-medium text-gray-700 mb-2">{label}</div>
              <input
                value={upstreamRoutes[group]?.base_url || ''}
                onChange={e => updateRoute(group, { base_url: e.target.value })}
                placeholder="https://api.example.com"
                className="w-full mb-2 px-2 py-1.5 border border-gray-200 rounded text-xs"
              />
              <select
                value={upstreamRoutes[group]?.preferred_key_id ?? ''}
                onChange={e => handleRouteKeyChange(group, e.target.value)}
                className="w-full px-2 py-1.5 border border-gray-200 rounded text-xs"
              >
                <option value="">不指定（按权重）</option>
                {keys
                  .filter(k => !upstreamRoutes[group]?.base_url || k.base_url === upstreamRoutes[group]?.base_url)
                  .map(k => (
                    <option key={k.id} value={k.id}>
                      #{k.id} {k.relay_name || k.note || k.api_key} {k.enabled ? '' : '(已禁用)'}
                    </option>
                  ))}
              </select>
            </div>
          ))}
        </div>
      </div>

      {(showAdd || editId !== null) && (
        <div className="mb-6 bg-white border border-gray-200 rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-700 mb-3">{editId !== null ? '编辑密钥' : '添加密钥'}</h3>
          <div className="grid grid-cols-2 gap-3">
            {editId === null && (
              <input placeholder="API Key *" value={form.api_key} onChange={e => setForm({...form, api_key: e.target.value})}
                className="col-span-2 px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            )}
            <input placeholder="Base URL *" value={form.base_url} onChange={e => setForm({...form, base_url: e.target.value})}
              className="col-span-2 px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            <input placeholder="中转名称" value={form.relay_name} onChange={e => setForm({...form, relay_name: e.target.value})}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            <input placeholder="中转 URL" value={form.relay_url} onChange={e => setForm({...form, relay_url: e.target.value})}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            <div>
              <label className="block text-xs text-gray-500 mb-1">最大并发 <span className="text-gray-400">— 同时处理的请求上限</span></label>
              <input type="number" value={form.max_concurrency} onChange={e => setForm({...form, max_concurrency: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">优先级 <span className="text-gray-400">— 越大越优先选用</span></label>
              <input type="number" value={form.priority} onChange={e => setForm({...form, priority: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">权重 <span className="text-gray-400">— 同优先级下的分配比例</span></label>
              <input type="number" value={form.weight} onChange={e => setForm({...form, weight: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <input placeholder="备注" value={form.note} onChange={e => setForm({...form, note: e.target.value})}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            <div>
              <label className="block text-xs text-gray-500 mb-1">分组倍率 <span className="text-gray-400">— 该密钥的计费倍率</span></label>
              <input type="number" step="0.1" value={form.group_multiplier} onChange={e => setForm({...form, group_multiplier: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">充值单价 (¥/站内$) <span className="text-gray-400">— 真实单价 = 充值单价 × 倍率</span></label>
              <input type="number" step="0.01" value={form.charge_rate} onChange={e => setForm({...form, charge_rate: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
          </div>
          <div className="flex gap-2 mt-3">
            <button onClick={editId !== null ? handleUpdate : handleAdd} disabled={loading}
              className="flex items-center gap-1.5 px-4 py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50">
              <Check size={14} /> {loading ? '保存中...' : '保存'}
            </button>
            <button onClick={() => { setShowAdd(false); setEditId(null); }}
              className="flex items-center gap-1.5 px-4 py-2 text-sm text-gray-600 bg-gray-100 rounded-lg hover:bg-gray-200">
              <X size={14} /> 取消
            </button>
          </div>
        </div>
      )}

      <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">状态</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">API Key</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">中转</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">并发</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">充值单价</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">倍率</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">真实单价</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">今日消耗</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">错误</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {keys.length === 0 && (
              <tr><td colSpan={9} className="px-4 py-8 text-center text-gray-400">暂无密钥，点击上方"添加密钥"开始</td></tr>
            )}
            {keys.map(k => (
              <tr key={k.id} className={`border-b border-gray-100 ${!k.enabled ? 'opacity-50' : ''}`}>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-2">
                    <StatusDot status={k.health_status} />
                    <span className="text-xs text-gray-500">{k.enabled ? '启用' : '禁用'}</span>
                  </div>
                </td>
                <td className="px-4 py-3 font-mono text-xs text-gray-600">{k.api_key}</td>
                <td className="px-4 py-3 text-gray-600">{k.relay_name || '-'}</td>
                <td className="px-4 py-3">
                  <span className="text-blue-600 font-medium">{getConcurrency(k.id)}</span>
                  <span className="text-gray-400">/{k.max_concurrency}</span>
                </td>
                <td className="px-4 py-3 text-gray-600">¥{(k.charge_rate || 0).toFixed(2)}</td>
                <td className="px-4 py-3 text-xs text-gray-600">{k.group_multiplier || 1.0}x</td>
                <td className="px-4 py-3 text-xs text-gray-600">¥{((k.charge_rate || 0) * (k.group_multiplier || 1)).toFixed(2)}</td>
                <td className="px-4 py-3 text-xs text-gray-500">
                  入 {((k.daily_tokens_input || 0) / 1000).toFixed(1)}K / 出 {((k.daily_tokens_output || 0) / 1000).toFixed(1)}K
                </td>
                <td className="px-4 py-3">
                  {k.consecutive_errors > 0 ? (
                    <span className="text-red-500 text-xs" title={k.last_error || ''}>{k.consecutive_errors} 次</span>
                  ) : <span className="text-green-500 text-xs">正常</span>}
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-1">
                    <button onClick={() => startEdit(k)} className="p-1.5 text-gray-400 hover:text-blue-500 rounded" title="编辑">
                      <Edit2 size={14} />
                    </button>
                    <button onClick={() => handleToggle(k.id)} className="p-1.5 text-gray-400 hover:text-yellow-500 rounded" title={k.enabled ? '禁用' : '启用'}>
                      <Power size={14} />
                    </button>
                    <button onClick={() => handleDelete(k.id)} className="p-1.5 text-gray-400 hover:text-red-500 rounded" title="删除">
                      <Trash2 size={14} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Recharge Modal */}
      {showRecharge && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center z-50" onClick={() => setShowRecharge(false)}>
          <div className="bg-white rounded-xl p-6 w-[480px] max-h-[80vh] overflow-y-auto" onClick={e => e.stopPropagation()}>
            <h3 className="text-sm font-medium text-gray-700 mb-4">录入充值</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-xs text-gray-500 mb-1">充值金额 (¥) *</label>
                <input type="number" step="0.01" placeholder="100.00" value={rechargeForm.amount_cny}
                  onChange={e => setRechargeForm({...rechargeForm, amount_cny: e.target.value})}
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
              </div>
              <div>
                <label className="block text-xs text-gray-500 mb-1">关联密钥（可多选）</label>
                <div className="max-h-32 overflow-y-auto border border-gray-200 rounded-lg p-2 space-y-1">
                  {keys.map(k => (
                    <label key={k.id} className="flex items-center gap-2 text-xs text-gray-600 cursor-pointer hover:bg-gray-50 px-1 py-0.5 rounded">
                      <input type="checkbox" checked={rechargeForm.key_ids.includes(k.id)}
                        onChange={e => {
                          const ids = e.target.checked
                            ? [...rechargeForm.key_ids, k.id]
                            : rechargeForm.key_ids.filter(id => id !== k.id);
                          setRechargeForm({...rechargeForm, key_ids: ids});
                        }} />
                      {k.note || k.relay_name || k.api_key} (#{k.id})
                    </label>
                  ))}
                </div>
              </div>
              <div>
                <label className="block text-xs text-gray-500 mb-1">备注</label>
                <input placeholder="可选备注" value={rechargeForm.remark}
                  onChange={e => setRechargeForm({...rechargeForm, remark: e.target.value})}
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
              </div>
            </div>
            <div className="flex gap-2 mt-4">
              <button onClick={async () => {
                if (!rechargeForm.amount_cny) { setError('请填写充值金额'); return; }
                try {
                  await addRecharge({ amount_cny: parseFloat(rechargeForm.amount_cny), key_ids: rechargeForm.key_ids, remark: rechargeForm.remark });
                  setShowRecharge(false);
                  if (showRechargeList) getRecharges().then(setRecharges).catch(() => {});
                } catch (e: any) { setError(e.message); }
              }} className="flex items-center gap-1.5 px-4 py-2 text-sm text-white bg-emerald-500 rounded-lg hover:bg-emerald-600">
                <Check size={14} /> 保存
              </button>
              <button onClick={() => setShowRecharge(false)}
                className="flex items-center gap-1.5 px-4 py-2 text-sm text-gray-600 bg-gray-100 rounded-lg hover:bg-gray-200">
                <X size={14} /> 取消
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Recharge Records */}
      <div className="mt-6">
        <button onClick={() => {
          setShowRechargeList(!showRechargeList);
          if (!showRechargeList) getRecharges().then(setRecharges).catch(() => {});
        }} className="flex items-center gap-1.5 text-sm text-gray-600 hover:text-gray-800">
          {showRechargeList ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          充值记录 ({recharges.length})
        </button>
        {showRechargeList && (
          <div className="mt-3 bg-white border border-gray-200 rounded-xl overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">时间</th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">金额</th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">关联密钥</th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">备注</th>
                  <th className="px-4 py-2 text-left text-gray-600 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {recharges.length === 0 && (
                  <tr><td colSpan={5} className="px-4 py-4 text-center text-gray-400">暂无充值记录</td></tr>
                )}
                {recharges.map((r: any) => (
                  <tr key={r.id} className="border-b border-gray-100">
                    <td className="px-4 py-2 text-gray-500">{r.created_at?.slice(0, 16)}</td>
                    <td className="px-4 py-2 text-emerald-600 font-medium">¥{r.amount_cny?.toFixed(2)}</td>
                    <td className="px-4 py-2 text-gray-500 text-xs">{(() => { try { const ids = JSON.parse(r.key_ids); return ids.length ? ids.join(', ') : '-'; } catch { return '-'; } })()}</td>
                    <td className="px-4 py-2 text-gray-500">{r.remark || '-'}</td>
                    <td className="px-4 py-2">
                      <button onClick={async () => {
                        if (!confirm('确定删除此充值记录？')) return;
                        try { await deleteRecharge(r.id); setRecharges(prev => prev.filter((x: any) => x.id !== r.id)); } catch (e: any) { setError(e.message); }
                      }} className="p-1 text-gray-400 hover:text-red-500"><Trash2 size={13} /></button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
