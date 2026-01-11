/**
 * Checkbox field component for settings
 *
 * Used for boolean settings like auto-start.
 */

interface CheckboxFieldProps {
  label: string;
  description?: string;
  value: boolean;
  onChange: (value: boolean) => void;
}

export function CheckboxField({ label, description, value, onChange }: CheckboxFieldProps) {
  return (
    <div className="flex items-start justify-between gap-4 py-2">
      <div className="flex flex-col">
        <span className="text-sm font-medium text-app-text">{label}</span>
        {description && <span className="text-xs text-app-text-muted">{description}</span>}
      </div>
      <label className="relative inline-flex cursor-pointer items-center">
        <input
          type="checkbox"
          checked={value}
          onChange={e => onChange(e.target.checked)}
          className="peer sr-only"
        />
        <div className="h-5 w-9 rounded-full bg-app-header-border transition-colors peer-checked:bg-app-accent-info peer-focus:ring-2 peer-focus:ring-app-accent-info/50 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-transform after:content-[''] peer-checked:after:translate-x-4" />
      </label>
    </div>
  );
}
