import ActionButton from "../../components/ActionButton";
import { dashboardActions } from "./actions";

function DashboardActions() {
  return (
    <section className="dashboard-actions">
      {dashboardActions.map((action) => (
        <ActionButton key={action.id} label={action.label} onClick={action.run} />
      ))}
    </section>
  );
}

export default DashboardActions;
