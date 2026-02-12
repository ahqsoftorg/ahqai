import { useEffect, useState, useRef, useCallback } from "react";
import { ChatInstance, Message } from "@/App/store/db/chats";
import { AIWSChat } from "@/App/server/ws";

export function useChatManager(chatId: number | undefined, temporary: boolean, newChat: boolean, sessionId: number) {
  const [state, setState] = useState<{
    chat: ChatInstance | null;
    ws: AIWSChat | null;
    messages: Message[];
    status: "loading" | "ready" | "connecting" | "connected" | "error";
  }>({
    chat: null,
    ws: null,
    messages: [],
    status: "loading",
  });

  const activeChatRef = useRef<ChatInstance | null>(null);
  const activeWsRef = useRef<AIWSChat | null>(null);

  useEffect(() => {
    let isMounted = true;

    async function setup() {
      // RAII: Cleanup previous instances
      if (activeWsRef.current) activeWsRef.current.disconnect();
      if (activeChatRef.current) activeChatRef.current.cleanup();

      activeWsRef.current = null;
      activeChatRef.current = null;

      setState({ chat: null, ws: null, messages: [], status: "loading" });

      try {
        const c = new ChatInstance();
        const mode = temporary ? "temporary" : (newChat && !chatId ? undefined : chatId);
        await c.init(mode as any);

        if (!isMounted) {
          c.cleanup();
          return;
        }

        // Initialize messages from cache/DB
        const initialMsgs = await Promise.all(
          c.cache.messages.map(id => c.getMessage(id))
        );

        activeChatRef.current = c;
        setState({ chat: c, ws: null, messages: initialMsgs.sort((am, bm) => new Date(am.created_at).getTime() - new Date(bm.created_at).getTime()), status: "ready" });
      } catch (e) {
        console.error("Chat Init Error:", e);
        if (isMounted) setState(prev => ({ ...prev, status: "error" }));
      }
    }

    setup();

    return () => {
      isMounted = false;
      if (activeWsRef.current) activeWsRef.current.disconnect();
      if (activeChatRef.current) activeChatRef.current.cleanup();
    };
  }, [sessionId]);

  const connectWS = useCallback(async (serverInstance: any, modelId: string) => {
    const currentChat = activeChatRef.current;
    if (!currentChat) return;

    setState(prev => ({ ...prev, status: "connecting" }));

    try {
      const ws = serverInstance.getWSCLass(modelId, currentChat);
      await ws.connect();
      await ws.restore();
      await ws.init();

      activeWsRef.current = ws;
      setState(prev => ({ ...prev, ws, status: "connected" }));
    } catch (e) {
      console.error("WS Connection Error:", e);
      setState(prev => ({ ...prev, status: "error" }));
      throw e;
    }
  }, []);

  const chatHook = useCallback(async (msgid: number, msg: string) => {
    if (activeWsRef.current) {
      const insertedMsg = await activeChatRef.current?.getMessage(msgid)!;
      setState((prev) => ({
        ...prev,
        messages: [...prev.messages, insertedMsg]
      }));

      const msgs = await activeWsRef.current.chat(msg);

      setState((prev) => ({
        ...prev,
        messages: [...prev.messages, ...msgs]
      }));
    }
  }, [activeWsRef.current, activeChatRef.current]);

  return { ...state, connectWS, chatHook };
}