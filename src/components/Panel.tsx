import type { ReactNode } from "react";
import "./Panel.css";

type PanelProps = {
  title: string;
  badge?: string;
  actions?: ReactNode;
  children: ReactNode;
};

function Panel({ title, badge, actions, children }: PanelProps) {
  return (
    <section className="panel">
      <header className="panel-header">
        <h2 className="panel-title">
          {title}
          {badge !== undefined && <span className="panel-badge">{badge}</span>}
        </h2>
        {actions && <div className="panel-actions">{actions}</div>}
      </header>
      <div className="panel-body">{children}</div>
    </section>
  );
}

export default Panel;
