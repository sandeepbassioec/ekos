import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router';
import { Plus, FileText, Clock, CheckCircle, Eye, Trash2, Loader2 } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import { Card, CardContent } from '../components/ui/card';
import { db } from '../lib/db';
import { useAuthStore } from '../store/authStore';
import { toast } from 'sonner';
import type { DesignerDocument, DesignerDocumentStatus } from '../lib/db/types';

const STATUS_ICON: Record<DesignerDocumentStatus, React.ReactNode> = {
  draft:    <Clock className="size-3 text-muted-foreground" />,
  review:   <Eye className="size-3 text-yellow-600" />,
  approved: <CheckCircle className="size-3 text-green-600" />,
};

const STATUS_VARIANT: Record<DesignerDocumentStatus, 'secondary' | 'outline' | 'default'> = {
  draft:    'secondary',
  review:   'outline',
  approved: 'default',
};

const CATEGORY_BADGE: Record<string, string> = {
  brd: 'BRD', hld: 'HLD', lld: 'LLD', architecture: 'ARCH', custom: 'DOC',
};

export default function DocumentsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { user } = useAuthStore();
  const [docs, setDocs] = useState<DesignerDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  useEffect(() => {
    if (!projectId) return;
    db.designerDocuments.getByProject(projectId)
      .then(setDocs)
      .catch(() => toast.error('Failed to load documents'))
      .finally(() => setLoading(false));
  }, [projectId]);

  const handleDelete = async (doc: DesignerDocument) => {
    if (!confirm(`Delete "${doc.title}"? This cannot be undone.`)) return;
    setDeletingId(doc.id);
    try {
      await db.designerDocuments.delete(doc.id);
      setDocs(prev => prev.filter(d => d.id !== doc.id));
      toast.success('Document deleted');
    } catch {
      toast.error('Delete failed');
    } finally {
      setDeletingId(null);
    }
  };

  return (
    <div className="container max-w-5xl mx-auto py-8 px-4 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Documents</h1>
          <p className="text-muted-foreground text-sm mt-0.5">Author BRDs, HLDs, LLDs and other project documents</p>
        </div>
        <Button onClick={() => navigate(`/projects/${projectId}/documents/new`)} className="gap-2">
          <Plus className="size-4" />
          New Document
        </Button>
      </div>

      {loading && (
        <div className="flex justify-center py-12">
          <Loader2 className="size-6 animate-spin text-muted-foreground" />
        </div>
      )}

      {!loading && docs.length === 0 && (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border py-16 text-center">
          <div className="flex size-12 items-center justify-center rounded-xl bg-muted mb-4">
            <FileText className="size-6 text-muted-foreground" />
          </div>
          <p className="font-medium">No documents yet</p>
          <p className="text-sm text-muted-foreground mt-1">Create a BRD, HLD, or LLD to get started</p>
          <Button
            onClick={() => navigate(`/projects/${projectId}/documents/new`)}
            className="mt-4 gap-2"
            variant="outline"
          >
            <Plus className="size-4" />
            Create first document
          </Button>
        </div>
      )}

      {!loading && docs.length > 0 && (
        <div className="grid gap-3">
          {docs.map(doc => (
            <Card
              key={doc.id}
              className="hover:shadow-md transition-shadow cursor-pointer"
              onClick={() => navigate(`/projects/${projectId}/documents/${doc.id}`)}
            >
              <CardContent className="flex items-center gap-4 py-4">
                <div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                  <span className="text-sm font-bold text-primary">{CATEGORY_BADGE[doc.category] ?? 'DOC'}</span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-semibold text-sm truncate">{doc.title}</p>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    Updated {new Date(doc.updatedAt).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })}
                    {' · v'}{doc.version}
                  </p>
                </div>
                <Badge variant={STATUS_VARIANT[doc.status]} className="gap-1 shrink-0">
                  {STATUS_ICON[doc.status]}
                  <span className="capitalize">{doc.status}</span>
                </Badge>
                <button
                  onClick={(e) => { e.stopPropagation(); handleDelete(doc); }}
                  disabled={deletingId === doc.id}
                  className="ml-2 rounded p-1 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                >
                  {deletingId === doc.id
                    ? <Loader2 className="size-4 animate-spin" />
                    : <Trash2 className="size-4" />}
                </button>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
