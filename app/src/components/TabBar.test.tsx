/**
 * Tests for TabBar component
 *
 * Displays tabs for switching between Clipboard and Launcher views.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TabBar } from './TabBar';

describe('TabBar', () => {
  it('renders clipboard and launcher tabs', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    expect(screen.getByRole('tab', { name: /clipboard/i })).toBeDefined();
    expect(screen.getByRole('tab', { name: /launcher/i })).toBeDefined();
  });

  it('marks clipboard tab as active when activeTab is clipboard', () => {
    render(<TabBar activeTab="clipboard" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    expect(clipboardTab.getAttribute('aria-selected')).toBe('true');
    expect(launcherTab.getAttribute('aria-selected')).toBe('false');
  });

  it('marks launcher tab as active when activeTab is launcher', () => {
    render(<TabBar activeTab="launcher" onTabChange={() => {}} />);

    const clipboardTab = screen.getByRole('tab', { name: /clipboard/i });
    const launcherTab = screen.getByRole('tab', { name: /launcher/i });

    expect(clipboardTab.getAttribute('aria-selected')).toBe('false');
    expect(launcherTab.getAttribute('aria-selected')).toBe('true');
  });

  it('calls onTabChange with clipboard when clipboard tab is clicked', () => {
    const handleTabChange = vi.fn();
    render(<TabBar activeTab="launcher" onTabChange={handleTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /clipboard/i }));

    expect(handleTabChange).toHaveBeenCalledWith('clipboard');
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
});
