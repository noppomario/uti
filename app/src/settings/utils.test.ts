/**
 * Tests for settings utility functions
 */

import { describe, expect, it } from 'vitest';
import { getNestedValue, setNestedValue } from './utils';

describe('getNestedValue', () => {
  it('returns value at simple path', () => {
    const obj = { name: 'test' };
    expect(getNestedValue(obj, 'name')).toBe('test');
  });

  it('returns value at nested path', () => {
    const obj = { theme: { color: 'dark' } };
    expect(getNestedValue(obj, 'theme.color')).toBe('dark');
  });

  it('returns deeply nested value', () => {
    const obj = { a: { b: { c: { d: 'value' } } } };
    expect(getNestedValue(obj, 'a.b.c.d')).toBe('value');
  });

  it('returns undefined for non-existent path', () => {
    const obj = { name: 'test' };
    expect(getNestedValue(obj, 'nonexistent')).toBeUndefined();
  });

  it('returns undefined for non-existent nested path', () => {
    const obj = { theme: { color: 'dark' } };
    expect(getNestedValue(obj, 'theme.size')).toBeUndefined();
  });

  it('returns undefined when intermediate path is null', () => {
    const obj = { theme: null } as unknown as Record<string, unknown>;
    expect(getNestedValue(obj, 'theme.color')).toBeUndefined();
  });

  it('returns undefined when intermediate path is undefined', () => {
    const obj = {} as Record<string, unknown>;
    expect(getNestedValue(obj, 'theme.color')).toBeUndefined();
  });
});

describe('setNestedValue', () => {
  it('sets value at simple path', () => {
    const obj = { name: 'old' };
    const result = setNestedValue(obj, 'name', 'new');
    expect(result.name).toBe('new');
  });

  it('sets value at nested path', () => {
    const obj = { theme: { color: 'dark', size: 'normal' } };
    const result = setNestedValue(obj, 'theme.color', 'light');
    expect(result.theme.color).toBe('light');
    expect(result.theme.size).toBe('normal');
  });

  it('creates intermediate objects when path does not exist', () => {
    const obj = {} as Record<string, unknown>;
    const result = setNestedValue(obj, 'theme.color', 'dark');
    expect((result.theme as Record<string, unknown>).color).toBe('dark');
  });

  it('does not mutate original object', () => {
    const obj = { theme: { color: 'dark' } };
    const result = setNestedValue(obj, 'theme.color', 'light');
    expect(obj.theme.color).toBe('dark');
    expect(result.theme.color).toBe('light');
  });

  it('preserves other properties in nested objects', () => {
    const obj = { theme: { color: 'dark', size: 'normal' }, other: 'value' };
    const result = setNestedValue(obj, 'theme.color', 'light');
    expect(result.theme.size).toBe('normal');
    expect(result.other).toBe('value');
  });

  it('handles setting undefined value', () => {
    const obj = { theme: { color: 'dark' } };
    const result = setNestedValue(obj, 'theme.color', undefined);
    expect(result.theme.color).toBeUndefined();
  });
});
