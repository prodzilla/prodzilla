import { useState, useEffect } from 'react';
import {
  fetchProbes,
  fetchStories,
  fetchProbeResults,
  fetchStoryResults,
} from '@/lib/api-client';
import { filterByTags, searchByName, getGroupedTags } from '@/lib/helpers';
import MonitorCard from './components/MonitorCard';
import FilterBar from './components/FilterBar';
import ResultsSidebar from './components/ResultsSidebar';

type MonitorItem = {
  name: string;
  status: string;
  last_probed: string;
  tags?: Record<string, string> | null;
};

type SelectedMonitor = {
  name: string;
  type: 'probe' | 'story';
};

export default function Dashboard() {
  const [probes, setProbes] = useState<MonitorItem[]>([]);
  const [stories, setStories] = useState<MonitorItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  // Sidebar state
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [selectedMonitor, setSelectedMonitor] =
    useState<SelectedMonitor | null>(null);
  const [results, setResults] = useState<any[] | null>(null);
  const [resultsLoading, setResultsLoading] = useState(false);
  const [resultsError, setResultsError] = useState<string | null>(null);

  useEffect(() => {
    const loadData = async () => {
      try {
        const [probesData, storiesData] = await Promise.all([
          fetchProbes(),
          fetchStories(),
        ]);
        setProbes(probesData);
        setStories(storiesData);
      } catch (error) {
        console.error('Failed to load data:', error);
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, []);

  const allItems = [
    ...probes.map((p) => ({ ...p, type: 'probe' as const })),
    ...stories.map((s) => ({ ...s, type: 'story' as const })),
  ];

  const groupedTags = getGroupedTags(allItems);

  const filteredItems = filterByTags(
    searchByName(allItems, searchTerm),
    selectedTags
  );

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
    );
  };

  const handleClearFilters = () => {
    setSearchTerm('');
    setSelectedTags([]);
  };

  const handleMonitorClick = async (name: string, type: 'probe' | 'story') => {
    setSelectedMonitor({ name, type });
    setSidebarOpen(true);
    setResultsLoading(true);
    setResultsError(null);
    setResults(null);

    try {
      const data =
        type === 'probe'
          ? await fetchProbeResults(name)
          : await fetchStoryResults(name);
      setResults(data);
    } catch (error) {
      setResultsError('Failed to fetch results');
      console.error('Error fetching results:', error);
    } finally {
      setResultsLoading(false);
    }
  };

  const handleCloseSidebar = () => {
    setSidebarOpen(false);
    setSelectedMonitor(null);
    setResults(null);
    setResultsError(null);
  };

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

        <FilterBar
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          selectedTags={selectedTags}
          groupedTags={groupedTags}
          onTagToggle={handleTagToggle}
          onClearFilters={handleClearFilters}
        />

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
                name={item.name}
                status={item.status}
                last_probed={item.last_probed}
                tags={item.tags}
                type={item.type}
                onTagClick={handleTagToggle}
                onClick={() => handleMonitorClick(item.name, item.type)}
              />
            ))}
          </div>
        )}
      </div>

      <ResultsSidebar
        isOpen={sidebarOpen}
        onClose={handleCloseSidebar}
        selectedMonitor={selectedMonitor}
        results={results}
        loading={resultsLoading}
        error={resultsError}
      />
    </div>
  );
}
