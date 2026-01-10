/**
 * Tests for TabBar component
 *
 * Displays tabs for switching between Clipboard, Snippets, and Launcher views.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TabBar } from './TabBar';

describe('TabBar', () => {
  it('renders all three tabs', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    expect(screen.getByRole('tab', { name: /clipboard/i })).toBeDefined();
    expect(screen.getByRole('tab', { name: /snippets/i })).toBeDefined();
    expect(screen.getByRole('tab', { name: /launcher/i })).toBeDefined();
  });

  it('marks clipboard tab as active when activeTab is clipboard', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const snippetsTab = screen.getByRole('tab', { name: /snippets/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    expect(clipboardTab.getAttribute('aria-selected')).toBe('true');
    expect(snippetsTab.getAttribute('aria-selected')).toBe('false');
    expect(launcherTab.getAttribute('aria-selected')).toBe('false');
  });

  it('marks snippets tab as active when activeTab is snippets', () => {
    render(<TabBar activeTab="snippets" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const snippetsTab = screen.getByRole('tab', { name: /snippets/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    expect(clipboardTab.getAttribute('aria-selected')).toBe('false');
    expect(snippetsTab.getAttribute('aria-selected')).toBe('true');
    expect(launcherTab.getAttribute('aria-selected')).toBe('false');
  });

  it('marks launcher tab as active when activeTab is launcher', () => {
    render(<TabBar activeTab="launcher" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const snippetsTab = screen.getByRole('tab', { name: /snippets/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    expect(clipboardTab.getAttribute('aria-selected')).toBe('false');
    expect(snippetsTab.getAttribute('aria-selected')).toBe('false');
    expect(launcherTab.getAttribute('aria-selected')).toBe('true');
  });

  it('calls onTabChange with clipboard when clipboard tab is clicked', () => {
    const handleTabChange = vi.fn();
    render(<TabBar activeTab="launcher" onTabChange={handleTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /clipboard/i }));

    expect(handleTabChange).toHaveBeenCalledWith('clipboard');
  });

  it('calls onTabChange with snippets when snippets tab is clicked', () => {
    const handleTabChange = vi.fn();
    render(<TabBar activeTab="clipboard" onTabChange={handleTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /snippets/i }));

    expect(handleTabChange).toHaveBeenCalledWith('snippets');
  });

  it('calls onTabChange with launcher when launcher tab is clicked', () => {
    const handleTabChange = vi.fn();
    render(<TabBar activeTab="clipboard" onTabChange={handleTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /launcher/i }));

    expect(handleTabChange).toHaveBeenCalledWith('launcher');
  });

  it('has tablist role for the container', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    expect(screen.getByRole('tablist')).toBeDefined();
  });

  it('applies active styling to selected tab', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });

    // Active tab should have font-semibold class
    expect(clipboardTab.className).toContain('font-semibold');
  });

  it('applies inactive styling to non-selected tab', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    // Inactive tab should have text-app-text-muted class
    expect(launcherTab.className).toContain('text-app-text-muted');
  });

  describe('icons and accessibility', () => {
    it('renders icons for each tab', () => {
      render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

      const tabs = screen.getAllByRole('tab');
      // Each tab should contain an SVG icon
      for (const tab of tabs) {
        expect(tab.querySelector('svg')).not.toBeNull();
      }
    });

    it('icons have aria-hidden attribute', () => {
      render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

      const tabs = screen.getAllByRole('tab');
      for (const tab of tabs) {
        const svg = tab.querySelector('svg');
        expect(svg?.getAttribute('aria-hidden')).toBe('true');
      }
    });

    it('has title attribute for tooltip on each tab', () => {
      render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

      const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
      const snippetsTab = screen.getByRole('tab', { name: /snippets/i });
      const launcherTab = screen.getByRole('tab', { name: /launcher/i });

      expect(clipboardTab.getAttribute('title')).toBe('Clipboard');
      expect(snippetsTab.getAttribute('title')).toBe('Snippets');
      expect(launcherTab.getAttribute('title')).toBe('Launcher');
    });

    it('renders text labels with tab-label class', () => {
      render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

      const tabs = screen.getAllByRole('tab');
      for (const tab of tabs) {
        const label = tab.querySelector('.tab-label');
        expect(label).not.toBeNull();
      }
    });
  });
});
