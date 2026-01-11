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
      <div className="relative">
        <select
          id={id}
          value={value}
          onChange={e => onChange(e.target.value)}
          className="appearance-none rounded border border-app-header-border bg-app-item py-1.5 pr-8 pl-3 text-app-text text-sm focus:outline-none focus:ring-1 focus:ring-app-item-selected"
        >
          {options.map(option => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <svg
          className="pointer-events-none absolute top-1/2 right-2 h-4 w-4 -translate-y-1/2 text-app-text-muted"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </div>
    </div>
  );
}
