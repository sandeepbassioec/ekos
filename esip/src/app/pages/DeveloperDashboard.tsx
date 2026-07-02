import { useState, useEffect } from 'react';
import { ApiClient } from '../lib/api';
import { useAuthStore } from '../store/authStore';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Textarea } from '../components/ui/textarea';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '../components/ui/dialog';
import { Label } from '../components/ui/label';
import { toast } from 'sonner';
import { ClipboardList, Clock, CheckCircle, XCircle } from 'lucide-react';

export default function DeveloperDashboard() {
  const [tasks, setTasks] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTask, setSelectedTask] = useState<any>(null);
  const [estimateHours, setEstimateHours] = useState('');
  const [notes, setNotes] = useState('');
  const user = useAuthStore((state) => state.user);

  useEffect(() => {
    fetchTasks();
  }, []);

  const fetchTasks = async () => {
    try {
      const projects = await ApiClient.getProjects();
      const allTasks: any[] = [];
      
      for (const project of projects) {
        const wbs = await ApiClient.getWbsTasks(project.id);
        allTasks.push(...wbs.map((t: any) => ({ ...t, projectName: project.name })));
      }
      
      setTasks(allTasks);
    } catch (error: any) {
      toast.error(error.message);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmitEstimate = async () => {
    if (!selectedTask || !estimateHours) return;
    
    try {
      await ApiClient.updateWbsTask(selectedTask.id, {
        developerEstimatedHours: parseFloat(estimateHours),
        developerNotes: notes,
      });
      
      toast.success('Estimate submitted for review');
      setSelectedTask(null);
      setEstimateHours('');
      setNotes('');
      fetchTasks();
    } catch (error: any) {
      toast.error(error.message);
    }
  };

  const getStatusBadge = (status: string) => {
    const styles = {
      pending: 'bg-yellow-100 text-yellow-800',
      approved: 'bg-green-100 text-green-800',
      rejected: 'bg-red-100 text-red-800',
    };
    return <Badge className={styles[status as keyof typeof styles] || ''}>{status}</Badge>;
  };

  if (loading) return <div className="p-8">Loading...</div>;

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex items-center gap-3 mb-8">
          <ClipboardList className="w-8 h-8 text-blue-600" />
          <h1 className="text-3xl font-bold">My Assigned Tasks</h1>
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
                  {getStatusBadge(task.status)}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {task.description && (
                    <p className="text-sm text-gray-600">{task.description}</p>
                  )}
                  
                  <div className="grid grid-cols-3 gap-4 text-sm">
                    <div>
                      <span className="text-gray-500">Complexity:</span>
                      <span className="ml-2 font-medium">{task.complexity || 'N/A'}</span>
                    </div>
                    <div>
                      <span className="text-gray-500">Manager Estimate:</span>
                      <span className="ml-2 font-medium">{task.estimatedHours || 'N/A'} hrs</span>
                    </div>
                    <div>
                      <span className="text-gray-500">Your Estimate:</span>
                      <span className="ml-2 font-medium">{task.developerEstimatedHours || 'N/A'} hrs</span>
                    </div>
                  </div>

                  {task.developerNotes && (
                    <div className="bg-blue-50 p-3 rounded">
                      <div className="text-xs text-blue-600 font-semibold mb-1">Your Notes:</div>
                      <div className="text-sm">{task.developerNotes}</div>
                    </div>
                  )}

                  {task.managerNotes && (
                    <div className="bg-gray-50 p-3 rounded">
                      <div className="text-xs text-gray-600 font-semibold mb-1">Manager Feedback:</div>
                      <div className="text-sm">{task.managerNotes}</div>
                    </div>
                  )}

                  <Dialog>
                    <DialogTrigger asChild>
                      <Button
                        variant="outline"
                        onClick={() => {
                          setSelectedTask(task);
                          setEstimateHours(task.developerEstimatedHours?.toString() || '');
                          setNotes(task.developerNotes || '');
                        }}
                      >
                        <Clock className="w-4 h-4 mr-2" />
                        {task.developerEstimatedHours ? 'Update' : 'Submit'} Estimate
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Submit Your Estimate</DialogTitle>
                      </DialogHeader>
                      <div className="space-y-4">
                        <div>
                          <Label>Task: {task.name}</Label>
                        </div>
                        <div>
                          <Label>Your Estimated Hours</Label>
                          <Input
                            type="number"
                            value={estimateHours}
                            onChange={(e) => setEstimateHours(e.target.value)}
                            placeholder="e.g., 8"
                            step="0.5"
                          />
                        </div>
                        <div>
                          <Label>Justification / Notes</Label>
                          <Textarea
                            value={notes}
                            onChange={(e) => setNotes(e.target.value)}
                            placeholder="Explain your estimate..."
                            rows={4}
                          />
                        </div>
                        <Button onClick={handleSubmitEstimate} className="w-full">
                          Submit for Review
                        </Button>
                      </div>
                    </DialogContent>
                  </Dialog>
                </div>
              </CardContent>
            </Card>
          ))}

          {tasks.length === 0 && (
            <div className="text-center py-12 text-gray-500">
              No tasks assigned yet.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
