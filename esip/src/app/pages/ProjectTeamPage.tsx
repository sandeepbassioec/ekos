import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router';
import {
  ArrowLeft, Users, Plus, Trash2, Edit2, Check, X, Shield,
  Crown, Code2, TestTube, Wrench, UserCheck, Eye, Loader2,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent } from '../components/ui/card';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription,
} from '../components/ui/dialog';
import { useTeamStore } from '../store/teamStore';
import { useAuthStore } from '../store/authStore';
import { db } from '../lib/db';
import {
  PROJECT_ROLE_LABELS, PROJECT_ROLE_DESCRIPTIONS,
  type ProjectRole, type ProjectMember,
} from '../lib/db/types';

const ROLES: ProjectRole[] = [
  'delivery_manager', 'product_owner', 'tech_lead',
  'qa_lead', 'developer', 'scrum_master', 'stakeholder',
];

const ROLE_COLORS: Record<ProjectRole, string> = {
  delivery_manager: 'bg-purple-100 text-purple-800 border-purple-200',
  product_owner:    'bg-blue-100 text-blue-800 border-blue-200',
  tech_lead:        'bg-emerald-100 text-emerald-800 border-emerald-200',
  qa_lead:          'bg-orange-100 text-orange-800 border-orange-200',
  developer:        'bg-sky-100 text-sky-800 border-sky-200',
  scrum_master:     'bg-pink-100 text-pink-800 border-pink-200',
  stakeholder:      'bg-gray-100 text-gray-700 border-gray-200',
};

const ROLE_ICONS: Record<ProjectRole, React.ReactNode> = {
  delivery_manager: <Crown className="w-3.5 h-3.5" />,
  product_owner:    <Shield className="w-3.5 h-3.5" />,
  tech_lead:        <Code2 className="w-3.5 h-3.5" />,
  qa_lead:          <TestTube className="w-3.5 h-3.5" />,
  developer:        <Wrench className="w-3.5 h-3.5" />,
  scrum_master:     <UserCheck className="w-3.5 h-3.5" />,
  stakeholder:      <Eye className="w-3.5 h-3.5" />,
};

function RoleBadge({ role }: { role: ProjectRole }) {
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium border ${ROLE_COLORS[role]}`}>
      {ROLE_ICONS[role]}
      {PROJECT_ROLE_LABELS[role]}
    </span>
  );
}

/** Multi-role checkbox grid used in both the Add dialog and inline edit. */
function RoleCheckboxGrid({
  selected,
  onChange,
}: {
  selected: ProjectRole[];
  onChange: (roles: ProjectRole[]) => void;
}) {
  const toggle = (role: ProjectRole) => {
    onChange(selected.includes(role) ? selected.filter(r => r !== role) : [...selected, role]);
  };

  return (
    <div className="grid gap-2">
      {ROLES.map(role => (
        <label
          key={role}
          className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors select-none ${
            selected.includes(role)
              ? 'border-primary bg-primary/5'
              : 'border-muted hover:border-muted-foreground/30'
          }`}
        >
          <input
            type="checkbox"
            checked={selected.includes(role)}
            onChange={() => toggle(role)}
            className="mt-0.5 shrink-0"
          />
          <div className="min-w-0">
            <div className="mb-0.5"><RoleBadge role={role} /></div>
            <p className="text-xs text-muted-foreground leading-snug">{PROJECT_ROLE_DESCRIPTIONS[role]}</p>
          </div>
        </label>
      ))}
    </div>
  );
}

export default function ProjectTeamPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { user } = useAuthStore();

  const {
    members, allUsers, isLoading, isSaving,
    fetchTeam, fetchAllUsers, addMember, updateMemberRoles, removeMember, currentUserRoles,
  } = useTeamStore();

  const [projectName, setProjectName] = useState('');
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [selectedUserId, setSelectedUserId] = useState('');
  const [selectedRoles, setSelectedRoles] = useState<ProjectRole[]>(['developer']);

  const [editingMemberId, setEditingMemberId] = useState<string | null>(null);
  const [editingRoles, setEditingRoles] = useState<ProjectRole[]>([]);
  const [confirmRemoveId, setConfirmRemoveId] = useState<string | null>(null);

  const myRoles = currentUserRoles(user?.id ?? '');
  const canManage = myRoles.includes('delivery_manager') || user?.role === 'admin' || user?.role === 'manager';

  useEffect(() => {
    if (!projectId) return;
    fetchTeam(projectId);
    fetchAllUsers();
    db.projects.getById(projectId, user?.id ?? '').then(p => setProjectName(p.name)).catch(() => {});
  }, [projectId]);

  const memberUserIds = new Set(members.map(m => m.userId));
  const availableUsers = allUsers.filter(u => !memberUserIds.has(u.id));

  const handleAdd = async () => {
    if (!selectedUserId || !projectId || !user || selectedRoles.length === 0) return;
    await addMember(projectId, selectedUserId, selectedRoles, user.id);
    setShowAddDialog(false);
    setSelectedUserId('');
    setSelectedRoles(['developer']);
  };

  const startEdit = (member: ProjectMember) => {
    setEditingMemberId(member.id);
    setEditingRoles([...member.projectRoles]);
  };

  const handleSaveRoles = async (memberId: string) => {
    if (editingRoles.length === 0) { return; }
    await updateMemberRoles(memberId, editingRoles);
    setEditingMemberId(null);
  };

  const handleRemove = async (memberId: string) => {
    await removeMember(memberId);
    setConfirmRemoveId(null);
  };

  return (
    <div className="max-w-4xl mx-auto px-6 py-8 space-y-6">
      {/* Header */}
      <div>
        <Button variant="ghost" onClick={() => navigate(`/projects/${projectId}`)} className="mb-3 -ml-2">
          <ArrowLeft className="w-4 h-4 mr-2" />Back to Project
        </Button>
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold flex items-center gap-2">
              <Users className="w-6 h-6 text-primary" />Project Team
            </h1>
            {projectName && <p className="text-muted-foreground mt-0.5 text-sm">{projectName}</p>}
          </div>
          {canManage && (
            <Button onClick={() => setShowAddDialog(true)} disabled={availableUsers.length === 0}>
              <Plus className="w-4 h-4 mr-2" />Add Member
            </Button>
          )}
        </div>
      </div>

      {/* Role legend */}
      <Card className="border bg-muted/30">
        <CardContent className="p-4">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-3">Role Capabilities</p>
          <p className="text-xs text-muted-foreground mb-3">
            One person can hold multiple roles — common in small teams where one person wears multiple hats.
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {ROLES.map(role => (
              <div key={role} className="flex items-start gap-2">
                <RoleBadge role={role} />
                <span className="text-xs text-muted-foreground leading-snug">{PROJECT_ROLE_DESCRIPTIONS[role]}</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Team list */}
      {isLoading ? (
        <div className="flex items-center justify-center py-16 text-muted-foreground gap-2">
          <Loader2 className="w-5 h-5 animate-spin" />Loading team...
        </div>
      ) : members.length === 0 ? (
        <Card className="border-2 border-dashed border-muted">
          <CardContent className="py-16 text-center">
            <Users className="w-12 h-12 mx-auto mb-3 text-muted-foreground/40" />
            <p className="text-lg font-semibold mb-1">No team members yet</p>
            <p className="text-sm text-muted-foreground mb-4">
              {canManage
                ? 'Add team members and assign their project roles to get started.'
                : 'The project manager has not added any team members yet.'}
            </p>
            {canManage && (
              <Button onClick={() => setShowAddDialog(true)} disabled={availableUsers.length === 0}>
                <Plus className="w-4 h-4 mr-2" />Add First Member
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {members.map(member => {
            const isMe = member.userId === user?.id;
            const isEditing = editingMemberId === member.id;
            const isConfirmRemove = confirmRemoveId === member.id;

            return (
              <Card key={member.id} className={`transition-shadow hover:shadow-sm ${isMe ? 'ring-1 ring-primary/30' : ''}`}>
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    {/* Avatar */}
                    <div className="w-9 h-9 rounded-full bg-primary/10 flex items-center justify-center shrink-0 text-sm font-semibold text-primary mt-0.5">
                      {(member.userFullName || member.userEmail).charAt(0).toUpperCase()}
                    </div>

                    {/* Name + roles */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap mb-1">
                        <span className="font-medium text-sm">{member.userFullName || member.userEmail}</span>
                        {isMe && (
                          <span className="text-xs bg-primary/10 text-primary px-1.5 py-0.5 rounded-full font-medium">You</span>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground mb-2">{member.userEmail}</p>

                      {/* Role badges or inline editor */}
                      {isEditing ? (
                        <div className="mt-2">
                          <p className="text-xs font-medium text-muted-foreground mb-2">Select roles (one or more):</p>
                          <RoleCheckboxGrid selected={editingRoles} onChange={setEditingRoles} />
                          <div className="flex gap-2 mt-3">
                            <Button
                              size="sm" className="h-7 text-xs px-3"
                              onClick={() => handleSaveRoles(member.id)}
                              disabled={isSaving || editingRoles.length === 0}
                            >
                              <Check className="w-3 h-3 mr-1" />Save
                            </Button>
                            <Button size="sm" variant="ghost" className="h-7 text-xs px-3" onClick={() => setEditingMemberId(null)}>
                              <X className="w-3 h-3 mr-1" />Cancel
                            </Button>
                            {editingRoles.length === 0 && (
                              <span className="text-xs text-red-500 self-center">Select at least one role</span>
                            )}
                          </div>
                        </div>
                      ) : (
                        <div className="flex flex-wrap gap-1.5">
                          {member.projectRoles.map(role => <RoleBadge key={role} role={role} />)}
                        </div>
                      )}
                    </div>

                    {/* Actions */}
                    {canManage && !isEditing && (
                      <div className="flex items-center gap-1 shrink-0">
                        {isConfirmRemove ? (
                          <>
                            <span className="text-xs text-destructive font-medium mr-1">Remove?</span>
                            <Button size="sm" variant="destructive" className="h-7 px-2 text-xs" onClick={() => handleRemove(member.id)} disabled={isSaving}>Yes</Button>
                            <Button size="sm" variant="ghost" className="h-7 px-2 text-xs" onClick={() => setConfirmRemoveId(null)}>No</Button>
                          </>
                        ) : (
                          <>
                            <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => startEdit(member)} title="Edit roles">
                              <Edit2 className="w-3.5 h-3.5" />
                            </Button>
                            {!isMe && (
                              <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => setConfirmRemoveId(member.id)} title="Remove">
                                <Trash2 className="w-3.5 h-3.5" />
                              </Button>
                            )}
                          </>
                        )}
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Stats footer */}
      {members.length > 0 && (
        <div className="flex items-center gap-3 flex-wrap pt-1">
          <span className="text-sm text-muted-foreground">{members.length} member{members.length !== 1 ? 's' : ''}</span>
          {ROLES.filter(r => members.some(m => m.projectRoles.includes(r))).map(role => (
            <span key={role} className="text-xs text-muted-foreground">
              {members.filter(m => m.projectRoles.includes(role)).length}× {PROJECT_ROLE_LABELS[role]}
            </span>
          ))}
        </div>
      )}

      {/* Add member dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Add Team Member</DialogTitle>
            <DialogDescription>
              Select a person and assign one or more project roles. Small teams can have one person cover multiple roles.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 pt-2">
            <div>
              <label className="text-sm font-medium block mb-1.5">Team Member</label>
              {availableUsers.length === 0 ? (
                <p className="text-sm text-muted-foreground">All organisation users are already on this team.</p>
              ) : (
                <select
                  value={selectedUserId}
                  onChange={e => setSelectedUserId(e.target.value)}
                  className="w-full border rounded-md px-3 py-2 text-sm bg-background"
                >
                  <option value="">— Select a person —</option>
                  {availableUsers.map(u => (
                    <option key={u.id} value={u.id}>
                      {u.fullName ? `${u.fullName} (${u.email})` : u.email}
                    </option>
                  ))}
                </select>
              )}
            </div>

            <div>
              <label className="text-sm font-medium block mb-1.5">
                Project Roles <span className="text-muted-foreground font-normal">(select one or more)</span>
              </label>
              <RoleCheckboxGrid selected={selectedRoles} onChange={setSelectedRoles} />
              {selectedRoles.length === 0 && (
                <p className="text-xs text-red-500 mt-1">Select at least one role</p>
              )}
            </div>

            <div className="flex gap-2 pt-1">
              <Button variant="outline" className="flex-1" onClick={() => setShowAddDialog(false)}>Cancel</Button>
              <Button
                className="flex-1"
                onClick={handleAdd}
                disabled={!selectedUserId || selectedRoles.length === 0 || isSaving}
              >
                {isSaving ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <Plus className="w-4 h-4 mr-2" />}
                Add to Team
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
