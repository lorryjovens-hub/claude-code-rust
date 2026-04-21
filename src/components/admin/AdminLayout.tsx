import React, { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { getUserProfile } from '../../api';
import { LayoutDashboard, Key, Users, ArrowLeft, Package, Ticket, Cpu, Bell } from 'lucide-react';

const NAV = [
  { path: '/admin', label: 'Dashboard', icon: LayoutDashboard },
  { path: '/admin/keys', label: '密钥池', icon: Key },
  { path: '/admin/models', label: '模型管理', icon: Cpu },
  { path: '/admin/users', label: '用户管理', icon: Users },
  { path: '/admin/announcements', label: '公告管理', icon: Bell },
  { path: '/admin/plans', label: '套餐管理', icon: Package },
  { path: '/admin/redemption', label: '兑换码', icon: Ticket },
];

export default function AdminLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const [authorized, setAuthorized] = useState<boolean | null>(null);

  useEffect(() => {
    getUserProfile().then((u: any) => {
      if (u.role === 'admin' || u.role === 'superadmin') setAuthorized(true);
      else { setAuthorized(false); navigate('/'); }
    }).catch(() => { setAuthorized(false); navigate('/login'); });
  }, []);

  if (authorized === null) return <div className="flex items-center justify-center h-screen text-gray-400">验证权限中...</div>;
  if (!authorized) return null;

  return (
    <div className="flex h-screen bg-gray-50">
      <aside className="w-56 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h1 className="text-lg font-semibold text-gray-800">管理后台</h1>
        </div>
        <nav className="flex-1 py-2">
          {NAV.map(item => {
            const active = item.path === '/admin'
              ? location.pathname === '/admin'
              : location.pathname.startsWith(item.path);
            return (
              <button
                key={item.path}
                onClick={() => navigate(item.path)}
                className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                  active ? 'bg-blue-50 text-blue-600 font-medium' : 'text-gray-600 hover:bg-gray-50'
                }`}
              >
                <item.icon size={18} />
                {item.label}
              </button>
            );
          })}
        </nav>
        <div className="p-3 border-t border-gray-200">
          <button
            onClick={() => navigate('/')}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-50 rounded-lg transition-colors"
          >
            <ArrowLeft size={16} />
            返回主页
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
