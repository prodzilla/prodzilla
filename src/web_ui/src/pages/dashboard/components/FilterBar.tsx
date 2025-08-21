type FilterBarProps = {
  searchTerm: string;
  onSearchChange: (term: string) => void;
  selectedTags: string[];
  availableTags: string[];
  onTagToggle: (tag: string) => void;
  onClearFilters: () => void;
};

export default function FilterBar({
  searchTerm,
  onSearchChange,
  selectedTags,
  availableTags,
  onTagToggle,
  onClearFilters,
}: FilterBarProps) {
  return (
    <div className="bg-white rounded-lg shadow p-4 mb-6">
      <div className="mb-4">
        <input
          type="text"
          placeholder="Search monitors by name..."
          value={searchTerm}
          onChange={(e) => onSearchChange(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {availableTags.length > 0 && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <span className="text-sm font-medium text-gray-800">
              Refine by Tags
            </span>
            {selectedTags.length > 0 && (
              <button
                onClick={onClearFilters}
                className="text-sm text-blue-600 hover:text-blue-800 font-medium"
              >
                Clear All Filters
              </button>
            )}
          </div>
          
          <div className="flex flex-wrap gap-2">
            {availableTags.map((tag) => (
              <button
                key={tag}
                onClick={() => onTagToggle(tag)}
                className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                  selectedTags.includes(tag)
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
                }`}
              >
                {tag}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
