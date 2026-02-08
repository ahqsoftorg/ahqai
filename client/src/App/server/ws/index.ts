import WebSocket from "@tauri-apps/plugin-websocket"

export class AIWSChat {
  session: string;
  url: string;
  model: string;

  ws: WebSocket | undefined = undefined;

  constructor(session: string, url: string, model: string) {
    this.session = session;
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

    console.log(ws);

    this.ws = ws;
  }

  async disconnect() {
    await this.ws!.disconnect();
  }
}