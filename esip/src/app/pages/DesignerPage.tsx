import { useParams } from 'react-router';
import { DesignerShell } from '../components/designer/DesignerShell';

export default function DesignerPage() {
  const { projectId, documentId } = useParams<{ projectId: string; documentId?: string }>();
  if (!projectId) return null;
  return <DesignerShell projectId={projectId} documentId={documentId} />;
}
