/**
 * Update dialog entry point
 *
 * Separate entry point for displaying update check results.
 * Parses URL parameters (title, message, kind) and shows a themed dialog.
 */

import { getCurrentWindow } from '@tauri-apps/api/window';
import type React from 'react';
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

/** Fixed styles for dialog - matches size-normal theme */
const dialogStyles = {
  container: { borderRadius: '0.75rem' } as React.CSSProperties, // 12px
  heading: { fontSize: '1rem' } as React.CSSProperties, // 16px - matches size-normal heading
  message: { fontSize: '0.875rem' } as React.CSSProperties, // 14px - matches size-normal base
  button: {
    fontSize: '0.875rem', // 14px - matches size-normal base
    padding: '0.5rem 0.75rem', // 8px 12px - matches size-normal padding
    borderRadius: '0.375rem', // 6px - matches size-normal radius
  } as React.CSSProperties,
};

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
    <div
      className="flex h-screen flex-col overflow-hidden border border-app-header-border bg-app-bg"
      style={dialogStyles.container}
    >
      {/* Custom title bar - draggable */}
      <div
        data-tauri-drag-region
        className="flex h-10 shrink-0 items-center justify-between border-b border-app-header-border bg-app-header px-4"
      >
        <span className="font-medium text-app-text text-sm">uti</span>
        <button
          type="button"
          onClick={handleClose}
          className="flex h-7 w-7 items-center justify-center rounded text-app-text-muted hover:bg-app-item-hover hover:text-app-text"
          aria-label="Close"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path
              d="M2 2L12 12M12 2L2 12"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      {/* Content area */}
      <div className="flex flex-1 flex-col p-5">
        {/* Title with accent color based on kind */}
        <h1
          className={`mb-4 font-semibold ${
            params.kind === 'error' ? 'text-app-accent-error' : 'text-app-accent-info'
          }`}
          style={dialogStyles.heading}
        >
          {params.title}
        </h1>

        {/* Message */}
        <p className="mb-5 flex-1 whitespace-pre-wrap text-app-text" style={dialogStyles.message}>
          {params.message}
        </p>

        {/* OK button */}
        <button
          type="button"
          onClick={handleClose}
          className="w-full bg-app-item text-app-text hover:bg-app-item-hover"
          style={dialogStyles.button}
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
