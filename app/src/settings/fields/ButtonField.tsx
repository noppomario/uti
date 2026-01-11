/**
 * Button field component for settings
 *
 * Variants:
 * - primary: Strong accent (Apply button)
 * - action: Visible border with text color (action buttons in settings)
 * - secondary: Subtle border (Cancel button)
 */

import { ExternalLink } from 'lucide-react';

interface ButtonFieldProps {
  label: string;
  variant?: 'primary' | 'action' | 'secondary';
  externalLink?: boolean;
  onClick: () => void;
}

function getVariantClasses(variant: 'primary' | 'action' | 'secondary'): string {
  switch (variant) {
    case 'primary':
      return 'bg-app-accent-info text-white hover:opacity-90';
    case 'action':
      return 'border border-app-text-muted bg-app-item text-app-text hover:bg-app-item-hover';
    case 'secondary':
      return 'border border-app-header-border bg-app-item text-app-text hover:bg-app-item-hover';
  }
}

export function ButtonField({
  label,
  variant = 'action',
  externalLink = false,
  onClick,
}: ButtonFieldProps) {
  const baseClasses = 'rounded px-4 py-2 text-sm font-medium transition-colors';

  return (
    <div className="py-2">
      <button
        type="button"
        onClick={onClick}
        className={`${baseClasses} ${getVariantClasses(variant)} inline-flex items-center gap-1.5`}
      >
        {label}
        {externalLink && <ExternalLink size={14} />}
      </button>
    </div>
  );
}
