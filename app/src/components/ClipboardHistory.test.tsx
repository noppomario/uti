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

    expect(screen.getByText('Item 1')).toBeDefined();
    expect(screen.getByText('Item 2')).toBeDefined();
    expect(screen.getByText('Item 3')).toBeDefined();
  });

  it('should call onSelect with item text on click', () => {
    const onSelect = vi.fn();
    render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const item = screen.getByText('Item 2');
    fireEvent.click(item);

    expect(onSelect).toHaveBeenCalledWith('Item 2');
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it('should highlight first item by default', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const firstItem = container.querySelector('[data-selected="true"]');
    expect(firstItem?.textContent).toContain('Item 1');
  });

  it('should navigate down with ArrowDown key', () => {
    const onSelect = vi.fn();
    const { container } = render(<ClipboardHistory items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selectedItem = container.querySelector('[data-selected="true"]');
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
});
