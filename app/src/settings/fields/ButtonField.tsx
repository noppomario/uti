/**
 * Button field component for settings
 */

interface ButtonFieldProps {
  label: string;
  variant?: 'primary' | 'secondary';
  onClick: () => void;
}

export function ButtonField({ label, variant = 'secondary', onClick }: ButtonFieldProps) {
  const baseClasses = 'rounded px-4 py-2 text-sm font-medium transition-colors';
  const variantClasses =
    variant === 'primary'
      ? 'bg-app-item-selected text-white hover:opacity-90'
      : 'bg-app-item text-app-text hover:bg-app-item-hover';

  return (
    <div className="py-2">
      <button type="button" onClick={onClick} className={`${baseClasses} ${variantClasses}`}>
        {label}
      </button>
    </div>
  );
}
