import { filterByTags, searchByName } from '@/lib/helpers';
import MonitorCard from './components/MonitorCard';
import FilterBar from './components/FilterBar';
import ResultsSidebar from './components/ResultsSidebar';
import { DashboardProvider } from './providers/DashboardProvider';
import { useDashboard, useTagFilters } from './hooks/useDashboard';

function DashboardContent() {
  const { probes, stories, loading } = useDashboard();
  const { searchTerm, selectedTags } = useTagFilters();

  const allItems = [
    ...probes.map((p) => ({ ...p, type: 'probe' as const })),
    ...stories.map((s) => ({ ...s, type: 'story' as const })),
  ];

  const filteredItems = filterByTags(
    searchByName(allItems, searchTerm),
    selectedTags
  );

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-lg text-gray-600">Loading monitors...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Prodzilla Dashboard
          </h1>
        </div>

        <FilterBar allItems={allItems} />

        <div className="mb-6 text-gray-600">
          {filteredItems.length === allItems.length ? (
            <span>Showing all {allItems.length} monitors</span>
          ) : (
            <span>
              Showing {filteredItems.length} of {allItems.length} monitors
            </span>
          )}
        </div>

        {filteredItems.length === 0 ? (
          <div className="text-center py-12">
            <div className="text-gray-500 text-lg">
              {allItems.length === 0
                ? 'No monitors configured'
                : 'No monitors match your filters'}
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredItems.map((item) => (
              <MonitorCard
                key={`${item.type}-${item.name}`}
                item={item}
              />
            ))}
          </div>
        )}
      </div>

      <ResultsSidebar />
    </div>
  );
}

export default function Dashboard() {
  return (
    <DashboardProvider>
      <DashboardContent />
    </DashboardProvider>
  );
}
