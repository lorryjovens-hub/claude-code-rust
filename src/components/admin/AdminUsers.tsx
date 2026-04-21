import React, { useEffect, useState, useCallback } from 'react';
import { getUsers, banUser, unbanUser, resetUserPassword, adjustUserQuota, setUserRole, getAdminMe } from '../../adminApi';
import { Search, Ban, ShieldCheck, KeyRound, Coins, ChevronLeft, ChevronRight, UserCog } from 'lucide-react';

interface User {
  id: string;
  email: string;
  nickname: string | null;
  role: string;
  plan: string;
  subscription_name: string | null;
  sub_status: string | null;
  sub_expires: string | null;
  sub_token_quota: number | null;
  sub_tokens_used: number | null;
  sub_storage_quota: number | null;
  banned: number;
  token_quota: number;
  token_used: number;
  storage_quota: number;
  storage_used: number;
  created_at: string;
}

export default function AdminUsers() {
  const [users, setUsers] = useState<User[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [error, setError] = useState('');
  const [modal, setModal] = useState<{ type: string; user: User } | null>(null);
  const [modalInput, setModalInput] = useState('');
  const [myRole, setMyRole] = useState('admin');
  const limit = 20;

  const load = useCallback(async () => {
    try {
      const params: any = { page, limit };
      if (search) params.search = search;
      const data = await getUsers(params);
      setUsers(data.users);
      setTotal(data.pagination.total);
    } catch (e: any) { setError(e.message); }
  }, [page, search]);

  useEffect(() => { load(); }, [load]);
  useEffect(() => { getAdminMe().then(d => setMyRole(d.role)).catch(() => {}); }, []);

  const handleSearch = () => { setPage(1); setSearch(searchInput); };

  const handleBan = async (u: User) => {
    if (!confirm(`确定封禁 ${u.email}？`)) return;
    try { await banUser(u.id); await load(); } catch (e: any) { setError(e.message); }
  };

  const handleUnban = async (u: User) => {
    try { await unbanUser(u.id); await load(); } catch (e: any) { setError(e.message); }
  };

  const handleResetPassword = async () => {
    if (!modal || !modalInput || modalInput.length < 6) { setError('密码至少 6 位'); return; }
    try {
      await resetUserPassword(modal.user.id, modalInput);
      setModal(null); setModalInput('');
    } catch (e: any) { setError(e.message); }
  };

  const handleAdjustQuota = async () => {
    if (!modal || !modalInput) return;
    try {
      const quota = parseInt(modalInput, 10);
      if (isNaN(quota) || quota < 0) { setError('请输入有效数字'); return; }
      await adjustUserQuota(modal.user.id, { token_quota: quota });
      setModal(null); setModalInput('');
      await load();
    } catch (e: any) { setError(e.message); }
  };

  const handleSetRole = async () => {
    if (!modal || !modalInput) return;
    try {
      await setUserRole(modal.user.id, modalInput);
      setModal(null); setModalInput('');
      await load();
    } catch (e: any) { setError(e.message); }
  };

  const totalPages = Math.ceil(total / limit);

  return (
    <div className="p-8">
      <h2 className="text-xl font-semibold text-gray-800 mb-6">用户管理</h2>

      {error && <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">{error} <button onClick={() => setError('')} className="ml-2 underline">关闭</button></div>}

      <div className="flex gap-2 mb-4">
        <div className="relative flex-1 max-w-md">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            value={searchInput}
            onChange={e => setSearchInput(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleSearch()}
            placeholder="搜索邮箱、昵称、ID..."
            className="w-full pl-9 pr-3 py-2 border border-gray-200 rounded-lg text-sm"
          />
        </div>
        <button onClick={handleSearch} className="px-4 py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600">搜索</button>
      </div>

      <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">用户</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">角色</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">套餐</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">Token 用量</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">Storage</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">状态</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">注册时间</th>
              <th className="px-4 py-3 text-left text-gray-600 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {users.length === 0 && (
              <tr><td colSpan={8} className="px-4 py-8 text-center text-gray-400">无数据</td></tr>
            )}
            {users.map(u => (
              <tr key={u.id} className="border-b border-gray-100">
                <td className="px-4 py-3">
                  <div className="font-medium text-gray-800">{u.nickname || '-'}</div>
                  <div className="text-xs text-gray-400">{u.email}</div>
                </td>
                <td className="px-4 py-3">
                  <span className={`text-xs px-2 py-0.5 rounded-full ${
                    u.role === 'superadmin' ? 'bg-red-100 text-red-600' :
                    u.role === 'admin' ? 'bg-purple-100 text-purple-600' : 'bg-gray-100 text-gray-600'
                  }`}>
                    {u.role === 'superadmin' ? '超管' : u.role === 'admin' ? '管理员' : '用户'}
                  </span>
                </td>
                <td className="px-4 py-3">
                  {u.subscription_name ? (
                    <div>
                      <span className="text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-600">{u.subscription_name}</span>
                      <div className="text-[10px] text-gray-400 mt-0.5">到期 {u.sub_expires?.slice(0, 10)}</div>
                    </div>
                  ) : (
                    <span className="text-xs text-gray-400">免费</span>
                  )}
                </td>
                <td className="px-4 py-3 text-xs text-gray-500">
                  {(() => {
                    const used = u.sub_token_quota ? (u.sub_tokens_used || 0) : (u.token_used || 0);
                    const quota = u.sub_token_quota || u.token_quota || 0;
                    return `$${(used / 10000).toFixed(2)} / $${(quota / 10000).toFixed(2)}`;
                  })()}
                </td>
                <td className="px-4 py-3 text-xs text-gray-500">
                  {(() => {
                    const quota = u.sub_storage_quota || u.storage_quota || 0;
                    return `${((u.storage_used || 0) / 1024 / 1024).toFixed(1)}MB / ${(quota / 1024 / 1024).toFixed(0)}MB`;
                  })()}
                </td>
                <td className="px-4 py-3">
                  {u.banned ? (
                    <span className="text-xs px-2 py-0.5 rounded-full bg-red-100 text-red-600">已封禁</span>
                  ) : (
                    <span className="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-600">正常</span>
                  )}
                </td>
                <td className="px-4 py-3 text-xs text-gray-400">{u.created_at?.slice(0, 10)}</td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-1">
                    {u.banned ? (
                      <button onClick={() => handleUnban(u)} className="p-1.5 text-gray-400 hover:text-green-500 rounded" title="解封">
                        <ShieldCheck size={14} />
                      </button>
                    ) : (
                      <button onClick={() => handleBan(u)} className="p-1.5 text-gray-400 hover:text-red-500 rounded" title="封禁">
                        <Ban size={14} />
                      </button>
                    )}
                    <button onClick={() => { setModal({ type: 'password', user: u }); setModalInput(''); }}
                      className="p-1.5 text-gray-400 hover:text-blue-500 rounded" title="重置密码">
                      <KeyRound size={14} />
                    </button>
                    <button onClick={() => { setModal({ type: 'quota', user: u }); setModalInput(String(u.sub_token_quota || u.token_quota)); }}
                      className="p-1.5 text-gray-400 hover:text-orange-500 rounded" title="调整额度">
                      <Coins size={14} />
                    </button>
                    {myRole === 'superadmin' && u.role !== 'superadmin' && (
                      <button onClick={() => { setModal({ type: 'role', user: u }); setModalInput(u.role || 'user'); }}
                        className="p-1.5 text-gray-400 hover:text-purple-500 rounded" title="修改角色">
                        <UserCog size={14} />
                      </button>
                    )}
                  </div>
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

      {modal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-2xl shadow-xl w-[400px] p-6">
            <h3 className="text-base font-semibold text-gray-800 mb-1">
              {modal.type === 'password' ? '重置密码' : modal.type === 'role' ? '修改角色' : '调整 Token 额度'}
            </h3>
            <p className="text-sm text-gray-500 mb-4">{modal.user.email}</p>
            {modal.type === 'role' ? (
              <select value={modalInput} onChange={e => setModalInput(e.target.value)}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm mb-4">
                <option value="user">用户</option>
                <option value="admin">管理员</option>
                <option value="superadmin">超级管理员</option>
              </select>
            ) : (
              <input
                type={modal.type === 'password' ? 'password' : 'number'}
                value={modalInput}
                onChange={e => setModalInput(e.target.value)}
                placeholder={modal.type === 'password' ? '输入新密码（至少 6 位）' : 'Token 额度'}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm mb-4"
              />
            )}
            <div className="flex justify-end gap-3">
              <button onClick={() => setModal(null)} className="px-4 py-2 text-sm text-gray-600 bg-gray-100 rounded-lg hover:bg-gray-200">取消</button>
              <button onClick={modal.type === 'password' ? handleResetPassword : modal.type === 'role' ? handleSetRole : handleAdjustQuota}
                className="px-4 py-2 text-sm text-white bg-blue-500 rounded-lg hover:bg-blue-600">确认</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
