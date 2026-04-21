const fs = require('fs');

// Fix Auth.tsx
let auth = fs.readFileSync('src/components/Auth.tsx', 'utf8');
if (!auth.includes('const isTauri')) {
  auth = "const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;\n" + auth;
}
auth = auth.replace(/isElectron/g, 'isTauri');
fs.writeFileSync('src/components/Auth.tsx', auth);
console.log('Auth.tsx fixed');

// Fix MessageAttachments.tsx
let ma = fs.readFileSync('src/components/MessageAttachments.tsx', 'utf8');
if (!ma.includes('const isTauri')) {
  ma = "const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;\n" + ma;
}
ma = ma.replace(/isElectron/g, 'isTauri');
fs.writeFileSync('src/components/MessageAttachments.tsx', ma);
console.log('MessageAttachments.tsx fixed');

// Fix Onboarding.tsx
let ob = fs.readFileSync('src/components/Onboarding.tsx', 'utf8');
ob = ob.replace(/api\?\.openExternal\?\./g, "import('../utils/tauriAPI').then(m => m.tauriAPI.openExternal).catch(() => {})");
ob = ob.replace(/try \{ api\?\.openExternal\?\./g, "try { import('../utils/tauriAPI').then(m => m.tauriAPI.openExternal");
fs.writeFileSync('src/components/Onboarding.tsx', ob);
console.log('Onboarding.tsx fixed');

// Fix SettingsPage.tsx - add 'models' to Tab type
let sp = fs.readFileSync('src/components/SettingsPage.tsx', 'utf8');
sp = sp.replace(/type Tab = 'general' \| 'account' \| 'usage'/g, "type Tab = 'general' | 'account' | 'usage' | 'models'");
sp = sp.replace(/__APP_VERSION__/g, "'1.6.12'");
fs.writeFileSync('src/components/SettingsPage.tsx', sp);
console.log('SettingsPage.tsx fixed');

// Fix MainContent.tsx - setOpenedResearchMsgId scope and warmEngine
let mc = fs.readFileSync('src/components/MainContent.tsx', 'utf8');
// Ensure warmEngine only called with string
mc = mc.replace(/warmEngine\(convId\);/g, 'if (convId) warmEngine(convId);');
// Fix materializeGithub call
mc = mc.replace(/const result = await materializeGithub\(\s*convId,\s*payload\.repoFullName,/g, 'const result = await materializeGithub(\n      convId!,\n      payload.repoFullName,');
fs.writeFileSync('src/components/MainContent.tsx', mc);
console.log('MainContent.tsx fixed');

console.log('All fixes applied!');
