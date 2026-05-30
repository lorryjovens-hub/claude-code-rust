import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

console.log('[Main] Starting application...');
console.log('[Main] __TAURI_INTERNALS__:', !!(window as any).__TAURI_INTERNALS__);

window.addEventListener('error', (event) => {
  console.error('[Global] Uncaught error:', event.error || event.message);
  event.preventDefault();
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('[Global] Unhandled promise rejection:', event.reason);
  event.preventDefault();
});

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null; errorInfo: React.ErrorInfo | null }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('[ErrorBoundary] React rendering error:', error);
    console.error('[ErrorBoundary] Component stack:', errorInfo.componentStack);
    this.setState({ errorInfo });
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: '40px',
          fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif',
          background: '#fff',
          color: '#333',
          maxWidth: '800px',
          margin: '0 auto',
        }}>
          <h1 style={{ color: '#e53e3e', marginBottom: '16px' }}>Application Error</h1>
          <div style={{
            padding: '16px',
            background: '#fff5f5',
            border: '1px solid #fed7d7',
            borderRadius: '8px',
            marginBottom: '16px',
          }}>
            <p style={{ fontWeight: 'bold', marginBottom: '8px' }}>{this.state.error?.message}</p>
            <pre style={{
              fontSize: '12px',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              maxHeight: '300px',
              overflow: 'auto',
            }}>
              {this.state.error?.stack}
            </pre>
          </div>
          {this.state.errorInfo && (
            <div style={{
              padding: '16px',
              background: '#f7fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '8px',
            }}>
              <p style={{ fontWeight: 'bold', marginBottom: '8px' }}>Component Stack:</p>
              <pre style={{
                fontSize: '12px',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                maxHeight: '300px',
                overflow: 'auto',
              }}>
                {this.state.errorInfo.componentStack}
              </pre>
            </div>
          )}
          <div style={{ display: 'flex', gap: '12px', marginTop: '16px' }}>
            <button
              onClick={this.handleReset}
              style={{
                padding: '8px 24px',
                background: '#48bb78',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontSize: '14px',
              }}
            >
              Try Recover
            </button>
            <button
              onClick={() => window.location.reload()}
              style={{
                padding: '8px 24px',
                background: '#4299e1',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontSize: '14px',
              }}
            >
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

const rootElement = document.getElementById('root');
if (!rootElement) {
  document.body.innerHTML = '<div style="padding:40px;color:red;font-size:24px;">ERROR: root element not found!</div>';
  throw new Error('Root element not found');
}

console.log('[Main] Root element found, rendering App...');

try {
  ReactDOM.createRoot(rootElement).render(
    <ErrorBoundary>
      <App />
    </ErrorBoundary>,
  );
  console.log('[Main] App rendered successfully');
} catch (e) {
  console.error('[Main] Failed to render App:', e);
  const errMsg = e instanceof Error ? e.message : String(e);
  rootElement.innerHTML = `<div style="padding:40px;color:red;">
    <h1>Fatal Render Error</h1>
    <pre>${errMsg}</pre>
    <button onclick="window.location.reload()" style="margin-top:16px;padding:8px 24px;background:#4299e1;color:white;border:none;border-radius:6px;cursor:pointer;">Reload</button>
  </div>`;
}
