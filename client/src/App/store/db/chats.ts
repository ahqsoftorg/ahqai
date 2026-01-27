import Database from "@tauri-apps/plugin-sql";

// TODO: Rewrite in rust once app becomes slow
class ChatDatabase {
  private static db: Database;

  async get() {
    if (!ChatDatabase.db) {
      ChatDatabase.db = await Database.load("sqlite:messages.db");

      const db = ChatDatabase.db;

      await db.execute("PRAGMA journal_mode = WAL;");
      await db.execute("PRAGMA synchronous = NORMAL;");
      await db.execute("PRAGMA page_size = 4096;");
      await db.execute("PRAGMA foreign_keys = ON;");
    }

    const chatid = await this.newchat("Hello world", "");
    console.log(await this.fetchchat(chatid));
    await this.listchats();
    await this.deletechat(chatid);
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

    return output.lastInsertId!!;
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

  async enforceChatLimit(limit: number = 120) {
    const db = ChatDatabase.db;

    // This query deletes chats that are not in the 'newest' set
    await db.execute(`
    DELETE FROM CHATS
    WHERE id NOT IN (
      SELECT id FROM CHATS
      ORDER BY created_at DESC
      LIMIT $1
    );
  `, [limit]);
  }

  async listchats(): Promise<number[]> {
    const db = ChatDatabase.db;

    const output = (await db.select<{ id: number }[]>("SELECT id FROM CHATS", [])).map((d) => d.id);

    console.log(output);

    return output;
  }

  async fetchchat(id: number): Promise<Chat | undefined> {
    const db = ChatDatabase.db;

    const output = await db.select<Chat[] | undefined>("SELECT * FROM CHATS WHERE id=$1", [id]);

    return output?.[0];
  }

  async deletechat(id: number) {
    const db = ChatDatabase.db;

    await db.execute(`
      DELETE FROM CHATS
      WHERE id=$1;
    `, [id]);
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
  content: string,
  created_at: string,
  updated_at: string
}

export class ChatInstance {
  messages: Message[] = [];

}