/**
 * Settings schema definition
 *
 * Declarative schema for settings UI auto-generation.
 * Add a new entry here to automatically generate the UI.
 */

/** Supported field types for settings UI */
export type FieldType = 'select' | 'number' | 'color' | 'button';

/** Base field definition */
interface BaseField {
  /** Unique identifier for the field */
  key: string;
  /** i18n key for label (e.g., 'settings.theme.color') */
  labelKey: string;
  /** i18n key for description (optional) */
  descriptionKey?: string;
  /** Field type */
  type: FieldType;
}

/** Select field (dropdown) */
interface SelectField extends BaseField {
  type: 'select';
  /** Available options with i18n keys for labels */
  options: Array<{
    value: string;
    labelKey: string;
  }>;
  /** Config path (dot notation): 'theme.color' */
  configPath: string;
}

/** Number field */
interface NumberField extends BaseField {
  type: 'number';
  /** Minimum value */
  min?: number;
  /** Maximum value */
  max?: number;
  /** Step increment */
  step?: number;
  /** Config path: 'clipboardHistoryLimit' */
  configPath: string;
}

/** Color picker field */
interface ColorField extends BaseField {
  type: 'color';
  /** Config path: 'theme.accentColor' */
  configPath: string;
  /** Allow clearing the color (reset to default) */
  allowClear?: boolean;
}

/** Action button field (no config binding) */
interface ButtonField extends BaseField {
  type: 'button';
  /** Action to perform */
  action: 'openConfigFolder' | 'reloadConfig';
  /** Button variant */
  variant?: 'primary' | 'secondary';
}

export type Field = SelectField | NumberField | ColorField | ButtonField;

/** Settings section definition */
export interface Section {
  /** Section ID */
  id: string;
  /** i18n key for section title */
  titleKey: string;
  /** Fields in this section */
  fields: Field[];
}

/** Complete settings schema */
export type SettingsSchema = Section[];

/** The settings schema definition */
export const settingsSchema: SettingsSchema = [
  {
    id: 'appearance',
    titleKey: 'sections.appearance',
    fields: [
      {
        key: 'theme.color',
        labelKey: 'theme.color.label',
        descriptionKey: 'theme.color.description',
        type: 'select',
        configPath: 'theme.color',
        options: [
          { value: 'midnight', labelKey: 'theme.color.options.midnight' },
          { value: 'dark', labelKey: 'theme.color.options.dark' },
          { value: 'light', labelKey: 'theme.color.options.light' },
        ],
      },
      {
        key: 'theme.size',
        labelKey: 'theme.size.label',
        type: 'select',
        configPath: 'theme.size',
        options: [
          { value: 'minimal', labelKey: 'theme.size.options.minimal' },
          { value: 'normal', labelKey: 'theme.size.options.normal' },
          { value: 'wide', labelKey: 'theme.size.options.wide' },
        ],
      },
      {
        key: 'theme.accentColor',
        labelKey: 'theme.accentColor.label',
        descriptionKey: 'theme.accentColor.description',
        type: 'color',
        configPath: 'theme.accentColor',
        allowClear: true,
      },
    ],
  },
  {
    id: 'clipboard',
    titleKey: 'sections.clipboard',
    fields: [
      {
        key: 'clipboardHistoryLimit',
        labelKey: 'clipboard.historyLimit.label',
        descriptionKey: 'clipboard.historyLimit.description',
        type: 'number',
        configPath: 'clipboardHistoryLimit',
        min: 10,
        max: 500,
        step: 10,
      },
    ],
  },
  {
    id: 'language',
    titleKey: 'sections.language',
    fields: [
      {
        key: 'language',
        labelKey: 'language.label',
        type: 'select',
        configPath: 'language',
        options: [
          { value: 'en', labelKey: 'language.options.en' },
          { value: 'ja', labelKey: 'language.options.ja' },
        ],
      },
    ],
  },
  {
    id: 'advanced',
    titleKey: 'sections.advanced',
    fields: [
      {
        key: 'openConfigFolder',
        labelKey: 'advanced.openConfigFolder',
        type: 'button',
        action: 'openConfigFolder',
        variant: 'secondary',
      },
      {
        key: 'reloadConfig',
        labelKey: 'advanced.reloadConfig',
        type: 'button',
        action: 'reloadConfig',
        variant: 'secondary',
      },
    ],
  },
];
