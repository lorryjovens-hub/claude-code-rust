import React, { useEffect, useState } from 'react';
import { getDashboard, getStatsTrends, getStatsCost } from '../../adminApi';
import { Users, MessageSquare, Key, Zap, UserPlus, CreditCard, DollarSign, TrendingUp, TrendingDown } from 'lucide-react';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface DashboardData {
  totalUsers: number;
  todayNewUsers: number;
  todayMessages: number;
  todayTokensInput: number;
  todayTokensOutput: number;
  keyPool: { total: number; enabled: number; healthy: number; down: number };
  activeSubscriptions: number;
  todayCost: number;
  todayRevenue: number;
  profit?: {
    monthRevenue: number;
    monthRecharge: number;
    totalRevenue: number;
    totalRecharge: number;
  };
}

function formatTokens(n: number) {
  if (n >= 1e6) return (n / 1e6).toFixed(1) + 'M';
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K';
  return String(n);
}

function formatYuan(n: number) {
  return '¥' + (n / 100).toFixed(2);
}

export default function AdminDashboard() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [error, setError] = useState('');
  const [trends, setTrends] = useState<any[]>([]);
  const [costTrends, setCostTrends] = useState<any[]>([]);
  const [days, setDays] = useState(30);

  const load = () => {
    getDashboard().then(setData).catch(e => setError(e.message));
  };

  const loadCharts = () => {
    getStatsTrends(days).then(setTrends).catch(() => {});
    getStatsCost(days).then(d => setCostTrends(d.dailyCost || [])).catch(() => {});
  };

  useEffect(() => { load(); const t = setInterval(load, 30000); return () => clearInterval(t); }, []);
  useEffect(() => { loadCharts(); }, [days]);

  if (error) return <div className="p-8 text-red-500">{error}</div>;
  if (!data) return <div className="p-8 text-gray-400">加载中...</div>;

  const todayCostDollar = (data.todayCost || 0) / 10000; // cost_units to dollars
  const todayRevenueYuan = (data.todayRevenue || 0) / 100;

  const cards = [
    { label: '总用户', value: data.totalUsers, icon: Users, color: 'bg-blue-500' },
    { label: '今日新增', value: data.todayNewUsers, icon: UserPlus, color: 'bg-green-500' },
    { label: '今日消息', value: data.todayMessages, icon: MessageSquare, color: 'bg-purple-500' },
    { label: '今日 Token (入)', value: formatTokens(data.todayTokensInput), icon: Zap, color: 'bg-orange-500' },
    { label: '今日 Token (出)', value: formatTokens(data.todayTokensOutput), icon: Zap, color: 'bg-red-500' },
    { label: '密钥池 (可用/总)', value: `${data.keyPool.healthy || 0}/${data.keyPool.total || 0}`, icon: Key, color: 'bg-cyan-500' },
    { label: '活跃订阅', value: data.activeSubscriptions, icon: CreditCard, color: 'bg-indigo-500' },
    { label: '今日成本', value: `$${todayCostDollar.toFixed(4)}`, icon: TrendingDown, color: 'bg-rose-500' },
    { label: '今日收入', value: `¥${todayRevenueYuan.toFixed(2)}`, icon: DollarSign, color: 'bg-emerald-500' },
  ];

  // Prepare chart data
  const chartData = trends.map(t => ({
    date: t.date?.slice(5),
    requests: t.requests,
    tokens_output: Math.round((t.tokens_output || 0) / 1000),
    active_users: t.active_users,
    new_users: t.new_users,
    revenue: (t.revenue || 0) / 100,
  }));

  const costChartData = costTrends.map(c => ({
    date: c.date?.slice(5),
    cost: parseFloat((c.total_cost || 0).toFixed(2)),
  }));

  // Merge revenue + cost for comparison
  const comparisonData = chartData.map(d => {
    const costItem = costChartData.find(c => c.date === d.date);
    return { date: d.date, revenue: d.revenue, cost: costItem?.cost || 0 };
  });

  return (
    <div className="p-8">
      <h2 className="text-xl font-semibold text-gray-800 mb-6">Dashboard</h2>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
        {cards.map(c => (
          <div key={c.label} className="bg-white rounded-xl border border-gray-200 p-5 flex items-start gap-4">
            <div className={`${c.color} p-2.5 rounded-lg text-white`}>
              <c.icon size={20} />
            </div>
            <div>
              <div className="text-2xl font-bold text-gray-800">{c.value}</div>
              <div className="text-sm text-gray-500 mt-0.5">{c.label}</div>
            </div>
          </div>
        ))}
      </div>
      {data.keyPool.down > 0 && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">
          ⚠ {data.keyPool.down} 个密钥状态异常 (down)
        </div>
      )}
      {/* Profit Overview */}
      {data.profit && (
        <div className="mt-6 bg-white rounded-xl border border-gray-200 p-5">
          <h3 className="text-sm font-semibold text-gray-700 mb-4">利润概览</h3>
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            {(() => {
              const p = data.profit!;
              const monthRevenueYuan = p.monthRevenue / 100;
              const monthProfit = monthRevenueYuan - p.monthRecharge;
              const monthRate = monthRevenueYuan > 0 ? ((monthProfit / monthRevenueYuan) * 100).toFixed(1) : '0.0';
              const totalRevenueYuan = p.totalRevenue / 100;
              const totalProfit = totalRevenueYuan - p.totalRecharge;
              const totalRate = totalRevenueYuan > 0 ? ((totalProfit / totalRevenueYuan) * 100).toFixed(1) : '0.0';
              return [
                { label: '本月收入', value: `¥${monthRevenueYuan.toFixed(2)}`, color: 'text-emerald-600' },
                { label: '本月充值', value: `¥${p.monthRecharge.toFixed(2)}`, color: 'text-orange-600' },
                { label: '本月利润', value: `¥${monthProfit.toFixed(2)}`, color: monthProfit >= 0 ? 'text-emerald-600' : 'text-red-600' },
                { label: '本月利润率', value: `${monthRate}%`, color: Number(monthRate) >= 0 ? 'text-blue-600' : 'text-red-600' },
                { label: '累计收入', value: `¥${totalRevenueYuan.toFixed(2)}`, color: 'text-emerald-600' },
                { label: '累计充值', value: `¥${p.totalRecharge.toFixed(2)}`, color: 'text-orange-600' },
                { label: '累计利润', value: `¥${totalProfit.toFixed(2)}`, color: totalProfit >= 0 ? 'text-emerald-600' : 'text-red-600' },
                { label: '累计利润率', value: `${totalRate}%`, color: Number(totalRate) >= 0 ? 'text-blue-600' : 'text-red-600' },
              ].map(item => (
                <div key={item.label} className="text-center">
                  <div className={`text-lg font-bold ${item.color}`}>{item.value}</div>
                  <div className="text-xs text-gray-500 mt-0.5">{item.label}</div>
                </div>
              ));
            })()}
          </div>
        </div>
      )}
      {/* Trend Charts */}
      <div className="mt-8">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-800">趋势图表</h3>
          <div className="flex gap-1">
            {[7, 30, 90].map(d => (
              <button key={d} onClick={() => setDays(d)}
                className={`px-3 py-1 text-xs rounded-lg ${days === d ? 'bg-blue-500 text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}`}>
                {d} 天
              </button>
            ))}
          </div>
        </div>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white rounded-xl border border-gray-200 p-5">
            <h4 className="text-sm font-medium text-gray-600 mb-3">每日请求数 & 输出 Token (K)</h4>
            <ResponsiveContainer width="100%" height={240}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" tick={{ fontSize: 11 }} />
                <YAxis yAxisId="left" tick={{ fontSize: 11 }} />
                <YAxis yAxisId="right" orientation="right" tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend />
                <Line yAxisId="left" type="monotone" dataKey="requests" stroke="#6366f1" name="请求数" dot={false} />
                <Line yAxisId="right" type="monotone" dataKey="tokens_output" stroke="#f97316" name="输出Token(K)" dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
          <div className="bg-white rounded-xl border border-gray-200 p-5">
            <h4 className="text-sm font-medium text-gray-600 mb-3">活跃用户 & 新注册</h4>
            <ResponsiveContainer width="100%" height={240}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" tick={{ fontSize: 11 }} />
                <YAxis tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend />
                <Line type="monotone" dataKey="active_users" stroke="#3b82f6" name="活跃用户" dot={false} />
                <Line type="monotone" dataKey="new_users" stroke="#10b981" name="新注册" dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
          <div className="bg-white rounded-xl border border-gray-200 p-5">
            <h4 className="text-sm font-medium text-gray-600 mb-3">每日收入 (¥)</h4>
            <ResponsiveContainer width="100%" height={240}>
              <BarChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" tick={{ fontSize: 11 }} />
                <YAxis tick={{ fontSize: 11 }} />
                <Tooltip />
                <Bar dataKey="revenue" fill="#10b981" name="收入(¥)" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
          <div className="bg-white rounded-xl border border-gray-200 p-5">
            <h4 className="text-sm font-medium text-gray-600 mb-3">成本 ($) vs 收入 (¥)</h4>
            <ResponsiveContainer width="100%" height={240}>
              <LineChart data={comparisonData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" tick={{ fontSize: 11 }} />
                <YAxis tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend />
                <Line type="monotone" dataKey="cost" stroke="#ef4444" name="成本" dot={false} />
                <Line type="monotone" dataKey="revenue" stroke="#10b981" name="收入" dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  );
}