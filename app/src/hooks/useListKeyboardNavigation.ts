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
  /** Whether navigation wraps around at ends (default: false) */
  wrapAround?: boolean;
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
  const { onSelect, onLeft, onRight, onEscape, wrapAround = false } = options;

  const [selectedIndex, setSelectedIndex] = useState(0);
  const containerRef = useRef<HTMLElement | null>(null);

  // Reset selection and focus when items change
  // biome-ignore lint/correctness/useExhaustiveDependencies: items change requires reset and focus
  useEffect(() => {
    setSelectedIndex(0);
    // Focus container when items are available
    containerRef.current?.focus();
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
          if (itemCount === 0) return;
          e.preventDefault();
          setSelectedIndex(prev =>
            wrapAround ? (prev - 1 + itemCount) % itemCount : Math.max(prev - 1, 0)
          );
          break;

        case 'ArrowLeft':
          if (onLeft) {
            e.preventDefault();
            onLeft();
          }
          break;

        case 'ArrowRight':
          if (onRight && itemCount > 0) {
            e.preventDefault();
            onRight(items[selectedIndex], selectedIndex);
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
    [items, selectedIndex, onSelect, onLeft, onRight, onEscape, wrapAround]
  );

  return {
    selectedIndex,
    setSelectedIndex,
    containerRef,
    handleKeyDown,
  };
}
