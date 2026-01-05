/**
 * Tab types supported by the application
 */
export type TabType = 'clipboard' | 'snippets' | 'launcher';

/**
 * Props for the TabBar component
 */
interface TabBarProps {
  /** Currently active tab */
  activeTab: TabType;
  /** Callback when tab is changed */
  onTabChange: (tab: TabType) => void;
}

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
 * Tab bar component for switching between Clipboard and Launcher views
 *
 * @param props - Component props
 * @returns The TabBar UI
 */
export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  const tabs: { id: TabType; label: string }[] = [
    { id: 'clipboard', label: 'Clipboard' },
    { id: 'snippets', label: 'Snippets' },
    { id: 'launcher', label: 'Launcher' },
  ];

  return (
    <div role="tablist" className="flex" style={containerStyles}>
      {tabs.map(tab => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={activeTab === tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`transition-colors ${
            activeTab === tab.id
              ? 'font-semibold text-app-text'
              : 'text-app-text-muted hover:text-app-text'
          }`}
          style={tabStyles}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
