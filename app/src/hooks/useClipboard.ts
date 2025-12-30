/**
 * Custom hook for monitoring clipboard changes
 *
 * Polls the clipboard every 1 second and adds new items to history.
 * Ignores duplicate or empty content.
 */

import { invoke } from '@tauri-apps/api/core';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { useEffect, useRef } from 'react';

const POLL_INTERVAL = 1000; // 1 second

/**
 * Monitors clipboard changes and adds new items to history
 *
 * @example
 * ```tsx
 * function App() {
 *   useClipboard(); // Start monitoring clipboard
 *   return <div>App content</div>;
 * }
 * ```
 */
export function useClipboard() {
  const lastClipboardRef = useRef<string | null>(null);

  useEffect(() => {
    const pollClipboard = async () => {
      try {
        const current = await readText();

        // Ignore null or empty content
        if (!current || current.trim() === '') {
          return;
        }

        // Ignore duplicates
        if (current === lastClipboardRef.current) {
          return;
        }

        // Add to history
        await invoke('add_clipboard_item', { text: current });
        lastClipboardRef.current = current;
      } catch (error) {
        console.error('Failed to read clipboard:', error);
      }
    };

    const interval = setInterval(pollClipboard, POLL_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, []);
}
