/**
 * i18n configuration
 *
 * Initializes react-i18next with English and Japanese translations.
 * Used by settings window and update dialog.
 * Language is loaded from config and can be changed at runtime.
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './en.json';
import ja from './ja.json';

const resources = {
  en: { settings: en },
  ja: { settings: ja },
};

i18n.use(initReactI18next).init({
  resources,
  lng: 'en', // Default language, will be updated from config
  fallbackLng: 'en',
  defaultNS: 'settings',
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
