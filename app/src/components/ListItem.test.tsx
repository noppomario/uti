/**
 * Tests for ListItem component
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ListItem } from './ListItem';

describe('ListItem', () => {
  it('renders with index and children', () => {
    render(
      <ListItem selected={false} index={0} onClick={() => {}}>
        Test Item
      </ListItem>
    );

    expect(screen.getByText(/1:.*Test Item/)).toBeDefined();
  });

  it('applies selected styling when selected', () => {
    const { container } = render(
      <ListItem selected={true} index={0} onClick={() => {}}>
        Selected Item
      </ListItem>
    );

    const button = container.querySelector('button');
    expect(button?.className).toContain('bg-app-item-selected');
  });

  it('applies non-selected styling when not selected', () => {
    const { container } = render(
      <ListItem selected={false} index={0} onClick={() => {}}>
        Normal Item
      </ListItem>
    );

    const button = container.querySelector('button');
    expect(button?.className).toContain('bg-app-item');
    expect(button?.className).not.toContain('bg-app-item-selected');
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(
      <ListItem selected={false} index={0} onClick={handleClick}>
        Clickable Item
      </ListItem>
    );

    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalled();
  });

  it('calls onMouseEnter and onMouseLeave', () => {
    const handleMouseEnter = vi.fn();
    const handleMouseLeave = vi.fn();
    render(
      <ListItem
        selected={false}
        index={0}
        onClick={() => {}}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        Hoverable Item
      </ListItem>
    );

    const button = screen.getByRole('button');
    fireEvent.mouseEnter(button);
    expect(handleMouseEnter).toHaveBeenCalled();

    fireEvent.mouseLeave(button);
    expect(handleMouseLeave).toHaveBeenCalled();
  });

  it('renders suffix when provided', () => {
    render(
      <ListItem
        selected={false}
        index={0}
        onClick={() => {}}
        suffix={<span data-testid="suffix">{'>'}</span>}
      >
        Item with suffix
      </ListItem>
    );

    expect(screen.getByTestId('suffix')).toBeDefined();
  });

  it('sets title attribute when provided', () => {
    render(
      <ListItem selected={false} index={0} onClick={() => {}} title="Full path here">
        Item with title
      </ListItem>
    );

    const button = screen.getByRole('button');
    expect(button.getAttribute('title')).toBe('Full path here');
  });

  it('sets data-selected attribute', () => {
    const { container, rerender } = render(
      <ListItem selected={false} index={0} onClick={() => {}}>
        Item
      </ListItem>
    );

    let button = container.querySelector('button');
    expect(button?.getAttribute('data-selected')).toBe('false');

    rerender(
      <ListItem selected={true} index={0} onClick={() => {}}>
        Item
      </ListItem>
    );

    button = container.querySelector('button');
    expect(button?.getAttribute('data-selected')).toBe('true');
  });

  it('displays correct 1-based index', () => {
    render(
      <ListItem selected={false} index={4} onClick={() => {}}>
        Fifth Item
      </ListItem>
    );

    expect(screen.getByText(/5:.*Fifth Item/)).toBeDefined();
  });

  it('calls onSuffixClick when suffix is clicked', () => {
    const handleClick = vi.fn();
    const handleSuffixClick = vi.fn();
    render(
      <ListItem
        selected={false}
        index={0}
        onClick={handleClick}
        suffix=">"
        onSuffixClick={handleSuffixClick}
      >
        Item with clickable suffix
      </ListItem>
    );

    // When onSuffixClick is provided, there are two buttons
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(2);

    // Click the suffix button (second one)
    const suffixButton = screen.getByRole('button', { name: '>' });
    fireEvent.click(suffixButton);

    expect(handleSuffixClick).toHaveBeenCalled();
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('calls onClick when main content is clicked (not suffix)', () => {
    const handleClick = vi.fn();
    const handleSuffixClick = vi.fn();
    render(
      <ListItem
        selected={false}
        index={0}
        onClick={handleClick}
        suffix=">"
        onSuffixClick={handleSuffixClick}
      >
        Item with clickable suffix
      </ListItem>
    );

    // Click the main content button (first one, contains the item name)
    const mainButton = screen.getByRole('button', { name: /Item with clickable suffix/ });
    fireEvent.click(mainButton);

    expect(handleClick).toHaveBeenCalled();
    expect(handleSuffixClick).not.toHaveBeenCalled();
  });

  it('does not make suffix clickable when onSuffixClick is not provided', () => {
    const handleClick = vi.fn();
    render(
      <ListItem selected={false} index={0} onClick={handleClick} suffix=">">
        Item with non-clickable suffix
      </ListItem>
    );

    // Only the main button should be present (standard single-button layout)
    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(1);

    fireEvent.click(buttons[0]);
    expect(handleClick).toHaveBeenCalled();
  });
});
