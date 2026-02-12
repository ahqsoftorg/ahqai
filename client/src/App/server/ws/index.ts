import { ChatInstance, Message } from "@/App/store/db/chats";
import WebSocket from "@tauri-apps/plugin-websocket"
import { toast } from "sonner";

export class AIWSChat {
  session: string;
  url: string;
  model: string;

  hinst: ChatInstance;
  ws: WebSocket | undefined = undefined;

  poll: { data: string, cb: (_: string) => void }[] = [];

  constructor(session: string, hinst: ChatInstance, url: string, model: string) {
    this.session = session;
    this.hinst = hinst;
    this.url = url.replace("http://", "ws://").replace("https://", "wss://");
    this.model = model;
  }

  async connect() {
    const ws = await WebSocket.connect(`${this.url}/chat`, {
      headers: {
        "Authorization": this.session,
        "model": this.model
      }
    });

    this.ws = ws;

    this.ws.addListener((msg) => {
      const data = this.poll.shift();

      data?.cb(msg.data as string);

      if (this.poll.length > 0) {
        this.wsSend(this.poll[0].data);
      }
    });
  }

  async wsSend(data: string) {
    try {
      await this.ws!.send({
        type: "Text",
        data
      });
    } catch (e) {
      toast.error("Could not send WS request!");
    }
  }

  sendAndPoll(data: string) {
    return new Promise((resolve) => {
      if (this.poll.length == 0) {
        this.wsSend(data);
      }

      this.poll.push({
        data,
        cb: resolve
      });
    });
  }

  async restore() {
    await this.sendAndPoll(JSON.stringify({
      event: "feed",
      history: (await Promise.all(this.hinst.cache.messages.slice(-200).map(this.hinst.getMessage.bind(this.hinst)))).sort((am, bm) => new Date(am.created_at).getTime() - new Date(bm.created_at).getTime())
        .map((msg) => {
          if (msg.responder == "user") {
            return {
              role: "user",
              content: [
                {
                  "type": "text",
                  text: msg.content
                }
              ]
            }
          } else if (msg.responder == "assistant") {
            return {
              role: "assistant",
              content: msg.content,
              thinking: null
            }
          } else {
            throw new Error("Unknown error");
          }
        })
    }));
  }

  async init() {
    await this.sendAndPoll(JSON.stringify({
      event: "init"
    }));
  }

  async disconnect() {
    await this.ws!.disconnect();
  }

  async chat(msg: string) {
    const data = JSON.parse(await this.sendAndPoll(JSON.stringify({
      event: "completion",
      msg: [
        {
          "type": "text",
          text: msg
        }
      ]
    })) as string);

    if (Array.isArray(data)) {
      const chats = data as Msg[];

      const msg = [] as Message[];

      for (let i = 0; i < chats.length; i++) {

        const chat = chats[i];

        const id = await this.hinst.insertMessage({
          responder: chat.role,
          content: chat.content,
          metadata: JSON.stringify(chat.thinking)
        });

        const chatmsg = await this.hinst.getMessage(id)!;

        msg.push(chatmsg);
      }

      return msg;
    }

    toast.error("Something went wrong while AI was processing");

    return [] as Message[];
  }
}

interface Msg {
  role: "assistant",
  content: string,
  thinking?: string
}