import ReactDOM from 'react-dom/client';
import App from './App';
import { getConfig } from './config';
import './index.css';

/**
 * Apply theme based on configuration
 *
 * @param config - Application configuration
 */
function applyTheme(config: { theme: string }) {
  const isDark = config.theme === 'dark';

  if (isDark) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
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
