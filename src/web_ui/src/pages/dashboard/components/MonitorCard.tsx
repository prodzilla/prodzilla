import StatusBadge from './StatusBadge';
import TagList from './TagList';
import { formatDateTime } from '@/lib/helpers';
import { useMonitorActions } from '../hooks/useDashboard';

type MonitorCardProps = {
  name: string;
  status: string;
  last_probed: string;
  tags?: Record<string, string> | null;
  type: 'probe' | 'story';
};

export default function MonitorCard({ 
  name, 
  status, 
  last_probed, 
  tags, 
  type
}: MonitorCardProps) {
  const { handleMonitorClick, handleTagToggle } = useMonitorActions();
  return (
    <div 
      className="bg-white rounded-lg shadow p-4 hover:shadow-md transition-shadow cursor-pointer hover:shadow-lg"
      onClick={() => handleMonitorClick(name, type)}
    >
      <div className="flex items-start justify-between mb-2">
        <h3 className="text-lg font-semibold text-gray-900">{name}</h3>
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 uppercase font-medium">{type}</span>
          <StatusBadge status={status} />
        </div>
      </div>
      
      <div className="text-sm text-gray-600 mb-3">
        Last probed: {formatDateTime(last_probed)}
      </div>
      
      <div onClick={(e) => e.stopPropagation()}>
        <TagList tags={tags} onTagClick={handleTagToggle} clickable />
      </div>
    </div>
  );
}