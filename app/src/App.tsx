import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useCallback, useEffect, useState } from 'react';
import { ClipboardHistory, type ClipboardItem } from './components/ClipboardHistory';
import { type AppConfig, defaultConfig, getConfig } from './config';
import { useClipboard } from './hooks/useClipboard';

/**
 * Main application component
 *
 * Displays clipboard history and handles window visibility toggling.
 *
 * @returns The main application UI
 */
function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);
  const [config, setConfig] = useState<AppConfig>(defaultConfig);

  // Start clipboard monitoring
  useClipboard();

  /**
   * Loads clipboard history from backend
   */
  const loadHistory = useCallback(async () => {
    try {
      const items = await invoke<ClipboardItem[]>('get_clipboard_history');
      setHistory(items);
    } catch (err) {
      console.error('Failed to load clipboard history:', err);
    }
  }, []);

  // Load config and clipboard history on initial mount
  useEffect(() => {
    getConfig().then(setConfig);
    loadHistory();
  }, [loadHistory]);

  // Listen for window focus to reload history
  useEffect(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        console.log('Window focused, reloading clipboard history');
        loadHistory();
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [loadHistory]);

  // Listen for double Ctrl press to toggle window
  useEffect(() => {
    /**
     * Sets up the event listener for double Ctrl press events
     *
     * @returns Cleanup function to remove the event listener
     */
    const setupListener = async () => {
      console.log('Setting up double-ctrl-pressed event listener...');
      const unlisten = await listen('double-ctrl-pressed', () => {
        console.log('Double Ctrl event received from Rust backend');
        console.log('Invoking toggle_window command...');
        invoke('toggle_window')
          .then(() => {
            console.log('toggle_window command completed');
            // History will be loaded by onFocusChanged event
          })
          .catch(err => {
            console.error('toggle_window command failed:', err);
          });
      });

      console.log('Event listener registered successfully');
      return unlisten;
    };

    let unlisten: (() => void) | undefined;
    setupListener()
      .then(fn => {
        unlisten = fn;
      })
      .catch(err => {
        console.error('Failed to setup event listener:', err);
      });

    return () => {
      if (unlisten) {
        console.log('Cleaning up event listener');
        unlisten();
      }
    };
  }, []);

  /**
   * Handles clipboard item selection
   *
   * @param text - The selected clipboard text
   */
  const handleSelect = async (text: string) => {
    try {
      // Write to system clipboard
      await writeText(text);
      console.log('Clipboard updated:', text);

      // Hide window after selection
      await invoke('toggle_window');
    } catch (err) {
      console.error('Failed to paste item:', err);
    }
  };

  return (
    <div className="h-screen bg-app-bg flex flex-col">
      <div className="border-b border-app-header-border px-2 py-1 bg-app-header">
        <div className="flex items-center justify-between">
          <h1 className="text-xs font-semibold text-app-text">Clipboard History</h1>
          <span className="text-xs text-app-text-muted" title="Dark mode indicator">
            <span className="hidden dark:inline">🌙</span>
            <span className="inline dark:hidden">☀️</span>
          </span>
        </div>
      </div>
      <div className="flex-1 min-h-0">
        <ClipboardHistory
          items={history}
          onSelect={handleSelect}
          showTooltip={config.showTooltip}
          tooltipDelay={config.tooltipDelay}
        />
      </div>
    </div>
  );
}

export default App;
