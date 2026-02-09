import { fetch } from "@tauri-apps/plugin-http";
import { satisfies } from "semver"
import { getKeys } from "./key";

import { parse } from "date-fns"

import { checkServerIntegrity } from "tauri-plugin-ahqai-api"
import { AIWSChat } from "./ws";
import { ChatInstance } from "../store/db/chats";

export const supportedServerSemver = ">=0.2.x";

export const StatusFlags = {
  Unavailable: 2,
  Unauthorized: 4,
  SessionExpired: 8,
  UnsupportedServerVersion: 16,
  ChallengeFailed: 32,
  ExpiresSoon: 64,
}

export enum AuthType {
  Unknown,
  OpenToAll,
  Account
}

interface ServerInformation {
  version: string;
  auth: "OpenToAll" | "Account";
  can_register: boolean;
  models: { id: string, name: string, capabilities: number }[];
}

export class HTTPServer {
  url: string;
  session: string;
  flags: number = 0;

  registration = false;
  auth = AuthType.Unknown;
  expiry: Date | undefined = undefined;

  models: { id: string, name: string, capabilities: number }[] = [];

  usable = true;

  constructor(url: string, session: string) {
    this.url = url, this.session = session;
  }

  async getFlags(session?: string) {
    const keys = (await getKeys(true)).keys;

    this.flags = 0;

    let output: ServerInformation;
    try {
      output = await fetch(`${this.url}/`, {
        connectTimeout: 1000
      })
        .then((d) => d.json());
    } catch (e) {
      console.warn(e);
      this.flags |= StatusFlags.Unavailable;

      return this.flags;
    }

    this.models = output.models;

    this.registration = output.can_register || false;

    const versionKey = `v${output.version}`;

    if (!keys[versionKey]) this.flags |= StatusFlags.ChallengeFailed;

    if (keys[versionKey]) {
      const pubkey = keys[versionKey].pubkey;
      const expiry = keys[versionKey]?.expiry;

      if (expiry) {
        const date = parse(expiry, 'EEE, dd MMM yyyy HH:mm:ss \'GMT\'', new Date());

        this.expiry = date;
        this.flags |= StatusFlags.ExpiresSoon;
      }

      const data = new Uint8Array(256);

      data.fill(0);

      for (let i = 0; i < 256; i++) {
        data[i] = Math.floor(Math.random() * 127);
      }

      const binaryString = atob(pubkey);
      const pkey = Uint8Array.from(binaryString, (char) => char.charCodeAt(0));

      const signature = await fetch(`${this.url}/challenge`, {
        method: "POST",
        body: data.buffer
      })
        .then((d) => d.arrayBuffer())
        .catch(() => new ArrayBuffer());

      if (!(await checkServerIntegrity(data.buffer, signature, pkey))) this.flags |= StatusFlags.ChallengeFailed;
    }

    if (!satisfies(output.version, supportedServerSemver)) this.flags |= StatusFlags.UnsupportedServerVersion;

    this.auth = (() => {
      switch (output.auth as string) {
        case "OpenToAll":
          this.session = "no-auth";
          return AuthType.OpenToAll;
        case "Account":
          return AuthType.Account;
        default:
          return AuthType.Unknown;
      }
    })();

    if (this.auth == AuthType.Unknown) {
      this.flags |= StatusFlags.Unauthorized;

      return this.flags;
    }

    if (this.auth == AuthType.OpenToAll) {
      return this.flags;
    }

    // Auth Check
    if (!session) {
      this.flags |= StatusFlags.SessionExpired;

      this.usable = false;

      return this.flags;
    }

    try {
      await this.ping(session);
      this.usable = false;
    } catch (e) {
      this.flags |= StatusFlags.SessionExpired;
    }

    return this.flags;
  }

  /**
   * Get a handle to a AIWSChat
   * @param model The model to use
   * @param chat Instance to use
   * @returns {AIWSChat}
   */
  getWSCLass(model: string, chat: ChatInstance): AIWSChat {
    return new AIWSChat(this.session, chat, this.url, model);
  }

  /**
   * Throws on error
   * @param session
   */
  async ping(session: string) {
    this.session = await fetch(`${this.url}/me`, {
      method: "POST",
      body: session
    }).then((d) => {
      if (!d.ok) {
        throw new Error("Invalid Credentials");
      }

      return d.text();
    });
  }

  /**
   * Throws on error
   * @param username 
   * @param pass 
   */
  async authenticate(username: string, pass: string) {
    this.session = await fetch(`${this.url}/login`, {
      method: "POST",
      body: JSON.stringify({
        username,
        pass
      })
    }).then((d) => {
      if (!d.ok) {
        throw new Error("Invalid Credentials");
      }

      return d.text();
    });
  }
}