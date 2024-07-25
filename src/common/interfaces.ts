export interface ClipboardItem {
  name: string; // "1672922494060.txt"
  folder: string; // "favorites"
  path: string; // "C:\\Users\\...\\AppData\\Local\\...\\data\\favorites"
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
