type TagListProps = {
  tags?: Record<string, string> | null;
  onTagClick?: (tag: string) => void;
  clickable?: boolean;
};

export default function TagList({
  tags,
  onTagClick,
  clickable = false,
}: TagListProps) {
  if (!tags || Object.keys(tags).length === 0) {
    return <span className="text-gray-400 text-sm">No tags</span>;
  }

  return (
    <div className="flex flex-wrap gap-1">
      {Object.entries(tags).map(([key, value]) => {
        const tagString = `${key}:${value}`;
        return (
          <span
            key={tagString}
            onClick={clickable ? () => onTagClick?.(tagString) : undefined}
            className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-800 ${
              clickable ? 'cursor-pointer hover:bg-blue-200' : ''
            }`}
          >
            {`${key}: ${value}`}
          </span>
        );
      })}
    </div>
  );
}
