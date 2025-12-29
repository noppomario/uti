import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

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
      console.log('Setting up double-ctrl-pressed event listener...');
      const unlisten = await listen('double-ctrl-pressed', () => {
        console.log('Double Ctrl event received from Rust backend');
        console.log('Invoking toggle_window command...');
        invoke('toggle_window')
          .then(() => {
            console.log('toggle_window command completed');
          })
          .catch(err => {
            console.error('toggle_window command failed:', err);
          });
      });

      console.log('Event listener registered successfully');
      return unlisten;
    };

    let unlisten: (() => void) | undefined;
    setupListener()
      .then(fn => {
        unlisten = fn;
      })
      .catch(err => {
        console.error('Failed to setup event listener:', err);
      });

    return () => {
      if (unlisten) {
        console.log('Cleaning up event listener');
        unlisten();
      }
    };
  }, []);

  return (
    <div className="h-screen bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-2xl p-8 max-w-md">
        <h1 className="text-3xl font-bold text-gray-800 mb-4">uti</h1>
        <p className="text-gray-600 mb-6">Press Ctrl twice to toggle visibility</p>
        <button
          type="button"
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
