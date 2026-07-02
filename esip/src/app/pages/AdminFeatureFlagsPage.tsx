import { SlidersHorizontal, RotateCcw } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { FEATURE_FLAGS, FLAG_DEFAULTS } from '../lib/featureFlags';
import { useFeatureFlagStore } from '../store/featureFlagStore';
import { toast } from 'sonner';

/**
 * Admin · Capabilities (Epic H). Toggle MVP capabilities on/off for this workspace. Flags default
 * ON; turning one OFF reverts that capability (its nav, routes, and entry points all respect it).
 */
export default function AdminFeatureFlagsPage() {
  const overrides = useFeatureFlagStore((s) => s.overrides);
  const setFlag = useFeatureFlagStore((s) => s.setFlag);
  const resetAll = useFeatureFlagStore((s) => s.resetAll);

  return (
    <div className="mx-auto max-w-3xl p-6 space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <SlidersHorizontal className="w-6 h-6" /> Capabilities
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Turn workspace capabilities on or off. Defaults are on; disabling one reverts that feature everywhere.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => { resetAll(); toast.success('Reset to defaults'); }}>
          <RotateCcw className="w-4 h-4 mr-1" /> Reset all
        </Button>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Feature flags</CardTitle>
          <CardDescription>{FEATURE_FLAGS.length} capabilities · changes apply immediately.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {FEATURE_FLAGS.map((f) => {
            const value = overrides[f.key] ?? FLAG_DEFAULTS[f.key];
            const overridden = overrides[f.key] !== undefined && overrides[f.key] !== FLAG_DEFAULTS[f.key];
            return (
              <div key={f.key} className="flex items-center justify-between gap-3 rounded-lg border border-border p-3">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{f.label}</span>
                    <Badge variant="outline" className="text-[10px]">Epic {f.epic}</Badge>
                    {overridden && <Badge variant="secondary" className="text-[10px]">overridden</Badge>}
                  </div>
                  <p className="text-xs text-muted-foreground">{f.description}</p>
                </div>
                <Button
                  size="sm"
                  variant={value ? 'default' : 'outline'}
                  className="w-16 shrink-0"
                  onClick={() => setFlag(f.key, !value)}
                >
                  {value ? 'On' : 'Off'}
                </Button>
              </div>
            );
          })}
        </CardContent>
      </Card>
    </div>
  );
}
