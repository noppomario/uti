/**
 * Hook for loading launcher configuration
 *
 * Fetches launcher commands from the backend and provides
 * state management for the launcher list.
 */

import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';

/**
 * History source configuration for jump list
 */
export interface HistorySource {
  type: 'recently-used' | 'vscode';
  appName?: string;
  path?: string;
}

/**
 * Launcher command item
 */
export interface LauncherItem {
  id: string;
  name: string;
  command: string;
  args: string[];
  historySource: HistorySource | null;
}

/**
 * Launcher configuration from backend
 */
interface LauncherConfig {
  commands: LauncherItem[];
}

/**
 * State returned by useLauncher hook
 */
interface UseLauncherResult {
  /** List of launcher commands */
  commands: LauncherItem[];
  /** Whether the config is being loaded */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
  /** Reload the configuration */
  reload: () => Promise<void>;
}

/**
 * Hook for managing launcher configuration
 *
 * @returns Launcher state and controls
 *
 * @example
 * ```typescript
 * const { commands, isLoading, error } = useLauncher();
 *
 * if (isLoading) return <Loading />;
 * if (error) return <Error message={error} />;
 *
 * return commands.map(cmd => <CommandItem key={cmd.id} {...cmd} />);
 * ```
 */
export function useLauncher(): UseLauncherResult {
  const [commands, setCommands] = useState<LauncherItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const config = await invoke<LauncherConfig>('get_launcher_config');
      setCommands(config.commands);
    } catch (err) {
      console.error('Failed to load launcher config:', err);
      setError('Failed to load launcher config');
      setCommands([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  return {
    commands,
    isLoading,
    error,
    reload: loadConfig,
  };
}
