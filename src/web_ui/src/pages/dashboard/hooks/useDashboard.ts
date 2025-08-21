import { useDashboardContext } from '../context/DashboardContext';

export function useDashboard() {
  return useDashboardContext();
}

export function useTagFilters() {
  const context = useDashboardContext();
  return {
    searchTerm: context.searchTerm,
    selectedTags: context.selectedTags,
    setSearchTerm: context.setSearchTerm,
    handleTagToggle: context.handleTagToggle,
    handleClearFilters: context.handleClearFilters,
  };
}

export function useSidebar() {
  const context = useDashboardContext();
  return {
    sidebarOpen: context.sidebarOpen,
    selectedMonitor: context.selectedMonitor,
    results: context.results,
    resultsLoading: context.resultsLoading,
    resultsError: context.resultsError,
    handleCloseSidebar: context.handleCloseSidebar,
  };
}

export function useMonitorActions() {
  const context = useDashboardContext();
  return {
    handleMonitorClick: context.handleMonitorClick,
    handleTagToggle: context.handleTagToggle,
  };
}