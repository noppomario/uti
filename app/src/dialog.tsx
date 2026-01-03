/**
 * Dialog window entry point
 *
 * This is a separate entry point for the update dialog window.
 * It parses URL parameters and displays a themed dialog.
 */

import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { getConfig } from './config';
import './index.css';

type DialogKind = 'info' | 'error';

interface DialogParams {
  title: string;
  message: string;
  kind: DialogKind;
}

/**
 * Parse dialog parameters from URL query string
 */
function parseDialogParams(): DialogParams {
  const params = new URLSearchParams(window.location.search);
  return {
    title: params.get('title') || 'Dialog',
    message: params.get('message') || '',
    kind: (params.get('kind') as DialogKind) || 'info',
  };
}

/**
 * Apply theme based on configuration
 */
async function applyTheme() {
  const config = await getConfig();
  if (config.theme === 'dark') {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

/**
 * Dialog page component
 *
 * Displays a themed dialog with custom title bar, message, and OK button.
 * Closes the window when OK is clicked or Escape is pressed.
 */
function DialogPage() {
  const [params] = useState<DialogParams>(parseDialogParams);

  // Handle Escape key to close window
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().close();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleClose = () => {
    getCurrentWindow().close();
  };

  return (
    <div className="flex h-screen flex-col overflow-hidden rounded-lg border border-app-header-border bg-app-bg">
      {/* Custom title bar - draggable */}
      <div
        data-tauri-drag-region
        className="flex h-9 shrink-0 items-center justify-between border-b border-app-header-border bg-app-header px-3"
      >
        <span className="text-xs font-medium text-app-text">uti</span>
        <button
          type="button"
          onClick={handleClose}
          className="flex h-6 w-6 items-center justify-center rounded text-app-text-muted hover:bg-app-item-hover hover:text-app-text"
          aria-label="Close"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path
              d="M2 2L10 10M10 2L2 10"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      {/* Content area */}
      <div className="flex flex-1 flex-col p-4">
        {/* Title with accent color based on kind */}
        <h1
          className={`mb-3 text-base font-semibold ${
            params.kind === 'error' ? 'text-app-accent-error' : 'text-app-accent-info'
          }`}
        >
          {params.title}
        </h1>

        {/* Message */}
        <p className="mb-4 flex-1 whitespace-pre-wrap text-sm text-app-text">{params.message}</p>

        {/* OK button */}
        <button
          type="button"
          onClick={handleClose}
          className="w-full rounded bg-app-item py-2 text-sm text-app-text hover:bg-app-item-hover"
        >
          OK
        </button>
      </div>
    </div>
  );
}

// Initialize theme and render
applyTheme();

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<DialogPage />);
