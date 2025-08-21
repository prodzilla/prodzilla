import { useState, useEffect, ReactNode } from 'react';
import {
  fetchProbes,
  fetchStories,
  fetchProbeResults,
  fetchStoryResults,
} from '@/lib/api-client';
import {
  DashboardContext,
  MonitorItem,
  SelectedMonitor,
} from '../context/DashboardContext';

type DashboardProviderProps = {
  children: ReactNode;
};

export function DashboardProvider({ children }: DashboardProviderProps) {
  const [probes, setProbes] = useState<MonitorItem[]>([]);
  const [stories, setStories] = useState<MonitorItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  // Sidebar state
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [selectedMonitor, setSelectedMonitor] = useState<SelectedMonitor | null>(null);
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

  const value = {
    // Data state
    probes,
    stories,
    loading,
    
    // Filter state
    searchTerm,
    selectedTags,
    setSearchTerm,
    
    // Sidebar state
    sidebarOpen,
    selectedMonitor,
    results,
    resultsLoading,
    resultsError,
    
    // Actions
    handleTagToggle,
    handleClearFilters,
    handleMonitorClick,
    handleCloseSidebar,
  };

  return (
    <DashboardContext.Provider value={value}>
      {children}
    </DashboardContext.Provider>
  );
}