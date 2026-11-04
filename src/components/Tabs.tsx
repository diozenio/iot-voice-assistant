import "./Tabs.css";

export type Tab<T extends string> = {
  id: T;
  label: string;
};

type TabsProps<T extends string> = {
  tabs: Tab<T>[];
  active: T;
  onChange: (id: T) => void;
};

function Tabs<T extends string>({ tabs, active, onChange }: TabsProps<T>) {
  return (
    <nav className="tabs">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          className={`tab ${tab.id === active ? "tab-active" : ""}`}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}

export default Tabs;
