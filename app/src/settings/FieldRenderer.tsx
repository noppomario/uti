/**
 * Field renderer component
 *
 * Renders the appropriate field component based on the field type.
 */

import { useTranslation } from 'react-i18next';
import { ButtonField } from './fields/ButtonField';
import { ColorField } from './fields/ColorField';
import { NumberField } from './fields/NumberField';
import { SelectField } from './fields/SelectField';
import type { Field } from './schema';

interface FieldRendererProps {
  field: Field;
  value: unknown;
  onChange: (path: string, value: unknown) => void;
  onAction: (action: string) => void;
}

export function FieldRenderer({ field, value, onChange, onAction }: FieldRendererProps) {
  const { t } = useTranslation('settings');

  const label = t(field.labelKey);
  const description = field.descriptionKey ? t(field.descriptionKey) : undefined;

  switch (field.type) {
    case 'select':
      return (
        <SelectField
          label={label}
          description={description}
          value={(value as string) ?? ''}
          options={field.options.map(opt => ({
            value: opt.value,
            label: t(opt.labelKey),
          }))}
          onChange={v => onChange(field.configPath, v)}
        />
      );

    case 'number':
      return (
        <NumberField
          label={label}
          description={description}
          value={(value as number) ?? 50}
          min={field.min}
          max={field.max}
          step={field.step}
          onChange={v => onChange(field.configPath, v)}
        />
      );

    case 'color':
      return (
        <ColorField
          label={label}
          description={description}
          value={value as string | undefined}
          allowClear={field.allowClear}
          onChange={v => onChange(field.configPath, v)}
        />
      );

    case 'button':
      return (
        <ButtonField label={label} variant={field.variant} onClick={() => onAction(field.action)} />
      );

    default:
      return null;
  }
}
