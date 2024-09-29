export interface ClipboardItem {
  name: string; // "1672922494060.txt"
  folder: string; // "favorites"
  path: string; // "C:\\Users\\...\\AppData\\Local\\...\\data\\favorites"
  size: number;
  extension: string;
  contents?: string;
}

export interface ClipboardFolder {
  children: [] | ClipboardItem[];
  name: string;
  path: string; // "C:\\Users\\...\\AppData\\Local\\...\\data\\favorites"
}

export type ClipboardData = ClipboardFolder[];

export enum SETTINGS_KEY {
  WINDOW_POS = "window_pos",
  WINDOW_SIZE = "window_size",
}

export interface AutoReplacementItem {
  key: string;
  value: string;
}

export interface AppItem {
  enabled: boolean;
  filename?: string;
  filepath: string;
  title?: string;
}

export interface KeyAppItem extends AppItem {
  lang_id?: number;
}

export interface KeyboardLayout {
  lang_id: number;
  lang_code: String;
  lang_name: String;
}
