/**
 * Tests for Snippets component
 *
 * Displays snippet items with keyboard navigation support.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Snippets } from './Snippets';

describe('Snippets', () => {
  const mockItems = [
    { id: 'uuid-1', label: 'Email', value: 'hello@example.com' },
    { id: 'uuid-2', label: null, value: 'Plain text snippet' },
    { id: 'uuid-3', label: 'Phone', value: '090-1234-5678' },
  ];

  it('should render empty state when no items', () => {
    const onSelect = vi.fn();
    render(<Snippets items={[]} onSelect={onSelect} />);

    expect(screen.getByText(/no snippets/i)).toBeDefined();
  });

  it('should render all snippet items', () => {
    const onSelect = vi.fn();
    render(<Snippets items={mockItems} onSelect={onSelect} />);

    expect(screen.getByText(/1: Email/)).toBeDefined();
    expect(screen.getByText(/2: Plain text snippet/)).toBeDefined();
    expect(screen.getByText(/3: Phone/)).toBeDefined();
  });

  it('should display label when present, value when label is null', () => {
    const onSelect = vi.fn();
    render(<Snippets items={mockItems} onSelect={onSelect} />);

    // Item with label shows label
    expect(screen.getByText(/1: Email/)).toBeDefined();
    // Item without label shows value
    expect(screen.getByText(/2: Plain text snippet/)).toBeDefined();
  });

  it('should call onSelect with item value on click', () => {
    const onSelect = vi.fn();
    render(<Snippets items={mockItems} onSelect={onSelect} />);

    const item = screen.getByText(/2: Plain text snippet/);
    fireEvent.click(item);

    expect(onSelect).toHaveBeenCalledWith('Plain text snippet');
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it('should highlight first item by default', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const firstItem = container.querySelector('[data-selected="true"]');
    expect(firstItem?.textContent).toContain('1:');
    expect(firstItem?.textContent).toContain('Email');
  });

  it('should navigate down with ArrowDown key', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('2:');
    expect(selectedItem?.textContent).toContain('Plain text snippet');
  });

  it('should navigate up with ArrowUp key', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to second item
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    // Move back to first
    fireEvent.keyDown(list, { key: 'ArrowUp' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('1:');
    expect(selectedItem?.textContent).toContain('Email');
  });

  it('should not go below first item with ArrowUp', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Try to go above first item
    fireEvent.keyDown(list, { key: 'ArrowUp' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('1:');
    expect(selectedItem?.textContent).toContain('Email');
  });

  it('should not go beyond last item with ArrowDown', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to last item
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    // Try to go beyond
    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.textContent).toContain('3:');
    expect(selectedItem?.textContent).toContain('Phone');
  });

  it('should call onSelect with selected item value on Enter key', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Move to second item and select
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    fireEvent.keyDown(list, { key: 'Enter' });

    expect(onSelect).toHaveBeenCalledWith('Plain text snippet');
  });

  it('should truncate long text with ellipsis', () => {
    const longText = 'A'.repeat(100);
    const onSelect = vi.fn();
    const { container } = render(
      <Snippets items={[{ id: 'uuid', label: null, value: longText }]} onSelect={onSelect} />
    );

    const item = container.querySelector('[data-snippet-item]');
    expect(item?.classList.contains('truncate')).toBe(true);
  });

  it('should display item numbers in format "1: text"', () => {
    const onSelect = vi.fn();
    render(<Snippets items={mockItems} onSelect={onSelect} />);

    expect(screen.getByText(/1:/)).toBeDefined();
    expect(screen.getByText(/2:/)).toBeDefined();
    expect(screen.getByText(/3:/)).toBeDefined();
  });

  it('should have semantic color classes for selected items', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const selectedItem = container.querySelector('[data-selected="true"]');
    expect(selectedItem?.className).toContain('bg-app-item-selected');
    expect(selectedItem?.className).toContain('text-app-text-on-selected');
  });

  it('should have semantic color classes for unselected items', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const unselectedItem = container.querySelector('[data-selected="false"]');
    expect(unselectedItem?.className).toContain('bg-app-item');
    expect(unselectedItem?.className).toContain('text-app-text');
    expect(unselectedItem?.className).toContain('hover:bg-app-item-hover');
  });

  it('should scroll selected item into view on keyboard navigation', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const list = container.querySelector('ul');
    if (!list) throw new Error('List not found');

    // Mock scrollIntoView on the button elements
    const scrollIntoViewMock = vi.fn();
    const buttons = container.querySelectorAll('[data-snippet-item]');
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

  it('should have title attribute showing full value for tooltip', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    const secondItem = container.querySelectorAll('[data-snippet-item]')[1];
    // Title shows the actual value, not the display text
    expect(secondItem.getAttribute('title')).toBe('Plain text snippet');
  });

  it('should show value in tooltip even when label is different', () => {
    const onSelect = vi.fn();
    const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

    // First item has label="Email" but value="hello@example.com"
    // Tooltip should show the value, not the label
    const firstItem = container.querySelectorAll('[data-snippet-item]')[0];
    expect(firstItem.getAttribute('title')).toBe('hello@example.com');
  });

  describe('number key selection', () => {
    it('should select item with number key 1-9', () => {
      const onSelect = vi.fn();
      const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '2' });

      expect(onSelect).toHaveBeenCalledWith('Plain text snippet');
    });

    it('should not select item if number exceeds item count', () => {
      const onSelect = vi.fn();
      const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '9' });

      expect(onSelect).not.toHaveBeenCalled();
    });

    it('should select first item with key 1', () => {
      const onSelect = vi.fn();
      const { container } = render(<Snippets items={mockItems} onSelect={onSelect} />);

      const list = container.querySelector('ul');
      if (!list) throw new Error('List not found');

      fireEvent.keyDown(list, { key: '1' });

      expect(onSelect).toHaveBeenCalledWith('hello@example.com');
    });
  });

  describe('empty label handling', () => {
    it('should show value when label is empty string', () => {
      const onSelect = vi.fn();
      render(
        <Snippets items={[{ id: 'uuid', label: '', value: 'actual value' }]} onSelect={onSelect} />
      );

      expect(screen.getByText(/1: actual value/)).toBeDefined();
    });

    it('should show value when label is whitespace only', () => {
      const onSelect = vi.fn();
      render(
        <Snippets
          items={[{ id: 'uuid', label: '   ', value: 'actual value' }]}
          onSelect={onSelect}
        />
      );

      expect(screen.getByText(/1: actual value/)).toBeDefined();
    });
  });
});
