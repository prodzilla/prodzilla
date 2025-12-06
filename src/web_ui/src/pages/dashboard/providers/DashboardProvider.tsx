import { useState, useEffect, type ReactNode, createContext } from 'react';
import {
  fetchProbes,
  fetchStories,
  fetchProbeResults,
  fetchStoryResults,
  bulkTriggerProbes,
  bulkTriggerStories,
} from '@/lib/api-client';
import { getCommonTags, filterByTags } from '@/lib/helpers';
import type { ProbeResult, StoryResult, MonitorItem } from '@/lib/api-client';

type BulkResult = Array<ProbeResult | StoryResult>;

type DashboardProviderProps = {
  children: ReactNode;
};

export type SelectedMonitor = {
  name: string;
  type: 'probe' | 'story';
};

function useContextValue() {
  const [probes, setProbes] = useState<MonitorItem[]>([]);
  const [stories, setStories] = useState<MonitorItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  // Selection state
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [bulkTriggerLoading, setBulkTriggerLoading] = useState(false);
  const [bulkTriggerResults, setBulkTriggerResults] = useState<
    BulkResult | undefined
  >(undefined);

  // Sidebar state
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [selectedMonitor, setSelectedMonitor] = useState<
    SelectedMonitor | undefined
  >(undefined);
  const [results, setResults] = useState<
    ProbeResult[] | StoryResult[] | undefined
  >(undefined);
  const [resultsLoading, setResultsLoading] = useState(false);
  const [resultsError, setResultsError] = useState<string | undefined>(
    undefined
  );

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
    setResultsError(undefined);
    setResults(undefined);

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
    setSelectedMonitor(undefined);
    setResults(undefined);
    setResultsError(undefined);
    setBulkTriggerResults(undefined);
  };

  const handleItemSelect = (name: string, type: 'probe' | 'story') => {
    const itemKey = `${type}:${name}`;
    setSelectedItems((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(itemKey)) {
        newSet.delete(itemKey);
      } else {
        newSet.add(itemKey);
      }
      return newSet;
    });
  };

  const handleSelectAll = () => {
    let itemsToSelect: MonitorItem[];
    
    if (selectedTags.length === 0) {
      // No tag filter - select all items
      itemsToSelect = [...probes, ...stories];
    } else {
      // Tag filter active - only select matching items
      const filteredProbes = filterByTags(probes, selectedTags);
      const filteredStories = filterByTags(stories, selectedTags);
      itemsToSelect = [...filteredProbes, ...filteredStories];
    }
    
    const allItems = itemsToSelect.map((item) => {
      const type = probes.includes(item as any) ? 'probe' : 'story';
      return `${type}:${item.name}`;
    });
    
    setSelectedItems(new Set(allItems));
  };

  const handleClearSelection = () => {
    setSelectedItems(new Set());
  };

  const handleBulkTrigger = async () => {
    if (selectedItems.size === 0) return;

    setBulkTriggerLoading(true);
    setBulkTriggerResults(undefined);
    setResultsError(undefined);

    // Extract common tags from selected items
    const selectedProbes = Array.from(selectedItems)
      .filter((item) => item.startsWith('probe:'))
      .map((item) => item.replace('probe:', ''));
    const selectedStories = Array.from(selectedItems)
      .filter((item) => item.startsWith('story:'))
      .map((item) => item.replace('story:', ''));

    try {
      if (selectedProbes.length > 0) {
        const selectedProbeItems = probes.filter((p) =>
          selectedProbes.includes(p.name)
        );
        const commonProbeTags = getCommonTags(selectedProbeItems);
        const probeResults = await bulkTriggerProbes(commonProbeTags);
        setBulkTriggerResults((prev) => [
          ...(prev || []),
          ...probeResults.results,
        ]);
      }

      if (selectedStories.length > 0) {
        const selectedStoryItems = stories.filter((s) =>
          selectedStories.includes(s.name)
        );
        const commonStoryTags = getCommonTags(selectedStoryItems);
        const storyResults = await bulkTriggerStories(commonStoryTags);
        setBulkTriggerResults((prev) => [
          ...(prev || []),
          ...storyResults.results,
        ]);
      }

      setSidebarOpen(true);
      setSelectedMonitor(undefined); // Clear individual monitor selection
    } catch (error) {
      setResultsError('Failed to trigger bulk execution');
      console.error('Error during bulk trigger:', error);
    } finally {
      setBulkTriggerLoading(false);
    }
  };

  return {
    // Data state
    probes,
    stories,
    loading,

    // Filter state
    searchTerm,
    selectedTags,

    // Selection state
    selectedItems,
    bulkTriggerLoading,
    bulkTriggerResults,

    // Sidebar state
    sidebarOpen,
    selectedMonitor,
    results,
    resultsLoading,
    resultsError,

    // Actions
    setSearchTerm,

    handleTagToggle,
    handleClearFilters,
    handleMonitorClick,
    handleCloseSidebar,

    // Selection actions
    handleItemSelect,
    handleSelectAll,
    handleClearSelection,
    handleBulkTrigger,
  } as const;
}

export const DashboardContext = createContext<
  ReturnType<typeof useContextValue> | undefined
>(undefined);

export function DashboardProvider({ children }: DashboardProviderProps) {
  const contextValue = useContextValue();

  return (
    <DashboardContext.Provider value={contextValue}>
      {children}
    </DashboardContext.Provider>
  );
}
