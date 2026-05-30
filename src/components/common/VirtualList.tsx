import React from 'react';
import { List as VirtualizedList } from 'react-window';
import { AutoSizer } from 'react-virtualized-auto-sizer';

interface VirtualListProps<T> {
  items: T[];
  itemHeight: number;
  renderItem: (item: T, index: number, style: React.CSSProperties) => React.ReactNode;
  overscanCount?: number;
  className?: string;
  defaultHeight?: number;
}

export function VirtualList<T>({
  items,
  itemHeight,
  renderItem,
  overscanCount = 5,
  className = '',
  defaultHeight = 500,
}: VirtualListProps<T>) {
  if (items.length === 0) return null;

  return (
    <AutoSizer
      renderProp={({ height, width }) => {
        if (!height || !width) {
          return <div style={{ height: defaultHeight }} />;
        }
        return (
          <VirtualizedList
            style={{ height, width } as React.CSSProperties}
            itemCount={items.length}
            rowHeight={itemHeight}
            overscanCount={overscanCount}
            className={className}
            rowComponent={({ index, style }) => (
              <div style={style}>
                {renderItem(items[index], index, style)}
              </div>
            )}
          />
        );
      }}
    />
  );
}
