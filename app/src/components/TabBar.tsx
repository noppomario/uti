/**
 * Tab types supported by the application
 */
export type TabType = 'clipboard' | 'launcher';

/**
 * Props for the TabBar component
 */
interface TabBarProps {
  /** Currently active tab */
  activeTab: TabType;
  /** Callback when tab is changed */
  onTabChange: (tab: TabType) => void;
}

/**
 * Tab bar component for switching between Clipboard and Launcher views
 *
 * @param props - Component props
 * @returns The TabBar UI
 */
export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  const tabs: { id: TabType; label: string }[] = [
    { id: 'clipboard', label: 'Clipboard' },
    { id: 'launcher', label: 'Launcher' },
  ];

  return (
    <div role="tablist" className="flex gap-1">
      {tabs.map(tab => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={activeTab === tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`px-2 py-0.5 text-xs rounded transition-colors ${
            activeTab === tab.id
              ? 'font-semibold text-app-text bg-app-item'
              : 'text-app-text-muted hover:text-app-text'
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
