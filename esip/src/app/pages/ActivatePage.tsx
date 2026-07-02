import { useEffect, useState } from 'react';
import { useNavigate, useParams, Link } from 'react-router';
import { useAuthStore } from '../store/authStore';
import { usePersonaStore } from '../store/personaStore';
import { db } from '../lib/db';
import type { Invite } from '../lib/db/types';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { licenseLabel } from '../lib/entitlements';
import { PERSONAS } from '../store/personaStore';
import { toast } from 'sonner';

/**
 * Invite activation (Epic G). A public page reached via an admin-issued link (/activate/:token).
 * It loads the invite, then the invitee sets their name + password to create their account with
 * the role, license, and persona the admin assigned. On success they're signed in and landed home.
 */
export default function ActivatePage() {
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const register = useAuthStore((s) => s.register);
  const setActivePersona = usePersonaStore((s) => s.setActivePersona);

  const [invite, setInvite] = useState<Invite | null>(null);
  const [state, setState] = useState<'loading' | 'ready' | 'invalid'>('loading');
  const [fullName, setFullName] = useState('');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    (async () => {
      if (!token) {
        setState('invalid');
        return;
      }
      try {
        const found = await db.invites.getByToken(token);
        if (found && found.status === 'pending') {
          setInvite(found);
          setState('ready');
        } else {
          setState('invalid');
        }
      } catch {
        setState('invalid');
      }
    })();
  }, [token]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!invite) return;
    setSubmitting(true);
    try {
      await register(invite.email, password, fullName || invite.email, invite.role, {
        licenseTier: invite.licenseTier,
        defaultPersonaId: invite.defaultPersonaId,
        status: 'active',
        invitedBy: invite.invitedByName,
      });
      await db.invites.update(invite.id, { status: 'accepted' });
      if (invite.defaultPersonaId) setActivePersona(invite.defaultPersonaId);
      toast.success('Account activated — welcome!');
      navigate('/home');
    } catch (error: any) {
      toast.error(error?.message || 'Activation failed');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold">Activate your account</CardTitle>
          <CardDescription>
            {state === 'ready' ? 'Set a password to join the ESIP workspace.' : 'Invite activation'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {state === 'loading' && <p className="text-sm text-muted-foreground">Checking your invite…</p>}

          {state === 'invalid' && (
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">
                This invitation link is invalid, has expired, or has already been used.
              </p>
              <Button variant="outline" className="w-full" onClick={() => navigate('/login')}>Go to login</Button>
            </div>
          )}

          {state === 'ready' && invite && (
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="rounded-lg border border-border bg-muted/40 p-3 text-sm">
                <div className="font-medium">{invite.email}</div>
                <div className="mt-1 flex flex-wrap gap-1.5">
                  <Badge variant="secondary">{invite.role}</Badge>
                  <Badge variant="outline">{licenseLabel(invite.licenseTier)}</Badge>
                  {invite.defaultPersonaId && (
                    <Badge variant="outline">{PERSONAS.find((p) => p.id === invite.defaultPersonaId)?.label ?? invite.defaultPersonaId}</Badge>
                  )}
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="fullName">Full Name</Label>
                <Input id="fullName" value={fullName} onChange={(e) => setFullName(e.target.value)} placeholder="John Doe" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input id="password" type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Minimum 8 characters" required minLength={8} />
              </div>
              <Button type="submit" className="w-full" disabled={submitting}>
                {submitting ? 'Activating…' : 'Activate & sign in'}
              </Button>
            </form>
          )}

          <div className="mt-4 text-center text-sm">
            Already have an account?{' '}
            <Link to="/login" className="text-blue-600 hover:underline">Login</Link>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
