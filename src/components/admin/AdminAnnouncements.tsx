import React, { useCallback, useEffect, useState } from 'react';
import {
  getAnnouncements,
  addAnnouncement,
  updateAnnouncement,
  deleteAnnouncement,
  toggleAnnouncement,
} from '../../adminApi';
import { Plus, RefreshCw, Edit2, Trash2, Power, Check, X } from 'lucide-react';

interface Announcement {
  id: number;
  title: string;
  content: string;
  is_active: number;
  read_count: number;
  created_at: string;
  updated_at: string;
}

const EMPTY_FORM = {
  title: '',
  content: '',
  is_active: true,
};

export default function AdminAnnouncements() {
  const [announcements, setAnnouncements] = useState<Announcement[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editId, setEditId] = useState<number | null>(null);
  const [form, setForm] = useState(EMPTY_FORM);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    try {
      setAnnouncements(await getAnnouncements());
    } catch (e: any) {
      setError(e.message || '加载公告失败');
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const resetForm = () => {
    setShowForm(false);
    setEditId(null);
    setForm(EMPTY_FORM);
  };

  const handleSave = async () => {
    if (!form.title.trim()) {
      setError('请填写公告标题');
      return;
    }
    if (!form.content.trim()) {
      setError('请填写公告内容');
      return;
    }

    setLoading(true);
    try {
      const payload = {
        title: form.title.trim(),
        content: form.content.trim(),
        is_active: form.is_active ? 1 : 0,
      };

      if (editId !== null) {
        await updateAnnouncement(editId, payload);
      } else {
        await addAnnouncement(payload);
      }

      resetForm();
      await load();
    } catch (e: any) {
      setError(e.message || '保存失败');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除这条公告？删除后无法恢复。')) return;
    try {
      await deleteAnnouncement(id);
      await load();
    } catch (e: any) {
      setError(e.message || '删除失败');
    }
  };

  const handleToggle = async (id: number) => {
    try {
      await toggleAnnouncement(id);
      await load();
    } catch (e: any) {
      setError(e.message || '切换状态失败');
    }
  };

  const startEdit = (item: Announcement) => {
    setEditId(item.id);
    setShowForm(true);
    setForm({
      title: item.title || '',
      content: item.content || '',
      is_active: !!item.is_active,
    });
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold text-gray-800">公告管理</h2>
          <p className="text-sm text-gray-500 mt-1">新增或发布公告后，用户下次进入页面会弹窗显示；在线用户会在轮询后自动收到。</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={load}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-600 bg-white border border-gray-200 rounded-lg hover:bg-gray-50"
          >
            <RefreshCw size={14} /> 刷新
          </button>
          <button
            onClick={() => { setShowForm(true); setEditId(null); setForm(EMPTY_FORM); }}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600"
          >
            <Plus size={14} /> 添加公告
          </button>
        </div>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">
          {error}
          <button onClick={() => setError('')} className="ml-2 underline">关闭</button>
        </div>
      )}

      {showForm && (
        <div className="mb-6 bg-white border border-gray-200 rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-700 mb-3">{editId !== null ? '编辑公告' : '添加公告'}</h3>
          <div className="space-y-3">
            <input
              placeholder="公告标题 *"
              value={form.title}
              onChange={e => setForm({ ...form, title: e.target.value })}
              className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm"
            />
            <textarea
              placeholder="公告内容 *"
              value={form.content}
              onChange={e => setForm({ ...form, content: e.target.value })}
              rows={8}
              className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm resize-y"
            />
            <label className="inline-flex items-center gap-2 text-sm text-gray-600">
              <input
                type="checkbox"
                checked={form.is_active}
                onChange={e => setForm({ ...form, is_active: e.target.checked })}
              />
              保存后立即发布
            </label>
          </div>
          <div className="flex gap-2 mt-4">
            <button
              onClick={handleSave}
              disabled={loading}
              className="flex items-center gap-1.5 px-4 py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50"
            >
              <Check size={14} /> {loading ? '保存中...' : '保存'}
            </button>
            <button
              onClick={resetForm}
              className="flex items-center gap-1.5 px-4 py-2 text-sm text-gray-600 bg-gray-100 rounded-lg hover:bg-gray-200"
            >
              <X size={14} /> 取消
            </button>
          </div>
        </div>
      )}

      <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">公告</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">状态</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">已读人数</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">创建时间</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {announcements.length === 0 && (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-gray-400">暂无公告</td>
              </tr>
            )}
            {announcements.map(item => (
              <tr key={item.id} className={`border-b border-gray-100 ${!item.is_active ? 'opacity-60' : ''}`}>
                <td className="px-4 py-3">
                  <div className="font-medium text-gray-800">{item.title}</div>
                  <div className="text-xs text-gray-500 mt-1 whitespace-pre-wrap break-words">
                    {item.content.length > 160 ? `${item.content.slice(0, 160)}...` : item.content}
                  </div>
                </td>
                <td className="px-4 py-3">
                  <span className={`text-xs px-2 py-0.5 rounded-full ${item.is_active ? 'bg-green-100 text-green-600' : 'bg-gray-100 text-gray-500'}`}>
                    {item.is_active ? '发布中' : '未发布'}
                  </span>
                </td>
                <td className="px-4 py-3 text-gray-600">{item.read_count || 0}</td>
                <td className="px-4 py-3 text-xs text-gray-500">{item.created_at?.slice(0, 16).replace('T', ' ')}</td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => startEdit(item)}
                      className="p-1.5 text-gray-400 hover:text-blue-500 rounded"
                      title="编辑"
                    >
                      <Edit2 size={14} />
                    </button>
                    <button
                      onClick={() => handleToggle(item.id)}
                      className="p-1.5 text-gray-400 hover:text-yellow-500 rounded"
                      title={item.is_active ? '下线' : '发布'}
                    >
                      <Power size={14} />
                    </button>
                    <button
                      onClick={() => handleDelete(item.id)}
                      className="p-1.5 text-gray-400 hover:text-red-500 rounded"
                      title="删除"
                    >
                      <Trash2 size={14} />
                    </button>
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
