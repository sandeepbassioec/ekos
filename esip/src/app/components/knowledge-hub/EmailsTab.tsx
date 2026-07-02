import React, { useState } from 'react';
import { Mail, Plus, Trash2, Pencil } from 'lucide-react';
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
import type { Email } from '../../types/knowledge-hub';

interface EmailsTabProps {
  roadmapItemId: string;
  emails: Email[];
}

function EmailForm({
  initial,
  onSubmit,
  onCancel,
  submitLabel,
}: {
  initial?: Partial<Email>;
  onSubmit: (data: Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>) => Promise<void>;
  onCancel: () => void;
  submitLabel: string;
}) {
  const [subject, setSubject] = useState(initial?.subject ?? '');
  const [from, setFrom] = useState(initial?.from ?? '');
  const [to, setTo] = useState(initial?.to?.join(', ') ?? '');
  const [cc, setCc] = useState(initial?.cc?.join(', ') ?? '');
  const [date, setDate] = useState(initial?.date ? initial.date.split('T')[0] : new Date().toISOString().split('T')[0]);
  const [body, setBody] = useState(initial?.body ?? '');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!subject || !from || !to || !body) return;
    setIsSubmitting(true);
    await onSubmit({
      subject,
      from: from.trim(),
      to: to.split(',').map(e => e.trim()).filter(Boolean),
      cc: cc ? cc.split(',').map(e => e.trim()).filter(Boolean) : undefined,
      date: new Date(date).toISOString(),
      body,
    });
    setIsSubmitting(false);
  };

  return (
    <div className="space-y-4 mt-4">
      <div>
        <Label>Subject *</Label>
        <Input value={subject} onChange={e => setSubject(e.target.value)} placeholder="Email subject line" />
      </div>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label>From *</Label>
          <Input value={from} onChange={e => setFrom(e.target.value)} placeholder="sender@example.com" />
        </div>
        <div>
          <Label>Date *</Label>
          <Input type="date" value={date} onChange={e => setDate(e.target.value)} />
        </div>
      </div>
      <div>
        <Label>To * (comma-separated)</Label>
        <Input value={to} onChange={e => setTo(e.target.value)} placeholder="recipient1@example.com, recipient2@example.com" />
      </div>
      <div>
        <Label>CC (optional, comma-separated)</Label>
        <Input value={cc} onChange={e => setCc(e.target.value)} placeholder="cc1@example.com" />
      </div>
      <div>
        <Label>Email Body *</Label>
        <Textarea value={body} onChange={e => setBody(e.target.value)} placeholder="Paste email content here..." rows={10} />
      </div>
      <div className="flex gap-2 pt-2">
        <Button onClick={handleSubmit} disabled={!subject || !from || !to || !body || isSubmitting} className="flex-1">
          {isSubmitting ? 'Saving...' : submitLabel}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={isSubmitting}>Cancel</Button>
      </div>
    </div>
  );
}

export function EmailsTab({ roadmapItemId, emails }: EmailsTabProps) {
  const { createEmail, updateEmail, deleteEmail } = useKnowledgeHubStore();
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [editingEmail, setEditingEmail] = useState<Email | null>(null);

  const handleCreate = async (data: Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>) => {
    await createEmail({ ...data, roadmapItemId });
    setIsAddOpen(false);
  };

  const handleUpdate = async (data: Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>) => {
    if (!editingEmail) return;
    await updateEmail(editingEmail.id, data);
    setEditingEmail(null);
  };

  const handleDelete = async (emailId: string) => {
    if (confirm('Delete this email?')) await deleteEmail(emailId);
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Email Threads</h3>
        <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
          <DialogTrigger asChild>
            <Button><Plus className="w-4 h-4 mr-2" />Add Email</Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <DialogHeader><DialogTitle>Add Email</DialogTitle></DialogHeader>
            <EmailForm onSubmit={handleCreate} onCancel={() => setIsAddOpen(false)} submitLabel="Add Email" />
          </DialogContent>
        </Dialog>
      </div>

      {/* Edit dialog */}
      <Dialog open={!!editingEmail} onOpenChange={open => !open && setEditingEmail(null)}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader><DialogTitle>Edit Email</DialogTitle></DialogHeader>
          {editingEmail && (
            <EmailForm
              initial={editingEmail}
              onSubmit={handleUpdate}
              onCancel={() => setEditingEmail(null)}
              submitLabel="Save Changes"
            />
          )}
        </DialogContent>
      </Dialog>

      {emails.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="py-16 text-center">
            <Mail className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Emails Yet</h3>
            <p className="text-muted-foreground mb-4">Capture important email threads and discussions</p>
            <Button onClick={() => setIsAddOpen(true)}><Plus className="w-4 h-4 mr-2" />Add First Email</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {emails.map(email => (
            <Card key={email.id} className="hover:shadow-md transition-shadow">
              <CardContent className="p-6">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex items-start gap-3 flex-1">
                    <Mail className="w-5 h-5 text-blue-500 mt-1 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <h4 className="font-semibold text-base mb-2">{email.subject}</h4>
                      <div className="space-y-1 text-sm text-muted-foreground mb-3">
                        <div><strong>From:</strong> {email.from}</div>
                        <div className="flex items-center gap-1 flex-wrap">
                          <strong>To:</strong>
                          {email.to.map((e, i) => <Badge key={i} variant="secondary" className="ml-1 text-xs">{e}</Badge>)}
                        </div>
                        {email.cc && email.cc.length > 0 && (
                          <div className="flex items-center gap-1 flex-wrap">
                            <strong>CC:</strong>
                            {email.cc.map((e, i) => <Badge key={i} variant="outline" className="ml-1 text-xs">{e}</Badge>)}
                          </div>
                        )}
                        <div><strong>Date:</strong> {new Date(email.date).toLocaleString()}</div>
                      </div>
                      <div className="bg-muted rounded-lg p-3">
                        <div className="text-sm whitespace-pre-wrap line-clamp-6">{email.body}</div>
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2 shrink-0">
                    <Button size="sm" variant="outline" onClick={() => setEditingEmail(email)}>
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => handleDelete(email.id)}>
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
