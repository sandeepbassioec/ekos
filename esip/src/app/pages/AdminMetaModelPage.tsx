import React, { useEffect, useState } from 'react';
import { Plus, Settings, Power, PowerOff, Pencil } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { MetaModelAPI, initializeMetaModel } from '../lib/metamodel-store';
import { ArtifactType } from '../types/metamodel';
import { useNavigate } from 'react-router';
import { toast } from 'sonner';

export default function AdminMetaModelPage() {
  const navigate = useNavigate();
  const [artifactTypes, setArtifactTypes] = useState<ArtifactType[]>([]);

  useEffect(() => {
    initializeMetaModel();
    loadArtifactTypes();
  }, []);

  const loadArtifactTypes = () => {
    const types = MetaModelAPI.getArtifactTypes();
    setArtifactTypes(types);
  };

  const toggleActive = (id: string, currentState: boolean) => {
    MetaModelAPI.updateArtifactType(id, { isActive: !currentState });
    loadArtifactTypes();
    toast.success(!currentState ? 'Artifact type activated' : 'Artifact type deactivated');
  };

  const categoryColors: Record<string, string> = {
    application: 'bg-blue-100 text-blue-800 border-blue-300',
    infrastructure: 'bg-purple-100 text-purple-800 border-purple-300',
    integration: 'bg-green-100 text-green-800 border-green-300',
    dependency: 'bg-orange-100 text-orange-800 border-orange-300',
    system: 'bg-gray-100 text-gray-800 border-gray-300',
    computation: 'bg-pink-100 text-pink-800 border-pink-300',
  };

  return (
    <div className="container mx-auto py-8 px-4">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2">Artifact Type Management</h1>
          <p className="text-muted-foreground">
            Configure artifact types, fields, rules, and templates
          </p>
        </div>
        <Button onClick={() => navigate('/admin/metamodel/create')}>
          <Plus className="w-4 h-4 mr-2" />
          Create Artifact Type
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {artifactTypes.map((artifact) => (
          <Card key={artifact.id} className={artifact.isActive ? '' : 'opacity-60'}>
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="text-3xl">{artifact.icon}</div>
                  <div>
                    <CardTitle className="text-lg">{artifact.name}</CardTitle>
                    <CardDescription className="text-xs mt-1">
                      {artifact.description || 'No description'}
                    </CardDescription>
                  </div>
                </div>
                {artifact.isActive ? (
                  <Power className="w-5 h-5 text-green-600" />
                ) : (
                  <PowerOff className="w-5 h-5 text-gray-400" />
                )}
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex items-center gap-2">
                <Badge className={categoryColors[artifact.category] || ''}>
                  {artifact.category}
                </Badge>
                <Badge variant={artifact.isActive ? 'default' : 'secondary'}>
                  {artifact.isActive ? 'Active' : 'Inactive'}
                </Badge>
              </div>

              <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
                <div>
                  <span className="font-semibold">Fields:</span>{' '}
                  {MetaModelAPI.getFieldDefinitions(artifact.id).length}
                </div>
                <div>
                  <span className="font-semibold">Rules:</span>{' '}
                  {MetaModelAPI.getComplexityRules(artifact.id).length}
                </div>
                <div>
                  <span className="font-semibold">WBS:</span>{' '}
                  {MetaModelAPI.getWBSTemplates(artifact.id).length}
                </div>
                <div>
                  <span className="font-semibold">Stories:</span>{' '}
                  {MetaModelAPI.getStoryTemplates(artifact.id).length}
                </div>
              </div>

              <div className="flex gap-2 pt-2 border-t">
                <Button
                  size="sm"
                  variant="outline"
                  className="flex-1"
                  onClick={() => navigate(`/admin/metamodel/configure/${artifact.id}`)}
                >
                  <Settings className="w-3 h-3 mr-1" />
                  Configure
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="flex-1"
                  onClick={() => navigate(`/admin/metamodel/edit/${artifact.id}`)}
                >
                  <Pencil className="w-3 h-3 mr-1" />
                  Edit
                </Button>
                <Button
                  size="sm"
                  variant={artifact.isActive ? 'destructive' : 'default'}
                  onClick={() => toggleActive(artifact.id, artifact.isActive)}
                >
                  {artifact.isActive ? <PowerOff className="w-3 h-3" /> : <Power className="w-3 h-3" />}
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {artifactTypes.length === 0 && (
        <div className="text-center py-20">
          <p className="text-muted-foreground mb-4">No artifact types configured yet.</p>
          <Button onClick={() => navigate('/admin/metamodel/create')}>
            <Plus className="w-4 h-4 mr-2" />
            Create First Artifact Type
          </Button>
        </div>
      )}
    </div>
  );
}
