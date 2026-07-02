import React, { useState, useEffect } from 'react';
import { User } from 'lucide-react';
import { Label } from '../ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { supabase } from '../../lib/supabase';

interface TeamMember {
  id: string;
  email: string;
  name?: string;
}

interface RoadmapOwnerSelectorProps {
  projectId: string;
  value?: string;
  onChange: (userId: string) => void;
  label?: string;
}

export function RoadmapOwnerSelector({ 
  projectId, 
  value, 
  onChange,
  label = "Roadmap Owner"
}: RoadmapOwnerSelectorProps) {
  const [teamMembers, setTeamMembers] = useState<TeamMember[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  
  useEffect(() => {
    const fetchTeamMembers = async () => {
      // Fetch project team members
      const { data: projectData } = await supabase
        .from('Project')
        .select('teamMembers')
        .eq('id', projectId)
        .single();
      
      if (!projectData?.teamMembers || projectData.teamMembers.length === 0) {
        setIsLoading(false);
        return;
      }
      
      // Fetch user details for team members
      const { data: usersData } = await supabase.auth.admin.listUsers();
      
      if (usersData?.users) {
        const members = projectData.teamMembers
          .map((memberId: string) => {
            const user = usersData.users.find(u => u.id === memberId);
            return user ? {
              id: user.id,
              email: user.email || '',
              name: user.user_metadata?.name || user.email?.split('@')[0] || 'Unknown',
            } : null;
          })
          .filter(Boolean) as TeamMember[];
        
        setTeamMembers(members);
      }
      
      setIsLoading(false);
    };
    
    fetchTeamMembers();
  }, [projectId]);
  
  if (isLoading) {
    return (
      <div>
        <Label>{label}</Label>
        <div className="h-10 bg-muted animate-pulse rounded-md" />
      </div>
    );
  }
  
  if (teamMembers.length === 0) {
    return (
      <div>
        <Label>{label}</Label>
        <div className="text-sm text-muted-foreground py-2">
          No team members assigned to this project
        </div>
      </div>
    );
  }
  
  return (
    <div>
      <Label htmlFor="owner">{label}</Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger id="owner">
          <SelectValue placeholder="Select owner">
            {value && (
              <div className="flex items-center gap-2">
                <User className="w-4 h-4" />
                {teamMembers.find(m => m.id === value)?.name}
              </div>
            )}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {teamMembers.map((member) => (
            <SelectItem key={member.id} value={member.id}>
              <div className="flex items-center gap-2">
                <User className="w-4 h-4" />
                <div>
                  <div className="font-medium">{member.name}</div>
                  <div className="text-xs text-muted-foreground">{member.email}</div>
                </div>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <p className="text-xs text-muted-foreground mt-1">
        Person responsible for delivering this roadmap
      </p>
    </div>
  );
}
