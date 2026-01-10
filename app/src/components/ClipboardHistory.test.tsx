/**
 * Tests for ClipboardHistory component
 *
 * Displays clipboard history with keyboard navigation support.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ClipboardHistory } from './ClipboardHistory';

describe('ClipboardHistory', () => {
  const mockItems = [
    { text: 'Item 1', timestamp: 1735567200 },
    { text: 'Item 2', timestamp: 1735567100 },
    { text: 'Item 3', timestamp: 1735567000 },
  ];

  it('should render empty state when no items', () => {
    const onSelect = vi.fn();
    render(<ClipboardHistory items={[]} onSelect={onSelect} />);

    expect(screen.getByText(/no clipboard history/i)).toBeDefined();
  });

  it('should render all clipboard items', () => {
    const onSelect = vi.fn();
    render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    expect(screen.getByText(/1: Item 1/)).toBeDefined();
    expect(screen.getByText(/2: Item 2/)).toBeDefined();
    expect(screen.getByText(/3: Item 3/)).toBeDefined();
  });

  it('should call onSelect with item text on click', () => {
    const onSelect = vi.fn();
    render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const item = screen.getByText(/2: Item 2/);
    fireEvent.click(item);

    expect(onSelect).toHaveBeenCalledWith('Item 2');
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it('should highlight first item by default', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const firstItem = container.querySelector('[data-selected="true"]');
    expect(firstItem?.textContent).toContain('1:');
    expect(firstItem?.textContent).toContain('Item 1');
  });

  it('should navigate down with ArrowDown key', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('2:');
    expect(selectedItem?.textContent).toContain('Item 2');
  });

  it('should navigate up with ArrowUp key', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to second item
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    // Move back to first
    fireEvent.keyDown(list, { key: 'ArrowUp' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('1:');
    expect(selectedItem?.textContent).toContain('Item 1');
  });

  it('should not go below first item with ArrowUp', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Try to go above first item
    fireEvent.keyDown(list, { key: 'ArrowUp' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('1:');
    expect(selectedItem?.textContent).toContain('Item 1');
  });

  it('should not go beyond last item with ArrowDown', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to last item
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    // Try to go beyond
    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('3:');
    expect(selectedItem?.textContent).toContain('Item 3');
  });

  it('should call onSelect with selected item on Enter key', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to second item and select
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    fireEvent.keyDown(list, { key: 'Enter' });

    expect(onSelect).toHaveBeenCalledWith('Item 2');
  });

  it('should truncate long text with ellipsis', () => {
    const longText = 'A'.repeat(100);
    const onSelect = vi.fn();
    const { container } = render(
      <ClipboardHistory items={[{ text: longText, timestamp: 123 }]} onSelect={onSelect} />
    );

    const item = container.querySelector('[data-clipboard-item]');
    expect(item?.classList.contains('truncate')).toBe(true);
  });

  it('should display item numbers in format "1: text"', () => {
    const onSelect = vi.fn();
    render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    expect(screen.getByText(/1:/)).toBeDefined();
    expect(screen.getByText(/2:/)).toBeDefined();
    expect(screen.getByText(/3:/)).toBeDefined();
  });

  it('should have semantic color classes for selected items', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.className).toContain('bg-app-item-selected');
    expect(selectedItem?.className).toContain('text-app-text-on-selected');
  });

  it('should have semantic color classes for unselected items', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const unselectedItem = container.querySelector('[data-selected="false"]');
    expect(unselectedItem?.className).toContain('bg-app-item');
    expect(unselectedItem?.className).toContain('text-app-text');
    expect(unselectedItem?.className).toContain('hover:bg-app-item-hover');
  });

  it('should scroll selected item into view on keyboard navigation', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Mock scrollIntoView on the button elements
    const scrollIntoViewMock = vi.fn();
    const buttons = container.querySelectorAll('[data-clipboard-item]');
    for (const button of buttons) {
      button.scrollIntoView = scrollIntoViewMock;
    }

    // Navigate down
    fireEvent.keyDown(list, { key: 'ArrowDown' });

    expect(scrollIntoViewMock).toHaveBeenCalledWith({
      block: 'nearest',
      behavior: 'smooth',
    });
  });

  it('should have title attribute for tooltip', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const secondItem = container.querySelectorAll('[data-clipboard-item]')[1];
    expect(secondItem.getAttribute('title')).toBe('Item 2');
  });

  describe('star button (pin to snippets)', () => {
    it('should show star button when onTogglePin is provided', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const starButtons = container.querySelectorAll('[aria-label="Pin to snippets"]');
      expect(starButtons.length).toBe(3);
    });

    it('should not show star button when onTogglePin is not provided', () => {
      const onSelect = vi.fn();
      const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

      const starButtons = container.querySelectorAll('[aria-label="Pin to snippets"]');
      expect(starButtons.length).toBe(0);
    });

    it('should show pinned state when item is in pendingPins', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set([1])} // Second item is pinned
        />
      );

      const pinnedButtons = container.querySelectorAll('[aria-label="Unpin from snippets"]');
      expect(pinnedButtons.length).toBe(1);
      expect(pinnedButtons[0]?.className).toContain('text-app-accent');
    });

    it('should call onTogglePin with index when star button is clicked', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const starButtons = container.querySelectorAll('[aria-label="Pin to snippets"]');
      fireEvent.click(starButtons[1]); // Click second item's star button

      expect(onTogglePin).toHaveBeenCalledWith(1);
      expect(onTogglePin).toHaveBeenCalledTimes(1);
    });

    it('should not trigger onSelect when star button is clicked', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const starButtons = container.querySelectorAll('[aria-label="Pin to snippets"]');
      fireEvent.click(starButtons[0]);

      expect(onSelect).not.toHaveBeenCalled();
      expect(onTogglePin).toHaveBeenCalledWith(0);
    });

    it('should toggle star with S key on selected item', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      // Press S on first item (selected by default)
      fireEvent.keyDown(list, { key: 's' });

      expect(onTogglePin).toHaveBeenCalledWith(0);
      expect(onTogglePin).toHaveBeenCalledTimes(1);
    });

    it('should toggle star with uppercase S key', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={mockItems}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      // Navigate to second item then press S
      fireEvent.keyDown(list, { key: 'ArrowDown' });
      fireEvent.keyDown(list, { key: 'S' });

      expect(onTogglePin).toHaveBeenCalledWith(1);
    });

    it('should not call onTogglePin with S key when onTogglePin is not provided', () => {
      const onSelect = vi.fn();
      const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      // Should not throw and should fall through to other handlers
      fireEvent.keyDown(list, { key: 's' });

      expect(onSelect).not.toHaveBeenCalled();
    });

    it('should not call onTogglePin with S key when items are empty', () => {
      const onSelect = vi.fn();
      const onTogglePin = vi.fn();
      const { container } = render(
        <ClipboardHistory
          items={[]}
          onSelect={onSelect}
          onTogglePin={onTogglePin}
          pendingPins={new Set()}
        />
      );

      const emptyDiv = container.querySelector('div');
      if (!emptyDiv) throw new Error('Empty state div not found');

      fireEvent.keyDown(emptyDiv, { key: 's' });

      expect(onTogglePin).not.toHaveBeenCalled();
    });
  });

  describe('number key selection', () => {
    it('should select item with number key 1-9', () => {
      const onSelect = vi.fn();
      const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '2' });

      expect(onSelect).toHaveBeenCalledWith('Item 2');
    });

    it('should not select item if number exceeds item count', () => {
      const onSelect = vi.fn();
      const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '9' });

      expect(onSelect).not.toHaveBeenCalled();
    });

    it('should select first item with key 1', () => {
      const onSelect = vi.fn();
      const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '1' });

      expect(onSelect).toHaveBeenCalledWith('Item 1');
    });
  });
});
