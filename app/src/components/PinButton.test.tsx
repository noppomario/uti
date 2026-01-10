/**
 * Tests for PinButton component
 *
 * A button that toggles window pinning (always-on-top) behavior.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PinButton } from './PinButton';

describe('PinButton', () => {
  it('renders pin icon when not pinned', () => {
    render(<PinButton isPinned={false} onToggle={() => {}} />);

    const button = screen.getByRole('button', { name: /pin window/i });
    expect(button).toBeDefined();
  });

  it('renders pinned icon when pinned', () => {
    render(<PinButton isPinned={true} onToggle={() => {}} />);

    const button = screen.getByRole('button', { name: /unpin window/i });
    expect(button).toBeDefined();
  });

  it('calls onToggle when clicked', () => {
    const handleToggle = vi.fn();
    render(<PinButton isPinned={false} onToggle={handleToggle} />);

    fireEvent.click(screen.getByRole('button'));

    expect(handleToggle).toHaveBeenCalledTimes(1);
  });

  it('has button type attribute', () => {
    render(<PinButton isPinned={false} onToggle={() => {}} />);

    const button = screen.getByRole('button');
    expect(button.getAttribute('type')).toBe('button');
  });

  it('applies muted color when not pinned', () => {
    render(<PinButton isPinned={false} onToggle={() => {}} />);

    const button = screen.getByRole('button');
    expect(button.className).toContain('text-app-text-muted');
  });

  it('applies accent color when pinned', () => {
    render(<PinButton isPinned={true} onToggle={() => {}} />);

    const button = screen.getByRole('button');
    expect(button.className).toContain('text-app-accent-info');
  });
});
