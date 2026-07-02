import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore, TeamMember, SprintDuration } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Card } from '../../components/ui/card';
import { Input } from '../../components/ui/input';
import { Label } from '../../components/ui/label';
import { 
  ArrowRight, 
  ArrowLeft, 
  Users, 
  Calendar, 
  TrendingUp, 
  Plus, 
  Trash2,
  AlertTriangle,
  CheckCircle,
  Clock
} from 'lucide-react';
import { toast } from 'sonner';

export function CapacityPlanning() {
  const navigate = useNavigate();
  const { 
    results, 
    project,
    capacityPlan,
    setCurrentStep, 
    generateCapacityPlan,
    setCapacityPlan,
    addTeamMember,
    updateTeamMember,
    removeTeamMember
  } = useEstimationStore();

  const [sprintDuration, setSprintDuration] = useState<SprintDuration>(
    capacityPlan?.sprintDuration || 2
  );
  const [productivityFactor, setProductivityFactor] = useState(
    capacityPlan?.productivityFactor || 75
  );
  const [includeBuffer, setIncludeBuffer] = useState(
    capacityPlan?.includeBufferTime || true
  );
  const [bufferPercentage, setBufferPercentage] = useState(
    capacityPlan?.bufferPercentage || 20
  );
  const [startDate, setStartDate] = useState(
    capacityPlan?.startDate ? new Date(capacityPlan.startDate).toISOString().split('T')[0] : 
    new Date().toISOString().split('T')[0]
  );

  const [showAddMember, setShowAddMember] = useState(false);
  const [newMember, setNewMember] = useState({
    name: '',
    role: '',
    allocation: 100,
    availableHoursPerWeek: 40
  });

  useEffect(() => {
    setCurrentStep(8);
  }, [setCurrentStep]);

  // Auto-generate capacity plan when inputs change
  useEffect(() => {
    if (results) {
      handleGeneratePlan();
    }
  }, [sprintDuration, productivityFactor, includeBuffer, bufferPercentage, startDate]);

  const handleGeneratePlan = () => {
    if (!results) {
      toast.error('No estimation results available');
      return;
    }

    // Update capacity plan settings
    setCapacityPlan({
      sprintDuration,
      teamMembers: capacityPlan?.teamMembers || [],
      productivityFactor: productivityFactor / 100,
      includeBufferTime: includeBuffer,
      bufferPercentage,
      startDate: new Date(startDate),
      sprints: []
    });

    // Generate sprints
    generateCapacityPlan();
    toast.success('Capacity plan generated!');
  };

  const handleAddMember = () => {
    if (!newMember.name || !newMember.role) {
      toast.error('Please fill in all team member details');
      return;
    }

    const member: TeamMember = {
      id: `member-${Date.now()}`,
      ...newMember
    };

    addTeamMember(member);
    setNewMember({ name: '', role: '', allocation: 100, availableHoursPerWeek: 40 });
    setShowAddMember(false);
    toast.success(`${member.name} added to team`);
  };

  const handleNext = () => {
    setCurrentStep(9);
    navigate('/wizard/step/9');
  };

  const handleBack = () => {
    setCurrentStep(7);
    navigate('/wizard/step/7');
  };

  if (!results) {
    return (
      <div className="text-center py-12">
        <AlertTriangle className="w-16 h-16 text-yellow-500 mx-auto mb-4" />
        <p className="text-gray-600">Please complete estimation first</p>
        <Button onClick={handleBack} className="mt-4">
          Go Back
        </Button>
      </div>
    );
  }

  const totalCapacityHours = capacityPlan?.sprints.reduce((sum, s) => sum + s.capacityHours, 0) || 0;
  const totalPlannedHours = capacityPlan?.sprints.reduce((sum, s) => sum + s.plannedHours, 0) || 0;
  const averageUtilization = totalCapacityHours > 0 
    ? (totalPlannedHours / totalCapacityHours) * 100 
    : 0;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Capacity Planning</h2>
        <p className="text-gray-600">Plan sprints, team allocation, and delivery timeline</p>
      </div>

      {/* Summary Cards */}
      <div className="grid md:grid-cols-4 gap-4">
        <Card className="p-4 bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200">
          <div className="flex items-center gap-3">
            <Calendar className="w-8 h-8 text-blue-600" />
            <div>
              <div className="text-sm text-blue-600 font-semibold">Total Sprints</div>
              <div className="text-2xl font-bold text-blue-700">
                {capacityPlan?.sprints.length || 0}
              </div>
            </div>
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-green-50 to-emerald-50 border-green-200">
          <div className="flex items-center gap-3">
            <Clock className="w-8 h-8 text-green-600" />
            <div>
              <div className="text-sm text-green-600 font-semibold">Planned Hours</div>
              <div className="text-2xl font-bold text-green-700">
                {Math.round(totalPlannedHours)}
              </div>
            </div>
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-purple-50 to-pink-50 border-purple-200">
          <div className="flex items-center gap-3">
            <TrendingUp className="w-8 h-8 text-purple-600" />
            <div>
              <div className="text-sm text-purple-600 font-semibold">Utilization</div>
              <div className="text-2xl font-bold text-purple-700">
                {Math.round(averageUtilization)}%
              </div>
            </div>
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-orange-50 to-red-50 border-orange-200">
          <div className="flex items-center gap-3">
            <Users className="w-8 h-8 text-orange-600" />
            <div>
              <div className="text-sm text-orange-600 font-semibold">Team Size</div>
              <div className="text-2xl font-bold text-orange-700">
                {project.teamSize}
              </div>
            </div>
          </div>
        </Card>
      </div>

      {/* Configuration */}
      <Card className="p-6">
        <h3 className="font-semibold text-gray-900 mb-4 flex items-center gap-2">
          <Calendar className="w-5 h-5" />
          Sprint Configuration
        </h3>
        
        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <Label htmlFor="sprintDuration">Sprint Duration (weeks)</Label>
            <select
              id="sprintDuration"
              value={sprintDuration}
              onChange={(e) => setSprintDuration(Number(e.target.value) as SprintDuration)}
              className="w-full mt-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value={1}>1 week</option>
              <option value={2}>2 weeks (Recommended)</option>
              <option value={3}>3 weeks</option>
              <option value={4}>4 weeks</option>
            </select>
          </div>

          <div>
            <Label htmlFor="startDate">Project Start Date</Label>
            <Input
              id="startDate"
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              className="mt-1"
            />
          </div>

          <div>
            <Label htmlFor="productivity">
              Productivity Factor ({productivityFactor}%)
            </Label>
            <input
              id="productivity"
              type="range"
              min="50"
              max="100"
              value={productivityFactor}
              onChange={(e) => setProductivityFactor(Number(e.target.value))}
              className="w-full mt-2"
            />
            <p className="text-xs text-gray-500 mt-1">
              Accounts for meetings, support, and overhead (60-80% is typical)
            </p>
          </div>

          <div>
            <Label htmlFor="buffer">
              Buffer Percentage ({bufferPercentage}%)
            </Label>
            <div className="flex items-center gap-3 mt-1">
              <input
                type="checkbox"
                id="includeBuffer"
                checked={includeBuffer}
                onChange={(e) => setIncludeBuffer(e.target.checked)}
                className="w-4 h-4"
              />
              <label htmlFor="includeBuffer" className="text-sm text-gray-700">
                Include buffer time
              </label>
            </div>
            {includeBuffer && (
              <input
                type="range"
                min="10"
                max="50"
                value={bufferPercentage}
                onChange={(e) => setBufferPercentage(Number(e.target.value))}
                className="w-full mt-2"
              />
            )}
          </div>
        </div>
      </Card>

      {/* Sprint Timeline */}
      {capacityPlan && capacityPlan.sprints.length > 0 && (
        <Card className="p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Sprint Timeline</h3>
          
          <div className="space-y-3">
            {capacityPlan.sprints.map((sprint) => {
              const isOverallocated = sprint.utilizationPercent > 100;
              const isUnderutilized = sprint.utilizationPercent < 70;
              
              return (
                <div key={sprint.id} className="border border-gray-200 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <div>
                      <h4 className="font-semibold text-gray-900">Sprint {sprint.number}</h4>
                      <p className="text-sm text-gray-600">
                        {new Date(sprint.startDate).toLocaleDateString()} - {new Date(sprint.endDate).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-gray-600">
                        {Math.round(sprint.plannedHours)} / {Math.round(sprint.capacityHours)} hrs
                      </div>
                      <div className={`text-lg font-bold ${
                        isOverallocated ? 'text-red-600' : 
                        isUnderutilized ? 'text-yellow-600' : 
                        'text-green-600'
                      }`}>
                        {Math.round(sprint.utilizationPercent)}%
                        {isOverallocated && <AlertTriangle className="inline w-4 h-4 ml-1" />}
                        {!isOverallocated && !isUnderutilized && <CheckCircle className="inline w-4 h-4 ml-1" />}
                      </div>
                    </div>
                  </div>

                  {/* Utilization Bar */}
                  <div className="w-full bg-gray-200 rounded-full h-3 overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all ${
                        isOverallocated ? 'bg-red-500' : 
                        isUnderutilized ? 'bg-yellow-500' : 
                        'bg-green-500'
                      }`}
                      style={{ width: `${Math.min(sprint.utilizationPercent, 100)}%` }}
                    />
                  </div>

                  {/* Phase Breakdown */}
                  <div className="grid grid-cols-3 gap-2 mt-3 text-xs">
                    <div>
                      <span className="text-gray-600">Dev:</span>
                      <span className="ml-1 font-semibold">{Math.round(sprint.tasks.development)}h</span>
                    </div>
                    <div>
                      <span className="text-gray-600">QA:</span>
                      <span className="ml-1 font-semibold">{Math.round(sprint.tasks.testing)}h</span>
                    </div>
                    <div>
                      <span className="text-gray-600">Hardening:</span>
                      <span className="ml-1 font-semibold">{Math.round(sprint.tasks.hardening)}h</span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </Card>
      )}

      {/* Warnings */}
      {averageUtilization > 95 && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-red-600 mt-0.5" />
          <div>
            <h4 className="font-semibold text-red-900">High Utilization Warning</h4>
            <p className="text-sm text-red-700">
              Team utilization is over 95%. Consider adding buffer time or increasing team size.
            </p>
          </div>
        </div>
      )}

      {averageUtilization < 60 && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5" />
          <div>
            <h4 className="font-semibold text-yellow-900">Low Utilization Notice</h4>
            <p className="text-sm text-yellow-700">
              Team utilization is below 60%. You might be able to reduce timeline or team size.
            </p>
          </div>
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
          Continue to Deliverables
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}