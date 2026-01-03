/**
 * Tests for Launcher component
 *
 * Displays launcher commands with keyboard navigation.
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { RecentFile } from './JumpList';
import { Launcher, type LauncherItem } from './Launcher';

describe('Launcher', () => {
  const mockItems: LauncherItem[] = [
    {
      id: 'nautilus',
      name: 'Files',
      command: 'nautilus',
      args: [],
      historySource: null,
    },
    {
      id: 'gnome-text-editor',
      name: 'Text Editor',
      command: 'gnome-text-editor',
      args: [],
      historySource: null,
    },
  ];

  it('renders empty state when no items', () => {
    render(<Launcher items={[]} onSelect={() => {}} />);

    expect(screen.getByText(/no launcher commands/i)).toBeDefined();
  });

  it('calls onSwitchToPrevTab when ArrowLeft is pressed with items', () => {
    const handleSwitchToPrevTab = vi.fn();
    const { container } = render(
      <Launcher items={mockItems} onSelect={() => {}} onSwitchToPrevTab={handleSwitchToPrevTab} />
    );
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: 'ArrowLeft' });

    expect(handleSwitchToPrevTab).toHaveBeenCalled();
  });

  it('calls onSwitchToPrevTab when ArrowLeft is pressed with empty items', () => {
    const handleSwitchToPrevTab = vi.fn();
    render(<Launcher items={[]} onSelect={() => {}} onSwitchToPrevTab={handleSwitchToPrevTab} />);

    const emptyState = screen.getByText(/no launcher commands/i);
    fireEvent.keyDown(emptyState, { key: 'ArrowLeft' });

    expect(handleSwitchToPrevTab).toHaveBeenCalled();
  });

  it('renders all launcher items', () => {
    render(<Launcher items={mockItems} onSelect={() => {}} />);

    expect(screen.getByText(/Files/)).toBeDefined();
    expect(screen.getByText(/Text Editor/)).toBeDefined();
  });

  it('calls onSelect with item when clicked', () => {
    const handleSelect = vi.fn();
    render(<Launcher items={mockItems} onSelect={handleSelect} />);

    fireEvent.click(screen.getByText(/Files/));

    expect(handleSelect).toHaveBeenCalledWith(mockItems[0]);
  });

  it('highlights first item by default', () => {
    const { container } = render(<Launcher items={mockItems} onSelect={() => {}} />);

    const firstItem = container.querySelector('[data-selected="true"]');
    expect(firstItem?.textContent).toContain('1:');
  });

  it('navigates with arrow keys', () => {
    const { container } = render(<Launcher items={mockItems} onSelect={() => {}} />);

    // Initially first item is selected
    let selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('1:');

    // Press down arrow
    const list = container.querySelector('ul') as Element;
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('2:');

    // Press up arrow
    fireEvent.keyDown(list, { key: 'ArrowUp' });
    selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('1:');
  });

  it('stays at last item when navigating past end', () => {
    const { container } = render(<Launcher items={mockItems} onSelect={() => {}} />);
    const list = container.querySelector('ul') as Element;

    // Go to second item
    fireEvent.keyDown(list, { key: 'ArrowDown' });
    // Go past end - should stay at last
    fireEvent.keyDown(list, { key: 'ArrowDown' });

    const selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('2:');
  });

  it('stays at first item when navigating before start', () => {
    const { container } = render(<Launcher items={mockItems} onSelect={() => {}} />);
    const list = container.querySelector('ul') as Element;

    // Go before start - should stay at first
    fireEvent.keyDown(list, { key: 'ArrowUp' });

    const selected = container.querySelector('[data-selected="true"]');
    expect(selected?.textContent).toContain('1:');
  });

  it('selects item with Enter key', () => {
    const handleSelect = vi.fn();
    const { container } = render(<Launcher items={mockItems} onSelect={handleSelect} />);
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: 'Enter' });

    expect(handleSelect).toHaveBeenCalledWith(mockItems[0]);
  });

  it('selects with number keys', () => {
    const handleSelect = vi.fn();
    const { container } = render(<Launcher items={mockItems} onSelect={handleSelect} />);
    const list = container.querySelector('ul') as Element;

    fireEvent.keyDown(list, { key: '2' });

    expect(handleSelect).toHaveBeenCalledWith(mockItems[1]);
  });

  describe('JumpList integration', () => {
    const mockRecentFiles: RecentFile[] = [
      {
        path: '/home/user/Documents/main.rs',
        name: 'main.rs',
        timestamp: '2024-01-01T12:00:00Z',
      },
      {
        path: '/home/user/Documents/config.ts',
        name: 'config.ts',
        timestamp: '2024-01-01T11:00:00Z',
      },
    ];

    it('calls onExpand when ArrowRight is pressed', () => {
      const handleExpand = vi.fn();
      const { container } = render(
        <Launcher items={mockItems} onSelect={() => {}} onExpand={handleExpand} />
      );
      const list = container.querySelector('ul') as Element;

      fireEvent.keyDown(list, { key: 'ArrowRight' });

      expect(handleExpand).toHaveBeenCalledWith(mockItems[0]);
    });

    it('shows jump list when expandedItemId matches an item', () => {
      render(
        <Launcher
          items={mockItems}
          onSelect={() => {}}
          expandedItemId="nautilus"
          recentFiles={mockRecentFiles}
        />
      );

      expect(screen.getByText(/main\.rs/)).toBeDefined();
      expect(screen.getByText(/config\.ts/)).toBeDefined();
    });

    it('calls onSelectWithFile when file is selected in jump list', () => {
      const handleSelectWithFile = vi.fn();
      render(
        <Launcher
          items={mockItems}
          onSelect={() => {}}
          expandedItemId="nautilus"
          recentFiles={mockRecentFiles}
          onSelectWithFile={handleSelectWithFile}
        />
      );

      fireEvent.click(screen.getByText(/main\.rs/));

      expect(handleSelectWithFile).toHaveBeenCalledWith(
        mockItems[0],
        '/home/user/Documents/main.rs'
      );
    });

    it('calls onCollapse when ArrowLeft is pressed in jump list', () => {
      const handleCollapse = vi.fn();
      const { container } = render(
        <Launcher
          items={mockItems}
          onSelect={() => {}}
          expandedItemId="nautilus"
          recentFiles={mockRecentFiles}
          onCollapse={handleCollapse}
        />
      );
      // Find the jump list (second ul in the container)
      const jumpList = container.querySelectorAll('ul')[1] as Element;

      fireEvent.keyDown(jumpList, { key: 'ArrowLeft' });

      expect(handleCollapse).toHaveBeenCalled();
    });

    it('shows expand indicator for items with historySource', () => {
      const itemsWithHistory: LauncherItem[] = [
        {
          id: 'vscode',
          name: 'VSCode',
          command: 'code',
          args: [],
          historySource: { type: 'vscode', path: '~/.config/Code' },
        },
        ...mockItems,
      ];
      render(<Launcher items={itemsWithHistory} onSelect={() => {}} />);

      // VSCode has historySource, so expand indicator button should exist
      const expandIndicator = screen.getByRole('button', { name: 'Expand' });
      expect(expandIndicator).toBeDefined();
    });

    it('calls onExpand when expand indicator is clicked', () => {
      const handleExpand = vi.fn();
      const handleSelect = vi.fn();
      const itemsWithHistory: LauncherItem[] = [
        {
          id: 'vscode',
          name: 'VSCode',
          command: 'code',
          args: [],
          historySource: { type: 'vscode', path: '~/.config/Code' },
        },
      ];
      render(<Launcher items={itemsWithHistory} onSelect={handleSelect} onExpand={handleExpand} />);

      // When item has historySource, there are two buttons: main and suffix
      const buttons = screen.getAllByRole('button');
      expect(buttons).toHaveLength(2);

      // Click the expand indicator (suffix button)
      const expandIndicator = screen.getByRole('button', { name: 'Expand' });
      fireEvent.click(expandIndicator);

      expect(handleExpand).toHaveBeenCalledWith(itemsWithHistory[0]);
      expect(handleSelect).not.toHaveBeenCalled();
    });

    it('calls onCollapse when expand indicator is clicked on expanded item', () => {
      const handleExpand = vi.fn();
      const handleCollapse = vi.fn();
      const itemsWithHistory: LauncherItem[] = [
        {
          id: 'vscode',
          name: 'VSCode',
          command: 'code',
          args: [],
          historySource: { type: 'vscode', path: '~/.config/Code' },
        },
      ];
      render(
        <Launcher
          items={itemsWithHistory}
          onSelect={() => {}}
          onExpand={handleExpand}
          onCollapse={handleCollapse}
          expandedItemId="vscode"
          recentFiles={[]}
        />
      );

      // Click the expand indicator on already-expanded item
      const expandIndicator = screen.getByRole('button', { name: 'Expand' });
      fireEvent.click(expandIndicator);

      expect(handleCollapse).toHaveBeenCalled();
      expect(handleExpand).not.toHaveBeenCalled();
    });

    it('calls onCollapse when clicking on empty space', () => {
      const handleCollapse = vi.fn();
      const itemsWithHistory: LauncherItem[] = [
        {
          id: 'vscode',
          name: 'VSCode',
          command: 'code',
          args: [],
          historySource: { type: 'vscode', path: '~/.config/Code' },
        },
      ];
      const { container } = render(
        <Launcher
          items={itemsWithHistory}
          onSelect={() => {}}
          onCollapse={handleCollapse}
          expandedItemId="vscode"
          recentFiles={[]}
        />
      );

      // Click directly on the ul (empty space), not on items
      const list = container.querySelector('ul') as HTMLElement;
      fireEvent.click(list);

      expect(handleCollapse).toHaveBeenCalled();
    });

    it('does not call onCollapse when clicking on item', () => {
      const handleCollapse = vi.fn();
      const handleSelect = vi.fn();
      const itemsWithHistory: LauncherItem[] = [
        {
          id: 'vscode',
          name: 'VSCode',
          command: 'code',
          args: [],
          historySource: { type: 'vscode', path: '~/.config/Code' },
        },
      ];
      render(
        <Launcher
          items={itemsWithHistory}
          onSelect={handleSelect}
          onCollapse={handleCollapse}
          expandedItemId="vscode"
          recentFiles={[]}
        />
      );

      // Click on the item content (left half button)
      const mainButton = screen.getByRole('button', { name: /VSCode/ });
      fireEvent.click(mainButton);

      // onCollapse should NOT be called (click was on item, not empty space)
      expect(handleCollapse).not.toHaveBeenCalled();
      expect(handleSelect).toHaveBeenCalled();
    });
  });
});
