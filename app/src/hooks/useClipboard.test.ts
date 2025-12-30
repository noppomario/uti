/**
 * Tests for useClipboard hook
 *
 * This hook polls the clipboard every 1 second and detects changes.
 */

import { renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboard } from './useClipboard';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(),
}));

describe('useClipboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should start polling on mount', async () => {
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
    vi.mocked(readText).mockResolvedValue('initial text');

    renderHook(() => useClipboard());

    // Should not call immediately
    expect(readText).not.toHaveBeenCalled();

    // Should call after 1 second
    await vi.advanceTimersByTimeAsync(1000);
    expect(readText).toHaveBeenCalledTimes(1);
  });

  it('should invoke add_clipboard_item when clipboard changes', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');

    vi.mocked(readText).mockResolvedValueOnce('first text').mockResolvedValueOnce('second text');

    renderHook(() => useClipboard());

    // First poll - should add first text
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledWith('add_clipboard_item', { text: 'first text' });

    // Second poll - should add second text
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledWith('add_clipboard_item', { text: 'second text' });

    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it('should not invoke add_clipboard_item for duplicate text', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');

    vi.mocked(readText)
      .mockResolvedValueOnce('same text')
      .mockResolvedValueOnce('same text')
      .mockResolvedValueOnce('same text');

    renderHook(() => useClipboard());

    // First poll - should add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(1);

    // Second poll - same text, should not add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(1);

    // Third poll - still same text, should not add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it('should ignore null or empty clipboard content', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');

    vi.mocked(readText)
      .mockResolvedValueOnce(null as unknown as string)
      .mockResolvedValueOnce('')
      .mockResolvedValueOnce('valid text');

    renderHook(() => useClipboard());

    // First poll - null, should not add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).not.toHaveBeenCalled();

    // Second poll - empty string, should not add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).not.toHaveBeenCalled();

    // Third poll - valid text, should add
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledWith('add_clipboard_item', { text: 'valid text' });
  });

  it('should cleanup interval on unmount', async () => {
    const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
    vi.mocked(readText).mockResolvedValue('text');

    const { unmount } = renderHook(() => useClipboard());

    // Verify polling is active
    await vi.advanceTimersByTimeAsync(1000);
    expect(readText).toHaveBeenCalledTimes(1);

    // Unmount
    unmount();

    // Advance timer - should not poll anymore
    await vi.advanceTimersByTimeAsync(1000);
    expect(readText).toHaveBeenCalledTimes(1);
  });
});
