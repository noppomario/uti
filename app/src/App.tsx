import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type React from 'react';
import { useCallback, useEffect, useState } from 'react';
import { ClipboardHistory, type ClipboardItem } from './components/ClipboardHistory';
import type { RecentFile } from './components/JumpList';
import { Launcher, type LauncherItem } from './components/Launcher';
import { TabBar, type TabType } from './components/TabBar';
import { useClipboard } from './hooks/useClipboard';
import { useLauncher } from './hooks/useLauncher';

/** Inline styles using CSS variables for theme-based sizing */
const headerStyles: React.CSSProperties = {
  padding: 'var(--size-padding-y) var(--size-padding-x)',
  boxShadow: 'var(--shadow-header)',
};

/**
 * Main application component
 *
 * Displays clipboard history and handles window visibility toggling.
 *
 * @returns The main application UI
 */
function App() {
  const [history, setHistory] = useState<ClipboardItem[]>([]);
  const [activeTab, setActiveTab] = useState<TabType>('clipboard');
  const [expandedItemId, setExpandedItemId] = useState<string | undefined>();
  const [recentFiles, setRecentFiles] = useState<RecentFile[]>([]);

  // Start clipboard monitoring
  useClipboard();

  // Load launcher configuration
  const { commands: launcherItems } = useLauncher();

  /**
   * Switch to the next tab
   */
  const switchTab = useCallback(
    (direction: 'left' | 'right') => {
      const tabs: TabType[] = ['clipboard', 'launcher'];
      const currentIndex = tabs.indexOf(activeTab);
      const newIndex =
        direction === 'right'
          ? (currentIndex + 1) % tabs.length
          : (currentIndex - 1 + tabs.length) % tabs.length;
      setActiveTab(tabs[newIndex]);
      // Clear jump list state when switching tabs
      setExpandedItemId(undefined);
      setRecentFiles([]);
    },
    [activeTab]
  );

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

  // Load clipboard history on initial mount
  useEffect(() => {
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
  const handleClipboardSelect = async (text: string) => {
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

  /**
   * Handles launcher item selection
   *
   * @param item - The selected launcher item
   */
  const handleLauncherSelect = async (item: LauncherItem) => {
    try {
      await invoke('execute_command', { command: item.command, args: item.args });
      console.log('Launched:', item.command);

      // Hide window after launching
      await invoke('toggle_window');
    } catch (err) {
      console.error('Failed to launch:', err);
    }
  };

  /**
   * Handles launcher item expansion to show recent files
   *
   * @param item - The item to expand
   */
  const handleLauncherExpand = async (item: LauncherItem) => {
    // Only expand if historySource is configured
    if (!item.historySource) {
      return;
    }

    try {
      let files: RecentFile[];

      if (item.historySource.type === 'recently-used') {
        // Use recently-used.xbel with optional appName and custom path
        // - System XBEL requires appName for filtering
        // - Per-app XBEL (custom path) can work without appName
        const appName = item.historySource.appName;
        const xbelPath = item.historySource.path;

        // Validate: system XBEL requires appName
        if (!xbelPath && !appName) {
          console.warn('System XBEL requires appName for:', item.name);
          return;
        }

        files = await invoke<RecentFile[]>('get_recent_files', { appName, xbelPath });
        console.log(
          'Loaded recent files for:',
          appName ?? 'all',
          xbelPath ?? 'system',
          files.length
        );
      } else if (item.historySource.type === 'vscode') {
        // Use VSCode storage.json
        const storagePath = item.historySource.path;
        if (!storagePath) {
          console.warn('No path configured for VSCode:', item.name);
          return;
        }
        files = await invoke<RecentFile[]>('get_vscode_recent_files', { storagePath });
        console.log('Loaded VSCode recent files from:', storagePath, files.length);
      } else {
        console.warn('Unknown historySource type:', item.historySource);
        return;
      }

      setRecentFiles(files);
      setExpandedItemId(item.id);
    } catch (err) {
      console.error('Failed to load recent files:', err);
    }
  };

  /**
   * Handles launcher jump list collapse
   */
  const handleLauncherCollapse = () => {
    setExpandedItemId(undefined);
    setRecentFiles([]);
  };

  /**
   * Handles file selection from jump list
   *
   * @param item - The launcher item
   * @param filePath - The selected file path
   */
  const handleLauncherSelectWithFile = async (item: LauncherItem, filePath: string) => {
    try {
      // Execute command with file path as argument
      await invoke('execute_command', {
        command: item.command,
        args: [...item.args, filePath],
      });
      console.log('Launched:', item.command, 'with file:', filePath);

      // Hide window after launching
      await invoke('toggle_window');
    } catch (err) {
      console.error('Failed to launch with file:', err);
    }
  };

  return (
    <div className="h-screen bg-app-bg flex flex-col rounded-lg overflow-hidden">
      <div
        className="relative z-10 border-b border-app-header-border bg-app-header"
        style={headerStyles}
      >
        <TabBar activeTab={activeTab} onTabChange={setActiveTab} />
      </div>
      <div className="flex-1 min-h-0">
        {activeTab === 'clipboard' && (
          <ClipboardHistory
            items={history}
            onSelect={handleClipboardSelect}
            onSwitchToNextTab={() => switchTab('right')}
          />
        )}
        {activeTab === 'launcher' && (
          <Launcher
            items={launcherItems}
            onSelect={handleLauncherSelect}
            expandedItemId={expandedItemId}
            recentFiles={recentFiles}
            onExpand={handleLauncherExpand}
            onCollapse={handleLauncherCollapse}
            onSelectWithFile={handleLauncherSelectWithFile}
            onSwitchToPrevTab={() => switchTab('left')}
          />
        )}
      </div>
    </div>
  );
}

export default App;
