/**
 * Settings section component
 *
 * Renders a section with a title and its fields.
 */

import { useTranslation } from 'react-i18next';
import { FieldRenderer } from './FieldRenderer';
import type { Field, Section } from './schema';

interface SettingsSectionProps {
  section: Section;
  config: Record<string, unknown>;
  onChange: (path: string, value: unknown) => void;
  onAction: (action: string) => void;
  getFieldValue: (field: Field) => unknown;
}

export function SettingsSection({
  section,
  config: _config,
  onChange,
  onAction,
  getFieldValue,
}: SettingsSectionProps) {
  const { t } = useTranslation('settings');

  return (
    <section>
      <h2 className="mb-4 font-semibold text-app-text text-sm">{t(section.titleKey)}</h2>
      <div className="space-y-3">
        {section.fields.map(field => (
          <FieldRenderer
            key={field.key}
            field={field}
            value={field.type !== 'button' ? getFieldValue(field) : undefined}
            onChange={onChange}
            onAction={onAction}
          />
        ))}
      </div>
    </section>
  );
}
