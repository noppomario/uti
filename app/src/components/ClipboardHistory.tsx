/**
 * ClipboardHistory component
 *
 * Displays clipboard history items with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 * Shows tooltip at bottom after hovering for a short delay.
 */
import type React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useListKeyboardNavigation } from '../hooks/useListKeyboardNavigation';
import { ListItem } from './ListItem';

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
  /** Called when user wants to switch to next tab */
  onSwitchToNextTab?: () => void;
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
  onSwitchToNextTab,
}: ClipboardHistoryProps) {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const [tooltipIndex, setTooltipIndex] = useState<number | null>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const { selectedIndex, containerRef, handleKeyDown } = useListKeyboardNavigation(items, {
    onSelect: item => onSelect(item.text),
    onRight: onSwitchToNextTab,
    wrapAround: false,
  });

  // Reset hover state when items change
  // biome-ignore lint/correctness/useExhaustiveDependencies: items change requires reset
  useEffect(() => {
    setHoveredIndex(null);
    setTooltipIndex(null);
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

  const setItemRef = useCallback(
    (index: number) => (el: HTMLButtonElement | null) => {
      itemRefs.current[index] = el;
    },
    []
  );

  const tooltipText = tooltipIndex !== null ? items[tooltipIndex]?.text : null;

  if (items.length === 0) {
    return (
      // biome-ignore lint/a11y/noStaticElementInteractions: Keyboard navigation requires handler
      <div
        ref={containerRef as React.RefObject<HTMLDivElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="p-4 text-center text-app-text-muted text-xs focus:outline-none"
      >
        No clipboard history
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Scrollable list */}
      <ul
        ref={containerRef as React.RefObject<HTMLUListElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="flex-1 flex flex-col gap-0.5 p-1 focus:outline-none overflow-y-auto min-h-0"
      >
        {items.map((item, index) => (
          <li key={`${item.timestamp}-${index}`}>
            <ListItem
              selected={index === selectedIndex}
              index={index}
              onClick={() => onSelect(item.text)}
              onMouseEnter={() => setHoveredIndex(index)}
              onMouseLeave={() => setHoveredIndex(null)}
              buttonRef={setItemRef(index)}
              dataAttributes={{ 'data-clipboard-item': true }}
            >
              {item.text}
            </ListItem>
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
