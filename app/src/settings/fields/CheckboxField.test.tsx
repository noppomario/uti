/**
 * Tests for CheckboxField component
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { CheckboxField } from './CheckboxField';

describe('CheckboxField', () => {
  it('renders label', () => {
    render(<CheckboxField label="Test Label" value={false} onChange={() => {}} />);
    expect(screen.getByText('Test Label')).toBeDefined();
  });

  it('renders description when provided', () => {
    render(
      <CheckboxField
        label="Test Label"
        description="Test description"
        value={false}
        onChange={() => {}}
      />
    );
    expect(screen.getByText('Test description')).toBeDefined();
  });

  it('renders checkbox with correct checked state', () => {
    const { rerender } = render(
      <CheckboxField label="Test Label" value={false} onChange={() => {}} />
    );
    const checkbox = screen.getByRole('checkbox') as HTMLInputElement;
    expect(checkbox.checked).toBe(false);

    rerender(<CheckboxField label="Test Label" value={true} onChange={() => {}} />);
    expect(checkbox.checked).toBe(true);
  });

  it('calls onChange when clicked', () => {
    const handleChange = vi.fn();
    render(<CheckboxField label="Test Label" value={false} onChange={handleChange} />);
    const checkbox = screen.getByRole('checkbox');
    fireEvent.click(checkbox);
    expect(handleChange).toHaveBeenCalledWith(true);
  });

  it('calls onChange with false when unchecking', () => {
    const handleChange = vi.fn();
    render(<CheckboxField label="Test Label" value={true} onChange={handleChange} />);
    const checkbox = screen.getByRole('checkbox');
    fireEvent.click(checkbox);
    expect(handleChange).toHaveBeenCalledWith(false);
  });
});
