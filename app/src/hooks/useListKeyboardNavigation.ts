/**
 * Hook for keyboard navigation in list components
 *
 * Provides common keyboard navigation logic for lists:
 * - Arrow up/down for item selection
 * - Enter for item activation
 * - Optional arrow left/right/escape handlers
 */

import { useCallback, useEffect, useRef, useState } from 'react';

export interface UseListKeyboardNavigationOptions<T> {
  /** Called when Enter is pressed on selected item */
  onSelect?: (item: T, index: number) => void;
  /** Called when ArrowLeft is pressed */
  onLeft?: () => void;
  /** Called when ArrowRight is pressed on selected item */
  onRight?: (item: T, index: number) => void;
  /** Called when Escape is pressed */
  onEscape?: () => void;
  /** Called when ArrowUp is pressed at first item (to move focus to search bar) */
  onUpAtTop?: () => void;
  /** Whether navigation wraps around at ends (default: false) */
  wrapAround?: boolean;
  /** External container ref for focus management (if not provided, internal ref is created) */
  containerRef?: React.RefObject<HTMLElement | null>;
}

export interface UseListKeyboardNavigationResult {
  /** Currently selected index */
  selectedIndex: number;
  /** Set the selected index */
  setSelectedIndex: React.Dispatch<React.SetStateAction<number>>;
  /** Ref for the container element */
  containerRef: React.RefObject<HTMLElement | null>;
  /** Keyboard event handler for the container */
  handleKeyDown: (e: React.KeyboardEvent) => void;
}

/**
 * Hook for managing keyboard navigation in list components
 *
 * @param items - Array of items to navigate
 * @param options - Navigation options
 * @returns Navigation state and handlers
 *
 * @example
 * ```tsx
 * const { selectedIndex, containerRef, handleKeyDown } = useListKeyboardNavigation(
 *   items,
 *   {
 *     onSelect: (item) => console.log('Selected:', item),
 *     onLeft: () => console.log('Left pressed'),
 *   }
 * );
 * ```
 */
export function useListKeyboardNavigation<T>(
  items: T[],
  options: UseListKeyboardNavigationOptions<T> = {}
): UseListKeyboardNavigationResult {
  const {
    onSelect,
    onLeft,
    onRight,
    onEscape,
    onUpAtTop,
    wrapAround = false,
    containerRef: externalContainerRef,
  } = options;

  const [selectedIndex, setSelectedIndex] = useState(0);
  const internalContainerRef = useRef<HTMLElement | null>(null);
  const containerRef = externalContainerRef ?? internalContainerRef;

  // Focus container on initial mount, but don't steal focus from search input
  // biome-ignore lint/correctness/useExhaustiveDependencies: only run on mount
  useEffect(() => {
    // Use requestAnimationFrame to ensure DOM is fully rendered before focusing
    const frameId = requestAnimationFrame(() => {
      // Don't steal focus if an input element is currently focused (e.g., search bar)
      const isInputFocused = document.activeElement?.tagName === 'INPUT';
      if (!isInputFocused) {
        containerRef.current?.focus();
      }
    });
    return () => cancelAnimationFrame(frameId);
  }, []);

  // Reset selection when items change (don't re-focus to avoid stealing focus from search input)
  // biome-ignore lint/correctness/useExhaustiveDependencies: items change requires reset
  useEffect(() => {
    setSelectedIndex(0);
  }, [items]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const itemCount = items.length;

      switch (e.key) {
        case 'ArrowDown':
          if (itemCount === 0) return;
          e.preventDefault();
          setSelectedIndex(prev =>
            wrapAround ? (prev + 1) % itemCount : Math.min(prev + 1, itemCount - 1)
          );
          break;

        case 'ArrowUp':
          e.preventDefault();
          // Allow navigation to search bar even when list is empty
          if (itemCount === 0) {
            if (onUpAtTop) {
              onUpAtTop();
            }
            return;
          }
          setSelectedIndex(prev => {
            if (prev === 0) {
              if (onUpAtTop) {
                onUpAtTop();
              }
              return wrapAround ? itemCount - 1 : 0;
            }
            return prev - 1;
          });
          break;

        case 'ArrowLeft':
          if (onLeft) {
            e.preventDefault();
            onLeft();
          }
          break;

        case 'ArrowRight':
          if (onRight) {
            e.preventDefault();
            // Allow tab switching even when list is empty
            if (itemCount > 0) {
              onRight(items[selectedIndex], selectedIndex);
            } else {
              // Call with placeholder values for empty list (tab switching uses no args)
              onRight(undefined as T, -1);
            }
          }
          break;

        case 'Escape':
          if (onEscape) {
            e.preventDefault();
            onEscape();
          }
          break;

        case 'Enter':
          if (onSelect && itemCount > 0) {
            e.preventDefault();
            onSelect(items[selectedIndex], selectedIndex);
          }
          break;

        default:
          break;
      }
    },
    [items, selectedIndex, onSelect, onLeft, onRight, onEscape, onUpAtTop, wrapAround]
  );

  return {
    selectedIndex,
    setSelectedIndex,
    containerRef,
    handleKeyDown,
  };
}
