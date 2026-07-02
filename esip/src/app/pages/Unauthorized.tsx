import { Link } from 'react-router';
import { Button } from '../components/ui/button';
import { ShieldAlert } from 'lucide-react';

export default function Unauthorized() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="text-center space-y-4">
        <ShieldAlert className="w-16 h-16 mx-auto text-red-500" />
        <h1 className="text-2xl font-bold">Access Denied</h1>
        <p className="text-gray-600">You don't have permission to access this page.</p>
        <Button asChild>
          <Link to="/">Go to Dashboard</Link>
        </Button>
      </div>
    </div>
  );
}
