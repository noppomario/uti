/**
 * Tests for ButtonField component
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ButtonField } from './ButtonField';

describe('ButtonField', () => {
  it('renders button with label', () => {
    render(<ButtonField label="Test Button" onClick={() => {}} />);
    expect(screen.getByRole('button', { name: 'Test Button' })).toBeDefined();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<ButtonField label="Test Button" onClick={handleClick} />);
    const button = screen.getByRole('button', { name: 'Test Button' });
    fireEvent.click(button);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('renders with action variant styling by default', () => {
    render(<ButtonField label="Test Button" onClick={() => {}} />);
    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button.className).toContain('border-app-text-muted');
  });

  it('renders with primary variant styling', () => {
    render(<ButtonField label="Test Button" variant="primary" onClick={() => {}} />);
    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button.className).toContain('bg-app-accent-info');
  });

  it('renders with secondary variant styling', () => {
    render(<ButtonField label="Test Button" variant="secondary" onClick={() => {}} />);
    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button.className).toContain('border-app-header-border');
    expect(button.className).toContain('text-app-text');
  });

  it('has type button to prevent form submission', () => {
    render(<ButtonField label="Test Button" onClick={() => {}} />);
    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button.getAttribute('type')).toBe('button');
  });
});
