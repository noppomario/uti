/**
 * ClipboardHistory component
 *
 * Displays clipboard history items with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 * Shows tooltip at bottom after hovering for a short delay.
 */
import { useCallback, useEffect, useRef, useState } from 'react';

export interface ClipboardItem {
  text: string;
  timestamp: number;
}

export interface ClipboardHistoryProps {
  /** List of clipboard items to display */
  items: ClipboardItem[];
  /** Callback when an item is selected */
  onSelect: (text: string) => void;
  /** Whether to show tooltip on hover (default: true) */
  showTooltip?: boolean;
  /** Delay before showing tooltip in ms (default: 500) */
  tooltipDelay?: number;
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
export function ClipboardHistory({
  items,
  onSelect,
  showTooltip = true,
  tooltipDelay = 500,
}: ClipboardHistoryProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [tooltipIndex, setTooltipIndex] = useState<number | null>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Reset selection when items array changes (referential equality)
  useEffect(() => {
    setSelectedIndex(0);
    setHoveredIndex(null);
    setTooltipIndex(null);
    // Auto-focus the list when items are loaded
    if (items.length > 0 && listRef.current) {
      listRef.current.focus();
    }
  }, [items]);

  // Scroll selected item into view when selectedIndex changes
  useEffect(() => {
    const selectedElement = itemRefs.current[selectedIndex];
    if (selectedElement) {
      selectedElement.scrollIntoView({
        block: 'nearest',
        behavior: 'smooth',
      });
    }
  }, [selectedIndex]);

  // Handle delayed tooltip display
  useEffect(() => {
    if (!showTooltip) {
      setTooltipIndex(null);
      return;
    }

    if (hoveredIndex !== null) {
      hoverTimerRef.current = setTimeout(() => {
        setTooltipIndex(hoveredIndex);
      }, tooltipDelay);
    } else {
      setTooltipIndex(null);
    }

    return () => {
      if (hoverTimerRef.current) {
        clearTimeout(hoverTimerRef.current);
      }
    };
  }, [hoveredIndex, showTooltip, tooltipDelay]);

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

  const setItemRef = useCallback(
    (index: number) => (el: HTMLButtonElement | null) => {
      itemRefs.current[index] = el;
    },
    []
  );

  const tooltipText = tooltipIndex !== null ? items[tooltipIndex]?.text : null;

  if (items.length === 0) {
    return <div className="p-4 text-center text-app-text-muted">No clipboard history</div>;
  }

  return (
    <div className="flex flex-col h-full">
      {/* Scrollable list */}
      <ul
        ref={listRef}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="flex-1 flex flex-col gap-0.5 p-1 focus:outline-none overflow-y-auto min-h-0"
      >
        {items.map((item, index) => (
          <li key={`${item.timestamp}-${index}`}>
            <button
              ref={setItemRef(index)}
              type="button"
              data-clipboard-item
              data-selected={index === selectedIndex}
              onClick={() => onSelect(item.text)}
              onMouseEnter={() => setHoveredIndex(index)}
              onMouseLeave={() => setHoveredIndex(null)}
              className={`
                w-full cursor-pointer rounded px-2 py-1 text-left text-xs
                truncate
                ${
                  index === selectedIndex
                    ? 'bg-app-item-selected text-app-text-on-selected'
                    : 'bg-app-item text-app-text hover:bg-app-item-hover'
                }
              `}
            >
              {index + 1}:&nbsp;&nbsp;&nbsp;&nbsp;{item.text}
            </button>
          </li>
        ))}
      </ul>

      {/* Fixed tooltip area at bottom */}
      {tooltipText && (
        <div
          data-tooltip
          className="shrink-0 mx-1 mb-1 px-2 py-1.5 rounded
            bg-slate-900 text-slate-100
            text-xs leading-normal
            max-h-24 overflow-y-auto whitespace-pre-wrap break-all"
        >
          {tooltipText}
        </div>
      )}
    </div>
  );
}
