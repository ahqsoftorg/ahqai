import { HTTPServer } from "@/App/server";
import { AIWSChat } from "@/App/server/ws";
import { ChatInstance } from "@/App/store/db/chats";
import { useEffect, useState } from "react";

export interface MsgsProps {
  chat: ChatInstance,
  server: HTTPServer | undefined,
  ws: AIWSChat | undefined
}

// @ts-ignore
export function Messages({ chat, server, ws }: MsgsProps) {
  const [msgList, setMsgList] = useState(chat.cache.messages);

  useEffect(() => {
    chat.cb = (msg) => {
      console.log(msg);

      setMsgList((m) => {
        m.push(msg.id);

        return m;
      });
    };

    return () => {
      chat.cb = () => { };
    }
  }, []);

  return <>
    Hello
    {msgList.map((x, i) => <li key={`${i}`}>{x}</li>)}
  </>;
}