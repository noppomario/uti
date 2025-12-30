/**
 * Application configuration
 *
 * This module provides centralized configuration for the application.
 * Configuration is loaded from ~/.config/uti/config.json via Tauri backend.
 */

import { invoke } from '@tauri-apps/api/core';

export interface AppConfig {
  /** Theme mode: 'light' or 'dark' */
  theme: 'light' | 'dark';

  /** Maximum number of clipboard items to store */
  clipboardHistoryLimit: number;
}

/**
 * Default application configuration
 *
 * Used as fallback if config file cannot be loaded.
 */
export const defaultConfig: AppConfig = {
  theme: 'dark',
  clipboardHistoryLimit: 50,
};

/**
 * Get current application configuration
 *
 * Loads configuration from ~/.config/uti/config.json via Tauri backend.
 * Falls back to default configuration if file cannot be loaded.
 *
 * @returns Promise resolving to current configuration
 *
 * @example
 * ```typescript
 * const config = await getConfig();
 * console.log(config.theme); // 'dark' | 'light'
 * ```
 */
export async function getConfig(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>('read_config');
    return { ...defaultConfig, ...config };
  } catch (error) {
    console.error('Failed to load config, using defaults:', error);
    return defaultConfig;
  }
}
