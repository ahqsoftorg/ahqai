import { HTTPServer } from "@/App/server";
import { AIWSChat } from "@/App/server/ws";
import { ChatInstance, Message } from "@/App/store/db/chats";
import { useEffect, useState } from "react";

export interface MsgsProps {
  chat: ChatInstance,
  server: HTTPServer | undefined,
  ws: AIWSChat | undefined
}

// @ts-ignore
export function Messages({ chat, server, ws }: MsgsProps) {
  const [msgList, setMsgList] = useState<Message[]>();

  useEffect(() => {
    try {
      const promises = chat.cache.messages.map(chat.getMessage.bind(chat));

      Promise.all(promises)
        .then(setMsgList);
    } catch (e) { }

    chat.cb = (msg) => {
      console.log(msg);

      setMsgList((m) => ([...m!, msg]));
    };

    return () => {
      chat.cb = () => { };
    }
  }, []);

  return <>
    Hello
    {msgList && msgList.map((x, i) => <li key={`${i}`}>{x.content}</li>)}
  </>;
}