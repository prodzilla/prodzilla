import ProbeResultItem from './ProbeResultItem';
import StoryResultItem from './StoryResultItem';
import { useSidebar, useSelection } from '../hooks/useDashboard';
import { useEscapeKey } from '../hooks/useEscapeKey';
import { XIcon } from '@/lib/icons';
import { isStoryResult } from '@/lib/api-client';

export default function ResultsSidebar() {
  const {
    sidebarOpen,
    selectedMonitor,
    results,
    resultsLoading,
    resultsError,
    handleCloseSidebar,
  } = useSidebar();
  const { bulkTriggerResults } = useSelection();
  const showingBulkResults =
    bulkTriggerResults !== undefined && !selectedMonitor;
  const displayResults = showingBulkResults ? bulkTriggerResults : results;

  // Use the escape key hook
  useEscapeKey(sidebarOpen, handleCloseSidebar);

  if (!sidebarOpen) return null;

  return (
    <>
      <div
        className={`fixed right-0 top-0 h-full w-96 bg-white shadow-xl z-50  ${
          sidebarOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              {showingBulkResults
                ? 'Bulk Trigger Results'
                : selectedMonitor?.name}
            </h2>
            <span className="text-sm text-gray-500 capitalize">
              {showingBulkResults
                ? `${bulkTriggerResults?.length || 0} items triggered`
                : `${selectedMonitor?.type} Results`}
            </span>
          </div>
          <button
            onClick={handleCloseSidebar}
            className="p-2 hover:bg-gray-100 rounded-full transition-colors"
          >
            <XIcon className="w-5 h-5 text-gray-500" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          {resultsLoading && (
            <div className="flex items-center justify-center h-32">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            </div>
          )}

          {resultsError && (
            <div className="p-4 text-red-600 bg-red-50 m-4 rounded">
              {resultsError}
            </div>
          )}

          {displayResults && !resultsLoading && !resultsError && (
            <div>
              {!displayResults ? (
                <div className="p-4 text-center text-gray-500">
                  No results found
                </div>
              ) : showingBulkResults ? (
                <div>
                  {/* Bulk Results Summary */}
                  <div className="p-4 border-b border-gray-200">
                    <div className="flex gap-4 text-sm">
                      <span className="text-green-600">
                        ✓ {displayResults.filter((r) => r.success).length}{' '}
                        successful
                      </span>
                      <span className="text-red-600">
                        ✗ {displayResults.filter((r) => !r.success).length}{' '}
                        failed
                      </span>
                    </div>
                  </div>

                  {/* Individual Results */}
                  <div>
                    {displayResults.map((result, index) =>
                      isStoryResult(result) ? (
                        <StoryResultItem
                          key={`bulk-${index}`}
                          result={result}
                          showName={true}
                        />
                      ) : (
                        <ProbeResultItem
                          key={`bulk-${index}`}
                          result={result}
                          showName={true}
                        />
                      )
                    )}
                  </div>
                </div>
              ) : (
                <div>
                  {displayResults.map((result, index) =>
                    isStoryResult(result) ? (
                      <StoryResultItem key={index} result={result} />
                    ) : (
                      <ProbeResultItem key={index} result={result} />
                    )
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
