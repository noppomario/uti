/**
 * Tests for useListKeyboardNavigation hook
 */

import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useListKeyboardNavigation } from './useListKeyboardNavigation';

describe('useListKeyboardNavigation', () => {
  const mockItems = ['item1', 'item2', 'item3'];

  it('initializes with selectedIndex 0', () => {
    const { result } = renderHook(() => useListKeyboardNavigation(mockItems));

    expect(result.current.selectedIndex).toBe(0);
  });

  it('resets selectedIndex when items change', () => {
    const { result, rerender } = renderHook(({ items }) => useListKeyboardNavigation(items), {
      initialProps: { items: mockItems },
    });

    // Change selection
    act(() => {
      result.current.setSelectedIndex(2);
    });
    expect(result.current.selectedIndex).toBe(2);

    // Change items
    rerender({ items: ['new1', 'new2'] });
    expect(result.current.selectedIndex).toBe(0);
  });

  describe('ArrowDown', () => {
    it('moves selection down', () => {
      const { result } = renderHook(() => useListKeyboardNavigation(mockItems));

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowDown',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(1);
    });

    it('stays at last item by default', () => {
      const { result } = renderHook(() => useListKeyboardNavigation(mockItems));

      // Move to last item
      act(() => {
        result.current.setSelectedIndex(2);
      });

      // Press down
      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowDown',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(2);
    });

    it('wraps around when wrapAround is true', () => {
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { wrapAround: true })
      );

      // Move to last item
      act(() => {
        result.current.setSelectedIndex(2);
      });

      // Press down
      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowDown',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(0);
    });

    it('does nothing with empty items', () => {
      const { result } = renderHook(() => useListKeyboardNavigation([]));

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowDown',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(0);
    });
  });

  describe('ArrowUp', () => {
    it('moves selection up', () => {
      const { result } = renderHook(() => useListKeyboardNavigation(mockItems));

      // Start at index 1
      act(() => {
        result.current.setSelectedIndex(1);
      });

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowUp',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(0);
    });

    it('stays at first item by default', () => {
      const { result } = renderHook(() => useListKeyboardNavigation(mockItems));

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowUp',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(result.current.selectedIndex).toBe(0);
    });

    it('calls onUpAtTop when at first item', () => {
      const handleUpAtTop = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onUpAtTop: handleUpAtTop })
      );

      // At first item (index 0)
      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowUp',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleUpAtTop).toHaveBeenCalled();
    });

    it('does not call onUpAtTop when not at first item', () => {
      const handleUpAtTop = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onUpAtTop: handleUpAtTop })
      );

      // Move to index 1
      act(() => {
        result.current.setSelectedIndex(1);
      });

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowUp',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleUpAtTop).not.toHaveBeenCalled();
      expect(result.current.selectedIndex).toBe(0);
    });

    it('calls onUpAtTop with empty items to allow navigation to search bar', () => {
      const handleUpAtTop = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation([], { onUpAtTop: handleUpAtTop })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowUp',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleUpAtTop).toHaveBeenCalled();
    });
  });

  describe('Enter', () => {
    it('calls onSelect with selected item', () => {
      const handleSelect = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onSelect: handleSelect })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'Enter',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleSelect).toHaveBeenCalledWith('item1', 0);
    });

    it('does not call onSelect with empty items', () => {
      const handleSelect = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation([], { onSelect: handleSelect })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'Enter',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleSelect).not.toHaveBeenCalled();
    });
  });

  describe('ArrowLeft', () => {
    it('calls onLeft when provided', () => {
      const handleLeft = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onLeft: handleLeft })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowLeft',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleLeft).toHaveBeenCalled();
    });

    it('works even with empty items', () => {
      const handleLeft = vi.fn();
      const { result } = renderHook(() => useListKeyboardNavigation([], { onLeft: handleLeft }));

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowLeft',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleLeft).toHaveBeenCalled();
    });
  });

  describe('ArrowRight', () => {
    it('calls onRight with selected item', () => {
      const handleRight = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onRight: handleRight })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowRight',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleRight).toHaveBeenCalledWith('item1', 0);
    });

    it('does not call onRight with empty items', () => {
      const handleRight = vi.fn();
      const { result } = renderHook(() => useListKeyboardNavigation([], { onRight: handleRight }));

      act(() => {
        result.current.handleKeyDown({
          key: 'ArrowRight',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleRight).not.toHaveBeenCalled();
    });
  });

  describe('Escape', () => {
    it('calls onEscape when provided', () => {
      const handleEscape = vi.fn();
      const { result } = renderHook(() =>
        useListKeyboardNavigation(mockItems, { onEscape: handleEscape })
      );

      act(() => {
        result.current.handleKeyDown({
          key: 'Escape',
          preventDefault: vi.fn(),
        } as unknown as React.KeyboardEvent);
      });

      expect(handleEscape).toHaveBeenCalled();
    });
  });
});
