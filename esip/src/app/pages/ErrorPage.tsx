import { useMemo } from 'react';
import { useRouteError, isRouteErrorResponse, Link } from 'react-router';
import { AlertTriangle, FileQuestion, Home, Mail, Phone } from 'lucide-react';
import { Button } from '../components/ui/button';

const SUPPORT_EMAIL = 'support@esip.local';
const SUPPORT_PHONE = '+1 (800) 000-0000';

function genCorrelationId() {
  return `ESIP-${Date.now().toString(36).toUpperCase()}-${Math.random().toString(36).slice(2, 6).toUpperCase()}`;
}

export default function ErrorPage() {
  const routeError = useRouteError();
  const correlationId = useMemo(() => genCorrelationId(), []);

  const is404 = !routeError || (isRouteErrorResponse(routeError) && routeError.status === 404);

  return (
    <div className="min-h-screen flex items-center justify-center bg-background px-4">
      <div className="max-w-lg w-full space-y-6">

        {/* Icon + heading */}
        <div className="flex flex-col items-center gap-3 text-center">
          <div className="flex size-16 items-center justify-center rounded-full bg-muted">
            {is404
              ? <FileQuestion className="size-8 text-muted-foreground" />
              : <AlertTriangle className="size-8 text-destructive" />
            }
          </div>
          <div>
            <h1 className="text-2xl font-bold">
              {is404 ? 'Page not found' : 'Something went wrong'}
            </h1>
            <p className="mt-1 text-muted-foreground text-sm">
              {is404
                ? "The page you're looking for doesn't exist or hasn't been built yet."
                : 'An unexpected error occurred. Our team has been notified and is looking into it.'}
            </p>
          </div>
        </div>

        {/* Correlation ID card */}
        <div className="rounded-lg border border-border bg-muted/40 px-4 py-3 text-center">
          <p className="text-xs text-muted-foreground">Reference ID</p>
          <p className="mt-0.5 font-mono text-sm font-semibold tracking-wider">{correlationId}</p>
          <p className="mt-1 text-xs text-muted-foreground">
            Quote this when contacting support
          </p>
        </div>

        {/* Support contact */}
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

        {/* Actions */}
        <div className="flex gap-3">
          <Button asChild className="flex-1">
            <Link to="/projects/list">
              <Home className="size-4 mr-2" />
              Go to Projects
            </Link>
          </Button>
          <Button variant="outline" className="flex-1" onClick={() => window.history.back()}>
            Go back
          </Button>
        </div>

      </div>
    </div>
  );
}
