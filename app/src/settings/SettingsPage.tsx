/**
 * Settings page component
 *
 * Main component for the settings window.
 * Uses sidebar navigation with section icons.
 */

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { DialogLayout } from '../components/DialogLayout';
import type { AppConfig } from '../config';
import i18n from '../i18n';
import { getSectionIcon, ICON_SIZE } from './icons';
import { SettingsSection } from './SettingsSection';
import { settingsSchema } from './schema';
import { setNestedValue } from './utils';

/**
 * Settings page component with sidebar navigation
 */
export function SettingsPage() {
  const { t } = useTranslation('settings');
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [hasChanges, setHasChanges] = useState(false);
  const [activeSection, setActiveSection] = useState(settingsSchema[0]?.id || '');
  const [version, setVersion] = useState<string>('');
  const [autoStart, setAutoStart] = useState<boolean>(false);

  // Load config and other state on mount
  useEffect(() => {
    async function loadData() {
      // Load config
      const cfg = await invoke<AppConfig>('read_config');
      setConfig(cfg);

      // Set i18n language from config
      if (cfg.language) {
        i18n.changeLanguage(cfg.language);
      }

      // Load version
      const ver = await invoke<string>('get_version');
      setVersion(ver);

      // Load autostart status
      const autoStartEnabled = await invoke<boolean>('get_autostart_status');
      setAutoStart(autoStartEnabled);
    }
    loadData();
  }, []);

  // Handle Escape key to close window
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().close();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Handle config change
  const handleChange = (path: string, value: unknown) => {
    // Handle autoStart separately (not in config)
    if (path === 'autoStart') {
      setAutoStart(value as boolean);
      setHasChanges(true);
      return;
    }

    if (!config) return;

    const configRecord = config as unknown as Record<string, unknown>;
    const newConfig = setNestedValue(configRecord, path, value) as unknown as AppConfig;
    setConfig(newConfig);
    setHasChanges(true);
  };

  // Handle action buttons
  const handleAction = async (action: string) => {
    switch (action) {
      case 'openConfigFolder':
        await invoke('open_config_folder');
        break;
      case 'openLauncherConfig':
        await invoke('open_launcher_config');
        break;
      case 'openSnippetsConfig':
        await invoke('open_snippets_config');
        break;
      case 'reloadConfig': {
        const newConfig = await invoke<AppConfig>('reload_config');
        setConfig(newConfig);
        setHasChanges(false);
        if (newConfig.language) {
          i18n.changeLanguage(newConfig.language);
        }
        await emit('config_changed', newConfig);
        break;
      }
      case 'checkForUpdates':
        await invoke('open_update_dialog');
        break;
      case 'openGitHub':
        await invoke('open_url', { url: 'https://github.com/noppomario/uti' });
        break;
    }
  };

  // Get value for a field path (handles special cases like version, autoStart)
  const getFieldValue = (field: { type: string; valueKey?: string; configPath?: string }) => {
    if (field.type === 'text' && field.valueKey === 'version') {
      return version;
    }
    if (field.type === 'checkbox' && field.configPath === 'autoStart') {
      return autoStart;
    }
    if (!config || !field.configPath) return undefined;

    // Navigate nested config path
    const parts = field.configPath.split('.');
    let value: unknown = config;
    for (const part of parts) {
      if (value && typeof value === 'object') {
        value = (value as Record<string, unknown>)[part];
      } else {
        return undefined;
      }
    }
    return value;
  };

  // Handle apply button
  const handleApply = async () => {
    if (!config) return;

    // Save config
    await invoke('save_config', { config });

    // Save autostart setting
    await invoke('set_autostart', { enabled: autoStart });

    setHasChanges(false);

    if (config.language) {
      i18n.changeLanguage(config.language);
    }

    await emit('config_changed', config);
  };

  // Handle cancel button
  const handleCancel = () => {
    getCurrentWindow().close();
  };

  if (!config) {
    return (
      <div className="flex h-screen items-center justify-center bg-app-bg">
        <span className="text-app-text-muted">Loading...</span>
      </div>
    );
  }

  const activeSchemaSection = settingsSchema.find(s => s.id === activeSection);

  const footer = (
    <>
      <button
        type="button"
        onClick={handleCancel}
        className="rounded border border-app-header-border bg-app-item px-4 py-2 font-medium text-app-text text-sm hover:bg-app-item-hover"
      >
        {t('cancel')}
      </button>
      <button
        type="button"
        onClick={handleApply}
        disabled={!hasChanges}
        className="rounded bg-app-accent-info px-4 py-2 font-medium text-white text-sm hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
      >
        {t('apply')}
      </button>
    </>
  );

  return (
    <DialogLayout title={t('title')} footer={footer}>
      <div className="flex h-full">
        {/* Sidebar - slightly darker than content for visual separation */}
        <nav className="w-48 shrink-0 border-r border-app-header-border bg-app-item p-2">
          <ul className="space-y-1">
            {settingsSchema.map(section => {
              const Icon = getSectionIcon(section.id);
              const isActive = section.id === activeSection;

              return (
                <li key={section.id}>
                  <button
                    type="button"
                    onClick={() => setActiveSection(section.id)}
                    className={`flex w-full items-center gap-3 rounded px-3 py-2 text-left text-sm transition-colors ${
                      isActive
                        ? 'bg-app-item-selected text-app-text'
                        : 'text-app-text-muted hover:bg-app-item-hover hover:text-app-text'
                    }`}
                  >
                    {Icon && <Icon size={ICON_SIZE} className="shrink-0" />}
                    <span>{t(section.titleKey)}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>

        {/* Main content */}
        <main className="flex-1 overflow-y-auto p-6">
          {activeSchemaSection && (
            <SettingsSection
              section={activeSchemaSection}
              config={config as unknown as Record<string, unknown>}
              onChange={handleChange}
              onAction={handleAction}
              getFieldValue={getFieldValue}
            />
          )}
        </main>
      </div>
    </DialogLayout>
  );
}
