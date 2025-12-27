import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';

/**
 * Mock Tauri API
 *
 * Tauri APIs are not available in test environment, so we mock them.
 */
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the application title', () => {
    // Arrange & Act
    render(<App />);

    // Assert
    const title = screen.getByText('uti');
    expect(title).toBeDefined();
    expect(title.tagName).toBe('H1');
  });

  it('renders the instruction text', () => {
    // Arrange & Act
    render(<App />);

    // Assert
    const instruction = screen.getByText('Press Ctrl twice to toggle visibility');
    expect(instruction).toBeDefined();
  });

  it('renders the toggle button', () => {
    // Arrange & Act
    render(<App />);

    // Assert
    const button = screen.getByRole('button', { name: /toggle window/i });
    expect(button).toBeDefined();
    expect(button).toHaveProperty('type', 'button');
  });

  it('calls invoke with toggle_window when button is clicked', async () => {
    // Arrange
    const { invoke } = await import('@tauri-apps/api/core');
    render(<App />);

    // Act
    const button = screen.getByRole('button', { name: /toggle window/i });
    fireEvent.click(button);

    // Assert
    expect(invoke).toHaveBeenCalledWith('toggle_window');
  });

  it('sets up event listener on mount', async () => {
    // Arrange
    const { listen } = await import('@tauri-apps/api/event');

    // Act
    render(<App />);

    // Wait for useEffect to complete
    await new Promise(resolve => setTimeout(resolve, 0));

    // Assert
    expect(listen).toHaveBeenCalledWith('double-ctrl-pressed', expect.any(Function));
  });
});
