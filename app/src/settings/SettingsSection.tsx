/**
 * Settings section component
 *
 * Renders a section with a title and its fields.
 */

import { useTranslation } from 'react-i18next';
import { FieldRenderer } from './FieldRenderer';
import type { Section } from './schema';
import { getNestedValue } from './utils';

interface SettingsSectionProps {
  section: Section;
  config: Record<string, unknown>;
  onChange: (path: string, value: unknown) => void;
  onAction: (action: string) => void;
}

export function SettingsSection({ section, config, onChange, onAction }: SettingsSectionProps) {
  const { t } = useTranslation('settings');

  return (
    <section className="mb-6">
      <h2 className="mb-3 border-b border-app-header-border pb-2 font-medium text-app-text text-sm">
        {t(section.titleKey)}
      </h2>
      <div className="space-y-1">
        {section.fields.map(field => (
          <FieldRenderer
            key={field.key}
            field={field}
            value={field.type !== 'button' ? getNestedValue(config, field.configPath) : undefined}
            onChange={onChange}
            onAction={onAction}
          />
        ))}
      </div>
    </section>
  );
}
