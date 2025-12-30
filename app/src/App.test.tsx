import { render, screen } from '@testing-library/react';
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

  it('renders the application title', () => {
    // Arrange & Act
    render(<App />);

    // Assert
    const title = screen.getByText('Clipboard History');
    expect(title).toBeDefined();
    expect(title.tagName).toBe('H1');
  });

  it('renders empty clipboard history state', () => {
    // Arrange & Act
    render(<App />);

    // Assert
    const emptyState = screen.getByText(/no clipboard history/i);
    expect(emptyState).toBeDefined();
  });

  it('sets up event listener on mount', async () => {
    // Arrange
    const { listen } = await import('@tauri-apps/api/event');

    // Act
    render(<App />);

    // Wait for useEffect
    await new Promise(resolve => setTimeout(resolve, 0));

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
    render(<App />);
    await new Promise(resolve => setTimeout(resolve, 0));

    // Trigger double ctrl event
    if (doubleCtrlHandler) {
      doubleCtrlHandler();
    }

    // Wait for promises
    await new Promise(resolve => setTimeout(resolve, 0));

    // Assert
    expect(invoke).toHaveBeenCalledWith('toggle_window');
  });
});
