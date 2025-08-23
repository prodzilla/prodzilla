import { useSelection } from '../hooks/useDashboard';

export default function BulkActionBar() {
  const {
    selectedItems,
    bulkTriggerLoading,
    handleSelectAll,
    handleClearSelection,
    handleBulkTrigger,
  } = useSelection();

  return (
    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <span className="text-sm font-medium text-blue-900">
            {selectedItems.size} item{selectedItems.size !== 1 ? 's' : ''}{' '}
            selected
          </span>
          <div className="flex gap-2">
            <button
              onClick={handleSelectAll}
              className="text-sm text-blue-600 hover:text-blue-800 font-medium cursor-pointer"
            >
              Select All
            </button>
            <span className="text-blue-300">|</span>
            <button
              onClick={handleClearSelection}
              className="text-sm text-blue-600 hover:text-blue-800 font-medium cursor-pointer"
            >
              Clear Selection
            </button>
          </div>
        </div>

        {selectedItems.size > 0 && (
          <button
            onClick={handleBulkTrigger}
            disabled={bulkTriggerLoading}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2"
          >
            {bulkTriggerLoading && (
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            )}
            {bulkTriggerLoading ? 'Triggering...' : 'Trigger Selected'}
          </button>
        )}
      </div>
    </div>
  );
}
