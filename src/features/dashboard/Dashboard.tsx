import DashboardActions from "./DashboardActions";
import "./Dashboard.css";

function Dashboard() {
  return (
    <main className="dashboard">
      <header className="dashboard-header">
        <h1>IoT Voice Assistant</h1>
        <p>Dashboard</p>
      </header>

      <section className="dashboard-content">
        <p>O aplicativo está em execução em segundo plano.</p>
        <p>Use o ícone na barra de ferramentas para abrir este painel ou fechar o programa.</p>
      </section>

      <DashboardActions />
    </main>
  );
}

export default Dashboard;
