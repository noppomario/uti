/**
 * Select field component for settings
 */

import { useId } from 'react';

interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps {
  label: string;
  description?: string;
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
}

export function SelectField({ label, description, value, options, onChange }: SelectFieldProps) {
  const id = useId();

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex-1">
        <label htmlFor={id} className="block font-medium text-app-text text-sm">
          {label}
        </label>
        {description && <p className="text-app-text-muted text-xs mt-0.5">{description}</p>}
      </div>
      <select
        id={id}
        value={value}
        onChange={e => onChange(e.target.value)}
        className="rounded border border-app-header-border bg-app-item px-3 py-1.5 text-app-text text-sm focus:outline-none focus:ring-1 focus:ring-app-item-selected"
      >
        {options.map(option => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
}
