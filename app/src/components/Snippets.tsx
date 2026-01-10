/**
 * Snippets component
 *
 * Displays snippet items (pinned clipboard text) with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 * Uses CSS variables for sizing to support theme-based scaling.
 */
import type React from 'react';
import { useCallback, useEffect, useRef } from 'react';
import { useListKeyboardNavigation } from '../hooks/useListKeyboardNavigation';
import { ListItem } from './ListItem';

export interface SnippetItem {
  id: string;
  label: string | null;
  value: string;
}

export interface SnippetsProps {
  /** List of snippet items to display */
  items: SnippetItem[];
  /** Callback when an item is selected */
  onSelect: (value: string) => void;
  /** Called when user wants to switch to next tab */
  onSwitchToNextTab?: () => void;
  /** Called when user wants to switch to previous tab */
  onSwitchToPrevTab?: () => void;
  /** Called when ArrowUp is pressed at first item (to focus search bar) */
  onUpAtTop?: () => void;
  /** Ref for the list container (for focus management) */
  listContainerRef?: React.RefObject<HTMLElement | null>;
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
 * Get display text for a snippet item
 *
 * Returns label if present and non-empty, otherwise returns value.
 */
function getDisplayText(item: SnippetItem): string {
  if (item.label && item.label.trim().length > 0) {
    return item.label;
  }
  return item.value;
}

/**
 * Renders snippets with keyboard navigation
 *
 * @example
 * ```tsx
 * <Snippets
 *   items={snippets}
 *   onSelect={(value) => copyToClipboard(value)}
 * />
 * ```
 */
export function Snippets({
  items,
  onSelect,
  onSwitchToNextTab,
  onSwitchToPrevTab,
  onUpAtTop,
  listContainerRef,
}: SnippetsProps) {
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const internalContainerRef = useRef<HTMLElement | null>(null);
  const containerRef = listContainerRef ?? internalContainerRef;

  const { selectedIndex, handleKeyDown: baseHandleKeyDown } = useListKeyboardNavigation(items, {
    onSelect: item => onSelect(item.value),
    onRight: onSwitchToNextTab,
    onLeft: onSwitchToPrevTab,
    onUpAtTop,
    wrapAround: false,
    containerRef,
  });

  /**
   * Extended keyboard handler with number key support
   */
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Number key selection (1-9)
      if (e.key >= '1' && e.key <= '9') {
        const index = Number.parseInt(e.key, 10) - 1;
        if (index < items.length) {
          e.preventDefault();
          onSelect(items[index].value);
          return;
        }
      }
      // Fall through to base handler
      baseHandleKeyDown(e);
    },
    [items, onSelect, baseHandleKeyDown]
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
        No snippets
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
      {items.map((item, index) => (
        <li key={item.id}>
          <ListItem
            selected={index === selectedIndex}
            index={index}
            title={item.value}
            onClick={() => onSelect(item.value)}
            buttonRef={setItemRef(index)}
            dataAttributes={{ 'data-snippet-item': true }}
          >
            {getDisplayText(item)}
          </ListItem>
        </li>
      ))}
    </ul>
  );
}
