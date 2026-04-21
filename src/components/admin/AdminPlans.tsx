import React, { useEffect, useState, useCallback } from 'react';
import { getPlans, addPlan, updatePlan, deletePlan, togglePlan } from '../../adminApi';
import { Plus, Trash2, Power, Edit2, X, Check, RefreshCw } from 'lucide-react';

interface Plan {
  id: number;
  name: string;
  price: number;
  duration_days: number;
  token_quota: number;
  storage_quota: number;
  description: string | null;
  is_active: number;
  created_at: string;
  window_budget: number;
  weekly_budget: number;
}

const EMPTY_FORM = {
  name: '', price_yuan: '', duration_days: 30, quota_dollar: '',
  storage_quota_mb: '', description: '', window_budget: '', weekly_budget: '',
};

export default function AdminPlans() {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editId, setEditId] = useState<number | null>(null);
  const [form, setForm] = useState(EMPTY_FORM);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    try { setPlans(await getPlans()); } catch (e: any) { setError(e.message); }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleSave = async () => {
    if (!form.name || !form.price_yuan || !form.quota_dollar) {
      setError('请填写名称、价格和美元额度'); return;
    }
    setLoading(true);
    const data = {
      name: form.name,
      price: Math.round(parseFloat(form.price_yuan) * 100),
      duration_days: form.duration_days,
      token_quota: Math.round(parseFloat(form.quota_dollar) * 10000),
      storage_quota: form.storage_quota_mb ? Math.round(parseFloat(form.storage_quota_mb) * 1024 * 1024) : 104857600,
      description: form.description || null,
      window_budget: form.window_budget ? parseFloat(form.window_budget) : 0,
      weekly_budget: form.weekly_budget ? parseFloat(form.weekly_budget) : 0,
    };
    try {
      if (editId !== null) await updatePlan(editId, data);
      else await addPlan(data);
      setShowForm(false); setEditId(null); setForm(EMPTY_FORM);
      await load();
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  };

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除此套餐？有活跃订阅时无法删除。')) return;
    try { await deletePlan(id); await load(); } catch (e: any) { setError(e.message); }
  };

  const handleToggle = async (id: number) => {
    try { await togglePlan(id); await load(); } catch (e: any) { setError(e.message); }
  };

  const startEdit = (p: Plan) => {
    setEditId(p.id); setShowForm(true);
    setForm({
      name: p.name,
      price_yuan: (p.price / 100).toString(),
      duration_days: p.duration_days,
      quota_dollar: (p.token_quota / 10000).toString(),
      storage_quota_mb: (p.storage_quota / 1024 / 1024).toFixed(0),
      description: p.description || '',
      window_budget: (p.window_budget || 0).toString(),
      weekly_budget: (p.weekly_budget || 0).toString(),
    });
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-800">套餐管理</h2>
        <div className="flex gap-2">
          <button onClick={load} className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-600 bg-white border border-gray-200 rounded-lg hover:bg-gray-50">
            <RefreshCw size={14} /> 刷新
          </button>
          <button onClick={() => { setShowForm(true); setEditId(null); setForm(EMPTY_FORM); }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600">
            <Plus size={14} /> 添加套餐
          </button>
        </div>
      </div>

      {error && <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">{error} <button onClick={() => setError('')} className="ml-2 underline">关闭</button></div>}

      {showForm && (
        <div className="mb-6 bg-white border border-gray-200 rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-700 mb-3">{editId !== null ? '编辑套餐' : '添加套餐'}</h3>
          <div className="grid grid-cols-2 gap-3">
            <input placeholder="套餐名称 *" value={form.name} onChange={e => setForm({...form, name: e.target.value})}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            <div>
              <label className="block text-xs text-gray-500 mb-1">价格（元）*</label>
              <input type="number" step="0.01" placeholder="49.00" value={form.price_yuan} onChange={e => setForm({...form, price_yuan: e.target.value})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">时长（天）</label>
              <input type="number" value={form.duration_days} onChange={e => setForm({...form, duration_days: Number(e.target.value)})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">美元额度 *</label>
              <input type="number" step="0.01" placeholder="10" value={form.quota_dollar} onChange={e => setForm({...form, quota_dollar: e.target.value})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">存储额度（MB）</label>
              <input type="number" placeholder="100" value={form.storage_quota_mb} onChange={e => setForm({...form, storage_quota_mb: e.target.value})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">窗口预算 (真$/5h)</label>
              <input type="number" step="0.1" placeholder="6.0" value={form.window_budget} onChange={e => setForm({...form, window_budget: e.target.value})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <div>
              <label className="block text-xs text-gray-500 mb-1">周预算 (真$/周)</label>
              <input type="number" step="0.1" placeholder="17.3" value={form.weekly_budget} onChange={e => setForm({...form, weekly_budget: e.target.value})}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm" />
            </div>
            <input placeholder="描述" value={form.description} onChange={e => setForm({...form, description: e.target.value})}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm" />
          </div>
          <div className="flex gap-2 mt-3">
            <button onClick={handleSave} disabled={loading}
              className="flex items-center gap-1.5 px-4 py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50">
              <Check size={14} /> {loading ? '保存中...' : '保存'}
            </button>
            <button onClick={() => { setShowForm(false); setEditId(null); }}
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
              <th className="px-4 py-3 text-left text-gray-600 font-medium">名称</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">价格</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">时长</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">美元额度</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">窗口/5h</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">周预算</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">存储</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">状态</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {plans.length === 0 && (
              <tr><td colSpan={9} className="px-4 py-8 text-center text-gray-400">暂无套餐</td></tr>
            )}
            {plans.map(p => (
              <tr key={p.id} className={`border-b border-gray-100 ${!p.is_active ? 'opacity-50' : ''}`}>
                <td className="px-4 py-3">
                  <div className="font-medium text-gray-800">{p.name}</div>
                  {p.description && <div className="text-xs text-gray-400 mt-0.5">{p.description}</div>}
                </td>
                <td className="px-4 py-3 text-gray-600">¥{(p.price / 100).toFixed(2)}</td>
                <td className="px-4 py-3 text-gray-600">{p.duration_days} 天</td>
                <td className="px-4 py-3 text-gray-600">${(p.token_quota / 10000).toFixed(2)}</td>
                <td className="px-4 py-3 text-gray-600">${(p.window_budget || 0).toFixed(1)}</td>
                <td className="px-4 py-3 text-gray-600">{p.weekly_budget ? `$${p.weekly_budget.toFixed(1)}` : '-'}</td>
                <td className="px-4 py-3 text-gray-600">{(p.storage_quota / 1024 / 1024).toFixed(0)}MB</td>
                <td className="px-4 py-3">
                  <span className={`text-xs px-2 py-0.5 rounded-full ${p.is_active ? 'bg-green-100 text-green-600' : 'bg-gray-100 text-gray-500'}`}>
                    {p.is_active ? '上架' : '下架'}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-1">
                    <button onClick={() => startEdit(p)} className="p-1.5 text-gray-400 hover:text-blue-500 rounded" title="编辑"><Edit2 size={14} /></button>
                    <button onClick={() => handleToggle(p.id)} className="p-1.5 text-gray-400 hover:text-yellow-500 rounded" title={p.is_active ? '下架' : '上架'}><Power size={14} /></button>
                    <button onClick={() => handleDelete(p.id)} className="p-1.5 text-gray-400 hover:text-red-500 rounded" title="删除"><Trash2 size={14} /></button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}