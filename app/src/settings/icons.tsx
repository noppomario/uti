/**
 * Section icons for settings UI
 *
 * Uses lucide-react icons to match the main app's TabBar.
 */

import type { LucideIcon } from 'lucide-react';
import { ClipboardList, Info, Palette, Rocket, Settings, Sliders, Star } from 'lucide-react';

/** Icon size for section icons */
export const ICON_SIZE = 16;

/** Map section ID to lucide-react icon */
const sectionIcons: Record<string, LucideIcon> = {
  general: Sliders,
  appearance: Palette,
  clipboard: ClipboardList,
  snippets: Star,
  launcher: Rocket,
  advanced: Settings,
  about: Info,
};

/** Get icon component by section ID */
export function getSectionIcon(sectionId: string): LucideIcon | null {
  return sectionIcons[sectionId] || null;
}
