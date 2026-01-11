/**
 * Application configuration
 *
 * This module provides centralized configuration for the application.
 * Configuration is loaded from ~/.config/uti/config.json via Tauri backend.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Color theme options
 */
export type ColorTheme = 'midnight' | 'dark' | 'light';

/**
 * Size theme options
 */
export type SizeTheme = 'minimal' | 'normal' | 'wide';

/**
 * Language options
 */
export type Language = 'en' | 'ja';

/**
 * Theme configuration
 */
export interface ThemeConfig {
  /** Color theme: 'midnight', 'dark', or 'light' */
  color: ColorTheme;
  /** Size theme: 'minimal', 'normal', or 'wide' */
  size: SizeTheme;
  /** Custom accent color (hex format, e.g., '#3584e4'). If not set, uses theme default. */
  accentColor?: string;
}

export interface AppConfig {
  /** Theme configuration */
  theme: ThemeConfig;

  /** Maximum number of clipboard items to store */
  clipboardHistoryLimit: number;

  /** UI language: 'en' or 'ja' */
  language: Language;
}

/**
 * Default theme configuration
 */
export const defaultTheme: ThemeConfig = {
  color: 'dark',
  size: 'normal',
};

/**
 * Default application configuration
 *
 * Used as fallback if config file cannot be loaded.
 */
export const defaultConfig: AppConfig = {
  theme: defaultTheme,
  clipboardHistoryLimit: 50,
  language: 'en',
};

/**
 * Check if color theme is a dark variant
 */
export function isDarkTheme(color: ColorTheme): boolean {
  return color === 'midnight' || color === 'dark';
}

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
 * console.log(config.theme.color); // 'midnight' | 'dark' | 'light'
 * console.log(config.theme.size);  // 'minimal' | 'normal' | 'wide'
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
