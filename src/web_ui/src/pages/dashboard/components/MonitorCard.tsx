import StatusBadge from './StatusBadge';
import TagList from './TagList';
import { formatDateTime } from '@/lib/helpers';
import { useMonitorActions, useSelection } from '../hooks/useDashboard';
import { useCallback } from 'react';

type MonitorCardProps = {
  item: {
    name: string;
    status: string;
    last_probed: string;
    tags?: Record<string, string> | undefined;
    type: 'probe' | 'story';
  };
};

export default function MonitorCard({ item }: MonitorCardProps) {
  const { name, status, last_probed, tags, type } = item;
  const { handleMonitorClick, handleTagToggle } = useMonitorActions();
  const { selectedItems, handleItemSelect } = useSelection();

  const itemKey = `${type}:${name}`;
  const isSelected = selectedItems.has(itemKey);

  const handleClickTitle = useCallback(() => {
    handleMonitorClick(name, type);
  }, [handleMonitorClick, name, type]);

  const handleSelectChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      e.stopPropagation();
      handleItemSelect(name, type);
    },
    [handleItemSelect, name, type]
  );

  return (
    <div
      className={`bg-white rounded-lg shadow p-4 transition-shadow ${isSelected ? 'ring-2 ring-blue-500 bg-blue-50' : ''}`}
    >
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-start gap-2">
          <input
            type="checkbox"
            checked={isSelected}
            onChange={handleSelectChange}
            className="mt-1 h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
            onClick={(e) => e.stopPropagation()}
          />
          <h3
            className="text-lg font-semibold text-gray-900 cursor-pointer hover:underline"
            onClick={handleClickTitle}
          >
            {name}
          </h3>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 uppercase font-medium">
            {type}
          </span>
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
