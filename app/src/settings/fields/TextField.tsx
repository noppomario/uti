/**
 * Text field component for settings
 *
 * Used for displaying static text like version info.
 * This is a read-only display field, not an input.
 */

interface TextFieldProps {
  label: string;
  description?: string;
  value: string;
}

export function TextField({ label, description, value }: TextFieldProps) {
  return (
    <div className="flex items-start justify-between gap-4 py-2">
      <div className="flex flex-col">
        <span className="text-sm font-medium text-app-text">{label}</span>
        {description && <span className="text-xs text-app-text-muted">{description}</span>}
      </div>
      <span className="text-sm text-app-text-muted">{value}</span>
    </div>
  );
}
