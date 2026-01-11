/**
 * Tests for SelectField component
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SelectField } from './SelectField';

describe('SelectField', () => {
  const defaultOptions = [
    { value: 'option1', label: 'Option 1' },
    { value: 'option2', label: 'Option 2' },
    { value: 'option3', label: 'Option 3' },
  ];

  it('renders label', () => {
    render(
      <SelectField
        label="Test Label"
        value="option1"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    expect(screen.getByText('Test Label')).toBeDefined();
  });

  it('renders description when provided', () => {
    render(
      <SelectField
        label="Test Label"
        description="Test description"
        value="option1"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    expect(screen.getByText('Test description')).toBeDefined();
  });

  it('does not render description when not provided', () => {
    render(
      <SelectField
        label="Test Label"
        value="option1"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    expect(screen.queryByText('Test description')).toBeNull();
  });

  it('renders all options', () => {
    render(
      <SelectField
        label="Test Label"
        value="option1"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    const select = screen.getByRole('combobox');
    expect(select).toBeDefined();
    expect(screen.getByText('Option 1')).toBeDefined();
    expect(screen.getByText('Option 2')).toBeDefined();
    expect(screen.getByText('Option 3')).toBeDefined();
  });

  it('selects the correct value', () => {
    render(
      <SelectField
        label="Test Label"
        value="option2"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('option2');
  });

  it('calls onChange when selection changes', () => {
    const handleChange = vi.fn();
    render(
      <SelectField
        label="Test Label"
        value="option1"
        options={defaultOptions}
        onChange={handleChange}
      />
    );
    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: 'option3' } });
    expect(handleChange).toHaveBeenCalledWith('option3');
  });

  it('associates label with select element', () => {
    render(
      <SelectField
        label="Test Label"
        value="option1"
        options={defaultOptions}
        onChange={() => {}}
      />
    );
    const label = screen.getByText('Test Label');
    const select = screen.getByRole('combobox');
    expect(label.getAttribute('for')).toBeDefined();
    expect(select.getAttribute('id')).toBeDefined();
    expect(label.getAttribute('for')).toBe(select.getAttribute('id'));
  });
});
