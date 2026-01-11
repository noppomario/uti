/**
 * Settings page component
 *
 * Main component for the settings window.
 * Renders the title bar, all sections from the schema, and action buttons.
 */

import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { AppConfig } from '../config';
import i18n from './i18n';
import { SettingsSection } from './SettingsSection';
import { settingsSchema } from './schema';
import { setNestedValue } from './utils';

/**
 * Settings page component
 */
export function SettingsPage() {
  const { t } = useTranslation('settings');
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Load config on mount
  useEffect(() => {
    async function loadConfig() {
      const cfg = await invoke<AppConfig>('read_config');
      setConfig(cfg);

      // Set i18n language from config
      if (cfg.language) {
        i18n.changeLanguage(cfg.language);
      }
    }
    loadConfig();
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
    if (!config) return;

    const configRecord = config as unknown as Record<string, unknown>;
    const newConfig = setNestedValue(configRecord, path, value) as unknown as AppConfig;
    setConfig(newConfig);
    setHasChanges(true);

    // Special handling for language change
    if (path === 'language' && typeof value === 'string') {
      i18n.changeLanguage(value);
    }
  };

  // Handle action buttons
  const handleAction = async (action: string) => {
    switch (action) {
      case 'openConfigFolder':
        await invoke('open_config_folder');
        break;
      case 'reloadConfig': {
        const newConfig = await invoke<AppConfig>('reload_config');
        setConfig(newConfig);
        setHasChanges(false);
        if (newConfig.language) {
          i18n.changeLanguage(newConfig.language);
        }
        break;
      }
    }
  };

  // Handle apply button
  const handleApply = async () => {
    if (!config) return;

    await invoke('save_config', { config });
    setHasChanges(false);
  };

  // Handle cancel button
  const handleCancel = () => {
    getCurrentWindow().close();
  };

  // Handle close button
  const handleClose = () => {
    getCurrentWindow().close();
  };

  if (!config) {
    return (
      <div className="flex h-screen items-center justify-center bg-app-bg">
        <span className="text-app-text-muted">Loading...</span>
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden rounded-xl border border-app-header-border bg-app-bg">
      {/* Custom title bar - draggable */}
      <div
        data-tauri-drag-region
        className="flex h-10 shrink-0 items-center justify-between border-b border-app-header-border bg-app-header px-4"
      >
        <span className="font-medium text-app-text text-sm">{t('title')}</span>
        <button
          type="button"
          onClick={handleClose}
          className="flex h-7 w-7 items-center justify-center rounded text-app-text-muted hover:bg-app-item-hover hover:text-app-text"
          aria-label="Close"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path
              d="M2 2L12 12M12 2L2 12"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      {/* Content area */}
      <div className="flex-1 overflow-y-auto p-4">
        {settingsSchema.map(section => (
          <SettingsSection
            key={section.id}
            section={section}
            config={config as unknown as Record<string, unknown>}
            onChange={handleChange}
            onAction={handleAction}
          />
        ))}
      </div>

      {/* Action bar */}
      <div className="flex shrink-0 justify-end gap-2 border-t border-app-header-border bg-app-header px-4 py-3">
        <button
          type="button"
          onClick={handleCancel}
          className="rounded bg-app-item px-4 py-2 font-medium text-app-text text-sm hover:bg-app-item-hover"
        >
          {t('cancel')}
        </button>
        <button
          type="button"
          onClick={handleApply}
          disabled={!hasChanges}
          className="rounded bg-app-item-selected px-4 py-2 font-medium text-white text-sm hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {t('apply')}
        </button>
      </div>
    </div>
  );
}
