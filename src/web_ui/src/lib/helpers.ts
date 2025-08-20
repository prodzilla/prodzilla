export function filterByTags<T extends { tags?: Record<string, string> | null }>(
  items: T[],
  selectedTags: string[]
): T[] {
  if (selectedTags.length === 0) return items;
  
  return items.filter(item => {
    if (!item.tags) return false;
    const itemTags = Object.entries(item.tags).map(([key, value]) => `${key}:${value}`);
    return selectedTags.some(tag => itemTags.includes(tag));
  });
}

export function searchByName<T extends { name: string }>(
  items: T[],
  searchTerm: string
): T[] {
  if (!searchTerm.trim()) return items;
  
  const term = searchTerm.toLowerCase();
  return items.filter(item => item.name.toLowerCase().includes(term));
}

export function formatDateTime(dateString: string): string {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  }).format(date);
}

export function getAllTags<T extends { tags?: Record<string, string> | null }>(
  items: T[]
): string[] {
  const tagSet = new Set<string>();
  
  items.forEach(item => {
    if (item.tags) {
      Object.entries(item.tags).forEach(([key, value]) => {
        tagSet.add(`${key}:${value}`);
      });
    }
  });
  
  return Array.from(tagSet).sort();
}