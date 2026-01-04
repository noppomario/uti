import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type React from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ClipboardHistory, type ClipboardItem } from './components/ClipboardHistory';
import type { RecentFile } from './components/JumpList';
import { Launcher, type LauncherItem } from './components/Launcher';
import { SearchBar } from './components/SearchBar';
import { TabBar, type TabType } from './components/TabBar';
import { useClipboard } from './hooks/useClipboard';
import { useLauncher } from './hooks/useLauncher';

/** Desktop application from .desktop file search */
interface DesktopApp {
  id: string;
  name: string;
  exec: string;
  icon: string | null;
  comment: string | null;
}

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
  const [searchQueries, setSearchQueries] = useState({ clipboard: '', launcher: '' });
  const [desktopApps, setDesktopApps] = useState<DesktopApp[]>([]);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const listContainerRef = useRef<HTMLElement>(null);

  // Start clipboard monitoring
  useClipboard();

  // Filter clipboard history based on search query
  const filteredHistory = useMemo(() => {
    const query = searchQueries.clipboard.toLowerCase();
    if (!query) return history;
    return history.filter(item => item.text.toLowerCase().includes(query));
  }, [history, searchQueries.clipboard]);

  // Load launcher configuration
  const { commands: launcherItems } = useLauncher();

  // Search desktop apps when launcher search query changes (debounced)
  useEffect(() => {
    const query = searchQueries.launcher;

    // Clear results if query is empty
    if (!query) {
      setDesktopApps([]);
      return;
    }

    // Debounce search
    const timer = setTimeout(async () => {
      try {
        const results = await invoke<DesktopApp[]>('search_desktop_files', { query });
        setDesktopApps(results);
      } catch (err) {
        console.error('Failed to search desktop files:', err);
        setDesktopApps([]);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [searchQueries.launcher]);

  // Convert DesktopApp to LauncherItem for display
  const desktopAppsAsLauncherItems: LauncherItem[] = useMemo(
    () =>
      desktopApps.map(app => ({
        id: app.id,
        name: app.name,
        command: app.exec.split(' ')[0], // First part of exec
        args: app.exec.split(' ').slice(1), // Rest as args
        historySource: null,
      })),
    [desktopApps]
  );

  // Determine which items to show in launcher
  const launcherDisplayItems = searchQueries.launcher ? desktopAppsAsLauncherItems : launcherItems;

  /**
   * Switch to the next tab (stops at ends, doesn't cycle)
   */
  const switchTab = useCallback(
    (direction: 'left' | 'right') => {
      const tabs: TabType[] = ['clipboard', 'launcher'];
      const currentIndex = tabs.indexOf(activeTab);
      const newIndex =
        direction === 'right'
          ? Math.min(currentIndex + 1, tabs.length - 1)
          : Math.max(currentIndex - 1, 0);
      if (newIndex !== currentIndex) {
        setActiveTab(tabs[newIndex]);
        // Clear jump list state when switching tabs
        setExpandedItemId(undefined);
        setRecentFiles([]);
      }
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
  const handleClipboardSelect = useCallback(async (text: string) => {
    try {
      // Write to system clipboard
      await writeText(text);
      console.log('Clipboard updated:', text);

      // Hide window after selection
      await invoke('toggle_window');
    } catch (err) {
      console.error('Failed to paste item:', err);
    }
  }, []);

  /**
   * Handles launcher item selection
   *
   * @param item - The selected launcher item
   */
  const handleLauncherSelect = useCallback(async (item: LauncherItem) => {
    try {
      await invoke('execute_command', { command: item.command, args: item.args });
      console.log('Launched:', item.command);

      // Hide window after launching
      await invoke('toggle_window');
    } catch (err) {
      console.error('Failed to launch:', err);
    }
  }, []);

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
   * Updates search query for current tab
   */
  const handleSearchChange = useCallback(
    (value: string) => {
      setSearchQueries(prev => ({ ...prev, [activeTab]: value }));
    },
    [activeTab]
  );

  /**
   * Focuses the search input (called from list via onUpAtTop)
   */
  const focusSearchInput = useCallback(() => {
    searchInputRef.current?.focus();
  }, []);

  /**
   * Focuses the list and clears search (called from SearchBar via onEscape)
   */
  const handleSearchEscape = useCallback(() => {
    setSearchQueries(prev => ({ ...prev, [activeTab]: '' }));
    listContainerRef.current?.focus();
  }, [activeTab]);

  /**
   * Moves focus from search bar to list (called from SearchBar via onArrowDown)
   * Only focuses if there are items to display
   */
  const focusList = useCallback(() => {
    const hasItems =
      activeTab === 'clipboard'
        ? filteredHistory.length > 0
        : (launcherDisplayItems?.length ?? 0) > 0;
    if (hasItems) {
      listContainerRef.current?.focus();
    }
  }, [activeTab, filteredHistory.length, launcherDisplayItems?.length]);

  /**
   * Selects the first item in the list (called from SearchBar via onEnter)
   */
  const handleSearchEnter = useCallback(() => {
    if (activeTab === 'clipboard') {
      if (filteredHistory.length > 0) {
        handleClipboardSelect(filteredHistory[0].text);
      }
    } else {
      if (launcherDisplayItems && launcherDisplayItems.length > 0) {
        handleLauncherSelect(launcherDisplayItems[0]);
      }
    }
  }, [
    activeTab,
    filteredHistory,
    launcherDisplayItems,
    handleClipboardSelect,
    handleLauncherSelect,
  ]);

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
        <div style={{ marginTop: 'var(--size-gap)' }}>
          <SearchBar
            value={searchQueries[activeTab]}
            onChange={handleSearchChange}
            onArrowDown={focusList}
            onEscape={handleSearchEscape}
            onArrowLeft={() => switchTab('left')}
            onArrowRight={() => switchTab('right')}
            onEnter={handleSearchEnter}
            inputRef={searchInputRef}
            placeholder={activeTab === 'clipboard' ? 'Search history...' : 'Search apps...'}
          />
        </div>
      </div>
      <div className="flex-1 min-h-0">
        {activeTab === 'clipboard' && (
          <ClipboardHistory
            items={filteredHistory}
            onSelect={handleClipboardSelect}
            onSwitchToNextTab={() => switchTab('right')}
            onUpAtTop={focusSearchInput}
            listContainerRef={listContainerRef}
          />
        )}
        {activeTab === 'launcher' && (
          <Launcher
            items={launcherDisplayItems}
            onSelect={handleLauncherSelect}
            expandedItemId={expandedItemId}
            recentFiles={recentFiles}
            onExpand={handleLauncherExpand}
            onCollapse={handleLauncherCollapse}
            onSelectWithFile={handleLauncherSelectWithFile}
            onSwitchToPrevTab={() => switchTab('left')}
            onUpAtTop={focusSearchInput}
            listContainerRef={listContainerRef}
          />
        )}
      </div>
    </div>
  );
}

export default App;
