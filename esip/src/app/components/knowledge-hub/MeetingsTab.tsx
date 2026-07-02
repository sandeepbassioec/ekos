import React, { useState } from 'react';
import { Video, Plus, Trash2, Calendar, Users, Lightbulb, Pencil } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import type { Meeting } from '../../types/knowledge-hub';

interface MeetingsTabProps {
  roadmapItemId: string;
  meetings: Meeting[];
}

function MeetingForm({
  initial,
  onSubmit,
  onCancel,
  submitLabel,
}: {
  initial?: Partial<Meeting>;
  onSubmit: (data: Omit<Meeting, 'id' | 'createdAt'>) => Promise<void>;
  onCancel: () => void;
  submitLabel: string;
}) {
  const [title, setTitle] = useState(initial?.title ?? '');
  const [date, setDate] = useState(initial?.date ? initial.date.split('T')[0] : new Date().toISOString().split('T')[0]);
  const [transcript, setTranscript] = useState(initial?.transcript ?? '');
  const [participants, setParticipants] = useState(initial?.participants?.join(', ') ?? '');
  const [keyDecisions, setKeyDecisions] = useState(initial?.keyDecisions?.join('\n') ?? '');
  const [notes, setNotes] = useState(initial?.notes ?? '');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!title || !transcript) return;
    setIsSubmitting(true);
    await onSubmit({
      roadmapItemId: initial?.roadmapItemId ?? '',
      title,
      date: new Date(date).toISOString(),
      transcript,
      participants: participants.split(',').map(p => p.trim()).filter(Boolean),
      keyDecisions: keyDecisions.split('\n').map(d => d.trim()).filter(Boolean),
      notes: notes || undefined,
    });
    setIsSubmitting(false);
  };

  return (
    <div className="space-y-4 mt-4">
      <div>
        <Label>Meeting Title *</Label>
        <Input value={title} onChange={e => setTitle(e.target.value)} placeholder="e.g., Architecture Review - Auth Module" />
      </div>
      <div>
        <Label>Meeting Date *</Label>
        <Input type="date" value={date} onChange={e => setDate(e.target.value)} />
      </div>
      <div>
        <Label>Participants (comma-separated)</Label>
        <Input value={participants} onChange={e => setParticipants(e.target.value)} placeholder="John Doe, Jane Smith, Bob Johnson" />
      </div>
      <div>
        <Label>Meeting Transcript / Notes *</Label>
        <Textarea value={transcript} onChange={e => setTranscript(e.target.value)} placeholder="Paste meeting transcript or detailed notes here..." rows={8} />
        <p className="text-xs text-muted-foreground mt-1">Paste the Teams meeting transcript or type detailed notes</p>
      </div>
      <div>
        <Label>Key Decisions (one per line)</Label>
        <Textarea value={keyDecisions} onChange={e => setKeyDecisions(e.target.value)} placeholder="OAuth 2.0 will be used for authentication&#10;Session timeout set to 30 minutes" rows={4} />
      </div>
      <div>
        <Label>Additional Notes</Label>
        <Textarea value={notes} onChange={e => setNotes(e.target.value)} placeholder="Any additional context or action items..." rows={3} />
      </div>
      <div className="flex gap-2 pt-2">
        <Button onClick={handleSubmit} disabled={!title || !transcript || isSubmitting} className="flex-1">
          {isSubmitting ? 'Saving...' : submitLabel}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={isSubmitting}>Cancel</Button>
      </div>
    </div>
  );
}

export function MeetingsTab({ roadmapItemId, meetings }: MeetingsTabProps) {
  const { createMeeting, updateMeeting, deleteMeeting } = useKnowledgeHubStore();
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [editingMeeting, setEditingMeeting] = useState<Meeting | null>(null);

  const handleCreate = async (data: Omit<Meeting, 'id' | 'createdAt'>) => {
    await createMeeting({ ...data, roadmapItemId });
    setIsAddOpen(false);
  };

  const handleUpdate = async (data: Omit<Meeting, 'id' | 'createdAt'>) => {
    if (!editingMeeting) return;
    await updateMeeting(editingMeeting.id, data);
    setEditingMeeting(null);
  };

  const handleDelete = async (meetingId: string) => {
    if (confirm('Delete this meeting?')) await deleteMeeting(meetingId);
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Teams Meetings & Recordings</h3>
        <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
          <DialogTrigger asChild>
            <Button><Plus className="w-4 h-4 mr-2" />Add Meeting</Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <DialogHeader><DialogTitle>Add Meeting</DialogTitle></DialogHeader>
            <MeetingForm onSubmit={handleCreate} onCancel={() => setIsAddOpen(false)} submitLabel="Add Meeting" />
          </DialogContent>
        </Dialog>
      </div>

      {/* Edit dialog */}
      <Dialog open={!!editingMeeting} onOpenChange={open => !open && setEditingMeeting(null)}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader><DialogTitle>Edit Meeting</DialogTitle></DialogHeader>
          {editingMeeting && (
            <MeetingForm
              initial={editingMeeting}
              onSubmit={handleUpdate}
              onCancel={() => setEditingMeeting(null)}
              submitLabel="Save Changes"
            />
          )}
        </DialogContent>
      </Dialog>

      {meetings.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="py-16 text-center">
            <Video className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Meetings Yet</h3>
            <p className="text-muted-foreground mb-4">Add Teams meeting transcripts and notes</p>
            <Button onClick={() => setIsAddOpen(true)}><Plus className="w-4 h-4 mr-2" />Add First Meeting</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {meetings.map(meeting => (
            <Card key={meeting.id} className="hover:shadow-md transition-shadow">
              <CardContent className="p-6">
                <div className="flex items-start justify-between gap-4 mb-4">
                  <div className="flex items-start gap-3 flex-1">
                    <Video className="w-5 h-5 text-primary mt-1 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <h4 className="font-semibold text-base mb-2">{meeting.title}</h4>
                      <div className="flex items-center gap-3 text-sm text-muted-foreground mb-3">
                        <div className="flex items-center gap-1">
                          <Calendar className="w-4 h-4" />
                          {new Date(meeting.date).toLocaleDateString()}
                        </div>
                        {meeting.participants.length > 0 && (
                          <div className="flex items-center gap-1">
                            <Users className="w-4 h-4" />
                            {meeting.participants.length} participants
                          </div>
                        )}
                      </div>

                      {meeting.participants.length > 0 && (
                        <div className="mb-3 flex flex-wrap gap-1">
                          {meeting.participants.map((p, i) => <Badge key={i} variant="secondary" className="text-xs">{p}</Badge>)}
                        </div>
                      )}

                      {meeting.keyDecisions.length > 0 && (
                        <div className="mb-3 bg-amber-50 border border-amber-200 rounded-lg p-3">
                          <div className="flex items-center gap-2 text-sm font-medium text-amber-900 mb-2">
                            <Lightbulb className="w-4 h-4" />Key Decisions:
                          </div>
                          <ul className="space-y-1 text-sm text-amber-800">
                            {meeting.keyDecisions.map((d, i) => (
                              <li key={i} className="flex items-start gap-2">
                                <span className="text-amber-600 mt-0.5">•</span><span>{d}</span>
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}

                      <div className="bg-muted rounded-lg p-3">
                        <div className="text-xs font-medium text-muted-foreground mb-2">Transcript / Notes:</div>
                        <div className="text-sm whitespace-pre-wrap line-clamp-4">{meeting.transcript}</div>
                      </div>

                      {meeting.notes && (
                        <div className="mt-3 text-sm text-muted-foreground italic">{meeting.notes}</div>
                      )}
                    </div>
                  </div>

                  <div className="flex gap-2 shrink-0">
                    <Button size="sm" variant="outline" onClick={() => setEditingMeeting(meeting)}>
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => handleDelete(meeting.id)}>
                      <Trash2 className="w-4 h-4 text-destructive" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
