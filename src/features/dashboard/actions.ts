import { openBrowser, openFileExplorer } from "../../api/systemCommands";

export type DashboardAction = {
  id: string;
  label: string;
  run: () => void;
};

export const dashboardActions: DashboardAction[] = [
  {
    id: "open-browser",
    label: "Abrir navegador",
    run: () => {
      openBrowser().catch(console.error);
    },
  },
  {
    id: "open-file-explorer",
    label: "Abrir explorador de arquivos",
    run: () => {
      openFileExplorer().catch(console.error);
    },
  },
];
