import { Component, ReactNode } from 'react';
import { AlertTriangle, Home, Mail, Phone, RefreshCw } from 'lucide-react';
import { Button } from './ui/button';

const SUPPORT_EMAIL = 'support@esip.local';
const SUPPORT_PHONE = '+1 (800) 000-0000';

function genCorrelationId() {
  return `ESIP-${Date.now().toString(36).toUpperCase()}-${Math.random().toString(36).slice(2, 6).toUpperCase()}`;
}

interface Props { children: ReactNode }
interface State { hasError: boolean; correlationId: string }

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, correlationId: '' };
  }

  static getDerivedStateFromError(): State {
    return { hasError: true, correlationId: genCorrelationId() };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // Log full details to console for devs; never expose to UI
    console.error('[ESIP] Unhandled error:', error, info);
  }

  handleTryAgain = () => {
    this.setState({ hasError: false, correlationId: '' });
  };

  render() {
    if (!this.state.hasError) return this.props.children;

    const { correlationId } = this.state;

    return (
      <div className="min-h-screen flex items-center justify-center bg-background px-4">
        <div className="max-w-lg w-full space-y-6">

          <div className="flex flex-col items-center gap-3 text-center">
            <div className="flex size-16 items-center justify-center rounded-full bg-muted">
              <AlertTriangle className="size-8 text-destructive" />
            </div>
            <div>
              <h1 className="text-2xl font-bold">Something went wrong</h1>
              <p className="mt-1 text-sm text-muted-foreground">
                An unexpected error occurred. Your work has been preserved — try refreshing
                the section or contact support if the problem persists.
              </p>
            </div>
          </div>

          <div className="rounded-lg border border-border bg-muted/40 px-4 py-3 text-center">
            <p className="text-xs text-muted-foreground">Reference ID</p>
            <p className="mt-0.5 font-mono text-sm font-semibold tracking-wider">{correlationId}</p>
            <p className="mt-1 text-xs text-muted-foreground">Quote this when contacting support</p>
          </div>

          <div className="rounded-lg border border-border bg-card p-4 space-y-2">
            <p className="text-sm font-medium">Need help? Contact support</p>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Phone className="size-3.5 shrink-0" />
              <span>{SUPPORT_PHONE}</span>
            </div>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Mail className="size-3.5 shrink-0" />
              <a href={`mailto:${SUPPORT_EMAIL}?subject=ESIP Error ${correlationId}`} className="hover:underline">
                {SUPPORT_EMAIL}
              </a>
            </div>
          </div>

          <div className="flex gap-3">
            <Button className="flex-1" onClick={this.handleTryAgain}>
              <RefreshCw className="size-4 mr-2" />
              Try again
            </Button>
            <Button variant="outline" className="flex-1" onClick={() => window.location.href = '/projects/list'}>
              <Home className="size-4 mr-2" />
              Go to Projects
            </Button>
          </div>

        </div>
      </div>
    );
  }
}
