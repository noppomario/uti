/**
 * Settings window entry point
 *
 * Separate entry point for the settings window.
 * Initializes i18n and renders the SettingsPage component.
 */

import { listen } from '@tauri-apps/api/event';
import ReactDOM from 'react-dom/client';
import type { AppConfig } from './config';
import { getConfig } from './config';
import './index.css';
import './i18n';
import { SettingsPage } from './settings/SettingsPage';

/** Available color themes */
const COLOR_THEMES = ['midnight', 'dark', 'light'] as const;
/** Available size themes */
const SIZE_THEMES = ['minimal', 'normal', 'wide'] as const;

/**
 * Apply theme from config
 */
function applyThemeFromConfig(config: AppConfig) {
  const root = document.documentElement;
  const { color, size, accentColor } = config.theme;

  // Remove all existing theme classes
  for (const c of COLOR_THEMES) {
    root.classList.remove(`theme-${c}`);
  }
  for (const s of SIZE_THEMES) {
    root.classList.remove(`size-${s}`);
  }

  // Apply color theme
  root.classList.add(`theme-${color}`);

  // Apply size theme (minimal is default, no class needed)
  if (size !== 'minimal') {
    root.classList.add(`size-${size}`);
  }

  // Apply custom accent color if specified
  if (accentColor) {
    root.style.setProperty('--color-app-item-selected', accentColor);
  } else {
    root.style.removeProperty('--color-app-item-selected');
  }
}

/**
 * Initialize theme from stored config
 */
async function initTheme() {
  const config = await getConfig();
  applyThemeFromConfig(config);
}

// Initialize theme and listen for changes
initTheme();
listen<AppConfig>('config_changed', event => {
  applyThemeFromConfig(event.payload);
});

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<SettingsPage />);
