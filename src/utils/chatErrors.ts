export function formatChatError(err: string): string {
  const lower = (err || '').toLowerCase();
  if (lower.includes('quota_exceeded') || lower.includes('额度已用完') || lower.includes('额度已用尽') || lower.includes('时段额度') || lower.includes('周期额度')) {
    return '⚠️ 当前额度已用完，请等待额度重置后再试。你可以在设置页查看额度详情。';
  }
  if (lower.includes('订阅已过期') || lower.includes('未激活') || lower.includes('inactive') || lower.includes('expired')) {
    return '⚠️ 你的订阅已过期或未激活，请续费后继续使用。';
  }
  if (lower.includes('invalid api key') || lower.includes('authentication')) {
    return '⚠️ API 认证失败，请重新登录。';
  }
  if (lower.includes('overloaded') || lower.includes('rate limit') || lower.includes('529')) {
    return '⚠️ 服务暂时繁忙，请稍后再试。';
  }
  return 'Error: ' + err;
}
