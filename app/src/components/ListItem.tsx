/**
 * ListItem component
 *
 * Shared list item button with consistent styling for keyboard-navigable lists.
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
  onClick,
  onMouseEnter,
  onMouseLeave,
  title,
  dataAttributes,
  buttonRef,
}: ListItemProps) {
  const baseClassName = `w-full cursor-pointer rounded px-2 py-1 text-left text-xs truncate flex items-center ${
    selected
      ? 'bg-app-item-selected text-app-text-on-selected'
      : 'bg-app-item text-app-text hover:bg-app-item-hover'
  }`;

  // When suffix has its own click handler, use div container with separate buttons
  if (onSuffixClick && suffix) {
    return (
      // biome-ignore lint/a11y/useSemanticElements: div is visual container, interaction via child buttons
      <div
        role="group"
        data-selected={selected}
        {...dataAttributes}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
        className={baseClassName}
      >
        <button
          ref={buttonRef}
          type="button"
          title={title}
          onClick={onClick}
          className="truncate flex-1 text-left bg-transparent border-none cursor-pointer p-0"
        >
          {showIndex && <>{index + 1}:&nbsp;&nbsp;&nbsp;&nbsp;</>}
          {children}
        </button>
        <button
          type="button"
          tabIndex={-1}
          onClick={onSuffixClick}
          className="ml-2 px-1 text-app-text-muted hover:text-app-text cursor-pointer bg-transparent border-none"
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
    >
      <span className="truncate flex-1">
        {showIndex && <>{index + 1}:&nbsp;&nbsp;&nbsp;&nbsp;</>}
        {children}
      </span>
      {suffix}
    </button>
  );
}
