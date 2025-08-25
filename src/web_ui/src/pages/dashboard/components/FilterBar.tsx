import { useTagFilters } from '../hooks/useDashboard';
import { getGroupedTags } from '@/lib/helpers';

type FilterBarProps = {
  allItems: Array<{ tags?: Record<string, string> | undefined }>;
};

export default function FilterBar({ allItems }: FilterBarProps) {
  const {
    searchTerm,
    selectedTags,
    setSearchTerm,
    handleTagToggle,
    handleClearFilters,
  } = useTagFilters();
  const groupedTags = getGroupedTags(allItems);

  return (
    <div className="bg-white rounded-lg shadow p-4 mb-6">
      <div className="mb-3">
        <input
          type="text"
          placeholder="Filter monitors by name"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {Object.keys(groupedTags).length > 0 && (
        <div>
          {selectedTags.length > 0 && (
            <div className="flex items-center justify-between mb-3">
              <button
                onClick={handleClearFilters}
                className="text-sm text-blue-600 hover:text-blue-800 font-medium hover:underline cursor:pointer"
              >
                Clear All Filters
              </button>
            </div>
          )}

          <div className="space-y-3 flex flex-wrap items-start gap-2">
            {Object.entries(groupedTags)
              .sort(([a], [b]) => a.localeCompare(b))
              .map(([tagKey, tagValues]) => (
                <div key={tagKey} className="bg-gray-100 p-2 rounded-sm">
                  <span className="block text-sm font-medium text-gray-700 px-1 pb-2">
                    {tagKey}
                  </span>
                  <div className="flex flex-wrap items-center gap-2">
                    {tagValues.map((value) => {
                      const fullTag = `${tagKey}:${value}`;
                      return (
                        <button
                          key={fullTag}
                          onClick={() => handleTagToggle(fullTag)}
                          className={`inline-flex items-center px-3 py-1 rounded-xs text-sm font-medium transition-colors ${
                            selectedTags.includes(fullTag)
                              ? 'bg-blue-600 text-white'
                              : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
                          }`}
                        >
                          {value}
                        </button>
                      );
                    })}
                  </div>
                </div>
              ))}
          </div>
        </div>
      )}
    </div>
  );
}
