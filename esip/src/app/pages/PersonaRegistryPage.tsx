import { PERSONAS } from '../store/personaStore';
import { Badge } from '../components/ui/badge';

/**
 * Read-only persona registry (Epic B / P1-B4). Lets admins SEE the persona config that
 * drives the workspace. Full editing (admin → project → user override, metamodel-driven)
 * lands in a later increment per catalog Vol 05 + Vol 11.
 */
export default function PersonaRegistryPage() {
  return (
    <div className="mx-auto max-w-5xl p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold">Personas</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          The architecture roles the suite supports. Each persona reshapes the workspace
          (foregrounded capabilities + default working mode) and — in later releases — its
          artifacts, diagrams, deliverable templates and AI expert prompt-packs. Read-only for now;
          configuration (admin → project → user override) is metamodel-driven and lands later.
        </p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        {PERSONAS.map((p) => (
          <div key={p.id} className="rounded-lg border border-border bg-card p-4">
            <div className="mb-1 flex items-center gap-2">
              <span className="inline-flex w-9 justify-center rounded bg-muted px-1 text-xs text-muted-foreground">{p.code}</span>
              <h2 className="font-medium">{p.label}</h2>
            </div>
            <p className="mb-3 text-sm text-muted-foreground">{p.description}</p>
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="text-xs text-muted-foreground">Workspace:</span>
              {p.enabledNav.map((n) => (
                <Badge key={n} variant="secondary">{n}</Badge>
              ))}
              <span className="ml-2 text-xs text-muted-foreground">Default mode:</span>
              <Badge variant="outline">{p.defaultWorkingMode}</Badge>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
