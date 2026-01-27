import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useMemo, useState } from "react";
import Splash from "./Splash";

import { ThemeContext } from "./theme";
import { initStore } from "./store";
import { getKeys } from "./server/key";
import { chatdb } from "./store/db/chats";

import Application from "./App";


export const PageId = {
  Splash: 0,
  Home: 1
};

export default function App() {
  const [page, setPage] = useState(PageId.Splash);

  const [, showDangerScreen] = useState(false);

  useEffect(() => {
    (async () => {
      // Show the window when its ready
      try {
        await initStore();
        try { await getCurrentWebviewWindow().show(); } catch (e) { }
        try {
          await getKeys();
        } catch (e) {
          console.error(e);
          showDangerScreen(true);
        }

        await chatdb.get();

        setTimeout(() => {
          setPage(PageId.Home);
        }, 2000);
      } catch (e) {
        console.error(e);
      }
    })()
  }, []);

  const pageContent = useMemo(() => {
    switch (page) {
      case PageId.Splash:
        return <Splash />;
      default:
        return <Application />;
    }
  }, [page]);

  return <ThemeContext>
    {pageContent}
  </ThemeContext>
}
