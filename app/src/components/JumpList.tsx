/**
 * JumpList component
 *
 * Displays recent files for a launcher command with keyboard navigation.
 */

import type React from 'react';
import { useListKeyboardNavigation } from '../hooks/useListKeyboardNavigation';
import { ListItem } from './ListItem';

/**
 * Recent file from backend
 */
export interface RecentFile {
  path: string;
  name: string;
  timestamp: string;
}

/**
 * Props for JumpList component
 */
interface JumpListProps {
  /** List of recent files */
  files: RecentFile[];
  /** Called when a file is selected */
  onSelect: (path: string) => void;
  /** Called when the jump list should close */
  onClose: () => void;
}

/**
 * JumpList component displaying recent files with keyboard navigation
 *
 * @param props - Component props
 * @returns The JumpList UI
 */
export function JumpList({ files, onSelect, onClose }: JumpListProps) {
  const { selectedIndex, containerRef, handleKeyDown } = useListKeyboardNavigation(files, {
    onSelect: file => onSelect(file.path),
    onLeft: onClose,
    onEscape: onClose,
  });

  if (files.length === 0) {
    return (
      // biome-ignore lint/a11y/noStaticElementInteractions: Keyboard navigation requires handler
      <div
        ref={containerRef as React.RefObject<HTMLDivElement | null>}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="p-4 text-center text-app-text-muted text-xs focus:outline-none"
      >
        No recent files
      </div>
    );
  }

  return (
    <ul
      ref={containerRef as React.RefObject<HTMLUListElement | null>}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: Keyboard navigation requires focus
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex-1 flex flex-col gap-0.5 p-1 focus:outline-none overflow-y-auto min-h-0"
    >
      {files.map((file, index) => (
        <li key={file.path}>
          <ListItem
            selected={index === selectedIndex}
            index={index}
            showIndex={false}
            title={file.path}
            onClick={() => onSelect(file.path)}
          >
            {file.name}
          </ListItem>
        </li>
      ))}
    </ul>
  );
}
