import type React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';

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
 * Props for the Prompt component
 */
interface PromptProps {
  /** Callback when text is submitted via Ctrl+Enter */
  onSubmit: (text: string) => void;
  /** Callback to switch to next tab (right) */
  onSwitchToNextTab?: () => void;
}

/**
 * Prompt component for text input with auto-paste functionality
 *
 * Features:
 * - Multi-line text input area
 * - Submit via Ctrl+Enter
 * - Auto-focus when rendered
 *
 * @param props - Component props
 * @returns The Prompt UI
 */
export function Prompt({ onSubmit, onSwitchToNextTab }: PromptProps) {
  const [text, setText] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-focus on mount
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Ctrl+Enter to submit
      if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        const trimmedText = text.trim();
        if (trimmedText) {
          onSubmit(trimmedText);
          setText('');
        }
        return;
      }

      // Arrow Right at end of text (or empty) switches to next tab
      if (e.key === 'ArrowRight' && onSwitchToNextTab) {
        const textarea = textareaRef.current;
        if (textarea) {
          const isAtEnd = textarea.selectionStart === text.length;
          if (isAtEnd) {
            e.preventDefault();
            onSwitchToNextTab();
          }
        }
      }
    },
    [text, onSubmit, onSwitchToNextTab]
  );

  return (
    <div className="flex flex-col h-full p-2">
      <textarea
        ref={textareaRef}
        value={text}
        onChange={e => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Type text and press Ctrl+Enter to paste..."
        className="flex-1 w-full resize-none bg-app-surface text-app-text placeholder-app-text-muted border border-app-border rounded focus:outline-none focus:ring-1 focus:ring-app-accent-info p-2"
        style={{
          fontSize: 'var(--size-font-base)',
        }}
      />
      <div className="mt-1 flex items-center justify-end gap-1 text-app-text-muted">
        <KbdBadge>Ctrl</KbdBadge>
        <KbdBadge>Enter</KbdBadge>
        <span style={{ fontSize: 'calc(var(--size-font-base) * 0.75)' }}>→</span>
        <KbdBadge>Ctrl+Shift+V</KbdBadge>
        <KbdBadge>↵</KbdBadge>
      </div>
    </div>
  );
}
