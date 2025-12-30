/**
 * ClipboardHistory component
 *
 * Displays clipboard history items with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 */
import { useEffect, useRef, useState } from 'react';

export interface ClipboardItem {
  text: string;
  timestamp: number;
}

export interface ClipboardHistoryProps {
  /** List of clipboard items to display */
  items: ClipboardItem[];
  /** Callback when an item is selected */
  onSelect: (text: string) => void;
}

/**
 * Renders clipboard history with keyboard navigation
 *
 * @example
 * ```tsx
 * <ClipboardHistory
 *   items={history}
 *   onSelect={(text) => pasteToClipboard(text)}
 * />
 * ```
 */
export function ClipboardHistory({ items, onSelect }: ClipboardHistoryProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLUListElement>(null);

  // Reset selection when items array changes (referential equality)
  useEffect(() => {
    setSelectedIndex(0);
    // Auto-focus the list when items are loaded
    if (items.length > 0 && listRef.current) {
      listRef.current.focus();
    }
  }, [items]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, items.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (items[selectedIndex]) {
          onSelect(items[selectedIndex].text);
        }
        break;
    }
  };

  if (items.length === 0) {
    return <div className="p-4 text-center text-gray-500">No clipboard history</div>;
  }

  return (
    <ul
      ref={listRef}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex flex-col gap-0.5 p-1 focus:outline-none"
    >
      {items.map((item, index) => (
        <li key={`${item.timestamp}-${index}`}>
          <button
            type="button"
            data-clipboard-item
            data-selected={index === selectedIndex}
            onClick={() => onSelect(item.text)}
            className={`
							w-full cursor-pointer rounded px-2 py-1 text-left text-xs
							truncate
							${
                index === selectedIndex
                  ? 'bg-blue-500 text-white'
                  : 'bg-gray-100 text-gray-800 hover:bg-gray-200'
              }
						`}
          >
            {item.text}
          </button>
        </li>
      ))}
    </ul>
  );
}
