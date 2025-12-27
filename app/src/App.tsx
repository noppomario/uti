import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

/**
 * Main application component
 *
 * Listens for double-ctrl-pressed events from the Rust backend and toggles
 * window visibility when the event is received.
 *
 * @returns The main application UI
 */
function App() {
  useEffect(() => {
    /**
     * Sets up the event listener for double Ctrl press events
     *
     * @returns Cleanup function to remove the event listener
     */
    const setupListener = async () => {
      const unlisten = await listen('double-ctrl-pressed', () => {
        console.log('Double Ctrl event received from Rust backend');
        invoke('toggle_window');
      });

      return unlisten;
    };

    let unlisten: (() => void) | undefined;
    setupListener().then(fn => { unlisten = fn; });

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="h-screen bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-2xl p-8 max-w-md">
        <h1 className="text-3xl font-bold text-gray-800 mb-4">
          uti
        </h1>
        <p className="text-gray-600 mb-6">
          Press Ctrl twice to toggle visibility
        </p>
        <button
          onClick={() => invoke('toggle_window')}
          className="w-full bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded transition duration-200"
        >
          Toggle Window
        </button>
      </div>
    </div>
  );
}

export default App;
