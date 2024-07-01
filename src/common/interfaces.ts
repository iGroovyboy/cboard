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
