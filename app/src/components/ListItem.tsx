/**
 * ListItem component
 *
 * Shared list item button with consistent styling for keyboard-navigable lists.
 * Uses CSS variables for sizing to support theme-based scaling.
 */

import type React from 'react';

export interface ListItemProps {
  /** Whether the item is currently selected */
  selected: boolean;
  /** Index for display (0-based, will be shown as 1-based) */
  index: number;
  /** Main content to display */
  children: React.ReactNode;
  /** Whether to show the index prefix (default: true) */
  showIndex?: boolean;
  /** Optional suffix element (e.g., expand indicator) */
  suffix?: React.ReactNode;
  /** Called when the suffix is clicked (if provided, suffix becomes clickable) */
  onSuffixClick?: () => void;
  /** Aria label for the suffix button (default: "Expand") */
  suffixAriaLabel?: string;
  /** Additional class name for the suffix button */
  suffixClassName?: string;
  /** Called when the item is clicked */
  onClick: () => void;
  /** Called when mouse enters the item */
  onMouseEnter?: () => void;
  /** Called when mouse leaves the item */
  onMouseLeave?: () => void;
  /** Optional title attribute for tooltip */
  title?: string;
  /** Optional data attributes */
  dataAttributes?: Record<string, string | boolean>;
  /** Optional ref callback for the button element */
  buttonRef?: (el: HTMLButtonElement | null) => void;
}

/** Inline styles using CSS variables for theme-based sizing */
const sizeStyles: React.CSSProperties = {
  fontSize: 'var(--size-font-base)',
  padding: 'var(--size-padding-y) var(--size-padding-x)',
  borderRadius: 'var(--size-radius)',
};

/**
 * List item button with consistent styling
 *
 * @example
 * ```tsx
 * <ListItem
 *   selected={index === selectedIndex}
 *   index={index}
 *   onClick={() => onSelect(item)}
 * >
 *   {item.name}
 * </ListItem>
 * ```
 */
export function ListItem({
  selected,
  index,
  children,
  showIndex = true,
  suffix,
  onSuffixClick,
  suffixAriaLabel = 'Expand',
  suffixClassName = '',
  onClick,
  onMouseEnter,
  onMouseLeave,
  title,
  dataAttributes,
  buttonRef,
}: ListItemProps) {
  const baseClassName = `w-full cursor-pointer text-left truncate flex items-center ${
    selected
      ? 'bg-app-item-selected text-app-text-on-selected'
      : 'bg-app-item text-app-text hover:bg-app-item-hover'
  }`;

  // When suffix has its own click handler, use div container with 50/50 split buttons
  if (onSuffixClick && suffix) {
    return (
      // biome-ignore lint/a11y/useSemanticElements: div is visual container, interaction via child buttons
      <div
        role="group"
        data-selected={selected}
        {...dataAttributes}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
        className={`${baseClassName} border border-solid border-current/30`}
        style={sizeStyles}
      >
        <button
          ref={buttonRef}
          type="button"
          title={title}
          onClick={onClick}
          className="w-1/2 truncate text-left bg-transparent border-none cursor-pointer p-0"
          style={{ fontSize: 'inherit' }}
        >
          {showIndex && <>{index + 1}:&nbsp;&nbsp;&nbsp;&nbsp;</>}
          {children}
        </button>
        <button
          type="button"
          tabIndex={-1}
          aria-label={suffixAriaLabel}
          onClick={onSuffixClick}
          className={`w-1/2 flex justify-end items-center cursor-pointer bg-transparent border-none hover:opacity-70 transition-opacity pr-2 ${suffixClassName}`}
          style={{ fontSize: 'inherit' }}
        >
          {suffix}
        </button>
      </div>
    );
  }

  // Standard single-button layout
  return (
    <button
      ref={buttonRef}
      type="button"
      title={title}
      data-selected={selected}
      {...dataAttributes}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      className={baseClassName}
      style={sizeStyles}
    >
      <span className="truncate flex-1">
        {showIndex && <>{index + 1}:&nbsp;&nbsp;&nbsp;&nbsp;</>}
        {children}
      </span>
      {suffix && <span className="mr-2 shrink-0 flex items-center">{suffix}</span>}
    </button>
  );
}
