import ActionButton from "../../components/ActionButton";
import Panel from "../../components/Panel";
import { dashboardActions } from "./actions";

function DashboardActions() {
  return (
    <Panel title="Atalhos do sistema">
      <div className="dashboard-actions">
        {dashboardActions.map((action) => (
          <ActionButton
            key={action.id}
            label={action.label}
            onClick={action.run}
          />
        ))}
      </div>
    </Panel>
  );
}

export default DashboardActions;
