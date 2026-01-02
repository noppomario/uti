/**
 * Tests for JumpList component
 *
 * Displays recent files for a launcher command.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { JumpList, type RecentFile } from './JumpList';

describe('JumpList', () => {
  const mockFiles: RecentFile[] = [
    {
      path: '/home/user/Documents/main.rs',
      name: 'main.rs',
      timestamp: '2024-01-01T12:00:00Z',
    },
    {
      path: '/home/user/Documents/config.ts',
      name: 'config.ts',
      timestamp: '2024-01-01T11:00:00Z',
    },
  ];

  it('renders empty state when no files', () => {
    render(<JumpList files={[]} onSelect={() => {}} onClose={() => {}} />);

    expect(screen.getByText(/no recent files/i)).toBeDefined();
  });

  it('renders all recent files', () => {
    render(<JumpList files={mockFiles} onSelect={() => {}} onClose={() => {}} />);

    expect(screen.getByText('main.rs')).toBeDefined();
    expect(screen.getByText('config.ts')).toBeDefined();
  });

  it('calls onSelect with file path when clicked', () => {
    const handleSelect = vi.fn();
    render(<JumpList files={mockFiles} onSelect={handleSelect} onClose={() => {}} />);

    fireEvent.click(screen.getByText('main.rs'));

    expect(handleSelect).toHaveBeenCalledWith('/home/user/Documents/main.rs');
  });

  it('highlights first item by default', () => {
    const { container } = render(
      <JumpList files={mockFiles} onSelect={() => {}} onClose={() => {}} />
    );

    const firstItem = container.querySelector('[data-selected="true"]');
    expect(firstItem?.textContent).toContain('main.rs');
  });

  it('navigates with arrow keys', () => {
    const { container } = render(
      <JumpList files={mockFiles} onSelect={() => {}} onClose={() => {}} />
    );
    const list = container.querySelector('ul') as Element;

    // Initially first item is selected
    let selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('main.rs');

    // Press down arrow
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('config.ts');

    // Press up arrow
    fireEvent.keyDown(list, { key: 'ArrowUp' });
    selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('main.rs');
  });

  it('selects file with Enter key', () => {
    const handleSelect = vi.fn();
    const { container } = render(
      <JumpList files={mockFiles} onSelect={handleSelect} onClose={() => {}} />
    );
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: 'Enter' });

    expect(handleSelect).toHaveBeenCalledWith('/home/user/Documents/main.rs');
  });

  it('calls onClose when ArrowLeft is pressed', () => {
    const handleClose = vi.fn();
    const { container } = render(
      <JumpList files={mockFiles} onSelect={() => {}} onClose={handleClose} />
    );
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: 'ArrowLeft' });

    expect(handleClose).toHaveBeenCalled();
  });

  it('calls onClose when Escape is pressed', () => {
    const handleClose = vi.fn();
    const { container } = render(
      <JumpList files={mockFiles} onSelect={() => {}} onClose={handleClose} />
    );
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: 'Escape' });

    expect(handleClose).toHaveBeenCalled();
  });

  it('displays file path in tooltip', () => {
    render(<JumpList files={mockFiles} onSelect={() => {}} onClose={() => {}} />);

    const button = screen.getByText('main.rs').closest('button');
    expect(button?.getAttribute('title')).toBe('/home/user/Documents/main.rs');
  });
});
