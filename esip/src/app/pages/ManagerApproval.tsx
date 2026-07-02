import { useState, useEffect } from 'react';
import { ApiClient } from '../lib/api';
import { Button } from '../components/ui/button';
import { Textarea } from '../components/ui/textarea';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../components/ui/dialog';
import { Label } from '../components/ui/label';
import { toast } from 'sonner';
import { CheckCircle, XCircle, Clock } from 'lucide-react';

export default function ManagerApproval() {
  const [tasks, setTasks] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTask, setSelectedTask] = useState<any>(null);
  const [feedback, setFeedback] = useState('');
  const [showDialog, setShowDialog] = useState(false);
  const [action, setAction] = useState<'approve' | 'reject'>('approve');

  useEffect(() => {
    fetchPendingTasks();
  }, []);

  const fetchPendingTasks = async () => {
    try {
      const projects = await ApiClient.getProjects();
      const allTasks: any[] = [];
      
      for (const project of projects) {
        const wbs = await ApiClient.getWbsTasks(project.id);
        const pending = wbs.filter((t: any) => t.status === 'pending' && t.developerEstimatedHours);
        allTasks.push(...pending.map((t: any) => ({ ...t, projectName: project.name })));
      }
      
      setTasks(allTasks);
    } catch (error: any) {
      toast.error(error.message);
    } finally {
      setLoading(false);
    }
  };

  const handleApprove = async () => {
    if (!selectedTask) return;
    
    try {
      await ApiClient.updateWbsTask(selectedTask.id, {
        status: 'approved',
        developerEstimatedHours: selectedTask.developerEstimatedHours,
        managerNotes: feedback,
      });
      
      toast.success('Estimate approved');
      setShowDialog(false);
      setFeedback('');
      fetchPendingTasks();
    } catch (error: any) {
      toast.error(error.message);
    }
  };

  const handleReject = async () => {
    if (!selectedTask) return;
    
    try {
      await ApiClient.updateWbsTask(selectedTask.id, {
        status: 'rejected',
        managerNotes: feedback,
      });
      
      toast.success('Estimate rejected with feedback');
      setShowDialog(false);
      setFeedback('');
      fetchPendingTasks();
    } catch (error: any) {
      toast.error(error.message);
    }
  };

  const openDialog = (task: any, actionType: 'approve' | 'reject') => {
    setSelectedTask(task);
    setAction(actionType);
    setShowDialog(true);
    setFeedback('');
  };

  if (loading) return <div className="p-8">Loading...</div>;

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex items-center gap-3 mb-8">
          <Clock className="w-8 h-8 text-orange-600" />
          <h1 className="text-3xl font-bold">Pending Approvals</h1>
          <Badge className="ml-2">{tasks.length}</Badge>
        </div>

        <div className="grid gap-4">
          {tasks.map((task) => (
            <Card key={task.id}>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  <div>
                    <div>{task.name}</div>
                    <div className="text-sm font-normal text-gray-500">{task.projectName}</div>
                  </div>
                  <Badge className="bg-yellow-100 text-yellow-800">Pending Review</Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4 bg-gray-50 p-4 rounded">
                    <div>
                      <div className="text-sm text-gray-500">Your Original Estimate</div>
                      <div className="text-2xl font-bold text-gray-900">
                        {task.estimatedHours || 'N/A'} hrs
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-500">Developer's Estimate</div>
                      <div className="text-2xl font-bold text-blue-600">
                        {task.developerEstimatedHours} hrs
                      </div>
                    </div>
                  </div>

                  {task.developerNotes && (
                    <div className="bg-blue-50 p-3 rounded">
                      <div className="text-xs text-blue-600 font-semibold mb-1">Developer's Justification:</div>
                      <div className="text-sm">{task.developerNotes}</div>
                    </div>
                  )}

                  <div className="flex gap-2">
                    <Button
                      onClick={() => openDialog(task, 'approve')}
                      className="flex-1"
                      variant="default"
                    >
                      <CheckCircle className="w-4 h-4 mr-2" />
                      Approve
                    </Button>
                    <Button
                      onClick={() => openDialog(task, 'reject')}
                      className="flex-1"
                      variant="destructive"
                    >
                      <XCircle className="w-4 h-4 mr-2" />
                      Reject
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}

          {tasks.length === 0 && (
            <div className="text-center py-12 text-gray-500">
              No pending approvals. All estimates are up to date!
            </div>
          )}
        </div>
      </div>

      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {action === 'approve' ? 'Approve' : 'Reject'} Estimate
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label>Feedback (Optional)</Label>
              <Textarea
                value={feedback}
                onChange={(e) => setFeedback(e.target.value)}
                placeholder="Add any comments for the developer..."
                rows={4}
              />
            </div>
            <Button
              onClick={action === 'approve' ? handleApprove : handleReject}
              className="w-full"
              variant={action === 'approve' ? 'default' : 'destructive'}
            >
              {action === 'approve' ? 'Approve' : 'Reject'} Estimate
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
