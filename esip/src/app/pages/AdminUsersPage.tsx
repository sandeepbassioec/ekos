import { useEffect, useState } from 'react';
import { Users, UserPlus, Copy, Trash2, Ban, CheckCircle2, Shield } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Input } from '../components/ui/input';
import { db } from '../lib/db';
import type { UserProfile, Invite } from '../lib/db/types';
import { useAuthStore } from '../store/authStore';
import { PERSONAS } from '../store/personaStore';
import { LICENSE_TIERS, licenseLabel } from '../lib/entitlements';
import { toast } from 'sonner';

/**
 * Admin · Users & Access (Epic G / catalog Vol 13 + 17). Tenant admins manage seats here:
 * assign org role + license tier, enable/disable accounts, and invite new users (the invitee
 * activates the link to create their account). Gated to the `admin` role by the route.
 */

const ROLES = ['admin', 'manager', 'architect', 'developer', 'viewer'];

const newToken = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);

const selectCls = 'rounded-md border border-border bg-background px-2 py-1 text-sm';

export default function AdminUsersPage() {
  const me = useAuthStore((s) => s.user);
  const [users, setUsers] = useState<UserProfile[]>([]);
  const [invites, setInvites] = useState<Invite[]>([]);
  const [loading, setLoading] = useState(true);

  // Invite form
  const [email, setEmail] = useState('');
  const [role, setRole] = useState('manager');
  const [tier, setTier] = useState('contributor');
  const [personaId, setPersonaId] = useState('');
  const [inviting, setInviting] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [u, i] = await Promise.all([db.users.getAll(), db.invites.getAll()]);
      setUsers(u);
      setInvites(i);
    } catch (e) {
      console.error('Failed to load users/invites:', e);
      toast.error('Failed to load users');
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => {
    load();
  }, []);

  const patchUser = async (id: string, updates: Partial<UserProfile>) => {
    setUsers((prev) => prev.map((u) => (u.id === id ? { ...u, ...updates } : u)));
    try {
      await db.users.update(id, updates);
      toast.success('User updated');
    } catch {
      toast.error('Update failed');
      load();
    }
  };

  const createInvite = async () => {
    if (!email.trim()) return;
    setInviting(true);
    try {
      const created = await db.invites.create({
        email: email.trim(),
        role,
        licenseTier: tier as Invite['licenseTier'],
        defaultPersonaId: personaId || undefined,
        token: newToken(),
        status: 'pending',
        invitedByName: me?.fullName || me?.email,
      });
      setInvites((prev) => [created, ...prev]);
      setEmail('');
      setPersonaId('');
      toast.success('Invite created — share the activation link');
    } catch {
      toast.error('Failed to create invite');
    } finally {
      setInviting(false);
    }
  };

  const revokeInvite = async (id: string) => {
    setInvites((prev) => prev.map((i) => (i.id === id ? { ...i, status: 'revoked' } : i)));
    try {
      await db.invites.update(id, { status: 'revoked' });
    } catch {
      toast.error('Failed to revoke');
      load();
    }
  };

  const linkFor = (token: string) => `${window.location.origin}/activate/${token}`;
  const copyLink = (token: string) => {
    navigator.clipboard?.writeText(linkFor(token));
    toast.success('Activation link copied');
  };

  const pending = invites.filter((i) => i.status === 'pending');

  return (
    <div className="mx-auto max-w-5xl p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-semibold flex items-center gap-2">
          <Users className="w-6 h-6" /> Users &amp; Access
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Manage org roles, license seats, and account status — and invite new users to the workspace.
        </p>
      </div>

      {/* Invite */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2"><UserPlus className="w-4 h-4" /> Invite a user</CardTitle>
          <CardDescription>Creates an activation link; the invitee sets their password to join with the role &amp; license you pick.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">Email</span>
              <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="person@company.com" />
            </label>
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">Role</span>
              <select value={role} onChange={(e) => setRole(e.target.value)} className={`w-full ${selectCls} py-2`}>
                {ROLES.map((r) => <option key={r} value={r}>{r}</option>)}
              </select>
            </label>
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">License</span>
              <select value={tier} onChange={(e) => setTier(e.target.value)} className={`w-full ${selectCls} py-2`}>
                {LICENSE_TIERS.map((t) => <option key={t.value} value={t.value}>{t.label}{t.free ? ' (free)' : ''}</option>)}
              </select>
            </label>
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">Default persona</span>
              <select value={personaId} onChange={(e) => setPersonaId(e.target.value)} className={`w-full ${selectCls} py-2`}>
                <option value="">(none)</option>
                {PERSONAS.map((p) => <option key={p.id} value={p.id}>{p.label}</option>)}
              </select>
            </label>
          </div>
          <Button onClick={createInvite} disabled={inviting || !email.trim()}>
            <UserPlus className="w-4 h-4 mr-1" /> {inviting ? 'Creating…' : 'Create invite'}
          </Button>
        </CardContent>
      </Card>

      {/* Pending invites */}
      {pending.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Pending invites ({pending.length})</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {pending.map((i) => (
              <div key={i.id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-border p-3">
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">{i.email}</p>
                  <p className="text-xs text-muted-foreground">{i.role} · {licenseLabel(i.licenseTier)}{i.defaultPersonaId ? ` · ${PERSONAS.find((p) => p.id === i.defaultPersonaId)?.label ?? i.defaultPersonaId}` : ''}</p>
                </div>
                <div className="flex items-center gap-2">
                  <Button size="sm" variant="outline" onClick={() => copyLink(i.token)}><Copy className="w-4 h-4 mr-1" /> Copy link</Button>
                  <Button size="sm" variant="ghost" onClick={() => revokeInvite(i.id)}><Trash2 className="w-4 h-4" /></Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Members */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Members ({users.length})</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <p className="text-sm text-muted-foreground">Loading…</p>
          ) : users.length === 0 ? (
            <p className="text-sm text-muted-foreground">No users found.</p>
          ) : (
            <div className="space-y-2">
              {users.map((u) => {
                const disabled = u.status === 'disabled';
                return (
                  <div key={u.id} className={`flex flex-wrap items-center gap-3 rounded-lg border border-border p-3 ${disabled ? 'opacity-60' : ''}`}>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium truncate">
                        {u.fullName || u.email}
                        {u.id === me?.id && <span className="ml-1 text-xs text-muted-foreground">(you)</span>}
                        {u.role === 'admin' && <Shield className="inline w-3.5 h-3.5 ml-1 text-amber-500" />}
                      </p>
                      <p className="text-xs text-muted-foreground truncate">{u.email}</p>
                    </div>
                    <label className="text-xs text-muted-foreground">
                      role
                      <select value={u.role ?? 'manager'} onChange={(e) => patchUser(u.id, { role: e.target.value })} className={`ml-1 ${selectCls}`}>
                        {ROLES.map((r) => <option key={r} value={r}>{r}</option>)}
                      </select>
                    </label>
                    <label className="text-xs text-muted-foreground">
                      license
                      <select value={u.licenseTier ?? 'contributor'} onChange={(e) => patchUser(u.id, { licenseTier: e.target.value as UserProfile['licenseTier'] })} className={`ml-1 ${selectCls}`}>
                        {LICENSE_TIERS.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
                      </select>
                    </label>
                    <Badge variant={disabled ? 'outline' : 'secondary'}>{disabled ? 'disabled' : 'active'}</Badge>
                    <Button size="sm" variant="outline" onClick={() => patchUser(u.id, { status: disabled ? 'active' : 'disabled' })}>
                      {disabled ? <><CheckCircle2 className="w-4 h-4 mr-1" /> Enable</> : <><Ban className="w-4 h-4 mr-1" /> Disable</>}
                    </Button>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
