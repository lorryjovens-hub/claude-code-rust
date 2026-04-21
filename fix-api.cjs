const fs = require('fs');
let content = fs.readFileSync('..\\claude-desktop-app-main\\src\\api.ts', 'utf8');
content = content.replace(
  "const isElectronApp = typeof window !== 'undefined' && !!(window as any).electronAPI?.isElectron;",
  "const isTauriApp = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;"
);
content = content.replace(/isElectronApp/g, 'isTauriApp');
fs.writeFileSync('src\\api.ts', content);
console.log('api.ts copied and updated successfully');
