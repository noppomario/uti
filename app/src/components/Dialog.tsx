/**
 * Modal dialog component for displaying messages.
 * Supports info and error styles, with keyboard and click dismissal.
 */

import { useEffect } from 'react';

export type DialogKind = 'info' | 'error';

export interface DialogProps {
  /** Whether the dialog is visible */
  open: boolean;
  /** Dialog title */
  title: string;
  /** Dialog message content */
  message: string;
  /** Dialog type for styling (info: blue accent, error: red accent) */
  kind: DialogKind;
  /** Called when dialog is dismissed */
  onClose: () => void;
}

/**
 * Modal dialog component that follows the app's theme system.
 *
 * @param props - Dialog properties
 * @returns Dialog element or null when closed
 *
 * @example
 * ```tsx
 * <Dialog
 *   open={isOpen}
 *   title="Update Available"
 *   message="Version 1.2.3 is available"
 *   kind="info"
 *   onClose={() => setIsOpen(false)}
 * />
 * ```
 */
export function Dialog({ open, title, message, kind, onClose }: DialogProps) {
  // Handle Escape key to close dialog
  useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <>
      {/* Overlay - clicking dismisses dialog */}
      <button
        type="button"
        data-testid="dialog-overlay"
        aria-label="Close dialog"
        className="fixed inset-0 z-50 cursor-default border-none bg-app-dialog-overlay"
        onClick={onClose}
      />
      {/* Dialog box */}
      <div
        data-testid="dialog-content"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        className="fixed top-1/2 left-1/2 z-50 w-full max-w-xs -translate-x-1/2 -translate-y-1/2 rounded-lg border border-app-header-border bg-app-bg p-4 shadow-lg"
      >
        {/* Title with accent color based on kind */}
        <h2
          id="dialog-title"
          className={`mb-2 text-sm font-semibold ${
            kind === 'error' ? 'text-app-accent-error' : 'text-app-accent-info'
          }`}
        >
          {title}
        </h2>
        {/* Message */}
        <p className="mb-4 whitespace-pre-wrap text-xs text-app-text">{message}</p>
        {/* Close button */}
        <button
          type="button"
          onClick={onClose}
          className="w-full rounded bg-app-item py-1.5 text-xs text-app-text hover:bg-app-item-hover"
        >
          OK
        </button>
      </div>
    </>
  );
}
