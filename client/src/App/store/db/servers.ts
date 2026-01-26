import { HTTPServer } from "@/App/server";
import { State } from "../state";

import { BaseDirectory, mkdir, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs"

export interface JSONServer {
  name: string;
  url: string;
  session: string;
}

export interface Server {
  name: string;
  url: string;
  instance: HTTPServer
}

export const ServersState = new State<Server[]>([]);

export async function initServerState() {
  await mkdir("chats", {
    baseDir: BaseDirectory.AppData,
    recursive: true
  }).catch(console.info);

  const serverJsonData = await readTextFile("server.json", {
    baseDir: BaseDirectory.AppData
  }).catch(() => "");

  let val: JSONServer[];

  try {
    val = JSON.parse(serverJsonData);

    if (!Array.isArray(val)) throw new Error("");
  } catch (e) {
    val = [];
    console.warn(e);
  }

  const outValue = await checkServers(val);

  ServersState.value = outValue;

  ServersState.registerListener((data) => {
    writeTextFile("server.json", JSON.stringify(data.map((server) => ({
      name: server.name,
      url: server.url,
      session: server.instance.session
    }))), {
      baseDir: BaseDirectory.AppData
    }).catch(console.error)
  });
}

async function checkServers(val: JSONServer[]): Promise<Server[]> {
  return await Promise.all(
    val.map(async (data) => {
      const inst = new HTTPServer(data.url, data.session);

      const flags = await inst.getFlags(data.session);

      return {
        name: data.name,
        session: data.session,
        url: data.url,

        /** Newly Injected Fields */
        status: flags,
        instance: inst,
      } as Server;
    })
  );
}