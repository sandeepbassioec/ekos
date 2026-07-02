import React, { useState } from 'react';
import { MessageSquare, Plus, Trash2, Pencil } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '../ui/select';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import type { Chat, ChatPlatform } from '../../types/knowledge-hub';

interface ChatsTabProps {
  roadmapItemId: string;
  chats: Chat[];
}

function ChatForm({
  initial,
  onSubmit,
  onCancel,
  submitLabel,
}: {
  initial?: Partial<Chat>;
  onSubmit: (data: Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>) => Promise<void>;
  onCancel: () => void;
  submitLabel: string;
}) {
  const [platform, setPlatform] = useState<ChatPlatform>(initial?.platform ?? 'Teams');
  const [title, setTitle] = useState(initial?.title ?? '');
  const [channelName, setChannelName] = useState(initial?.channelName ?? '');
  const [date, setDate] = useState(initial?.date ? initial.date.split('T')[0] : new Date().toISOString().split('T')[0]);
  const [participants, setParticipants] = useState(initial?.participants?.join(', ') ?? '');
  const [messages, setMessages] = useState(initial?.messages ?? '');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!messages) return;
    setIsSubmitting(true);
    await onSubmit({
      platform,
      title: title || undefined,
      channelName: channelName || undefined,
      date: new Date(date).toISOString(),
      participants: participants.split(',').map(p => p.trim()).filter(Boolean),
      messages,
    });
    setIsSubmitting(false);
  };

  return (
    <div className="space-y-4 mt-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label>Platform *</Label>
          <Select value={platform} onValueChange={v => setPlatform(v as ChatPlatform)}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="Teams">Teams</SelectItem>
              <SelectItem value="Slack">Slack</SelectItem>
              <SelectItem value="Other">Other</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div>
          <Label>Date *</Label>
          <Input type="date" value={date} onChange={e => setDate(e.target.value)} />
        </div>
      </div>
      <div>
        <Label>Title (optional)</Label>
        <Input value={title} onChange={e => setTitle(e.target.value)} placeholder="e.g., Discussion about API design" />
      </div>
      <div>
        <Label>Channel Name (optional)</Label>
        <Input value={channelName} onChange={e => setChannelName(e.target.value)} placeholder="e.g., #backend-team" />
      </div>
      <div>
        <Label>Participants (comma-separated)</Label>
        <Input value={participants} onChange={e => setParticipants(e.target.value)} placeholder="John Doe, Jane Smith" />
      </div>
      <div>
        <Label>Chat Messages *</Label>
        <Textarea value={messages} onChange={e => setMessages(e.target.value)} placeholder="Paste chat conversation here..." rows={12} />
        <p className="text-xs text-muted-foreground mt-1">Paste the entire conversation from Teams/Slack</p>
      </div>
      <div className="flex gap-2 pt-2">
        <Button onClick={handleSubmit} disabled={!messages || isSubmitting} className="flex-1">
          {isSubmitting ? 'Saving...' : submitLabel}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={isSubmitting}>Cancel</Button>
      </div>
    </div>
  );
}

export function ChatsTab({ roadmapItemId, chats }: ChatsTabProps) {
  const { createChat, updateChat, deleteChat } = useKnowledgeHubStore();
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [editingChat, setEditingChat] = useState<Chat | null>(null);

  const handleCreate = async (data: Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>) => {
    await createChat({ ...data, roadmapItemId });
    setIsAddOpen(false);
  };

  const handleUpdate = async (data: Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>) => {
    if (!editingChat) return;
    await updateChat(editingChat.id, data);
    setEditingChat(null);
  };

  const handleDelete = async (chatId: string) => {
    if (confirm('Delete this chat?')) await deleteChat(chatId);
  };

  const getPlatformColor = (p: ChatPlatform) => {
    const map: Record<ChatPlatform, string> = {
      Teams: 'bg-purple-100 text-purple-800',
      Slack: 'bg-green-100 text-green-800',
      Other: 'bg-gray-100 text-gray-800',
    };
    return map[p];
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Chat Conversations</h3>
        <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
          <DialogTrigger asChild>
            <Button><Plus className="w-4 h-4 mr-2" />Add Chat</Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <DialogHeader><DialogTitle>Add Chat Conversation</DialogTitle></DialogHeader>
            <ChatForm onSubmit={handleCreate} onCancel={() => setIsAddOpen(false)} submitLabel="Add Chat" />
          </DialogContent>
        </Dialog>
      </div>

      {/* Edit dialog */}
      <Dialog open={!!editingChat} onOpenChange={open => !open && setEditingChat(null)}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader><DialogTitle>Edit Chat</DialogTitle></DialogHeader>
          {editingChat && (
            <ChatForm
              initial={editingChat}
              onSubmit={handleUpdate}
              onCancel={() => setEditingChat(null)}
              submitLabel="Save Changes"
            />
          )}
        </DialogContent>
      </Dialog>

      {chats.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="py-16 text-center">
            <MessageSquare className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Chats Yet</h3>
            <p className="text-muted-foreground mb-4">Capture important Teams or Slack conversations</p>
            <Button onClick={() => setIsAddOpen(true)}><Plus className="w-4 h-4 mr-2" />Add First Chat</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {chats.map(chat => (
            <Card key={chat.id} className="hover:shadow-md transition-shadow">
              <CardContent className="p-6">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex items-start gap-3 flex-1">
                    <MessageSquare className="w-5 h-5 text-primary mt-1 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <Badge className={getPlatformColor(chat.platform)}>{chat.platform}</Badge>
                        {chat.channelName && <Badge variant="outline">{chat.channelName}</Badge>}
                      </div>
                      <h4 className="font-semibold text-base mb-2">{chat.title || 'Chat Conversation'}</h4>
                      <div className="text-sm text-muted-foreground mb-3">
                        <div><strong>Date:</strong> {new Date(chat.date).toLocaleDateString()}</div>
                        {chat.participants.length > 0 && (
                          <div className="mt-1 flex items-center gap-1 flex-wrap">
                            <strong>Participants:</strong>
                            {chat.participants.map((p, i) => <Badge key={i} variant="secondary" className="ml-1 text-xs">{p}</Badge>)}
                          </div>
                        )}
                      </div>
                      <div className="bg-muted rounded-lg p-3">
                        <div className="text-sm whitespace-pre-wrap line-clamp-8 font-mono text-xs">{chat.messages}</div>
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2 shrink-0">
                    <Button size="sm" variant="outline" onClick={() => setEditingChat(chat)}>
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => handleDelete(chat.id)}>
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
