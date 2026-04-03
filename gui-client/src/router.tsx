import { createBrowserRouter } from "react-router-dom";
import { App } from "./App";
import { ChatPage } from "./components/chat/ChatPage";
import { TaskPage } from "./components/tasks/TaskPage";
import { ModelPage } from "./components/models/ModelPage";
import { SettingsPage } from "./components/settings/SettingsPage";
import { NotFound } from "./components/common/NotFound";

export function createRouter() {
  return createBrowserRouter([
    {
      path: "/",
      element: <App />,
      children: [
        {
          path: "",
          element: <ChatPage />,
        },
        {
          path: "tasks",
          element: <TaskPage />,
        },
        {
          path: "models",
          element: <ModelPage />,
        },
        {
          path: "settings",
          element: <SettingsPage />,
        },
        {
          path: "*",
          element: <NotFound />,
        },
      ],
    },
  ]);
}
