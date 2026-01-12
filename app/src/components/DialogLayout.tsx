/**
 * Dialog layout component
 *
 * Shared layout for dialog windows (settings, update dialog, etc.)
 * Provides consistent title bar, content area, and optional footer.
 */

import { getCurrentWindow } from '@tauri-apps/api/window';
import type { ReactNode } from 'react';

interface DialogLayoutProps {
  /** Dialog title displayed in title bar */
  title: string;
  /** Main content */
  children: ReactNode;
  /** Optional footer content (action buttons) */
  footer?: ReactNode;
  /** Custom close handler (defaults to closing window) */
  onClose?: () => void;
}

/**
 * Close icon component
 */
function CloseIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path
        d="M2 2L12 12M12 2L2 12"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}

/**
 * Dialog layout with title bar and optional footer
 */
export function DialogLayout({ title, children, footer, onClose }: DialogLayoutProps) {
  const handleClose = () => {
    if (onClose) {
      onClose();
    } else {
      getCurrentWindow().close();
    }
  };

  return (
    <div className="flex h-screen flex-col overflow-hidden rounded-lg border border-app-header-border bg-app-bg">
      {/* Title bar - draggable */}
      <div
        data-tauri-drag-region
        className="flex h-10 shrink-0 items-center justify-between border-b border-app-header-border bg-app-header px-4"
      >
        <span className="font-medium text-app-text text-sm">{title}</span>
        <button
          type="button"
          onClick={handleClose}
          className="flex h-7 w-7 items-center justify-center rounded text-app-text-muted hover:bg-app-item-hover hover:text-app-text"
          aria-label="Close"
        >
          <CloseIcon />
        </button>
      </div>

      {/* Content area */}
      <div className="flex flex-1 flex-col overflow-hidden">{children}</div>

      {/* Optional footer */}
      {footer && (
        <div className="flex shrink-0 justify-end gap-2 border-t border-app-header-border bg-app-item px-4 py-2">
          {footer}
        </div>
      )}
    </div>
  );
}
