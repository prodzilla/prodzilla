import { useEffect } from 'react';
import ProbeResultItem from './ProbeResultItem';
import StoryResultItem from './StoryResultItem';
import { useSidebar } from '../hooks/useDashboard';

export default function ResultsSidebar() {
  const { sidebarOpen, selectedMonitor, results, resultsLoading, resultsError, handleCloseSidebar } = useSidebar();
  // Close on escape key
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        handleCloseSidebar();
      }
    };

    if (sidebarOpen) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [sidebarOpen, handleCloseSidebar]);

  if (!sidebarOpen) return null;

  return (
    <>
      <div
        className={`fixed right-0 top-0 h-full w-96 bg-white shadow-xl z-50 transform transition-transform duration-300 ease-in-out ${
          sidebarOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              {selectedMonitor?.name}
            </h2>
            <span className="text-sm text-gray-500 capitalize">
              {selectedMonitor?.type} Results
            </span>
          </div>
          <button
            onClick={handleCloseSidebar}
            className="p-2 hover:bg-gray-100 rounded-full transition-colors"
          >
            <svg
              className="w-5 h-5 text-gray-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
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

          {results && !resultsLoading && !resultsError && (
            <div>
              {results.length === 0 ? (
                <div className="p-4 text-center text-gray-500">
                  No results found
                </div>
              ) : (
                <div>
                  {selectedMonitor?.type === 'probe'
                    ? results.map((result, index) => (
                        <ProbeResultItem key={index} result={result} />
                      ))
                    : results.map((result, index) => (
                        <StoryResultItem key={index} result={result} />
                      ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
