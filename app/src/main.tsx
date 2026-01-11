import { listen } from '@tauri-apps/api/event';
import ReactDOM from 'react-dom/client';
import App from './App';
import type { AppConfig, ColorTheme, SizeTheme } from './config';
import { getConfig } from './config';
import './index.css';

/** Valid color theme classes */
const COLOR_THEMES: ColorTheme[] = ['midnight', 'dark', 'light'];

/** Valid size theme classes */
const SIZE_THEMES: SizeTheme[] = ['minimal', 'normal', 'wide'];

/**
 * Apply theme based on configuration
 *
 * Adds theme classes to document root element.
 *
 * @param config - Application configuration
 */
function applyTheme(config: AppConfig) {
  const root = document.documentElement;
  const { color, size, accentColor } = config.theme;

  // Remove all existing theme classes
  for (const c of COLOR_THEMES) {
    root.classList.remove(`theme-${c}`);
  }
  for (const s of SIZE_THEMES) {
    root.classList.remove(`size-${s}`);
  }

  // Apply new theme classes
  root.classList.add(`theme-${color}`);
  if (size !== 'minimal') {
    // minimal is default, no class needed
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
 * Initialize theme
 *
 * Loads configuration from file and applies theme.
 */
async function initTheme() {
  const config = await getConfig();
  applyTheme(config);
}

// Apply theme on load
initTheme();

// Listen for config changes from settings window
listen<AppConfig>('config_changed', event => {
  applyTheme(event.payload);
});

/**
 * Application entry point
 *
 * Renders the main App component into the root DOM element.
 */
const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<App />);
