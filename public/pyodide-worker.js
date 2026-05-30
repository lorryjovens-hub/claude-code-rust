/**
 * Pyodide Web Worker — 在用户浏览器中执行 Python 代码
 * 懒加载 Pyodide，预装 numpy/pandas/matplotlib 等包
 */

const PYODIDE_CDN = 'https://cdn.jsdelivr.net/pyodide/v0.27.0/full/';
let pyodide = null;
let loading = false;

async function loadPyodideAndPackages() {
  if (pyodide) return pyodide;
  if (loading) {
    // 等待其他加载完成
    while (loading) await new Promise(r => setTimeout(r, 100));
    return pyodide;
  }
  loading = true;
  try {
    importScripts(PYODIDE_CDN + 'pyodide.js');
    pyodide = await loadPyodide({ indexURL: PYODIDE_CDN });
    // 预装常用包
    await pyodide.loadPackage('micropip');
    const micropip = pyodide.pyimport('micropip');
    await micropip.install(['numpy', 'pandas', 'matplotlib', 'openpyxl', 'scipy']);
    // 下载中文字体到 Pyodide 虚拟文件系统
    try {
      const fontUrl = 'https://cdn.jsdelivr.net/gh/notofonts/noto-cjk/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf';
      const fontResp = await fetch(fontUrl);
      if (fontResp.ok) {
        const fontBuf = await fontResp.arrayBuffer();
        pyodide.FS.mkdirTree('/usr/share/fonts/chinese');
        pyodide.FS.writeFile('/usr/share/fonts/chinese/NotoSansCJKsc-Regular.otf', new Uint8Array(fontBuf));
      }
    } catch (e) {
      console.warn('Failed to load Chinese font:', e);
    }
    // 配置 matplotlib 使用 Agg 后端 + 中文字体
    pyodide.runPython(`
import matplotlib
matplotlib.use('Agg')
import warnings
warnings.filterwarnings('ignore', category=UserWarning, module='matplotlib')

# 注册中文字体
import matplotlib.font_manager as fm
import os
font_path = '/usr/share/fonts/chinese/NotoSansCJKsc-Regular.otf'
if os.path.exists(font_path):
    fm.fontManager.addfont(font_path)
    prop = fm.FontProperties(fname=font_path)
    font_name = prop.get_name()
    matplotlib.rcParams['font.sans-serif'] = [font_name, 'DejaVu Sans', 'Arial', 'sans-serif']
else:
    matplotlib.rcParams['font.sans-serif'] = ['DejaVu Sans', 'Arial', 'sans-serif']
matplotlib.rcParams['axes.unicode_minus'] = False
`);
    loading = false;
    return pyodide;
  } catch (e) {
    loading = false;
    throw e;
  }
}

async function downloadFiles(files) {
  if (!files || files.length === 0) return;
  // 创建 /data/ 目录
  pyodide.runPython(`
import os
os.makedirs('/data', exist_ok=True)
`);
  for (const file of files) {
    try {
      const response = await fetch(file.url);
      if (!response.ok) {
        let detail = '';
        try {
          detail = (await response.text()).slice(0, 120);
        } catch {}
        throw new Error(`HTTP ${response.status}${detail ? `: ${detail}` : ''}`);
      }
      const buffer = await response.arrayBuffer();
      const uint8 = new Uint8Array(buffer);
      pyodide.FS.writeFile('/data/' + file.name, uint8);
    } catch (e) {
      throw new Error(`下载文件失败 ${file.name}: ${e.message || String(e)}`);
    }
  }
}

async function executeCode(code, files) {
  const py = await loadPyodideAndPackages();
  await downloadFiles(files);

  // 捕获 stdout/stderr
  py.runPython(`
import sys, io
__stdout_capture = io.StringIO()
__stderr_capture = io.StringIO()
sys.stdout = __stdout_capture
sys.stderr = __stderr_capture
__captured_images = []

import warnings
warnings.filterwarnings('ignore', category=UserWarning, module='matplotlib')

# Monkey-patch plt.show()
import matplotlib.pyplot as plt
_original_show = plt.show
def _capture_show(*args, **kwargs):
    import base64
    buf = io.BytesIO()
    plt.savefig(buf, format='png', dpi=150, bbox_inches='tight')
    buf.seek(0)
    __captured_images.append(base64.b64encode(buf.read()).decode('utf-8'))
    plt.close('all')
plt.show = _capture_show
`);

  let error = null;
  try {
    // 执行用户代码（带超时）
    py.runPython(code);
  } catch (e) {
    error = e.message || String(e);
  }

  // 收集结果
  const stdout = py.runPython('__stdout_capture.getvalue()');
  let stderr = py.runPython('__stderr_capture.getvalue()');
  const imagesList = py.runPython('__captured_images');
  const images = imagesList.toJs ? imagesList.toJs() : [];

  // 过滤掉 matplotlib 字体相关的警告
  if (stderr) {
    stderr = stderr.split('\n').filter(line => {
      if (line.includes('UserWarning: Glyph')) return false;
      if (line.includes('font cache')) return false;
      if (line.includes('missing from current font')) return false;
      if (line.trim() === '') return false;
      // 过滤掉 warnings.warn 的堆栈行
      if (/^\s+warnings\.warn/.test(line)) return false;
      return true;
    }).join('\n').trim();
  }

  // 恢复 stdout/stderr
  py.runPython(`
sys.stdout = sys.__stdout__
sys.stderr = sys.__stderr__
`);

  return { stdout, stderr, images: Array.from(images), error };
}

// 消息处理
self.onmessage = async function(e) {
  const { type, code, files, id } = e.data;

  if (type === 'execute') {
    try {
      self.postMessage({ type: 'status', status: 'loading', id });
      const result = await executeCode(code, files);
      self.postMessage({ type: 'result', id, ...result });
    } catch (err) {
      self.postMessage({
        type: 'result',
        id,
        stdout: '',
        stderr: '',
        images: [],
        error: err.message || String(err),
      });
    }
  }
};
