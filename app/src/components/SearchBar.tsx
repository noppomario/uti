import { Search, X } from 'lucide-react';
import type React from 'react';

export interface SearchBarProps {
  /** Current search query value */
  value: string;
  /** Called when input value changes */
  onChange: (value: string) => void;
  /** Called when ArrowDown is pressed (to move focus to list) */
  onArrowDown: () => void;
  /** Called when Escape is pressed (to clear search and exit) */
  onEscape: () => void;
  /** Called when ArrowLeft is pressed (to switch to previous tab) */
  onArrowLeft?: () => void;
  /** Called when ArrowRight is pressed (to switch to next tab) */
  onArrowRight?: () => void;
  /** Called when Enter is pressed (to select first item) */
  onEnter?: () => void;
  /** Ref to focus the input programmatically */
  inputRef?: React.RefObject<HTMLInputElement | null>;
  /** Placeholder text */
  placeholder?: string;
}

/**
 * Keyboard shortcut badge component
 */
function KbdBadge({ children }: { children: React.ReactNode }) {
  return (
    <kbd
      className="inline-flex items-center justify-center rounded border border-app-border bg-app-bg text-app-text-muted"
      style={{
        padding: '0.125em 0.375em',
        fontSize: 'calc(var(--size-font-base) * 0.75)',
        fontFamily: 'inherit',
        lineHeight: 1,
      }}
    >
      {children}
    </kbd>
  );
}

/**
 * Search bar component for filtering items
 *
 * Handles keyboard navigation:
 * - ArrowDown: Move focus to list
 * - ArrowLeft/ArrowRight: Switch tabs
 * - Escape: Clear search and exit search mode
 */
export function SearchBar({
  value,
  onChange,
  onArrowDown,
  onEscape,
  onArrowLeft,
  onArrowRight,
  onEnter,
  inputRef,
  placeholder = 'Search...',
}: SearchBarProps) {
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        onArrowDown();
        break;
      case 'ArrowLeft':
        if (onArrowLeft) {
          e.preventDefault();
          onArrowLeft();
        }
        break;
      case 'ArrowRight':
        if (onArrowRight) {
          e.preventDefault();
          onArrowRight();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onEscape();
        break;
      case 'Enter':
        if (onEnter) {
          e.preventDefault();
          onEnter();
        }
        break;
    }
  };

  const handleClear = () => {
    onChange('');
    inputRef?.current?.focus();
  };

  // Icon size based on font size
  const iconSize = 14;

  return (
    <div className="relative flex items-center">
      {/* Search icon on left */}
      <Search
        size={iconSize}
        className="absolute left-0 text-app-text-muted pointer-events-none"
        style={{ marginLeft: 'var(--size-padding-x)' }}
      />

      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={e => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className="w-full bg-app-item text-app-text placeholder-app-text-muted rounded-md outline-none focus:ring-1 focus:ring-app-accent-info"
        style={{
          padding: 'var(--size-padding-y) var(--size-padding-x)',
          paddingLeft: 'calc(var(--size-padding-x) * 2 + 14px)',
          paddingRight: 'calc(var(--size-padding-x) + 4.5em)',
          fontSize: 'var(--size-font-base)',
        }}
      />

      {/* Right side: Ctrl+F hint when empty, clear button when has value */}
      <div
        className="absolute right-0 top-1/2 -translate-y-1/2 flex items-center gap-1"
        style={{ marginRight: 'var(--size-padding-x)' }}
      >
        {value ? (
          <button
            type="button"
            onClick={handleClear}
            className="flex items-center gap-1 text-app-text-muted hover:text-app-text focus:outline-none"
            aria-label="Clear search (Esc)"
          >
            <X size={iconSize} />
            <KbdBadge>Esc</KbdBadge>
          </button>
        ) : (
          <div className="flex items-center gap-0.5 pointer-events-none">
            <KbdBadge>Ctrl</KbdBadge>
            <KbdBadge>F</KbdBadge>
          </div>
        )}
      </div>
    </div>
  );
}
