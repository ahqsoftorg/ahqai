import Database from "@tauri-apps/plugin-sql";

// TODO: Rewrite in rust once app becomes slow
export class ChatDatabase {
  static db: Database;
  private static inprog = false;

  static cb: (_: Chat) => void = () => { };

  async get() {
    if (ChatDatabase.inprog) { return; }

    if (!ChatDatabase.db) {
      ChatDatabase.inprog = true;
      ChatDatabase.db = await Database.load("sqlite:messages.db");

      const db = ChatDatabase.db;

      await db.execute("PRAGMA journal_mode = WAL;");
      await db.execute("PRAGMA synchronous = NORMAL;");
      await db.execute("PRAGMA page_size = 4096;");
      await db.execute("PRAGMA foreign_keys = ON;");
    }
  }

  async newchat(title: string, metadata: string): Promise<number> {
    const db = ChatDatabase.db;

    const output = await db.execute(`
      INSERT INTO CHATS (title, metadata)
      VALUES ($1, $2);
    `, [title, metadata]);

    // We don't 'await' this. We let it run in the background 
    // so the new chat ID is returned to the UI immediately.
    this.checkAndPrune(120, 10);

    const lastInsert = output.lastInsertId!!;

    try {
      ChatDatabase.cb((await this.fetchchat(lastInsert))!.chat!);
    } catch (e) {
      console.log(e);
    }

    return lastInsert;
  }

  private async checkAndPrune(limit: number, buffer: number) {
    const db = ChatDatabase.db;

    // Efficiently get the count without loading all IDs
    const result = await db.select<{ "COUNT(*)": number }[]>("SELECT COUNT(*) FROM CHATS");
    const count = result[0]?.["COUNT(*)"] || 0;

    if (count > limit + buffer) {
      await this.enforceChatLimit(limit);
    }
  }

  private async enforceChatLimit(limit: number = 120) {
    const db = ChatDatabase.db;

    // This query deletes chats that are not in the 'newest' set
    await db.execute(`
    DELETE FROM CHATS
    WHERE id NOT IN (
      SELECT id FROM CHATS
      ORDER BY created_at DESC
      LIMIT $1
    );`, [limit]);
  }

  async deletechat(id: number) {
    const db = ChatDatabase.db;

    await db.execute(`
      DELETE FROM CHATS
      WHERE id=$1;
    `, [id]);
  }

  async listchats(): Promise<number[]> {
    const db = ChatDatabase.db;

    const output = (await db.select<{ id: number }[]>("SELECT id FROM CHATS", [])).map((d) => d.id);

    return output;
  }

  async fetchchat(id: number): Promise<ChatInstance | undefined> {
    const db = ChatDatabase.db;
    const rows = await db.select<Chat[]>("SELECT * FROM CHATS WHERE id=$1", [id]);

    if (!rows || rows.length === 0) return undefined;

    const instance = new ChatInstance();
    await instance.init(rows[0]);
    return instance;
  }
}

export const chatdb = new ChatDatabase();

export interface Chat {
  created_at: string,
  id: number,
  metadata: string,
  title: string
}

export interface Message {
  id: number,
  chat_id: number,
  responder: string,
  metadata: string,
  content: string,
  created_at: string,
  updated_at: string
}

export interface MessageData {
  responder: string,
  content: string,
  metadata: string,
}

export class ChatInstance {
  cache = {
    // message ids in order
    messages: [] as number[],
    msgMap: {} as { [key: number]: Message }
  };
  chat_id: number | undefined | "temporary" = undefined;
  chat: Chat | undefined = undefined;
  db = ChatDatabase.db;

  cb: (msg: Message) => void = (_) => { };

  async init(ch: Chat | number | "temporary" | undefined) {
    const db = ChatDatabase.db;

    if (ch === 'temporary') {
      this.chat_id = "temporary";
      return;
    }

    if (ch === undefined) return;

    if (typeof (ch) == 'number') {
      this.chat_id = ch;
      this.chat = (await db.select<Chat[]>("SELECT * FROM CHATS WHERE id=$1", [ch]))[0]!!;
      await this.loadMsgs();
    }

    if (typeof (ch) == 'object') {
      this.chat_id = ch.id;
      this.chat = ch;
      await this.loadMsgs();
    }
  }

  private async loadMsgs() {
    const db = ChatDatabase.db;
    this.cache.messages = (await db.select<{ id: number }[]>("SELECT id FROM MESSAGES WHERE chat_id=$1", [this.chat_id])).map((({ id }) => id));
  }

  async insertMessage(msg: MessageData) {
    if (this.chat_id == "temporary") {
      const mid = this.cache.messages.length;

      this.cache.msgMap[mid] = {
        chat_id: 0,
        content: msg.content,
        metadata: msg.metadata,
        responder: msg.responder,
        created_at: new Date().toDateString(),
        id: mid,
        updated_at: new Date().toDateString()
      };
      this.cache.messages.push(mid);
      return mid;
    }

    if (this.chat_id == undefined) {
      console.log("Allocating chat id");
      const chatid = await chatdb.newchat(msg.content, "");

      this.chat_id = chatid;
      this.chat = {
        created_at: new Date().toDateString(),
        metadata: "",
        id: chatid,
        title: msg.content
      };
    }

    const msgid = (await this.db.execute(`
      INSERT INTO MESSAGES (chat_id, responder, content, metadata)
      VALUES ($1, $2, $3, $4);
    `, [this.chat_id!!, msg.responder, msg.content, msg.metadata])).lastInsertId!!;

    this.cache.messages.push(msgid);

    const msgdata = await this.getMessage(msgid);

    this.cache.msgMap[msgid] = msgdata;
    this.cb(msgdata);

    return msgid;
  }

  async getMessage(id: number) {
    if (this && this.cache && this.cache.msgMap[id]) {
      return this.cache.msgMap[id];
    }

    const db = ChatDatabase.db;

    const msg = (await db.select<Message[]>("SELECT * FROM MESSAGES WHERE id=$1", [id]))[0]!!;

    this.cache.msgMap[id] = msg;

    return msg;
  }

  async deletechat() {
    if (typeof (this.chat_id) == "number") {
      const db = ChatDatabase.db;

      await db.execute(`
        DELETE FROM CHATS
        WHERE id=$1;
      `, [this.chat_id]);
    }
  }

  cleanup() {
    this.cache.messages = [];
    this.cache.msgMap = {};
  }
}