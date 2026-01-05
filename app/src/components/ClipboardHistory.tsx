/**
 * ClipboardHistory component
 *
 * Displays clipboard history items with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 * Uses CSS variables for sizing to support theme-based scaling.
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
  /** Called when user wants to switch to next tab */
  onSwitchToNextTab?: () => void;
  /** Called when ArrowUp is pressed at first item (to focus search bar) */
  onUpAtTop?: () => void;
  /** Ref for the list container (for focus management) */
  listContainerRef?: React.RefObject<HTMLElement | null>;
  /** Set of item indices that are pinned (will be moved to snippets on window close) */
  pendingPins?: Set<number>;
  /** Callback when star button is clicked */
  onTogglePin?: (index: number) => void;
}

/** Inline styles using CSS variables for theme-based sizing */
const listStyles: React.CSSProperties = {
  gap: 'var(--size-gap)',
  padding: 'var(--size-gap)',
};

const emptyStyles: React.CSSProperties = {
  fontSize: 'var(--size-font-base)',
  padding: 'calc(var(--size-padding-x) * 2)',
};

/**
 * Star icon SVG component (filled version for pinned state)
 * Uses em units for size to match text size
 */
function StarIconFilled() {
  return (
    <svg width="1.25em" height="1.25em" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
  );
}

/**
 * Star icon SVG component (outline version for unpinned state)
 * Uses em units for size to match text size
 */
function StarIconOutline() {
  return (
    <svg
      width="1.25em"
      height="1.25em"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      aria-hidden="true"
    >
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
  );
}

interface StarButtonProps {
  isPinned: boolean;
  onClick: () => void;
  isVisible: boolean;
}

/**
 * Star button component with hover/focus visibility
 * Uses span with role="button" to avoid invalid button nesting in ListItem
 */
function StarButton({ isPinned, onClick, isVisible }: StarButtonProps) {
  return (
    // biome-ignore lint/a11y/useSemanticElements: Cannot use button to avoid invalid nesting (button inside ListItem button)
    <span
      role="button"
      tabIndex={-1}
      aria-label={isPinned ? 'Unpin from snippets' : 'Pin to snippets'}
      onClick={e => {
        e.stopPropagation();
        onClick();
      }}
      onKeyDown={e => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          e.stopPropagation();
          onClick();
        }
      }}
      className={`inline-flex items-center rounded transition-opacity cursor-pointer hover:bg-app-item-hover ${
        isPinned ? 'text-app-accent' : 'text-app-text-muted hover:text-app-text'
      } ${isVisible || isPinned ? 'opacity-100' : 'opacity-0'}`}
    >
      {isPinned ? <StarIconFilled /> : <StarIconOutline />}
    </span>
  );
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
  onSwitchToNextTab,
  onUpAtTop,
  listContainerRef,
  pendingPins,
  onTogglePin,
}: ClipboardHistoryProps) {
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const internalContainerRef = useRef<HTMLElement | null>(null);
  const containerRef = listContainerRef ?? internalContainerRef;
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  const { selectedIndex, handleKeyDown: baseHandleKeyDown } = useListKeyboardNavigation(items, {
    onSelect: item => onSelect(item.text),
    onRight: onSwitchToNextTab,
    onUpAtTop,
    wrapAround: false,
    containerRef,
  });

  /**
   * Extended keyboard handler with number key and star toggle support
   */
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Star toggle with S key
      if (e.key === 's' || e.key === 'S') {
        if (onTogglePin && items.length > 0) {
          e.preventDefault();
          onTogglePin(selectedIndex);
          return;
        }
      }
      // Number key selection (1-9)
      if (e.key >= '1' && e.key <= '9') {
        const index = Number.parseInt(e.key, 10) - 1;
        if (index < items.length) {
          e.preventDefault();
          onSelect(items[index].text);
          return;
        }
      }
      // Fall through to base handler
      baseHandleKeyDown(e);
    },
    [items, onSelect, baseHandleKeyDown, onTogglePin, selectedIndex]
  );

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

  const setItemRef = useCallback(
    (index: number) => (el: HTMLButtonElement | null) => {
      itemRefs.current[index] = el;
    },
    []
  );

  if (items.length === 0) {
    return (
      // biome-ignore lint/a11y/noStaticElementInteractions: Keyboard navigation requires handler
      <div
        ref={containerRef as React.RefObject<HTMLDivElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="text-center text-app-text-muted focus:outline-none"
        style={emptyStyles}
      >
        No clipboard history
      </div>
    );
  }

  return (
    <ul
      ref={containerRef as React.RefObject<HTMLUListElement | null>}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="h-full flex flex-col focus:outline-none overflow-y-auto"
      style={listStyles}
    >
      {items.map((item, index) => {
        const isPinned = pendingPins?.has(index) ?? false;
        const isVisible = hoveredIndex === index || selectedIndex === index;
        return (
          <li key={`${item.timestamp}-${index}`}>
            <ListItem
              selected={index === selectedIndex}
              index={index}
              title={item.text}
              onClick={() => onSelect(item.text)}
              onMouseEnter={() => setHoveredIndex(index)}
              onMouseLeave={() => setHoveredIndex(null)}
              buttonRef={setItemRef(index)}
              dataAttributes={{ 'data-clipboard-item': true }}
              suffix={
                onTogglePin ? (
                  <StarButton
                    isPinned={isPinned}
                    onClick={() => onTogglePin(index)}
                    isVisible={isVisible}
                  />
                ) : undefined
              }
            >
              {item.text}
            </ListItem>
          </li>
        );
      })}
    </ul>
  );
}
