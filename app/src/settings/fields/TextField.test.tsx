/**
 * Tests for TextField component
 */

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { TextField } from './TextField';

describe('TextField', () => {
  it('renders label', () => {
    render(<TextField label="Version" value="1.0.0" />);
    expect(screen.getByText('Version')).toBeDefined();
  });

  it('renders value', () => {
    render(<TextField label="Version" value="1.0.0" />);
    expect(screen.getByText('1.0.0')).toBeDefined();
  });

  it('renders description when provided', () => {
    render(<TextField label="Version" description="Current version" value="1.0.0" />);
    expect(screen.getByText('Current version')).toBeDefined();
  });

  it('does not render description when not provided', () => {
    render(<TextField label="Version" value="1.0.0" />);
    expect(screen.queryByText('Current version')).toBeNull();
  });
});
