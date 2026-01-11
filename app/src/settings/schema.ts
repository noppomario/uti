/**
 * Settings schema definition
 *
 * Declarative schema for settings UI auto-generation.
 * Add a new entry here to automatically generate the UI.
 */

/** Supported field types for settings UI */
export type FieldType = 'select' | 'number' | 'color' | 'button' | 'checkbox' | 'text';

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

/** Checkbox field (toggle) */
interface CheckboxField extends BaseField {
  type: 'checkbox';
  /** Config path: 'autoStart' */
  configPath: string;
}

/** Text field (read-only display) */
interface TextField extends BaseField {
  type: 'text';
  /** Value key to fetch from app state (not config) */
  valueKey: 'version';
}

/** Action button field (no config binding) */
interface ButtonField extends BaseField {
  type: 'button';
  /** Action to perform */
  action:
    | 'openConfigFolder'
    | 'openLauncherConfig'
    | 'openSnippetsConfig'
    | 'reloadConfig'
    | 'checkForUpdates'
    | 'openGitHub';
  /** Button variant: action (default), primary, or secondary */
  variant?: 'primary' | 'action' | 'secondary';
  /** Show external link icon */
  externalLink?: boolean;
}

export type Field =
  | SelectField
  | NumberField
  | ColorField
  | ButtonField
  | CheckboxField
  | TextField;

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
    id: 'general',
    titleKey: 'sections.general',
    fields: [
      {
        key: 'autoStart',
        labelKey: 'general.autoStart.label',
        descriptionKey: 'general.autoStart.description',
        type: 'checkbox',
        configPath: 'autoStart',
      },
      {
        key: 'language',
        labelKey: 'general.language.label',
        type: 'select',
        configPath: 'language',
        options: [
          { value: 'en', labelKey: 'general.language.options.en' },
          { value: 'ja', labelKey: 'general.language.options.ja' },
        ],
      },
    ],
  },
  {
    id: 'appearance',
    titleKey: 'sections.appearance',
    fields: [
      {
        key: 'theme.color',
        labelKey: 'appearance.theme.color.label',
        descriptionKey: 'appearance.theme.color.description',
        type: 'select',
        configPath: 'theme.color',
        options: [
          { value: 'midnight', labelKey: 'appearance.theme.color.options.midnight' },
          { value: 'dark', labelKey: 'appearance.theme.color.options.dark' },
          { value: 'light', labelKey: 'appearance.theme.color.options.light' },
        ],
      },
      {
        key: 'theme.size',
        labelKey: 'appearance.theme.size.label',
        type: 'select',
        configPath: 'theme.size',
        options: [
          { value: 'minimal', labelKey: 'appearance.theme.size.options.minimal' },
          { value: 'normal', labelKey: 'appearance.theme.size.options.normal' },
          { value: 'wide', labelKey: 'appearance.theme.size.options.wide' },
        ],
      },
      // {
      //   key: 'theme.accentColor',
      //   labelKey: 'appearance.theme.accentColor.label',
      //   descriptionKey: 'appearance.theme.accentColor.description',
      //   type: 'color',
      //   configPath: 'theme.accentColor',
      //   allowClear: true,
      // },
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
    id: 'snippets',
    titleKey: 'sections.snippets',
    fields: [
      {
        key: 'openSnippetsConfig',
        labelKey: 'snippets.openConfig.label',
        descriptionKey: 'snippets.openConfig.description',
        type: 'button',
        action: 'openSnippetsConfig',
      },
    ],
  },
  {
    id: 'launcher',
    titleKey: 'sections.launcher',
    fields: [
      {
        key: 'openLauncherConfig',
        labelKey: 'launcher.openConfig.label',
        descriptionKey: 'launcher.openConfig.description',
        type: 'button',
        action: 'openLauncherConfig',
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
      },
      {
        key: 'reloadConfig',
        labelKey: 'advanced.reloadConfig',
        type: 'button',
        action: 'reloadConfig',
      },
    ],
  },
  {
    id: 'about',
    titleKey: 'sections.about',
    fields: [
      {
        key: 'version',
        labelKey: 'about.version.label',
        type: 'text',
        valueKey: 'version',
      },
      {
        key: 'checkForUpdates',
        labelKey: 'about.checkForUpdates.label',
        type: 'button',
        action: 'checkForUpdates',
      },
      {
        key: 'openGitHub',
        labelKey: 'about.openGitHub.label',
        descriptionKey: 'about.openGitHub.description',
        type: 'button',
        action: 'openGitHub',
        externalLink: true,
      },
    ],
  },
];
