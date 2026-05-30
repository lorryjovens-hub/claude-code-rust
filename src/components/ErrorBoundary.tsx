import React from 'react';

interface ErrorBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorStack: string | null;
  errorCount: number;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private lastErrorTime: number = 0;

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorStack: null, errorCount: 0 };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error, errorStack: error.stack || null };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const now = Date.now();
    const errorCount = now - this.lastErrorTime < 3000
      ? this.state.errorCount + 1
      : 1;
    this.lastErrorTime = now;

    this.setState({ errorCount });

    console.error(
      `[ErrorBoundary] Caught error (#${errorCount}):`,
      error.message,
      '\nComponent Stack:', errorInfo.componentStack,
      '\nError Stack:', error.stack,
    );

    this.props.onError?.(error, errorInfo);

    if (errorCount >= 5) {
      console.error('[ErrorBoundary] Too many errors in quick succession. Consider full page reload.');
    }
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null, errorStack: null, errorCount: 0 });
  };

  handleReload = () => {
    window.location.reload();
  };

  handleCopyError = async () => {
    if (!this.state.error) return;
    const errorText = [
      `Error: ${this.state.error.message}`,
      `Count: ${this.state.errorCount}`,
      `Time: ${new Date().toISOString()}`,
      `URL: ${window.location.href}`,
      `UserAgent: ${navigator.userAgent}`,
      '',
      'Stack:',
      this.state.errorStack || 'No stack trace available',
    ].join('\n');

    try {
      await navigator.clipboard.writeText(errorText);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = errorText;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      const showReload = this.state.errorCount >= 3;
      const isRecoverable = this.state.errorCount < 5;

      return (
        <div className="flex flex-col items-center justify-center min-h-[400px] p-8 text-center bg-claude-bg">
          <div className="w-16 h-16 mb-4 rounded-full bg-red-500/10 flex items-center justify-center border border-red-500/20">
            <svg className="w-8 h-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
          </div>

          <h2 className="text-lg font-semibold text-claude-text mb-1">Something went wrong</h2>
          <p className="text-[13px] text-claude-textSecondary mb-2 max-w-md">
            An unexpected error occurred while rendering this component.
          </p>

          <div className="mb-4 px-4 py-2 bg-red-500/5 border border-red-500/10 rounded-lg max-w-lg w-full text-left">
            <p className="text-[12px] text-red-300 font-mono break-all leading-relaxed">
              {this.state.error?.message || 'Unknown error'}
            </p>
            {this.state.errorStack && (
              <details className="mt-2">
                <summary className="text-[11px] text-claude-textSecondary cursor-pointer hover:text-claude-text">
                  Stack trace
                </summary>
                <pre className="mt-1 text-[10px] text-claude-textSecondary font-mono max-h-[200px] overflow-auto whitespace-pre-wrap break-all">
                  {this.state.errorStack}
                </pre>
              </details>
            )}
          </div>

          {this.state.errorCount > 1 && (
            <p className="text-[11px] text-amber-400 mb-3">
              {this.state.errorCount} errors detected.
              {showReload && ' A full reload is recommended.'}
            </p>
          )}

          <div className="flex items-center gap-2">
            {isRecoverable && (
              <button
                onClick={this.handleRetry}
                className="px-4 py-2 text-[13px] font-medium text-white bg-claude-textPrimary rounded-lg hover:bg-claude-textPrimary/80 transition-colors"
              >
                Try Again
              </button>
            )}
            <button
              onClick={this.handleCopyError}
              className="px-3 py-2 text-[13px] text-claude-textSecondary hover:text-claude-text hover:bg-claude-btn-hover rounded-lg transition-colors"
            >
              Copy Error
            </button>
            <button
              onClick={this.handleReload}
              className="px-3 py-2 text-[13px] text-claude-textSecondary hover:text-claude-text hover:bg-claude-btn-hover rounded-lg transition-colors"
            >
              Reload App
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}