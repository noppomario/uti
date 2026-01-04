import { X } from 'lucide-react';
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

  return (
    <div className="relative">
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
          paddingRight: value ? 'calc(var(--size-padding-x) + 1.5em)' : 'var(--size-padding-x)',
          fontSize: 'var(--size-font-base)',
        }}
      />
      {value && (
        <button
          type="button"
          onClick={handleClear}
          className="absolute right-0 top-1/2 -translate-y-1/2 text-app-text-muted hover:text-app-text focus:outline-none"
          style={{
            padding: 'var(--size-padding-y) var(--size-padding-x)',
          }}
          aria-label="Clear search"
        >
          <X size={14} />
        </button>
      )}
    </div>
  );
}
