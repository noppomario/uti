/**
 * Color picker field component for settings
 */

import { useId } from 'react';

interface ColorFieldProps {
  label: string;
  description?: string;
  value: string | undefined;
  allowClear?: boolean;
  onChange: (value: string | undefined) => void;
}

export function ColorField({ label, description, value, allowClear, onChange }: ColorFieldProps) {
  const id = useId();

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex-1">
        <label htmlFor={id} className="block font-medium text-app-text text-sm">
          {label}
        </label>
        {description && <p className="text-app-text-muted text-xs mt-0.5">{description}</p>}
      </div>
      <div className="flex items-center gap-2">
        <input
          id={id}
          type="color"
          value={value || '#3584e4'}
          onChange={e => onChange(e.target.value)}
          className="h-8 w-12 cursor-pointer rounded border border-app-header-border bg-transparent"
        />
        {allowClear && value && (
          <button
            type="button"
            onClick={() => onChange(undefined)}
            className="rounded px-2 py-1 text-app-text-muted text-xs hover:bg-app-item hover:text-app-text"
          >
            Clear
          </button>
        )}
      </div>
    </div>
  );
}
