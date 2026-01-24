import { platform } from '@tauri-apps/plugin-os';

export const os = platform();

export const isApple = os == "ios" || os == "macos";
