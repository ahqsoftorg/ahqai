import Database from "@tauri-apps/plugin-sql";

class ChatDatabase {
  private static db: Database;

  async get() {
    if (!ChatDatabase.db) {
      ChatDatabase.db = await Database.load("sqlite:messages.db");
    }
  }
}

export const chatdb = new ChatDatabase();