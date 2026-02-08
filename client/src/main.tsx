import ReactDOM from "react-dom/client";
import App from "./App/index.tsx";

import "./global.css"

import { ContextMenu, ContextMenuTrigger } from "./components/ui/context-menu.tsx";
import { Toaster } from "@/components/ui/sonner"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ContextMenu>
    <Toaster />
    <ContextMenuTrigger>
      <App />
    </ContextMenuTrigger>
  </ContextMenu>,
);
