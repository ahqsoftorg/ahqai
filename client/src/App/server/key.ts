import { fetch } from "@tauri-apps/plugin-http";
import { checkFileIntegrity } from "tauri-plugin-ahqai-api";

const AHQAI_TRUSTED_KEYS_URL = "https://ahqsoftorg.github.io/ahqai/keys.json";
const AHQAI_TRUSTED_KEYS_INTEGRITY_URL = "https://ahqsoftorg.github.io/ahqai/keys.integrity";

const TWO_MINS_IN_SECS = 2 * 60;

let keys: { [key: string]: { pubkey: string, expiry?: string } | undefined } = {};
let old_danger = false;

let expiry = 0;

export async function getKeys(dangerous = false) {
  const now = Math.floor(Date.now() / 1000);

  // must fetch
  if (expiry < now) {
    const data = await fetch(AHQAI_TRUSTED_KEYS_URL).then((d) => d.arrayBuffer());
    const sig = await fetch(AHQAI_TRUSTED_KEYS_INTEGRITY_URL).then((d) => d.arrayBuffer());

    const validFile = await checkFileIntegrity(data, sig);

    if (validFile) {
      const decoder = new TextDecoder();

      keys = JSON.parse(decoder.decode(data));
      old_danger = false;
      expiry = Math.floor(Date.now() / 1000) + TWO_MINS_IN_SECS;

      return {
        keys,
        dangerous: false
      };
    }

    if (!dangerous) {
      throw new Error("Insecure Source, verification failed");
    }

    expiry = Math.floor(Date.now() / 1000) + TWO_MINS_IN_SECS;
    const decoder = new TextDecoder();

    keys = JSON.parse(decoder.decode(data));
    old_danger = true;
  }

  if (!dangerous && old_danger) {
    throw new Error("Insecure Source, verification failed");
  }

  return {
    keys,
    dangerous: old_danger
  };
}