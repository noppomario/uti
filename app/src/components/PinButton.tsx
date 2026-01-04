/**
 * Props for the PinButton component
 */
interface PinButtonProps {
  /** Whether the window is currently pinned */
  isPinned: boolean;
  /** Callback when pin state is toggled */
  onToggle: () => void;
}

/** Inline styles using CSS variables for theme-based sizing */
const buttonStyles: React.CSSProperties = {
  fontSize: 'var(--size-font-base)',
  padding: 'var(--size-padding-y) var(--size-padding-x)',
};

/**
 * Pin button component for toggling window always-on-top behavior
 *
 * @param props - Component props
 * @returns The PinButton UI
 */
export function PinButton({ isPinned, onToggle }: PinButtonProps) {
  return (
    <button
      type="button"
      onClick={onToggle}
      className={`transition-colors ${
        isPinned ? 'text-app-accent-info' : 'text-app-text-muted hover:text-app-text'
      }`}
      style={buttonStyles}
      aria-label={isPinned ? 'Unpin window' : 'Pin window'}
    >
      {isPinned ? (
        // Filled pin icon (pinned state)
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="currentColor"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <path d="M12 17v5" />
          <path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
        </svg>
      ) : (
        // Outline pin icon (unpinned state)
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <path d="M12 17v5" />
          <path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
        </svg>
      )}
    </button>
  );
}
