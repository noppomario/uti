import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

/**
 * Application entry point
 *
 * Renders the main App component into the root DOM element.
 */
const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(<App />);
