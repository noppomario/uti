/**
 * Number field component for settings
 */

import { useId } from 'react';

interface NumberFieldProps {
  label: string;
  description?: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  onChange: (value: number) => void;
}

export function NumberField({
  label,
  description,
  value,
  min,
  max,
  step,
  onChange,
}: NumberFieldProps) {
  const id = useId();

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex-1">
        <label htmlFor={id} className="block font-medium text-app-text text-sm">
          {label}
        </label>
        {description && <p className="text-app-text-muted text-xs mt-0.5">{description}</p>}
      </div>
      <input
        id={id}
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={e => onChange(Number(e.target.value))}
        className="w-24 rounded border border-app-header-border bg-app-item px-3 py-1.5 text-app-text text-sm focus:outline-none focus:ring-1 focus:ring-app-item-selected"
      />
    </div>
  );
}
