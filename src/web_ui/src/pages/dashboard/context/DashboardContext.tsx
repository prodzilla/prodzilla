import { createContext, useContext } from 'react';

export type MonitorItem = {
  name: string;
  status: string;
  last_probed: string;
  tags?: Record<string, string> | null;
};

export type SelectedMonitor = {
  name: string;
  type: 'probe' | 'story';
};

export type DashboardContextType = {
  // Data state
  probes: MonitorItem[];
  stories: MonitorItem[];
  loading: boolean;
  
  // Filter state
  searchTerm: string;
  selectedTags: string[];
  setSearchTerm: (term: string) => void;
  
  // Sidebar state
  sidebarOpen: boolean;
  selectedMonitor: SelectedMonitor | null;
  results: any[] | null;
  resultsLoading: boolean;
  resultsError: string | null;
  
  // Actions
  handleTagToggle: (tag: string) => void;
  handleClearFilters: () => void;
  handleMonitorClick: (name: string, type: 'probe' | 'story') => void;
  handleCloseSidebar: () => void;
};

export const DashboardContext = createContext<DashboardContextType | null>(null);

export function useDashboardContext() {
  const context = useContext(DashboardContext);
  if (!context) {
    throw new Error('useDashboardContext must be used within a DashboardProvider');
  }
  return context;
}