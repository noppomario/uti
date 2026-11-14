/**
 * Update dialog entry point
 *
 * Handles the full update flow: check -> display -> update -> complete.
 * Uses DialogLayout for consistent UI and supports i18n.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useCallback, useEffect, useRef, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { useTranslation } from 'react-i18next';
import { DialogLayout } from './components/DialogLayout';
import type { AppConfig } from './config';
import { getConfig } from './config';
import './index.css';
import i18n from './i18n';

/**
 * Dialog state machine
 */
type DialogState =
  | { status: 'checking' }
  | { status: 'noUpdate'; currentVersion: string }
  | { status: 'available'; currentVersion: string; latestVersion: string; releaseUrl: string }
  | { status: 'updating'; progress: string[] }
  | { status: 'success' }
  | { status: 'error'; message: string };

/**
 * Full update check result from backend
 */
interface UpdateCheckResultFull {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  release_url: string;
  uti_rpm_url: string | null;
  daemon_rpm_url: string | null;
  gnome_extension_url: string | null;
}

/**
 * Update progress event payload
 */
interface UpdateProgress {
  step: string;
  message: string;
  is_error: boolean;
}

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

  for (const c of COLOR_THEMES) {
    root.classList.remove(`theme-${c}`);
  }
  for (const s of SIZE_THEMES) {
    root.classList.remove(`size-${s}`);
  }

  root.classList.add(`theme-${color}`);
  if (size !== 'minimal') {
    root.classList.add(`size-${size}`);
  }

  if (accentColor) {
    root.style.setProperty('--color-app-item-selected', accentColor);
  } else {
    root.style.removeProperty('--color-app-item-selected');
  }
}

/**
 * Initialize theme and language
 */
async function initThemeAndLanguage() {
  const config = await getConfig();
  applyThemeFromConfig(config);
  i18n.changeLanguage(config.language);
}

/**
 * Update dialog page component
 */
function UpdateDialogPage() {
  const { t } = useTranslation('settings');
  const [state, setState] = useState<DialogState>({ status: 'checking' });
  const progressRef = useRef<HTMLDivElement>(null);

  const checkForUpdates = useCallback(async () => {
    try {
      const result = await invoke<UpdateCheckResultFull>('check_for_updates_full');
      if (result.update_available) {
        setState({
          status: 'available',
          currentVersion: result.current_version,
          latestVersion: result.latest_version,
          releaseUrl: result.release_url,
        });
      } else {
        setState({
          status: 'noUpdate',
          currentVersion: result.current_version,
        });
      }
    } catch (err) {
      setState({
        status: 'error',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, []);

  // Check for updates on mount
  useEffect(() => {
    checkForUpdates();
  }, [checkForUpdates]);

  // Listen for progress events
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const setupListener = async () => {
      unlisten = await listen<UpdateProgress>('update-progress', event => {
        setState(prev => {
          if (prev.status !== 'updating') return prev;
          const newProgress = [...prev.progress, event.payload.message];
          // Auto-scroll to bottom
          setTimeout(() => {
            progressRef.current?.scrollTo({
              top: progressRef.current.scrollHeight,
              behavior: 'smooth',
            });
          }, 0);

          if (event.payload.step === 'complete') {
            return { status: 'success' };
          }
          if (event.payload.is_error) {
            return { status: 'error', message: event.payload.message };
          }
          return { ...prev, progress: newProgress };
        });
      });
    };

    setupListener();
    return () => {
      unlisten?.();
    };
  }, []);

  // Escape key handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().close();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleUpdate = async () => {
    setState({ status: 'updating', progress: [] });
    try {
      await invoke('perform_update_gui');
      // Success is handled via event listener when 'complete' step is received
    } catch (err) {
      setState({
        status: 'error',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const handleOpenRelease = (url: string) => {
    invoke('open_url', { url });
  };

  const handleClose = () => {
    getCurrentWindow().close();
  };

  // Render content based on state
  const renderContent = () => {
    switch (state.status) {
      case 'checking':
        return (
          <div className="flex flex-1 flex-col items-center justify-center p-6">
            <div className="mb-4 h-8 w-8 animate-spin rounded-full border-4 border-app-text-muted border-t-app-accent-info" />
            <p className="text-app-text">{t('update.checking')}</p>
          </div>
        );

      case 'noUpdate':
        return (
          <div className="flex flex-1 flex-col items-center justify-center p-6">
            <svg
              className="mb-4 h-12 w-12 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-app-text font-medium">{t('update.upToDate')}</p>
            <p className="mt-1 text-app-text-muted text-sm">v{state.currentVersion}</p>
          </div>
        );

      case 'available':
        return (
          <div className="flex flex-1 flex-col p-4">
            <div className="mb-4 text-center">
              <h2 className="text-app-accent-info font-semibold text-lg">
                {t('update.available', { version: state.latestVersion })}
              </h2>
              <p className="mt-1 text-app-text-muted text-sm">
                {t('update.currentVersion', { version: state.currentVersion })}
              </p>
            </div>
            <button
              type="button"
              onClick={() => handleOpenRelease(state.releaseUrl)}
              className="mb-4 text-app-accent-info hover:underline text-sm"
            >
              {t('update.viewRelease')} &rarr;
            </button>
          </div>
        );

      case 'updating':
        return (
          <div className="flex flex-1 flex-col p-4">
            <p className="mb-2 text-app-text font-medium">{t('update.updating')}</p>
            <div
              ref={progressRef}
              className="flex-1 overflow-y-auto rounded bg-app-item p-3 font-mono text-sm"
            >
              {state.progress.map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: progress lines are append-only
                <div key={i} className="text-app-text-muted">
                  {line}
                </div>
              ))}
            </div>
          </div>
        );

      case 'success':
        return (
          <div className="flex flex-1 flex-col items-center justify-center p-6">
            <svg
              className="mb-4 h-12 w-12 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-app-text font-medium">{t('update.success')}</p>
            <p className="mt-2 text-center text-app-text-muted text-sm">
              {t('update.successMessage')}
            </p>
          </div>
        );

      case 'error':
        return (
          <div className="flex flex-1 flex-col items-center justify-center p-6">
            <svg
              className="mb-4 h-12 w-12 text-app-accent-error"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-app-accent-error font-medium">{t('update.error')}</p>
            <p className="mt-2 text-center text-app-text-muted text-sm">{state.message}</p>
          </div>
        );
    }
  };

  // Render footer based on state
  const renderFooter = () => {
    switch (state.status) {
      case 'available':
        return (
          <>
            <button
              type="button"
              onClick={handleClose}
              className="rounded bg-app-item px-4 py-2 text-app-text hover:bg-app-item-hover text-sm"
            >
              {t('update.closeButton')}
            </button>
            <button
              type="button"
              onClick={handleUpdate}
              className="rounded bg-app-accent-info px-4 py-2 text-white hover:opacity-90 text-sm"
            >
              {t('update.updateButton')}
            </button>
          </>
        );

      case 'updating':
        return null; // No buttons during update

      default:
        return (
          <button
            type="button"
            onClick={handleClose}
            className="rounded bg-app-item px-4 py-2 text-app-text hover:bg-app-item-hover text-sm"
          >
            {t('update.closeButton')}
          </button>
        );
    }
  };

  return (
    <DialogLayout title={t('update.title')} footer={renderFooter()}>
      {renderContent()}
    </DialogLayout>
  );
}

// Initialize theme, language, and render
initThemeAndLanguage();

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<UpdateDialogPage />);
