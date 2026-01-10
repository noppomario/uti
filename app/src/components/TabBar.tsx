import type { LucideIcon } from 'lucide-react';
import { ClipboardList, MessageSquare, Rocket, Star } from 'lucide-react';

/**
 * Tab types supported by the application
 */
export type TabType = 'prompt' | 'clipboard' | 'snippets' | 'launcher';

/**
 * Props for the TabBar component
 */
interface TabBarProps {
  /** Currently active tab */
  activeTab: TabType;
  /** Callback when tab is changed */
  onTabChange: (tab: TabType) => void;
}

/** Tab definition with icon */
interface TabDefinition {
  id: TabType;
  label: string;
  icon: LucideIcon;
}

/** Icon size using em units to scale with text size */
const ICON_SIZE = '1.25em';

/** Inline styles using CSS variables for theme-based sizing */
const tabStyles: React.CSSProperties = {
  fontSize: 'var(--size-font-base)',
  padding: 'var(--size-padding-y) var(--size-padding-x)',
  borderRadius: 'var(--size-radius)',
};

const containerStyles: React.CSSProperties = {
  gap: 'var(--size-gap)',
};

/**
 * Tab bar component for switching between Clipboard, Snippets, and Launcher views
 *
 * Display modes based on size theme:
 * - minimal: Icon only
 * - normal/wide: Icon + text
 *
 * @param props - Component props
 * @returns The TabBar UI
 */
export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  const tabs: TabDefinition[] = [
    { id: 'prompt', label: 'Prompt', icon: MessageSquare },
    { id: 'clipboard', label: 'Clipboard', icon: ClipboardList },
    { id: 'snippets', label: 'Snippets', icon: Star },
    { id: 'launcher', label: 'Launcher', icon: Rocket },
  ];

  return (
    <div role="tablist" className="flex" style={containerStyles}>
      {tabs.map(tab => {
        const Icon = tab.icon;
        return (
          <button
            key={tab.id}
            type="button"
            role="tab"
            aria-selected={activeTab === tab.id}
            aria-label={tab.label}
            title={tab.label}
            onClick={() => onTabChange(tab.id)}
            className={`flex items-center transition-colors ${
              activeTab === tab.id
                ? 'font-semibold text-app-text'
                : 'text-app-text-muted hover:text-app-text'
            }`}
            style={tabStyles}
          >
            <Icon size={ICON_SIZE} aria-hidden="true" />
            <span className="tab-label ml-1">{tab.label}</span>
          </button>
        );
      })}
    </div>
  );
}
