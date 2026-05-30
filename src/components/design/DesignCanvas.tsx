import React, { useRef, useEffect, useCallback, useImperativeHandle, forwardRef } from 'react';

export interface DesignCanvasMessage {
  type: 'insert_component' | 'update_style' | 'apply_layout' | 'get_state' | 'reset' | 'set_content';
  payload: any;
}

export interface DesignCanvasRef {
  sendMessage: (msg: DesignCanvasMessage) => void;
  getPreviewContent: () => string;
  reloadPreview: () => void;
}

interface DesignCanvasProps {
  initialContent?: string;
  onContentChange?: (content: string) => void;
  className?: string;
}

const CANVAS_BOOTSTRAP = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Design Preview</title>
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; min-height: 100vh; }
  #design-canvas-root { min-height: 100vh; }
</style>
<style id="design-canvas-styles"></style>
</head>
<body>
<div id="design-canvas-root">
  <div style="display:flex;align-items:center;justify-content:center;min-height:100vh;color:#999;font-size:14px;">
    Design Canvas Ready
  </div>
</div>
<script>
(function() {
  var COMPONENTS = {
    button: '<button class="dc-btn" style="padding:10px 20px;border:none;border-radius:6px;background:#3B82F6;color:#fff;font-size:14px;cursor:pointer;">Button</button>',
    input: '<input class="dc-input" type="text" placeholder="Enter text..." style="padding:10px 14px;border:1px solid #d1d5db;border-radius:6px;font-size:14px;width:100%;max-width:300px;" />',
    card: '<div class="dc-card" style="background:#fff;border-radius:12px;box-shadow:0 1px 3px rgba(0,0,0,0.1);padding:24px;max-width:360px;"><h3 style="margin-bottom:8px;font-size:16px;">Card Title</h3><p style="color:#6b7280;font-size:14px;">Card description goes here.</p></div>',
    badge: '<span class="dc-badge" style="display:inline-block;padding:4px 10px;border-radius:999px;background:#EF4444;color:#fff;font-size:12px;">New</span>',
    avatar: '<div class="dc-avatar" style="width:40px;height:40px;border-radius:50%;background:#6366F1;color:#fff;display:flex;align-items:center;justify-content:center;font-size:16px;">A</div>',
    divider: '<hr class="dc-divider" style="border:none;border-top:1px solid #e5e7eb;margin:16px 0;" />',
    heading: '<h2 class="dc-heading" style="font-size:24px;font-weight:700;margin-bottom:12px;">Heading</h2>',
    paragraph: '<p class="dc-paragraph" style="color:#374151;font-size:14px;line-height:1.6;max-width:600px;">Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>',
    image: '<div class="dc-image" style="width:100%;max-width:400px;height:200px;background:#f3f4f6;border-radius:8px;display:flex;align-items:center;justify-content:center;color:#9ca3af;">Image Placeholder</div>',
    grid: '<div class="dc-grid" style="display:grid;grid-template-columns:repeat(3,1fr);gap:16px;max-width:600px;"><div style="padding:20px;background:#f9fafb;border-radius:8px;text-align:center;">Item 1</div><div style="padding:20px;background:#f9fafb;border-radius:8px;text-align:center;">Item 2</div><div style="padding:20px;background:#f9fafb;border-radius:8px;text-align:center;">Item 3</div></div>',
    navbar: '<nav class="dc-navbar" style="display:flex;align-items:center;justify-content:space-between;padding:12px 24px;background:#fff;border-bottom:1px solid #e5e7eb;"><span style="font-weight:700;">Logo</span><div style="display:flex;gap:16px;"><a href="#" style="color:#374151;text-decoration:none;">Home</a><a href="#" style="color:#374151;text-decoration:none;">About</a><a href="#" style="color:#374151;text-decoration:none;">Contact</a></div></nav>',
    hero: '<section class="dc-hero" style="text-align:center;padding:60px 24px;background:linear-gradient(135deg,#667eea,#764ba2);color:#fff;border-radius:12px;"><h1 style="font-size:36px;margin-bottom:12px;">Welcome</h1><p style="font-size:16px;opacity:0.9;margin-bottom:24px;">Build beautiful interfaces with ease.</p><button style="padding:12px 28px;background:#fff;color:#764ba2;border:none;border-radius:8px;font-size:14px;font-weight:600;cursor:pointer;">Get Started</button></section>'
  };

  var LAYOUTS = {
    flex_row: { display: 'flex', flexDirection: 'row', gap: '16px', flexWrap: 'wrap' },
    flex_col: { display: 'flex', flexDirection: 'column', gap: '16px' },
    grid_2: { display: 'grid', gridTemplateColumns: 'repeat(2,1fr)', gap: '16px', maxWidth: '800px' },
    grid_3: { display: 'grid', gridTemplateColumns: 'repeat(3,1fr)', gap: '16px', maxWidth: '800px' },
    centered: { display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '100vh' },
    sidebar: { display: 'grid', gridTemplateColumns: '240px 1fr', gap: '0', minHeight: '100vh' }
  };

  var root = document.getElementById('design-canvas-root');
  var styleEl = document.getElementById('design-canvas-styles');

  function applyStyle(style) {
    var rules = '';
    if (style.colors) {
      if (style.colors.primary) rules += '.dc-btn { background-color: ' + style.colors.primary + ' !important; }';
      if (style.colors.background) rules += 'body { background-color: ' + style.colors.background + '; }';
      if (style.colors.text) rules += 'body { color: ' + style.colors.text + '; }';
    }
    if (style.typography) {
      if (style.typography.fontFamily) rules += 'body { font-family: ' + style.typography.fontFamily + '; }';
      if (style.typography.fontSize) rules += 'body { font-size: ' + style.typography.fontSize + 'px; }';
      if (style.typography.headingSize) rules += 'h1,h2,h3 { font-size: ' + style.typography.headingSize + 'px; }';
    }
    if (style.spacing) {
      if (style.spacing.padding) rules += '#design-canvas-root { padding: ' + style.spacing.padding + 'px; }';
    }
    if (style.borderRadius) {
      rules += '.dc-card, .dc-btn, .dc-input, .dc-image { border-radius: ' + style.borderRadius + 'px !important; }';
    }
    if (style.shadow) {
      var s = style.shadow;
      var shadowStr = (s.x||0) + 'px ' + (s.y||0) + 'px ' + (s.blur||s.r||10) + 'px ' + (s.spread||0) + 'px ' + (s.color||'rgba(0,0,0,0.1)');
      rules += '.dc-card { box-shadow: ' + shadowStr + ' !important; }';
    }
    styleEl.textContent = rules;
  }

  function insertComponent(name) {
    var html = COMPONENTS[name];
    if (html) { root.innerHTML += html; }
  }

  function applyLayout(name) {
    var layout = LAYOUTS[name];
    if (layout) { Object.assign(root.style, layout); }
    if (name === 'reset') { root.style.cssText = ''; }
  }

  function getCanvasState() {
    return { html: root.innerHTML, styles: styleEl.textContent };
  }

  function setContent(html) {
    root.innerHTML = html;
  }

  window.addEventListener('message', function(event) {
    var msg = event.data;
    if (!msg || !msg.type) return;
    switch (msg.type) {
      case 'insert_component':
        if (msg.payload && msg.payload.name) insertComponent(msg.payload.name);
        break;
      case 'update_style':
        if (msg.payload) applyStyle(msg.payload);
        break;
      case 'apply_layout':
        if (msg.payload && msg.payload.name) applyLayout(msg.payload.name);
        break;
      case 'reset':
        root.innerHTML = '<div style="display:flex;align-items:center;justify-content:center;min-height:100vh;color:#999;font-size:14px;">Design Canvas Ready</div>';
        root.style.cssText = '';
        styleEl.textContent = '';
        break;
      case 'set_content':
        if (msg.payload && msg.payload.html) setContent(msg.payload.html);
        break;
      case 'get_state':
        window.parent.postMessage({ type: 'canvas_state', payload: getCanvasState() }, '*');
        break;
    }
  });

  console.log('[DesignCanvas] Bridge initialized');
})();
</script>
</body>
</html>`;

const DesignCanvas = forwardRef<DesignCanvasRef, DesignCanvasProps>(
  ({ initialContent, onContentChange, className = '' }, ref) => {
    const iframeRef = useRef<HTMLIFrameElement>(null);
    const contentRef = useRef(initialContent || CANVAS_BOOTSTRAP);

    const postMessage = useCallback((msg: DesignCanvasMessage) => {
      if (iframeRef.current?.contentWindow) {
        iframeRef.current.contentWindow.postMessage(msg, '*');
      }
    }, []);

    const sendMessage = useCallback((msg: DesignCanvasMessage) => {
      postMessage(msg);
    }, [postMessage]);

    const getPreviewContent = useCallback(() => {
      return contentRef.current;
    }, []);

    const reloadPreview = useCallback(() => {
      if (iframeRef.current) {
        iframeRef.current.srcdoc = contentRef.current;
      }
    }, []);

    useImperativeHandle(ref, () => ({
      sendMessage,
      getPreviewContent,
      reloadPreview,
    }), [sendMessage, getPreviewContent, reloadPreview]);

    useEffect(() => {
      const handleMessage = (event: MessageEvent) => {
        if (event.data?.type === 'canvas_state' && event.data?.payload?.html) {
          contentRef.current = buildFullHtml(event.data.payload.html, event.data.payload.styles);
          onContentChange?.(contentRef.current);
        }
      };

      window.addEventListener('message', handleMessage);
      return () => window.removeEventListener('message', handleMessage);
    }, [onContentChange]);

    return (
      <iframe
        ref={iframeRef}
        srcDoc={CANVAS_BOOTSTRAP}
        className={`w-full h-full border-0 bg-white ${className}`}
        title="Design Canvas"
        sandbox="allow-scripts"
      />
    );
  }
);

DesignCanvas.displayName = 'DesignCanvas';

function buildFullHtml(html: string, styles: string): string {
  return CANVAS_BOOTSTRAP.replace(
    '<div id="design-canvas-root">',
    '<div id="design-canvas-root">' + html
  ).replace(
    '<style id="design-canvas-styles"></style>',
    '<style id="design-canvas-styles">' + styles + '</style>'
  );
}

export default DesignCanvas;