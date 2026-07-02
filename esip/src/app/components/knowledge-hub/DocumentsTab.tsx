import React, { useState } from 'react';
import { FileText, Upload, Download, Trash2, File, Pencil } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '../ui/select';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import type { Document, DocumentType } from '../../types/knowledge-hub';

interface DocumentsTabProps {
  roadmapItemId: string;
  documents: Document[];
}

export function DocumentsTab({ roadmapItemId, documents }: DocumentsTabProps) {
  const { uploadDocument, updateDocument, deleteDocument, getDocumentDownloadUrl } = useKnowledgeHubStore();

  const [isUploadOpen, setIsUploadOpen] = useState(false);
  const [editingDoc, setEditingDoc] = useState<Document | null>(null);

  // Upload form state
  const [file, setFile] = useState<File | null>(null);
  const [name, setName] = useState('');
  const [type, setType] = useState<DocumentType>('BRD');
  const [version, setVersion] = useState('');
  const [isUploading, setIsUploading] = useState(false);

  // Edit form state
  const [editName, setEditName] = useState('');
  const [editType, setEditType] = useState<DocumentType>('BRD');
  const [editVersion, setEditVersion] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  const openEdit = (doc: Document) => {
    setEditName(doc.name);
    setEditType(doc.type);
    setEditVersion(doc.version || '');
    setEditingDoc(doc);
  };

  const handleSaveEdit = async () => {
    if (!editingDoc || !editName) return;
    setIsSaving(true);
    await updateDocument(editingDoc.id, { name: editName, type: editType, version: editVersion || undefined });
    setIsSaving(false);
    setEditingDoc(null);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (f) {
      setFile(f);
      if (!name) setName(f.name.replace(/\.[^/.]+$/, ''));
    }
  };

  const handleUpload = async () => {
    if (!file) return;
    setIsUploading(true);
    const success = await uploadDocument({ roadmapItemId, name: name || file.name, type, version: version || undefined, file });
    if (success) { setIsUploadOpen(false); setFile(null); setName(''); setType('BRD'); setVersion(''); }
    setIsUploading(false);
  };

  const handleDownload = async (doc: Document) => {
    const url = await getDocumentDownloadUrl(doc.id);
    if (url) window.open(url, '_blank');
  };

  const handleDelete = async (documentId: string) => {
    if (confirm('Delete this document?')) await deleteDocument(documentId);
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return 'Unknown';
    const mb = bytes / (1024 * 1024);
    return mb < 1 ? `${(bytes / 1024).toFixed(1)} KB` : `${mb.toFixed(1)} MB`;
  };

  const getFileIcon = (mimeType?: string) => {
    if (!mimeType) return <File className="w-5 h-5" />;
    if (mimeType.includes('pdf')) return <FileText className="w-5 h-5 text-red-500" />;
    if (mimeType.includes('word') || mimeType.includes('document')) return <FileText className="w-5 h-5 text-blue-500" />;
    return <File className="w-5 h-5" />;
  };

  const getTypeBadgeColor = (t: DocumentType) => {
    const map: Record<DocumentType, string> = {
      BRD: 'bg-purple-100 text-purple-800',
      Specification: 'bg-blue-100 text-blue-800',
      'Technical Doc': 'bg-green-100 text-green-800',
      'Architecture Doc': 'bg-orange-100 text-orange-800',
      Other: 'bg-gray-100 text-gray-800',
    };
    return map[t] || 'bg-gray-100 text-gray-800';
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Documents & BRDs</h3>
        <Dialog open={isUploadOpen} onOpenChange={setIsUploadOpen}>
          <DialogTrigger asChild>
            <Button><Upload className="w-4 h-4 mr-2" />Upload Document</Button>
          </DialogTrigger>
          <DialogContent className="max-w-md">
            <DialogHeader><DialogTitle>Upload Document</DialogTitle></DialogHeader>
            <div className="space-y-4 mt-4">
              <div>
                <Label>Select File *</Label>
                <Input type="file" onChange={handleFileChange} accept=".pdf,.doc,.docx,.txt" />
                {file && <p className="text-xs text-muted-foreground mt-1">{file.name} ({formatFileSize(file.size)})</p>}
              </div>
              <div>
                <Label>Document Name *</Label>
                <Input value={name} onChange={e => setName(e.target.value)} placeholder="e.g., BRD - User Authentication" />
              </div>
              <div>
                <Label>Document Type *</Label>
                <Select value={type} onValueChange={v => setType(v as DocumentType)}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {['BRD','Specification','Technical Doc','Architecture Doc','Other'].map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>Version (Optional)</Label>
                <Input value={version} onChange={e => setVersion(e.target.value)} placeholder="e.g., v1.0" />
              </div>
              <div className="flex gap-2 pt-2">
                <Button onClick={handleUpload} disabled={!file || !name || isUploading} className="flex-1">
                  {isUploading ? 'Uploading...' : 'Upload'}
                </Button>
                <Button variant="outline" onClick={() => setIsUploadOpen(false)} disabled={isUploading}>Cancel</Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {/* Edit dialog */}
      <Dialog open={!!editingDoc} onOpenChange={open => !open && setEditingDoc(null)}>
        <DialogContent className="max-w-md">
          <DialogHeader><DialogTitle>Edit Document</DialogTitle></DialogHeader>
          <div className="space-y-4 mt-2">
            <div>
              <Label>Document Name *</Label>
              <Input value={editName} onChange={e => setEditName(e.target.value)} />
            </div>
            <div>
              <Label>Document Type</Label>
              <Select value={editType} onValueChange={v => setEditType(v as DocumentType)}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {['BRD','Specification','Technical Doc','Architecture Doc','Other'].map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>Version</Label>
              <Input value={editVersion} onChange={e => setEditVersion(e.target.value)} placeholder="e.g., v1.0" />
            </div>
            <p className="text-xs text-muted-foreground">Note: To replace the file, delete and re-upload.</p>
            <div className="flex gap-2 pt-1">
              <Button onClick={handleSaveEdit} disabled={!editName || isSaving} className="flex-1">
                {isSaving ? 'Saving...' : 'Save Changes'}
              </Button>
              <Button variant="outline" onClick={() => setEditingDoc(null)}>Cancel</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {documents.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="py-16 text-center">
            <FileText className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Documents Yet</h3>
            <p className="text-muted-foreground mb-4">Upload BRDs, specifications, and other documents</p>
            <Button onClick={() => setIsUploadOpen(true)}><Upload className="w-4 h-4 mr-2" />Upload First Document</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {documents.map(doc => (
            <Card key={doc.id} className="hover:shadow-md transition-shadow">
              <CardContent className="p-6">
                <div className="flex items-start gap-4">
                  <div className="mt-1">{getFileIcon((doc as any).mimeType)}</div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1">
                        <h4 className="font-semibold text-base mb-1">{doc.name}</h4>
                        <div className="flex items-center gap-2 flex-wrap">
                          <Badge className={getTypeBadgeColor(doc.type)}>{doc.type}</Badge>
                          {doc.version && <Badge variant="outline">{doc.version}</Badge>}
                          <span className="text-xs text-muted-foreground">{formatFileSize(doc.fileSize)}</span>
                          <span className="text-xs text-muted-foreground">Uploaded {new Date(doc.uploadedAt).toLocaleDateString()}</span>
                        </div>
                      </div>
                      <div className="flex gap-2">
                        <Button size="sm" variant="outline" onClick={() => handleDownload(doc)}>
                          <Download className="w-4 h-4" />
                        </Button>
                        <Button size="sm" variant="outline" onClick={() => openEdit(doc)}>
                          <Pencil className="w-4 h-4" />
                        </Button>
                        <Button size="sm" variant="outline" onClick={() => handleDelete(doc.id)}>
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      </div>
                    </div>
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
