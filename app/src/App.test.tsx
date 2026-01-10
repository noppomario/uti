import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';

/**
 * Mock Tauri API
 *
 * Tauri APIs are not available in test environment, so we mock them.
 */
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve([])),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    onFocusChanged: vi.fn(() => Promise.resolve(() => {})),
  })),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(() => Promise.resolve(null)),
  writeText: vi.fn(() => Promise.resolve()),
}));

vi.mock('./hooks/useClipboard', () => ({
  useClipboard: vi.fn(),
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the tab bar with clipboard and launcher tabs', async () => {
    // Arrange & Act
    await act(async () => {
      render(<App />);
    });

    // Assert
    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });
    expect(clipboardTab).toBeDefined();
    expect(launcherTab).toBeDefined();
    // Clipboard tab should be active by default
    expect(clipboardTab.getAttribute('aria-selected')).toBe('true');
  });

  it('renders empty clipboard history state', async () => {
    // Arrange & Act
    await act(async () => {
      render(<App />);
    });

    // Assert
    const emptyState = screen.getByText(/no clipboard history/i);
    expect(emptyState).toBeDefined();
  });

  it('sets up event listener on mount', async () => {
    // Arrange
    const { listen } = await import('@tauri-apps/api/event');

    // Act
    await act(async () => {
      render(<App />);
    });

    // Assert
    expect(listen).toHaveBeenCalledWith('double-ctrl-pressed', expect.any(Function));
  });

  it('loads clipboard history when double ctrl is pressed', async () => {
    // Arrange
    const { invoke } = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');

    const mockHistory = [
      { text: 'Test item 1', timestamp: 123 },
      { text: 'Test item 2', timestamp: 456 },
    ];

    vi.mocked(invoke).mockResolvedValueOnce(mockHistory);

    let doubleCtrlHandler: (() => void) | undefined;
    vi.mocked(listen).mockImplementation((event, handler) => {
      if (event === 'double-ctrl-pressed') {
        doubleCtrlHandler = handler as () => void;
      }
      return Promise.resolve(() => {});
    });

    // Act
    await act(async () => {
      render(<App />);
    });

    // Trigger double ctrl event
    await act(async () => {
      if (doubleCtrlHandler) {
        doubleCtrlHandler();
      }
    });

    // Assert
    expect(invoke).toHaveBeenCalledWith('toggle_window');
  });

  describe('keyboard shortcuts', () => {
    it('focuses search bar with Ctrl+F', async () => {
      // Arrange
      await act(async () => {
        render(<App />);
      });

      // Get search input
      const searchInput = screen.getByPlaceholderText(/search history/i);

      // Ensure search is not focused initially
      expect(document.activeElement).not.toBe(searchInput);

      // Act - Press Ctrl+F
      await act(async () => {
        fireEvent.keyDown(document, { key: 'f', ctrlKey: true });
      });

      // Assert - Search bar should be focused
      expect(document.activeElement).toBe(searchInput);
    });

    it('focuses search bar with Ctrl+F on launcher tab', async () => {
      // Arrange
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'get_clipboard_history') return [];
        if (cmd === 'get_launcher_config') {
          return { commands: [] };
        }
        return [];
      });

      await act(async () => {
        render(<App />);
      });

      // Switch to launcher tab
      const launcherTab = screen.getByRole('tab', { name: /launcher/i });
      await act(async () => {
        fireEvent.click(launcherTab);
      });

      // Wait for launcher config to load
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 50));
      });

      // Get search input
      const searchInput = screen.getByPlaceholderText(/search apps/i);

      // Ensure search is not focused initially
      expect(document.activeElement).not.toBe(searchInput);

      // Act - Press Ctrl+F
      await act(async () => {
        fireEvent.keyDown(document, { key: 'f', ctrlKey: true });
      });

      // Assert - Search bar should be focused
      expect(document.activeElement).toBe(searchInput);
    });
  });

  describe('keyboard navigation', () => {
    it('focuses list from search bar with ArrowDown on launcher tab', async () => {
      // Arrange
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'get_clipboard_history') return [];
        if (cmd === 'get_launcher_config') {
          return { commands: [{ id: '1', name: 'Test App', command: 'test', args: [] }] };
        }
        return [];
      });

      await act(async () => {
        render(<App />);
      });

      // Switch to launcher tab
      const launcherTab = screen.getByRole('tab', { name: /launcher/i });
      await act(async () => {
        fireEvent.click(launcherTab);
      });

      // Focus search bar
      const searchInput = screen.getByPlaceholderText(/search apps/i);
      await act(async () => {
        searchInput.focus();
      });

      // Press ArrowDown to focus list
      await act(async () => {
        fireEvent.keyDown(searchInput, { key: 'ArrowDown' });
      });

      // Wait for requestAnimationFrame
      await act(async () => {
        await new Promise(resolve => requestAnimationFrame(resolve));
      });

      // List should be focused (document.activeElement should not be input)
      expect(document.activeElement).not.toBe(searchInput);
    });

    it('focuses search bar from list with ArrowUp at first item on launcher tab', async () => {
      // Arrange
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'get_clipboard_history') return [];
        if (cmd === 'get_launcher_config') {
          return { commands: [{ id: '1', name: 'Test App', command: 'test', args: [] }] };
        }
        return [];
      });

      await act(async () => {
        render(<App />);
      });

      // Switch to launcher tab
      const launcherTab = screen.getByRole('tab', { name: /launcher/i });
      await act(async () => {
        fireEvent.click(launcherTab);
      });

      // Wait for launcher config to load
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 50));
      });

      // Find and focus the list
      const list = document.querySelector('ul[tabindex="0"]');
      expect(list).not.toBeNull();

      await act(async () => {
        (list as HTMLElement).focus();
      });

      // Get search input
      const searchInput = screen.getByPlaceholderText(/search apps/i);

      // Press ArrowUp to focus search bar
      await act(async () => {
        fireEvent.keyDown(list as HTMLElement, { key: 'ArrowUp' });
      });

      // Search bar should be focused
      expect(document.activeElement).toBe(searchInput);
    });
  });
});
