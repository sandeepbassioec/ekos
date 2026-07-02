import { useState, useEffect } from 'react';
import { ApiClient } from '../lib/api';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '../components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select';
import { toast } from 'sonner';
import { Users, UserPlus, Mail } from 'lucide-react';

export default function TeamManagement() {
  const [teams, setTeams] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [newTeamName, setNewTeamName] = useState('');
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState('member');
  const [selectedTeam, setSelectedTeam] = useState<string | null>(null);

  useEffect(() => {
    fetchTeams();
  }, []);

  const fetchTeams = async () => {
    try {
      const data = await ApiClient.getTeams();
      setTeams(data);
    } catch (error: any) {
      toast.error(error.message);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateTeam = async () => {
    if (!newTeamName.trim()) return;
    try {
      await ApiClient.createTeam(newTeamName);
      toast.success('Team created');
      setNewTeamName('');
      fetchTeams();
    } catch (error: any) {
      toast.error(error.message);
    }
  };

  const handleInvite = async () => {
    if (!inviteEmail.trim() || !selectedTeam) return;
    try {
      const result = await ApiClient.inviteToTeam(selectedTeam, inviteEmail, inviteRole);
      toast.success(`Invitation sent to ${inviteEmail}`);
      toast.info(`Share link: ${result.inviteLink}`);
      setInviteEmail('');
    } catch (error: any) {
      toast.error(error.message);
    }
  };

  if (loading) return <div className="p-8">Loading...</div>;

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <Users className="w-8 h-8 text-blue-600" />
            <h1 className="text-3xl font-bold">Team Management</h1>
          </div>
          <Dialog>
            <DialogTrigger asChild>
              <Button>
                <UserPlus className="w-4 h-4 mr-2" />
                Create Team
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Create New Team</DialogTitle>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <Label>Team Name</Label>
                  <Input
                    value={newTeamName}
                    onChange={(e) => setNewTeamName(e.target.value)}
                    placeholder="Engineering Team"
                  />
                </div>
                <Button onClick={handleCreateTeam} className="w-full">
                  Create
                </Button>
              </div>
            </DialogContent>
          </Dialog>
        </div>

        <div className="grid gap-6">
          {teams.map((team) => (
            <Card key={team.id}>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  <span>{team.name}</span>
                  <span className="text-sm font-normal text-gray-500">
                    {team.members?.length || 0} members • {team._count?.projects || 0} projects
                  </span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div>
                    <h4 className="font-semibold mb-2">Members</h4>
                    <div className="space-y-2">
                      {team.members?.map((member: any) => (
                        <div key={member.userId} className="flex items-center justify-between bg-gray-50 p-2 rounded">
                          <div>
                            <div className="font-medium">{member.user.fullName || member.user.email}</div>
                            <div className="text-sm text-gray-500">{member.user.role}</div>
                          </div>
                          <span className="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">
                            {member.role}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="border-t pt-4">
                    <h4 className="font-semibold mb-2">Invite Member</h4>
                    <div className="flex gap-2">
                      <Input
                        placeholder="email@example.com"
                        value={selectedTeam === team.id ? inviteEmail : ''}
                        onChange={(e) => {
                          setSelectedTeam(team.id);
                          setInviteEmail(e.target.value);
                        }}
                      />
                      <Select
                        value={inviteRole}
                        onValueChange={setInviteRole}
                      >
                        <SelectTrigger className="w-32">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="manager">Manager</SelectItem>
                          <SelectItem value="member">Member</SelectItem>
                        </SelectContent>
                      </Select>
                      <Button
                        onClick={() => {
                          setSelectedTeam(team.id);
                          handleInvite();
                        }}
                        disabled={!inviteEmail.trim()}
                      >
                        <Mail className="w-4 h-4 mr-2" />
                        Invite
                      </Button>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}

          {teams.length === 0 && (
            <div className="text-center py-12 text-gray-500">
              No teams yet. Create your first team to get started.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
