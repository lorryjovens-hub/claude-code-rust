import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft } from 'lucide-react';
import { login, register, sendCode, forgotPassword, resetPassword, gatewayLogin } from '../api';

type View = 'login' | 'register' | 'verify' | 'forgot' | 'reset';

const Auth = () => {
  const navigate = useNavigate();
  const [view, setView] = useState<View>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [nickname, setNickname] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [code, setCode] = useState('');
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [countdown, setCountdown] = useState(0);

  useEffect(() => {
    if (countdown <= 0) return;
    const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
    return () => clearTimeout(timer);
  }, [countdown]);

  const getPasswordStrength = (pwd: string): { level: string; color: string; width: string } => {
    if (!pwd) return { level: '', color: '', width: '0%' };
    const hasLetter = /[a-zA-Z]/.test(pwd);
    const hasNumber = /[0-9]/.test(pwd);
    const hasSpecial = /[^a-zA-Z0-9]/.test(pwd);
    const long = pwd.length >= 12;
    if (pwd.length < 8 || !hasLetter || !hasNumber) return { level: '弱', color: '#EF4444', width: '33%' };
    if (long && hasSpecial) return { level: '强', color: '#22C55E', width: '100%' };
    return { level: '中', color: '#F59E0B', width: '66%' };
  };

  const showStrength = (view === 'register' || view === 'reset') && password;
  const strength = showStrength ? getPasswordStrength(password) : null;

  const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(''); setLoading(true);
    try {
      if (isTauri) {
        // Electron app: login via US gateway, get API key for Claude Code SDK
        const data = await gatewayLogin(email, password);
        if (data.api_key) {
          window.location.hash = '#/'; window.location.reload();
        } else {
          setError('登录成功但未获取到 API Key');
        }
      } else {
        // Web: login via Chengdu backend
        const data = await login(email, password);
        if (data.token) {
          localStorage.setItem('auth_token', data.token);
          localStorage.setItem('user', JSON.stringify(data.user));
          window.location.hash = '#/'; window.location.reload();
        }
      }
    } catch (err: any) {
      setError(err.message || '登录失败');
    } finally { setLoading(false); }
  };

  const handleSendCode = async () => {
    if (countdown > 0) return;
    setError(''); setLoading(true);
    try {
      await sendCode(email);
      setCountdown(60);
      setView('verify');
      setMessage('验证码已发送到您的邮箱');
    } catch (err: any) {
      setError(err.message || '发送失败');
    } finally { setLoading(false); }
  };

  const handleRegisterStep1 = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password.length < 8 || !/[a-zA-Z]/.test(password) || !/[0-9]/.test(password)) {
      setError('密码至少 8 位，需包含字母和数字');
      return;
    }
    if (password !== confirmPassword) {
      setError('两次输入的密码不一致');
      return;
    }
    await handleSendCode();
  };

  const handleRegisterStep2 = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(''); setLoading(true);
    try {
      const data = await register(email, password, nickname, code);
      if (data.token) {
        localStorage.setItem('auth_token', data.token);
        localStorage.setItem('user', JSON.stringify(data.user));
        window.location.hash = '#/'; window.location.reload();
      }
    } catch (err: any) {
      setError(err.message || '注册失败');
    } finally { setLoading(false); }
  };

  const handleForgotPassword = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(''); setLoading(true);
    try {
      const data = await forgotPassword(email);
      setMessage(data.message || '验证码已发送');
      setCountdown(60);
      setView('reset');
    } catch (err: any) {
      setError(err.message || '发送失败');
    } finally { setLoading(false); }
  };

  const handleResetPassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password.length < 8 || !/[a-zA-Z]/.test(password) || !/[0-9]/.test(password)) {
      setError('密码至少 8 位，需包含字母和数字');
      return;
    }
    if (password !== confirmPassword) {
      setError('两次输入的密码不一致');
      return;
    }
    setError(''); setLoading(true);
    try {
      await resetPassword(email, code, password);
      setMessage('密码重置成功，请登录');
      setView('login');
      setPassword(''); setCode('');
    } catch (err: any) {
      setError(err.message || '重置失败');
    } finally { setLoading(false); }
  };

  const switchView = (v: View) => {
    setView(v); setError(''); setMessage(''); setCode(''); setConfirmPassword('');
  };

  const inputClass = "w-full px-3 py-2 border border-[#E5E5E5] rounded-lg focus:outline-none focus:ring-2 focus:ring-[#CC7C5E] focus:border-transparent transition-all";
  const btnClass = "w-full py-2.5 bg-[#CC7C5E] hover:bg-[#B96B4E] text-white font-medium rounded-lg transition-colors disabled:opacity-70 disabled:cursor-not-allowed";

  const renderPasswordField = () => (
    <>
      <div>
        <label className="block text-sm font-medium text-[#393939] mb-1">密码</label>
        <input type="password" required value={password} onChange={(e) => setPassword(e.target.value)}
          className={inputClass} placeholder="••••••••" />
        {strength && (
          <div className="mt-2">
            <div className="h-1.5 w-full bg-gray-200 rounded-full overflow-hidden">
              <div className="h-full rounded-full transition-all duration-300"
                style={{ width: strength.width, backgroundColor: strength.color }} />
            </div>
            <p className="text-xs mt-1" style={{ color: strength.color }}>
              密码强度：{strength.level}
              {strength.level === '弱' && '（至少8位，需包含字母和数字）'}
            </p>
          </div>
        )}
      </div>
      <div>
        <label className="block text-sm font-medium text-[#393939] mb-1">确认密码</label>
        <input type="password" required value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)}
          className={inputClass} placeholder="再次输入密码" />
        {confirmPassword && password !== confirmPassword && (
          <p className="text-xs mt-1 text-red-500">两次输入的密码不一致</p>
        )}
      </div>
    </>
  );

  const userMode = localStorage.getItem('user_mode');
  const showClawparrotHint = isTauri && userMode === 'clawparrot';

  const handleSkipLogin = () => {
    localStorage.setItem('user_mode', 'selfhosted');
    localStorage.removeItem('ANTHROPIC_API_KEY');
    localStorage.removeItem('ANTHROPIC_BASE_URL');
    localStorage.removeItem('gateway_user');
    localStorage.removeItem('auth_token');
    window.location.hash = '#/';
    window.location.reload();
  };

  const openClawparrot = () => {
    try { import('../utils/tauriAPI').then(m => m.tauriAPI.openExternal('http://127.0.0.1:30090/register')); } catch {}
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#F8F8F6] font-sans relative">
      {/* Back button — lets users leave the login page without logging in.
          Uses navigate(-1) if there's history, otherwise navigates to / */}
      <button
        onClick={() => {
          if (window.history.length > 1) navigate(-1);
          else navigate('/');
        }}
        className="absolute top-6 left-6 flex items-center gap-1.5 px-3 py-1.5 text-sm text-[#747474] hover:text-[#222] hover:bg-black/5 rounded-md transition-colors"
      >
        <ArrowLeft size={16} />
        返回
      </button>
      {showClawparrotHint && (
        <button
          onClick={handleSkipLogin}
          className="absolute top-14 right-6 px-3 py-1.5 text-sm text-[#747474] hover:text-[#222] hover:bg-black/5 rounded-md transition-colors"
        >
          跳过登录
        </button>
      )}
      <div className="w-full max-w-md p-8 bg-white rounded-2xl shadow-sm border border-[#E5E5E5]">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-serif-claude text-[#222] mb-2">Claude</h1>
          <p className="text-[#747474]">欢迎回来</p>
        </div>

        {showClawparrotHint && view === 'login' && (
          <div className="mb-4 p-3 bg-[#F5F0E9] border border-[#E5D9C8] text-[#6B4E3D] text-[13px] rounded-lg leading-relaxed">
            首次使用请先在{' '}
            <button onClick={openClawparrot} className="text-[#CC7C5E] underline hover:text-[#B96B4E] font-medium">
              Claude Rust
            </button>
            {' '}用邮箱注册账号，然后回到这里登录。右上角可跳过登录、改用自部署模式。
          </div>
        )}

        {error && <div className="mb-4 p-3 bg-red-50 text-red-600 text-sm rounded-lg">{error}</div>}
        {message && <div className="mb-4 p-3 bg-green-50 text-green-600 text-sm rounded-lg">{message}</div>}

        {/* 登录 */}
        {view === 'login' && (
          <form onSubmit={handleLogin} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-[#393939] mb-1">邮箱</label>
              <input type="email" required value={email} onChange={(e) => setEmail(e.target.value)}
                className={inputClass} placeholder="name@example.com" />
            </div>
            <div>
              <label className="block text-sm font-medium text-[#393939] mb-1">密码</label>
              <input type="password" required value={password} onChange={(e) => setPassword(e.target.value)}
                className={inputClass} placeholder="••••••••" />
            </div>
            <button type="submit" disabled={loading} className={btnClass}>
              {loading ? '登录中...' : '登录'}
            </button>
          </form>
        )}

      </div>
    </div>
  );
};

export default Auth;
