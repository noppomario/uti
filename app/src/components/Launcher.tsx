/**
 * Launcher component
 *
 * Displays a list of launcher commands with keyboard navigation.
 * Uses CSS variables for sizing to support theme-based scaling.
 */

import { FolderOpen } from 'lucide-react';
import type React from 'react';
import { useCallback } from 'react';
import { useListKeyboardNavigation } from '../hooks/useListKeyboardNavigation';
import { JumpList, type RecentFile } from './JumpList';
import { ListItem } from './ListItem';

/**
 * History source configuration for jump list
 */
export interface HistorySource {
  type: 'recently-used' | 'vscode';
  appName?: string;
  path?: string;
}

/**
 * Launcher item from backend
 */
export interface LauncherItem {
  id: string;
  name: string;
  command: string;
  args: string[];
  historySource: HistorySource | null;
}

/**
 * Props for Launcher component
 */
interface LauncherProps {
  /** List of launcher items */
  items: LauncherItem[];
  /** Called when an item is selected */
  onSelect: (item: LauncherItem) => void;
  /** ID of the item whose jump list is expanded */
  expandedItemId?: string;
  /** Recent files for the expanded item */
  recentFiles?: RecentFile[];
  /** Called when user wants to expand an item's jump list */
  onExpand?: (item: LauncherItem) => void;
  /** Called when user wants to collapse the jump list */
  onCollapse?: () => void;
  /** Called when a file is selected from the jump list */
  onSelectWithFile?: (item: LauncherItem, filePath: string) => void;
  /** Called when user wants to switch to previous tab */
  onSwitchToPrevTab?: () => void;
}

/** Inline styles using CSS variables for theme-based sizing */
const listStyles: React.CSSProperties = {
  gap: 'var(--size-gap)',
  padding: 'var(--size-gap)',
};

const emptyStyles: React.CSSProperties = {
  fontSize: 'var(--size-font-base)',
  padding: 'calc(var(--size-padding-x) * 2)',
};

/**
 * Launcher component displaying commands with keyboard navigation
 *
 * @param props - Component props
 * @returns The Launcher UI
 */
export function Launcher({
  items,
  onSelect,
  expandedItemId,
  recentFiles,
  onExpand,
  onCollapse,
  onSelectWithFile,
  onSwitchToPrevTab,
}: LauncherProps) {
  // Find the expanded item
  const expandedItem = expandedItemId ? items.find(item => item.id === expandedItemId) : null;

  const {
    selectedIndex,
    containerRef,
    handleKeyDown: baseHandleKeyDown,
  } = useListKeyboardNavigation(items, {
    onSelect: item => onSelect(item),
    onLeft: onSwitchToPrevTab,
    onRight: item => onExpand?.(item),
  });

  /**
   * Extended keyboard handler with number key support
   */
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Number key selection (1-9)
      if (e.key >= '1' && e.key <= '9') {
        const index = Number.parseInt(e.key, 10) - 1;
        if (index < items.length) {
          e.preventDefault();
          onSelect(items[index]);
          return;
        }
      }
      // Fall through to base handler
      baseHandleKeyDown(e);
    },
    [items, onSelect, baseHandleKeyDown]
  );

  /**
   * Handle file selection from jump list
   */
  const handleFileSelect = useCallback(
    (filePath: string) => {
      if (expandedItem && onSelectWithFile) {
        onSelectWithFile(expandedItem, filePath);
      }
    },
    [expandedItem, onSelectWithFile]
  );

  /**
   * Handle jump list close
   */
  const handleJumpListClose = useCallback(() => {
    if (onCollapse) {
      onCollapse();
    }
    // Return focus to launcher list
    containerRef.current?.focus();
  }, [onCollapse, containerRef]);

  /**
   * Handle click on empty space (not on items) to close jump list
   */
  const handleListClick = useCallback(
    (e: React.MouseEvent<HTMLUListElement>) => {
      // Only close if clicking directly on the ul (empty space), not on items
      if (e.target === e.currentTarget && onCollapse) {
        onCollapse();
      }
    },
    [onCollapse]
  );

  if (items.length === 0) {
    return (
      // biome-ignore lint/a11y/noStaticElementInteractions: Keyboard navigation requires handler
      <div
        ref={containerRef as React.RefObject<HTMLDivElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="text-center text-app-text-muted focus:outline-none"
        style={emptyStyles}
      >
        No launcher commands configured
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <ul
        ref={containerRef as React.RefObject<HTMLUListElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        onClick={handleListClick}
        className={`flex-1 flex flex-col focus:outline-none overflow-y-auto min-h-0 ${expandedItem ? 'w-1/2 border-r border-app-header-border' : 'w-full'}`}
        style={listStyles}
      >
        {items.map((item, index) => (
          <li key={item.id}>
            <ListItem
              selected={index === selectedIndex}
              index={index}
              onClick={() => onSelect(item)}
              suffix={item.historySource ? <FolderOpen size={14} /> : undefined}
              onSuffixClick={
                item.historySource
                  ? () => {
                      if (expandedItemId === item.id) {
                        onCollapse?.();
                      } else {
                        onExpand?.(item);
                      }
                    }
                  : undefined
              }
            >
              {item.name}
            </ListItem>
          </li>
        ))}
      </ul>
      {expandedItem && recentFiles && (
        <div className="w-1/2 h-full">
          <JumpList files={recentFiles} onSelect={handleFileSelect} onClose={handleJumpListClose} />
        </div>
      )}
    </div>
  );
}
