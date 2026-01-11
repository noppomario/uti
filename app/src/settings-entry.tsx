/**
 * Settings window entry point
 *
 * Separate entry point for the settings window.
 * Initializes i18n and renders the SettingsPage component.
 */

import ReactDOM from 'react-dom/client';
import { getConfig } from './config';
import './index.css';
import './settings/i18n';
import { SettingsPage } from './settings/SettingsPage';

/** Available color themes */
const COLOR_THEMES = ['midnight', 'dark', 'light'] as const;
/** Available size themes */
const SIZE_THEMES = ['minimal', 'normal', 'wide'] as const;

/**
 * Apply theme based on configuration
 */
async function applyTheme() {
  const config = await getConfig();
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
  }
}

// Initialize theme and render
applyTheme();

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<SettingsPage />);
