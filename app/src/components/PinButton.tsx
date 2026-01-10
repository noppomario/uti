import { Pin } from 'lucide-react';

/**
 * Props for the PinButton component
 */
interface PinButtonProps {
  /** Whether the window is currently pinned */
  isPinned: boolean;
  /** Callback when pin state is toggled */
  onToggle: () => void;
}

/** Icon size using em units to scale with text size */
const ICON_SIZE = '1.25em';

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
      <Pin size={ICON_SIZE} fill={isPinned ? 'currentColor' : 'none'} aria-hidden="true" />
    </button>
  );
}
