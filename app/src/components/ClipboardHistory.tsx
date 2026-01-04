/**
 * ClipboardHistory component
 *
 * Displays clipboard history items with keyboard navigation support.
 * Users can select items with mouse click or keyboard (ArrowUp/Down + Enter).
 * Uses CSS variables for sizing to support theme-based scaling.
 */
import type React from 'react';
import { useCallback, useEffect, useRef } from 'react';
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
}: ClipboardHistoryProps) {
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const internalContainerRef = useRef<HTMLElement | null>(null);
  const containerRef = listContainerRef ?? internalContainerRef;

  const { selectedIndex, handleKeyDown } = useListKeyboardNavigation(items, {
    onSelect: item => onSelect(item.text),
    onRight: onSwitchToNextTab,
    onUpAtTop,
    wrapAround: false,
    containerRef,
  });

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
      {items.map((item, index) => (
        <li key={`${item.timestamp}-${index}`}>
          <ListItem
            selected={index === selectedIndex}
            index={index}
            title={item.text}
            onClick={() => onSelect(item.text)}
            buttonRef={setItemRef(index)}
            dataAttributes={{ 'data-clipboard-item': true }}
          >
            {item.text}
          </ListItem>
        </li>
      ))}
    </ul>
  );
}
