import React, { useState, useEffect, useRef } from 'react';
import { Crown, Zap, Star, Gem, Clock, Calendar, Check, X } from 'lucide-react';
import { getPlans, createPaymentOrder, getPaymentStatus, getUserUsage, redeemCode, getUserModels } from '../api';

interface UpgradePlanProps {
  onClose: () => void;
}

interface Plan {
  id: number;
  name: string;
  price: number;
  duration_days: number;
  token_quota: number;
  window_budget: number;
  weekly_budget: number;
  description: string;
}

interface BillingModel {
  id: string;
  name: string;
  enabled: number;
  model_multiplier?: number;
  output_multiplier?: number;
  cache_read_multiplier?: number;
}

interface BillingConfig {
  model_multiplier: number;
  output_multiplier: number;
  cache_read_multiplier: number;
}

const PLAN_STYLES: Record<string, { icon: React.ElementType; color: string; bg: string; border: string; badge?: string }> = {
  '体验包': { icon: Zap, color: 'text-amber-600 dark:text-amber-400', bg: 'bg-amber-50 dark:bg-amber-900/20', border: 'border-amber-200 dark:border-amber-700', badge: '入门' },
  '日卡': { icon: Clock, color: 'text-sky-600 dark:text-sky-400', bg: 'bg-sky-50 dark:bg-sky-900/20', border: 'border-sky-200 dark:border-sky-700' },
  '周卡': { icon: Calendar, color: 'text-violet-600 dark:text-violet-400', bg: 'bg-violet-50 dark:bg-violet-900/20', border: 'border-violet-200 dark:border-violet-700' },
  '基础月卡': { icon: Star, color: 'text-emerald-600 dark:text-emerald-400', bg: 'bg-emerald-50 dark:bg-emerald-900/20', border: 'border-emerald-200 dark:border-emerald-700' },
  '专业月卡': { icon: Crown, color: 'text-blue-600 dark:text-blue-400', bg: 'bg-blue-50 dark:bg-blue-900/20', border: 'border-blue-200 dark:border-blue-700', badge: '热门' },
  '尊享月卡': { icon: Gem, color: 'text-purple-600 dark:text-purple-400', bg: 'bg-purple-50 dark:bg-purple-900/20', border: 'border-purple-200 dark:border-purple-700', badge: '旗舰' },
};
const DEFAULT_STYLE: { icon: React.ElementType; color: string; bg: string; border: string; badge?: string } = { icon: Star, color: 'text-gray-600 dark:text-gray-400', bg: 'bg-gray-50 dark:bg-gray-800', border: 'border-gray-200 dark:border-gray-600' };

const UpgradePlan = ({ onClose }: UpgradePlanProps) => {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [billingModels, setBillingModels] = useState<BillingModel[]>([]);
  const [currentPlanId, setCurrentPlanId] = useState<number | null>(null);
  const [currentPlanPrice, setCurrentPlanPrice] = useState<number | null>(null);
  const [currentPlanQuota, setCurrentPlanQuota] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [detailPlan, setDetailPlan] = useState<Plan | null>(null);
  const [payStep, setPayStep] = useState<'select' | 'paying' | 'success' | 'timeout'>('select');
  const [orderId, setOrderId] = useState('');
  const [payUrl, setPayUrl] = useState('');
  const [qrcodeUrl, setQrcodeUrl] = useState('');
  const [qrcodeError, setQrcodeError] = useState(false);
  const [error, setError] = useState('');
  const [redeemInput, setRedeemInput] = useState('');
  const [redeeming, setRedeeming] = useState(false);
  const [redeemResult, setRedeemResult] = useState<any>(null);
  const [redeemError, setRedeemError] = useState('');
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    Promise.allSettled([getPlans(), getUserUsage(), getUserModels()])
      .then(([plansResult, usageResult, modelsResult]) => {
        const plansData = plansResult.status === 'fulfilled' ? plansResult.value : [];
        const usage = usageResult.status === 'fulfilled' ? usageResult.value : {};
        const modelCatalog = modelsResult.status === 'fulfilled' ? modelsResult.value : {};
        setPlans(Array.isArray(plansData) ? plansData : []);
        setBillingModels(Array.isArray(modelCatalog?.all) ? modelCatalog.all : []);
        if (usage.plan && typeof usage.plan.price === 'number') setCurrentPlanPrice(usage.plan.price);
        if (usage.plan && typeof usage.plan.id === 'number') setCurrentPlanId(usage.plan.id);
        if (usage.plan && Array.isArray(plansData)) {
          const matched = plansData.find((p: Plan) => p.id === usage.plan.id);
          if (matched) setCurrentPlanQuota(matched.token_quota);
        }
        setLoading(false);
      });
    return () => { clearPolling(); };
  }, []);

  const formatPrice = (cents: number) => `¥${(cents / 100).toFixed(cents % 100 === 0 ? 0 : 1)}`;

  const BASE_PRICE_PER_M = 2.0;
  const AVG_INPUT = 70000;
  const AVG_PREV_INPUT = 55000;
  const AVG_OUTPUT = 3500;
  const CACHE_DISCOUNT = 0.3;
  // Short conversation: simple Q&A
  const SHORT_INPUT = 5000;
  const SHORT_OUTPUT = 1000;

  const resolveConfig = (family: 'opus' | 'sonnet'): BillingConfig => {
    const fallback = family === 'opus'
      ? { model_multiplier: 5.0, output_multiplier: 5.0, cache_read_multiplier: 0.1 }
      : { model_multiplier: 3.0, output_multiplier: 5.0, cache_read_multiplier: 0.1 };
    const enabled = billingModels.filter(m => Number(m.enabled) === 1);
    const exactId = family === 'opus' ? 'claude-opus-4-6' : 'claude-sonnet-4-6';
    const model = enabled.find(m => m.id === exactId) || enabled.find(m => new RegExp(family, 'i').test(m.id));
    if (!model) return fallback;
    return {
      model_multiplier: Number(model.model_multiplier ?? fallback.model_multiplier),
      output_multiplier: Number(model.output_multiplier ?? fallback.output_multiplier),
      cache_read_multiplier: Number(model.cache_read_multiplier ?? fallback.cache_read_multiplier),
    };
  };

  const calcCostPerRound = (config: BillingConfig) => {
    const promptPrice = BASE_PRICE_PER_M * config.model_multiplier;
    const cachedCost = (AVG_PREV_INPUT / 1e6) * promptPrice * CACHE_DISCOUNT;
    const freshCost = ((AVG_INPUT - AVG_PREV_INPUT) / 1e6) * promptPrice;
    const outputCost = (AVG_OUTPUT / 1e6) * promptPrice * config.output_multiplier;
    return cachedCost + freshCost + outputCost;
  };

  const calcShortCostPerRound = (config: BillingConfig) => {
    const promptPrice = BASE_PRICE_PER_M * config.model_multiplier;
    const inputCost = (SHORT_INPUT / 1e6) * promptPrice;
    const outputCost = (SHORT_OUTPUT / 1e6) * promptPrice * config.output_multiplier;
    return inputCost + outputCost;
  };

  const opusCost = calcCostPerRound(resolveConfig('opus'));
  const sonnetCost = calcCostPerRound(resolveConfig('sonnet'));
  const opusShortCost = calcShortCostPerRound(resolveConfig('opus'));
  const sonnetShortCost = calcShortCostPerRound(resolveConfig('sonnet'));

  const estimateRounds = (budget: number, costPerRound: number) => budget > 0 ? Math.floor(budget / costPerRound) : 0;

  const MONTHLY_IDS = [2, 3, 4];
  const isCurrentMonthly = currentPlanId !== null && MONTHLY_IDS.includes(currentPlanId);

  const getUpgradePrice = (plan: Plan) => {
    if (currentPlanPrice === null) return plan.price;
    // Only monthly→monthly upgrades get price difference
    if (isCurrentMonthly && MONTHLY_IDS.includes(plan.id)) {
      return Math.max(plan.price - currentPlanPrice, 0);
    }
    return plan.price;
  };

  // Can this plan be purchased given the current subscription?
  const canPurchase = (plan: Plan) => {
    if (currentPlanId === null) return true; // no active sub
    const isCurrent = currentPlanPrice !== null && plan.price === currentPlanPrice;
    if (isCurrent) return false;
    const isTargetMonthly = MONTHLY_IDS.includes(plan.id);
    if (isCurrentMonthly && isTargetMonthly) {
      // 月卡→月卡：只允许升级（高于当前）
      if (currentPlanQuota !== null && plan.token_quota <= currentPlanQuota) return false;
      return true;
    }
    if (!isCurrentMonthly) {
      // 当前是短期卡：允许原价购买任何套餐
      return true;
    }
    // 当前是月卡，目标是短期卡：不允许
    return false;
  };

  // Is this an upgrade (monthly→monthly, pays difference)?
  const isMonthlyUpgrade = (plan: Plan) => {
    return currentPlanId !== null && isCurrentMonthly && MONTHLY_IDS.includes(plan.id)
      && canPurchase(plan);
  };

  const handleBuy = async (plan: Plan) => {
    setDetailPlan(null);
    setError('');
    setPayStep('paying');
    setPayUrl('');
    setQrcodeUrl('');
    setQrcodeError(false);
    try {
      const data = await createPaymentOrder(plan.id, 'alipay');
      setOrderId(data.orderId);
      setDetailPlan(plan);
      if (data.payUrl) setPayUrl(data.payUrl);
      if (data.qrcodeUrl) {
        setQrcodeUrl(data.qrcodeUrl);
      } else {
        throw new Error('支付网关未返回二维码，请稍后重试');
      }
      startPolling(data.orderId);
    } catch (err: any) {
      setError(err.message || '创建订单失败');
      setPayStep('select');
    }
  };

  const startPolling = (oid: string) => {
    pollRef.current = setInterval(async () => {
      try {
        const status = await getPaymentStatus(oid);
        if (status.status === 'paid') { clearPolling(); setPayStep('success'); }
      } catch {}
    }, 2000);
    timeoutRef.current = setTimeout(() => { clearPolling(); setPayStep('timeout'); }, 5 * 60 * 1000);
  };

  const clearPolling = () => {
    if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
    if (timeoutRef.current) { clearTimeout(timeoutRef.current); timeoutRef.current = null; }
  };

  const handleRetry = () => {
    setPayStep('select'); setOrderId(''); setPayUrl(''); setQrcodeUrl(''); setQrcodeError(false); setError('');
  };

  const formatRedeemInput = (val: string) => {
    const cleaned = val.replace(/[^a-zA-Z0-9]/g, '').toUpperCase().slice(0, 16);
    const parts = [];
    for (let i = 0; i < cleaned.length; i += 4) parts.push(cleaned.slice(i, i + 4));
    return parts.join('-');
  };

  const handleRedeem = async () => {
    if (!redeemInput) return;
    setRedeeming(true); setRedeemError(''); setRedeemResult(null);
    try {
      const data = await redeemCode(redeemInput);
      setRedeemResult(data); setRedeemInput('');
      setTimeout(() => window.location.reload(), 3000);
    } catch (err: any) {
      setRedeemError(err.message || '兑换失败');
    } finally { setRedeeming(false); }
  };

  if (loading) {
    return <div className="flex-1 flex items-center justify-center h-full bg-claude-bg"><div className="text-[14px] text-claude-textSecondary">加载中...</div></div>;
  }

  // --- Detail modal ---
  const renderDetailModal = (plan: Plan) => {
    const isCurrent = currentPlanPrice !== null && plan.price === currentPlanPrice;
    const purchasable = canPurchase(plan);
    const disabled = isCurrent || !purchasable;
    const isUpgrade = purchasable && isMonthlyUpgrade(plan);
    const hasActiveSub = currentPlanId !== null;
    const blockedBySub = hasActiveSub && !isCurrent && !purchasable;
    const totalUSD = plan.token_quota / 10000;
    const hasWindow = plan.window_budget > 0;
    const hasWeekly = plan.weekly_budget > 0;
    const style = PLAN_STYLES[plan.name] || DEFAULT_STYLE;
    const Icon = style.icon;

    const opusTotal = estimateRounds(totalUSD, opusCost);
    const sonnetTotal = estimateRounds(totalUSD, sonnetCost);
    const opusShortTotal = estimateRounds(totalUSD, opusShortCost);
    const sonnetShortTotal = estimateRounds(totalUSD, sonnetShortCost);
    const opusWindow = hasWindow ? estimateRounds(plan.window_budget, opusCost) : 0;
    const sonnetWindow = hasWindow ? estimateRounds(plan.window_budget, sonnetCost) : 0;
    const opusShortWindow = hasWindow ? estimateRounds(plan.window_budget, opusShortCost) : 0;
    const opusWeekly = hasWeekly ? estimateRounds(plan.weekly_budget, opusCost) : 0;
    const sonnetWeekly = hasWeekly ? estimateRounds(plan.weekly_budget, sonnetCost) : 0;

    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={() => setDetailPlan(null)}>
        <div className="bg-claude-input rounded-2xl w-[460px] max-h-[90vh] overflow-y-auto shadow-xl" onClick={e => e.stopPropagation()}>
          {/* Header */}
          <div className={`flex items-center justify-between px-6 pt-5 pb-4 border-b border-claude-border`}>
            <div className="flex items-center gap-3">
              <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${style.bg} ${style.color}`}>
                <Icon size={20} />
              </div>
              <div>
                <h2 className="text-[18px] font-semibold text-claude-text">{plan.name}</h2>
                <p className="text-[12px] text-claude-textSecondary">{plan.description}</p>
              </div>
            </div>
            <button onClick={() => setDetailPlan(null)} className="text-claude-textSecondary hover:text-claude-text"><X size={20} /></button>
          </div>

          <div className="px-6 py-5">
            {/* Price + quota */}
            <div className="flex items-baseline gap-2 mb-1">
              <span className="text-[32px] font-bold text-claude-text">{formatPrice(plan.price)}</span>
              <span className="text-[14px] text-claude-textSecondary">/ {plan.duration_days} 天</span>
            </div>
            <div className="text-[13px] text-claude-textSecondary mb-5">总额度 ${totalUSD.toFixed(2)}</div>

            {/* Rounds table */}
            <div className={`rounded-xl p-4 mb-4 ${style.bg} border ${style.border}`}>
              <h4 className={`text-[13px] font-semibold ${style.color} mb-3`}>预估对话轮数（Opus 4.6）</h4>
              <table className="w-full text-[13px]">
                <thead>
                  <tr className="text-claude-textSecondary">
                    <th className="text-left pb-2 font-normal">额度类型</th>
                    <th className="text-right pb-2 font-normal">长对话</th>
                    <th className="text-right pb-2 font-normal">短对话</th>
                  </tr>
                </thead>
                <tbody className="text-claude-text">
                  {hasWindow && (
                    <tr className="border-t border-black/5">
                      <td className="py-2">每 5 小时</td>
                      <td className="text-right py-2 font-medium">~{opusWindow} 轮</td>
                      <td className="text-right py-2 font-medium">~{opusShortWindow} 轮</td>
                    </tr>
                  )}
                  {hasWeekly && (
                    <tr className="border-t border-black/5">
                      <td className="py-2">每周</td>
                      <td className="text-right py-2 font-medium">~{opusWeekly} 轮</td>
                      <td className="text-right py-2 font-medium">~{estimateRounds(plan.weekly_budget, opusShortCost)} 轮</td>
                    </tr>
                  )}
                  <tr className="border-t border-black/5">
                    <td className="py-2">{plan.duration_days} 天总额度</td>
                    <td className="text-right py-2 font-medium">~{opusTotal} 轮</td>
                    <td className="text-right py-2 font-medium">~{opusShortTotal} 轮</td>
                  </tr>
                </tbody>
              </table>
              <div className="mt-2 pt-2 border-t border-black/5 text-[11px] text-claude-textSecondary">
                Sonnet 4.6 更省额度：长对话约 {sonnetTotal} 轮 / 短对话约 {sonnetShortTotal} 轮
              </div>
            </div>

            {/* Estimation note */}
            <div className="bg-claude-hover rounded-xl p-4 mb-4 text-[12px] text-claude-textSecondary leading-relaxed">
              <p className="mb-1.5"><span className="font-medium text-claude-textSecondary">长对话</span>：多轮深度讨论、上传文件分析、写论文/代码等场景，每轮约消耗 ~70k input + ~3.5k output tokens（相当于 15 轮历史上下文或一次性发送 5 万字论文）。</p>
              <p className="mb-1.5"><span className="font-medium text-claude-textSecondary">短对话</span>：简单问答、翻译、快速查询等场景，每轮约消耗 ~5k input + ~1k output tokens。</p>
              <p>具体额度消耗取决于每一轮的实际用量，问简单问题消耗很少，不用担心不划算。大部分日常使用介于长短对话之间。</p>
            </div>

            {/* Features */}
            <div className="mb-5">
              <h4 className="text-[13px] font-semibold text-claude-text mb-2">包含功能</h4>
              <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-[12px] text-claude-textSecondary">
                {[
                  'Claude Opus 4.6 深度推理',
                  'Claude Sonnet 4.6 快速响应',
                  'GPT-5.4 多模型切换',
                  'Extended Thinking 模式',
                  '联网搜索实时信息',
                  '文档生成 (PPT/Word/Excel/PDF)',
                  'Python 代码执行',
                  '图片上传与分析',
                  '文件上传与解读',
                  '数据图表绘制',
                ].map(f => (
                  <div key={f} className="flex items-center gap-1.5">
                    <Check size={12} className="text-[#4B9C68] flex-shrink-0" />
                    <span>{f}</span>
                  </div>
                ))}
              </div>
            </div>

            {error && <div className="mb-3 p-2 bg-red-50 text-red-600 text-[13px] rounded-lg">{error}</div>}

            <button
              onClick={() => !disabled && handleBuy(plan)}
              disabled={disabled}
              className={`w-full py-3 rounded-xl text-[15px] font-medium transition-colors ${
                isCurrent ? 'bg-[#4B9C68]/10 text-[#4B9C68] cursor-default'
                : disabled ? 'bg-claude-btnHover text-claude-textSecondary cursor-not-allowed'
                : 'bg-claude-accent hover:opacity-90 text-white'
              }`}
            >
              {isCurrent ? '当前套餐' : blockedBySub ? '月卡有效期内无法购买' : isUpgrade ? `升级 (${formatPrice(getUpgradePrice(plan))})` : '立即购买'}
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="flex-1 flex flex-col h-full overflow-y-auto bg-claude-bg">
      <div className="max-w-[1000px] w-full mx-auto px-6 py-10">
        <div className="mb-8 text-center">
          <h1 className="text-[28px] font-serif-claude text-claude-text mb-2">选择适合您的套餐</h1>
          <p className="text-[15px] text-claude-textSecondary">升级后即可享受更多对话额度</p>
        </div>

        {/* Plan Cards */}
        {payStep === 'select' && (
          <>
            {/* Feature banner */}
            <div className="mb-6 px-5 py-4 rounded-2xl bg-claude-hover border border-claude-border">
              <p className="text-[13px] text-claude-textSecondary text-center leading-relaxed mb-2">
                所有套餐功能完全相同，仅额度不同 — 支持 Claude Opus 4.6 / Sonnet 4.6 / GPT-5.4 等模型，含 Extended Thinking 深度推理、联网搜索、文档生成、代码执行、文件上传与分析
              </p>
              <p className="text-[12px] text-claude-textSecondary/70 text-center leading-relaxed">
                额度按实际用量扣费：问简单问题消耗很少（约 $0.06~0.10/轮），深度长对话消耗较多（约 $0.30~0.50/轮），不用担心问简单问题不划算
              </p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5 mb-8">
              {plans.map(plan => {
                const style = PLAN_STYLES[plan.name] || DEFAULT_STYLE;
                const Icon = style.icon;
                const isCurrent = currentPlanPrice !== null && plan.price === currentPlanPrice;
                const hasWindow = plan.window_budget > 0;
                const hasWeekly = plan.weekly_budget > 0;
                const opusWeekly = hasWeekly ? estimateRounds(plan.weekly_budget, opusCost) : 0;
                const opusShortWeekly = hasWeekly ? estimateRounds(plan.weekly_budget, opusShortCost) : 0;
                return (
                  <div
                    key={plan.id}
                    onClick={() => setDetailPlan(plan)}
                    className={`relative flex flex-col p-5 rounded-2xl border-2 transition-all cursor-pointer hover:shadow-md bg-claude-input ${
                      isCurrent ? 'border-[#4B9C68] bg-[#4B9C68]/5 dark:bg-[#4B9C68]/10' : `${style.border} hover:border-claude-textSecondary`
                    }`}
                  >
                    {isCurrent && (
                      <div className="absolute top-3 right-3 flex items-center gap-0.5 px-2 py-0.5 bg-[#4B9C68]/10 text-[#4B9C68] text-[11px] font-medium rounded-full">
                        <Check size={11} /> 当前
                      </div>
                    )}
                    {!isCurrent && style.badge && (
                      <div className={`absolute top-3 right-3 px-2 py-0.5 ${style.bg} ${style.color} text-[11px] font-medium rounded-full`}>
                        {style.badge}
                      </div>
                    )}
                    <div className="flex items-center gap-3 mb-3">
                      <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${style.bg} ${style.color}`}>
                        <Icon size={20} />
                      </div>
                      <div>
                        <h3 className="text-[16px] font-semibold text-claude-text">{plan.name}</h3>
                        <div className="text-[12px] text-claude-textSecondary">{plan.duration_days} 天</div>
                      </div>
                    </div>
                    <div className="text-[24px] font-bold text-claude-text mb-2">{formatPrice(plan.price)}</div>
                    <div className="text-[12px] text-claude-textSecondary space-y-1 mb-3">
                      {hasWindow && <div>5h限额 ${plan.window_budget.toFixed(2)}{hasWeekly ? ` · 每周 $${plan.weekly_budget.toFixed(2)}` : ''}</div>}
                      {!hasWindow && hasWeekly && <div>每周 ${plan.weekly_budget.toFixed(2)}</div>}
                      {hasWeekly ? (
                        <div>约 {opusShortWeekly} 轮 Opus 短对话 / {opusWeekly} 轮长对话 每周</div>
                      ) : (
                        <div>总额度 ${(plan.token_quota / 10000).toFixed(2)}</div>
                      )}
                    </div>
                    <div className={`mt-auto text-center py-2 rounded-lg text-[13px] font-medium ${style.bg} ${style.color}`}>
                      查看详情 →
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Redemption Code */}
            <div className="max-w-[480px] mx-auto rounded-2xl border border-claude-border p-6">
              <h3 className="text-[15px] font-semibold text-claude-text mb-3">已有兑换码？</h3>
              <div className="flex gap-3 mb-2">
                <input
                  type="text" value={redeemInput}
                  onChange={e => { setRedeemInput(formatRedeemInput(e.target.value)); setRedeemError(''); setRedeemResult(null); }}
                  placeholder="请输入兑换码"
                  className="flex-1 px-4 py-2.5 bg-claude-input border border-claude-border rounded-lg text-[14px] text-claude-text font-mono tracking-wider focus:outline-none focus:border-claude-accent focus:shadow-sm transition-all"
                  maxLength={19}
                  onKeyDown={e => { if (e.key === 'Enter') handleRedeem(); }}
                />
                <button
                  onClick={handleRedeem}
                  disabled={redeeming || redeemInput.replace(/[^a-zA-Z0-9]/g, '').length !== 16}
                  className={`px-5 py-2.5 text-white text-[14px] font-medium rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap ${
                    redeeming || redeemInput.replace(/[^a-zA-Z0-9]/g, '').length !== 16 ? 'bg-claude-btnHover text-claude-textSecondary' : 'bg-claude-accent hover:opacity-90'
                  }`}
                >{redeeming ? '兑换中...' : '兑换'}</button>
              </div>
              {redeemError && <div className="mt-2 p-2 bg-red-500/10 text-red-500 dark:text-red-400 text-[13px] rounded-lg">{redeemError}</div>}
              {redeemResult && (
                <div className="mt-2 p-2 bg-green-500/10 text-green-700 dark:text-green-400 text-[13px] rounded-lg">
                  兑换成功！已激活 {redeemResult.plan_name}，页面即将刷新...
                </div>
              )}
            </div>
          </>
        )}

        {/* Detail Modal */}
        {payStep === 'select' && detailPlan && renderDetailModal(detailPlan)}

        {/* Paying */}
        {payStep === 'paying' && detailPlan && (() => {
          const displayPrice = ((currentPlanPrice !== null && getUpgradePrice(detailPlan) < detailPlan.price
            ? getUpgradePrice(detailPlan) : detailPlan.price) / 100).toFixed(2);
          return (
          <div className="max-w-[460px] mx-auto">
            <div className="bg-claude-input rounded-t-2xl border border-b-0 border-claude-border px-6 py-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-2">
                  <div className="w-[28px] h-[28px] bg-[#1677FF] rounded-[6px] flex items-center justify-center text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" fill="currentColor" viewBox="0 0 16 16">
                      <path d="M2.541 0H13.5a2.55 2.55 0 0 1 2.54 2.563v8.297c-.006 0-.531-.046-2.978-.813-.412-.14-.916-.327-1.479-.536q-.456-.17-.957-.353a13 13 0 0 0 1.325-3.373H8.822V4.649h3.831v-.634h-3.83V2.121H7.26c-.274 0-.274.273-.274.273v1.621H3.11v.634h3.875v1.136h-3.2v.634H9.99c-.227.789-.532 1.53-.894 2.202-2.013-.67-4.161-1.212-5.51-.878-.864.214-1.42.597-1.746.998-1.499 1.84-.424 4.633 2.741 4.633 1.872 0 3.675-1.053 5.072-2.787 2.08 1.008 6.37 2.738 6.387 2.745v.105A2.55 2.55 0 0 1 13.5 16H2.541A2.55 2.55 0 0 1 0 13.437V2.563A2.55 2.55 0 0 1 2.541 0"/>
                      <path d="M2.309 9.27c-1.22 1.073-.49 3.034 1.978 3.034 1.434 0 2.868-.925 3.994-2.406-1.602-.789-2.959-1.353-4.425-1.207-.397.04-1.14.217-1.547.58Z"/>
                    </svg>
                  </div>
                  <span className="text-[#1677FF] text-[16px] font-bold tracking-tight">支付宝</span>
                </div>
                <div className="w-px h-5 bg-[#ddd] mx-1" />
                <span className="text-[15px] text-claude-text">我的收银台</span>
              </div>
            </div>
            <div className="bg-claude-hover border-x border-claude-border">
              <div className="bg-claude-input mx-4 mt-4 rounded-lg px-5 py-4">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-[13px] text-claude-textSecondary">商品名称</span>
                  <span className="text-[14px] text-claude-text">{detailPlan.name}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[13px] text-claude-textSecondary">付款金额</span>
                  <div className="flex items-baseline">
                    <span className="text-[14px] text-[#FF6A00] mr-0.5">¥</span>
                    <span className="text-[24px] font-semibold text-[#FF6A00]">{displayPrice}</span>
                  </div>
                </div>
              </div>
              <div className="bg-claude-input mx-4 mt-3 mb-4 rounded-lg px-5 py-5 flex flex-col items-center">
                <div className="text-[14px] text-claude-text mb-4">
                  请使用<span className="text-[#1677FF] font-medium"> 支付宝 </span>扫描下方二维码完成支付
                </div>
                {qrcodeUrl ? (
                  qrcodeError ? (
                    <div className="w-[206px] h-[206px] rounded-lg bg-[#FFF5F5] border border-[#FFD0D0] flex flex-col items-center justify-center gap-3">
                      <div className="text-center px-4">
                        <div className="text-[13px] text-[#FF6B6B] mb-2">二维码加载失败</div>
                        <button onClick={() => { setQrcodeError(false); const u = qrcodeUrl; setQrcodeUrl(''); setTimeout(() => setQrcodeUrl(u), 100); }} className="text-[12px] text-[#1677FF] hover:underline">点击重试</button>
                      </div>
                    </div>
                  ) : (
                    <img src={qrcodeUrl} alt="支付二维码" className="w-[206px] h-[206px] rounded-lg" onError={() => setQrcodeError(true)} />
                  )
                ) : (
                  <div className="w-[206px] h-[206px] rounded-lg bg-claude-hover flex items-center justify-center">
                    <div className="text-[13px] text-claude-textSecondary">加载中...</div>
                  </div>
                )}
              </div>
            </div>
            <div className="bg-claude-input rounded-b-2xl border border-t-0 border-claude-border px-6 py-3 flex justify-between items-center">
              <button onClick={handleRetry} className="text-[13px] text-claude-textSecondary hover:text-claude-text">← 返回选择</button>
              {payUrl && <a href={payUrl} target="_blank" rel="noopener noreferrer" className="text-[13px] text-[#1677FF] hover:underline">手动打开支付页 →</a>}
            </div>
          </div>
          );
        })()}

        {/* Success */}
        {payStep === 'success' && (
          <div className="max-w-[400px] mx-auto">
            <div className="rounded-2xl border border-claude-border p-8 text-center bg-transparent">
              <div className="w-16 h-16 mx-auto bg-[#4B9C68]/10 rounded-full flex items-center justify-center mb-4">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#4B9C68" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              </div>
              <h3 className="text-[18px] font-semibold text-claude-text mb-2">支付成功</h3>
              <p className="text-[14px] text-claude-textSecondary mb-6">您的套餐已激活，现在可以开始使用了</p>
              <button onClick={onClose} className="w-full py-2.5 bg-claude-accent hover:opacity-90 text-white rounded-lg text-[14px] font-medium transition-colors">开始使用</button>
            </div>
          </div>
        )}

        {/* Timeout */}
        {payStep === 'timeout' && (
          <div className="max-w-[400px] mx-auto">
            <div className="rounded-2xl border border-claude-border p-8 text-center bg-transparent">
              <h3 className="text-[16px] font-semibold text-claude-text mb-2">订单已超时</h3>
              <p className="text-[14px] text-claude-textSecondary mb-6">支付超时，请重新下单</p>
              <button onClick={handleRetry} className="w-full py-2.5 bg-claude-accent hover:opacity-90 text-white rounded-lg text-[14px] font-medium transition-colors">重新支付</button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default UpgradePlan;
