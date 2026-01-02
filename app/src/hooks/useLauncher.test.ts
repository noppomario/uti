/**
 * Tests for useLauncher hook
 *
 * Loads launcher configuration from backend.
 */

import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useLauncher } from './useLauncher';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('useLauncher', () => {
  const mockConfig = {
    commands: [
      {
        id: 'nautilus',
        name: 'Files',
        command: 'nautilus',
        args: [],
        historySource: null,
      },
      {
        id: 'gnome-text-editor',
        name: 'Text Editor',
        command: 'gnome-text-editor',
        args: [],
        historySource: null,
      },
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns empty commands initially', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useLauncher());

    expect(result.current.commands).toEqual([]);
    expect(result.current.isLoading).toBe(true);
  });

  it('loads commands from backend', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useLauncher());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledWith('get_launcher_config');
    expect(result.current.commands).toHaveLength(2);
    expect(result.current.commands[0].id).toBe('nautilus');
  });

  it('handles error when loading fails', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Failed to load'));

    const { result } = renderHook(() => useLauncher());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.commands).toEqual([]);
    expect(result.current.error).toBe('Failed to load launcher config');
  });

  it('provides reload function', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useLauncher());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // First load
    expect(invoke).toHaveBeenCalledTimes(1);

    // Reload
    await act(async () => {
      await result.current.reload();
    });

    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
