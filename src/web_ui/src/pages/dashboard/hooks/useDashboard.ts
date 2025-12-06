import { useContext } from 'react';
import { DashboardContext } from '../providers/DashboardProvider';

function useDashboardContext() {
  const context = useContext(DashboardContext);
  if (!context) {
    throw new Error(
      'useDashboardContext must be used within a DashboardProvider'
    );
  }
  return context;
}

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
  } as const;
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
  } as const;
}

export function useMonitorActions() {
  const context = useDashboardContext();
  return {
    handleMonitorClick: context.handleMonitorClick,
    handleTagToggle: context.handleTagToggle,
  } as const;
}

export function useSelection() {
  const context = useDashboardContext();
  return {
    selectedItems: context.selectedItems,
    bulkTriggerLoading: context.bulkTriggerLoading,
    bulkTriggerResults: context.bulkTriggerResults,
    handleItemSelect: context.handleItemSelect,
    handleSelectAll: context.handleSelectAll,
    handleClearSelection: context.handleClearSelection,
    handleBulkTrigger: context.handleBulkTrigger,
  } as const;
}
