import express from 'express';
import cors from 'cors';
import Database from 'better-sqlite3';
import jwt from 'jsonwebtoken';
import bcrypt from 'bcryptjs';
import { v4 as uuidv4 } from 'uuid';
import https from 'https';
import http from 'http';
import crypto from 'crypto';

const app = express();
const PORT = 30090;
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key-change-in-production';
const KIE_API_URL = process.env.KIE_API_URL || 'https://api.kie.ai/v1';
const KIE_API_KEY = process.env.KIE_API_KEY || '';

app.use(cors());
app.use(express.json());

const db = new Database('proxy.db');

db.exec(`
  CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    subscription_status TEXT DEFAULT 'active',
    subscription_plan TEXT DEFAULT 'free',
    api_usage INTEGER DEFAULT 0
  );

  CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_used_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
  );

  CREATE TABLE IF NOT EXISTS usage_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cost REAL DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
  );
`);

function hashApiKey(key) {
  return crypto.createHash('sha256').update(key).digest('hex');
}

function verifyApiKey(key, storedHash) {
  return hashApiKey(key) === storedHash;
}

function generateApiKey() {
  return 'sk-' + uuidv4().replace(/-/g, '') + uuidv4().replace(/-/g, '').substring(0, 32);
}

function authenticate(req, res, next) {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: 'Missing or invalid authorization header' });
  }

  const token = authHeader.substring(7);
  try {
    const decoded = jwt.verify(token, JWT_SECRET);
    req.user = decoded;
    next();
  } catch (err) {
    return res.status(401).json({ error: 'Invalid or expired token' });
  }
}

function optionalAuth(req, res, next) {
  const authHeader = req.headers.authorization;
  if (authHeader && authHeader.startsWith('Bearer ')) {
    const token = authHeader.substring(7);
    try {
      const decoded = jwt.verify(token, JWT_SECRET);
      req.user = decoded;
    } catch (err) {
    }
  }
  next();
}

function calculateCost(model, inputTokens, outputTokens) {
  const pricing = {
    'claude-opus-4-6': { input: 0.015, output: 0.075 },
    'claude-sonnet-4-6': { input: 0.003, output: 0.015 },
    'claude-haiku-4-5-20251001': { input: 0.0008, output: 0.004 },
    'claude-opus-4': { input: 0.015, output: 0.075 },
    'claude-sonnet-4': { input: 0.003, output: 0.015 },
    'claude-haiku-3-5': { input: 0.0008, output: 0.004 },
  };
  const modelKey = Object.keys(pricing).find(k => model.includes(k.split('-').pop()));
  const p = modelKey ? pricing[modelKey] : pricing['claude-sonnet-4-6'];
  return (inputTokens * p.input + outputTokens * p.output) / 1000;
}

app.post('/api/auth/register', async (req, res) => {
  try {
    const { email, password } = req.body;
    if (!email || !password) {
      return res.status(400).json({ error: 'Email and password are required' });
    }

    const existingUser = db.prepare('SELECT id FROM users WHERE email = ?').get(email);
    if (existingUser) {
      return res.status(409).json({ error: 'Email already registered' });
    }

    const id = uuidv4();
    const passwordHash = await bcrypt.hash(password, 10);
    const apiKey = generateApiKey();
    const apiKeyHash = hashApiKey(apiKey);

    db.prepare('INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)').run(id, email, passwordHash);
    db.prepare('INSERT INTO api_keys (id, user_id, key_hash) VALUES (?, ?, ?)').run(uuidv4(), id, apiKeyHash);

    const token = jwt.sign({ userId: id, email }, JWT_SECRET, { expiresIn: '30d' });

    res.json({
      token,
      user: { id, email },
      apiKey,
      message: 'Registration successful'
    });
  } catch (err) {
    console.error('Registration error:', err);
    res.status(500).json({ error: 'Registration failed' });
  }
});

app.post('/api/auth/login', async (req, res) => {
  try {
    const { email, password } = req.body;
    if (!email || !password) {
      return res.status(400).json({ error: 'Email and password are required' });
    }

    const user = db.prepare('SELECT * FROM users WHERE email = ?').get(email);
    if (!user) {
      return res.status(401).json({ error: 'Invalid email or password' });
    }

    const validPassword = await bcrypt.compare(password, user.password_hash);
    if (!validPassword) {
      return res.status(401).json({ error: 'Invalid email or password' });
    }

    const token = jwt.sign({ userId: user.id, email: user.email }, JWT_SECRET, { expiresIn: '30d' });

    res.json({
      token,
      user: { id: user.id, email: user.email },
      subscription: {
        status: user.subscription_status,
        plan: user.subscription_plan,
        apiUsage: user.api_usage
      }
    });
  } catch (err) {
    console.error('Login error:', err);
    res.status(500).json({ error: 'Login failed' });
  }
});

app.get('/api/auth/me', authenticate, (req, res) => {
  const user = db.prepare('SELECT id, email, subscription_status, subscription_plan, api_usage FROM users WHERE id = ?').get(req.user.userId);
  if (!user) {
    return res.status(404).json({ error: 'User not found' });
  }
  res.json({
    user: { id: user.id, email: user.email },
    subscription: {
      status: user.subscription_status,
      plan: user.subscription_plan,
      apiUsage: user.api_usage
    }
  });
});

app.get('/api/subscription', authenticate, (req, res) => {
  const user = db.prepare('SELECT subscription_status, subscription_plan, api_usage FROM users WHERE id = ?').get(req.user.userId);
  if (!user) {
    return res.status(404).json({ error: 'User not found' });
  }
  res.json({
    status: user.subscription_status,
    plan: user.subscription_plan,
    usage: user.api_usage,
    limits: {
      free: { maxRequests: 100, maxTokens: 100000 },
      pro: { maxRequests: 1000, maxTokens: 1000000 }
    }
  });
});

app.post('/api/subscription/upgrade', authenticate, (req, res) => {
  const { plan } = req.body;
  if (!['free', 'pro'].includes(plan)) {
    return res.status(400).json({ error: 'Invalid plan' });
  }

  db.prepare('UPDATE users SET subscription_plan = ? WHERE id = ?').run(plan, req.user.userId);
  res.json({ success: true, plan });
});

app.get('/api/usage', authenticate, (req, res) => {
  const logs = db.prepare('SELECT * FROM usage_logs WHERE user_id = ? ORDER BY created_at DESC LIMIT 100').all(req.user.userId);
  const totals = db.prepare('SELECT SUM(input_tokens) as totalInput, SUM(output_tokens) as totalOutput, SUM(cost) as totalCost FROM usage_logs WHERE user_id = ?').get(req.user.userId);

  res.json({
    logs,
    totals: {
      inputTokens: totals.totalInput || 0,
      outputTokens: totals.totalOutput || 0,
      cost: totals.totalCost || 0
    }
  });
});

app.get('/api/keys', authenticate, (req, res) => {
  const keys = db.prepare('SELECT id, created_at, last_used_at FROM api_keys WHERE user_id = ?').all(req.user.userId);
  res.json({ keys });
});

app.post('/api/keys', authenticate, (req, res) => {
  const apiKey = generateApiKey();
  const apiKeyHash = hashApiKey(apiKey);
  db.prepare('INSERT INTO api_keys (id, user_id, key_hash) VALUES (?, ?, ?)').run(uuidv4(), req.user.userId, apiKeyHash);
  res.json({ apiKey, message: 'API key created. Save it securely - it will not be shown again.' });
});

app.delete('/api/keys/:id', authenticate, (req, res) => {
  db.prepare('DELETE FROM api_keys WHERE id = ? AND user_id = ?').run(req.params.id, req.user.userId);
  res.json({ success: true });
});

function proxyToKIE(req, res, endpoint, body) {
  return new Promise((resolve, reject) => {
    const kieUrl = new URL(endpoint, KIE_API_URL);
    const data = JSON.stringify(body);

    const options = {
      hostname: kieUrl.hostname,
      port: kieUrl.port || 443,
      path: kieUrl.pathname,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(data),
        'Authorization': `Bearer ${KIE_API_KEY}`
      }
    };

    const protocol = kieUrl.protocol === 'https:' ? https : http;
    const proxyReq = protocol.request(options, (proxyRes) => {
      res.status(proxyRes.statusCode);
      proxyRes.headers['content-type'] && res.setHeader('Content-Type', proxyRes.headers['content-type']);
      proxyRes.pipe(res);
    });

    proxyReq.on('error', reject);
    proxyReq.write(data);
    proxyReq.end();
  });
}

function parseSSEStream(data) {
  const lines = data.split('\n');
  let eventType = 'message';
  let eventData = '';

  for (const line of lines) {
    if (line.startsWith('event:')) {
      eventType = line.substring(6).trim();
    } else if (line.startsWith('data:')) {
      eventData = line.substring(5).trim();
    }
  }

  return { eventType, data: eventData };
}

app.post('/api/v1/messages', optionalAuth, async (req, res) => {
  try {
    const { model, messages, max_tokens = 8192, stream = false } = req.body;

    if (!model) {
      return res.status(400).json({ error: 'Model is required' });
    }

    if (req.user) {
      const user = db.prepare('SELECT * FROM users WHERE id = ?').get(req.user.userId);
      if (user.subscription_status !== 'active') {
        return res.status(403).json({ error: 'Subscription not active' });
      }
    }

    const requestBody = {
      model,
      messages,
      max_tokens
    };

    if (stream) {
      res.setHeader('Content-Type', 'text/event-stream');
      res.setHeader('Cache-Control', 'no-cache');
      res.setHeader('Connection', 'keep-alive');

      const kieUrl = new URL('/messages', KIE_API_URL);
      const data = JSON.stringify(requestBody);

      const options = {
        hostname: kieUrl.hostname,
        port: kieUrl.port || 443,
        path: kieUrl.pathname,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(data),
          'Authorization': `Bearer ${KIE_API_KEY}`,
          'Accept': 'text/event-stream'
        }
      };

      const protocol = kieUrl.protocol === 'https:' ? https : http;
      const proxyReq = protocol.request(options, (proxyRes) => {
        if (proxyRes.statusCode !== 200) {
          res.status(proxyRes.statusCode);
          let errorData = '';
          proxyRes.on('data', chunk => errorData += chunk);
          proxyRes.on('end', () => {
            res.end(`data: ${JSON.stringify({ error: errorData || 'KIE API error' })}\n\n`);
          });
          return;
        }

        proxyRes.on('data', (chunk) => {
          const lines = chunk.toString().split('\n');
          for (const line of lines) {
            if (line.trim()) {
              res.write(line + '\n');
            }
          }
        });

        proxyRes.on('end', () => {
          res.end();
        });
      });

      proxyReq.on('error', (err) => {
        console.error('KIE proxy error:', err);
        res.status(500).end(`data: ${JSON.stringify({ error: 'Proxy error: ' + err.message })}\n\n`);
      });

      proxyReq.write(data);
      proxyReq.end();

    } else {
      const result = await proxyToKIE(req, res, '/messages', requestBody);

      if (req.user) {
        const inputTokens = JSON.stringify(messages).length / 4;
        const outputTokens = result?.content?.[0]?.text?.length / 4 || 0;
        const cost = calculateCost(model, inputTokens, outputTokens);

        db.prepare('UPDATE users SET api_usage = api_usage + 1 WHERE id = ?').run(req.user.userId);
        db.prepare('INSERT INTO usage_logs (id, user_id, model, input_tokens, output_tokens, cost) VALUES (?, ?, ?, ?, ?, ?)')
          .run(uuidv4(), req.user.userId, model, Math.round(inputTokens), Math.round(outputTokens), cost);
      }
    }
  } catch (err) {
    console.error('API error:', err);
    res.status(500).json({ error: err.message || 'Internal server error' });
  }
});

app.post('/api/v1/chat/completions', optionalAuth, async (req, res) => {
  try {
    const { model, messages, max_tokens = 8192, stream = false } = req.body;

    if (!model) {
      return res.status(400).json({ error: 'Model is required' });
    }

    const requestBody = {
      model,
      messages,
      max_tokens,
      stream
    };

    if (stream) {
      res.setHeader('Content-Type', 'text/event-stream');
      res.setHeader('Cache-Control', 'no-cache');
      res.setHeader('Connection', 'keep-alive');

      const kieUrl = new URL('/chat/completions', KIE_API_URL);
      const data = JSON.stringify(requestBody);

      const options = {
        hostname: kieUrl.hostname,
        port: kieUrl.port || 443,
        path: kieUrl.pathname,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(data),
          'Authorization': `Bearer ${KIE_API_KEY}`,
          'Accept': 'text/event-stream'
        }
      };

      const protocol = kieUrl.protocol === 'https:' ? https : http;
      const proxyReq = protocol.request(options, (proxyRes) => {
        if (proxyRes.statusCode !== 200) {
          res.status(proxyRes.statusCode);
          let errorData = '';
          proxyRes.on('data', chunk => errorData += chunk);
          proxyRes.on('end', () => {
            res.end(`data: ${JSON.stringify({ error: errorData || 'KIE API error' })}\n\n`);
          });
          return;
        }

        proxyRes.on('data', (chunk) => {
          const lines = chunk.toString().split('\n');
          for (const line of lines) {
            if (line.trim()) {
              res.write(line + '\n');
            }
          }
        });

        proxyRes.on('end', () => {
          res.end();
        });
      });

      proxyReq.on('error', (err) => {
        console.error('KIE proxy error:', err);
        res.status(500).end(`data: ${JSON.stringify({ error: 'Proxy error: ' + err.message })}\n\n`);
      });

      proxyReq.write(data);
      proxyReq.end();

    } else {
      await proxyToKIE(req, res, '/chat/completions', requestBody);
    }
  } catch (err) {
    console.error('API error:', err);
    res.status(500).json({ error: err.message || 'Internal server error' });
  }
});

app.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

app.get('/register', (req, res) => {
  res.send(`<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>注册 - Claude Rust</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #F8F8F6; min-height: 100vh; display: flex; align-items: center; justify-content: center; }
    .container { background: white; padding: 40px; border-radius: 16px; box-shadow: 0 2px 12px rgba(0,0,0,0.08); width: 400px; max-width: 90vw; }
    h1 { font-size: 28px; margin-bottom: 8px; color: #222; }
    .subtitle { color: #747474; margin-bottom: 30px; font-size: 14px; }
    .form-group { margin-bottom: 20px; }
    label { display: block; font-size: 14px; font-weight: 500; color: #393939; margin-bottom: 6px; }
    input { width: 100%; padding: 12px; border: 1px solid #E5E5E5; border-radius: 10px; font-size: 15px; outline: none; transition: border-color 0.2s; }
    input:focus { border-color: #3b82f6; }
    button { width: 100%; padding: 14px; background: #222; color: white; border: none; border-radius: 10px; font-size: 15px; font-weight: 500; cursor: pointer; transition: opacity 0.2s; }
    button:hover { opacity: 0.9; }
    button:disabled { opacity: 0.5; cursor: not-allowed; }
    .message { padding: 12px; border-radius: 8px; margin-bottom: 20px; font-size: 14px; }
    .error { background: #fef2f2; color: #dc2626; border: 1px solid #fecaca; }
    .success { background: #f0fdf4; color: #16a34a; border: 1px solid #bbf7d0; }
    .login-link { text-align: center; margin-top: 20px; font-size: 14px; color: #747474; }
    .login-link a { color: #CC7C5E; text-decoration: none; }
    .login-link a:hover { text-decoration: underline; }
  </style>
</head>
<body>
  <div class="container">
    <h1>Claude Rust</h1>
    <p class="subtitle">创建你的账号</p>
    <div id="message"></div>
    <form id="registerForm">
      <div class="form-group">
        <label>邮箱</label>
        <input type="email" id="email" required placeholder="name@example.com">
      </div>
      <div class="form-group">
        <label>密码</label>
        <input type="password" id="password" required placeholder="••••••••" minlength="6">
      </div>
      <button type="submit" id="submitBtn">注册</button>
    </form>
    <p class="login-link">已有账号? <a href="/login">登录</a></p>
  </div>
  <script>
    document.getElementById('registerForm').addEventListener('submit', async (e) => {
      e.preventDefault();
      const btn = document.getElementById('submitBtn');
      const msg = document.getElementById('message');
      btn.disabled = true;
      btn.textContent = '注册中...';
      msg.innerHTML = '';

      try {
        const res = await fetch('/api/auth/register', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            email: document.getElementById('email').value,
            password: document.getElementById('password').value
          })
        });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || '注册失败');

        msg.className = 'message success';
        msg.textContent = '注册成功！请在应用中登录。';
        btn.textContent = '注册成功';
        setTimeout(() => { window.location.href = '/login'; }, 1500);
      } catch (err) {
        msg.className = 'message error';
        msg.textContent = err.message;
        btn.disabled = false;
        btn.textContent = '注册';
      }
    });
  </script>
</body>
</html>`);
});

app.get('/login', (req, res) => {
  res.send(`<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>登录 - Claude Rust</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #F8F8F6; min-height: 100vh; display: flex; align-items: center; justify-content: center; }
    .container { background: white; padding: 40px; border-radius: 16px; box-shadow: 0 2px 12px rgba(0,0,0,0.08); width: 400px; max-width: 90vw; }
    h1 { font-size: 28px; margin-bottom: 8px; color: #222; }
    .subtitle { color: #747474; margin-bottom: 30px; font-size: 14px; }
    .form-group { margin-bottom: 20px; }
    label { display: block; font-size: 14px; font-weight: 500; color: #393939; margin-bottom: 6px; }
    input { width: 100%; padding: 12px; border: 1px solid #E5E5E5; border-radius: 10px; font-size: 15px; outline: none; transition: border-color 0.2s; }
    input:focus { border-color: #3b82f6; }
    button { width: 100%; padding: 14px; background: #222; color: white; border: none; border-radius: 10px; font-size: 15px; font-weight: 500; cursor: pointer; transition: opacity 0.2s; }
    button:hover { opacity: 0.9; }
    button:disabled { opacity: 0.5; cursor: not-allowed; }
    .message { padding: 12px; border-radius: 8px; margin-bottom: 20px; font-size: 14px; }
    .error { background: #fef2f2; color: #dc2626; border: 1px solid #fecaca; }
    .success { background: #f0fdf4; color: #16a34a; border: 1px solid #bbf7d0; }
    .api-key { background: #f5f5f5; padding: 12px; border-radius: 8px; margin-top: 15px; font-family: monospace; font-size: 12px; word-break: break-all; }
    .register-link { text-align: center; margin-top: 20px; font-size: 14px; color: #747474; }
    .register-link a { color: #CC7C5E; text-decoration: none; }
    .register-link a:hover { text-decoration: underline; }
  </style>
</head>
<body>
  <div class="container">
    <h1>Claude Rust</h1>
    <p class="subtitle">登录你的账号</p>
    <div id="message"></div>
    <form id="loginForm">
      <div class="form-group">
        <label>邮箱</label>
        <input type="email" id="email" required placeholder="name@example.com">
      </div>
      <div class="form-group">
        <label>密码</label>
        <input type="password" id="password" required placeholder="••••••••">
      </div>
      <button type="submit" id="submitBtn">登录</button>
    </form>
    <p class="register-link">没有账号? <a href="/register">注册</a></p>
  </div>
  <script>
    document.getElementById('loginForm').addEventListener('submit', async (e) => {
      e.preventDefault();
      const btn = document.getElementById('submitBtn');
      const msg = document.getElementById('message');
      btn.disabled = true;
      btn.textContent = '登录中...';
      msg.innerHTML = '';
      msg.className = '';

      try {
        const res = await fetch('/api/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            email: document.getElementById('email').value,
            password: document.getElementById('password').value
          })
        });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || '登录失败');

        msg.className = 'message success';
        msg.innerHTML = '登录成功！<br><br>你的 API Key:<div class="api-key">' + (data.apiKey || '已生成') + '</div><br>请在 Claude Rust 应用中使用此 Key。';
        btn.textContent = '登录成功';
        setTimeout(() => { window.close(); }, 5000);
      } catch (err) {
        msg.className = 'message error';
        msg.textContent = err.message;
        btn.disabled = false;
        btn.textContent = '登录';
      }
    });
  </script>
</body>
</html>`);
});

app.listen(PORT, () => {
  console.log(`[API Proxy] Server running on http://127.0.0.1:${PORT}`);
  console.log(`[API Proxy] KIE API URL: ${KIE_API_URL}`);
  console.log(`[API Proxy] Database: proxy.db`);
  console.log(`[API Proxy] Register page: http://127.0.0.1:${PORT}/register`);
  console.log(`[API Proxy] Login page: http://127.0.0.1:${PORT}/login`);
});

process.on('SIGINT', () => {
  console.log('\n[API Proxy] Shutting down...');
  db.close();
  process.exit(0);
});
