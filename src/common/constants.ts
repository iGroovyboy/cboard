export enum Folder {
  Clipboard = 0,
  Favorites = 1,
}

export const FILE_NAME = {
  Autoreplace: "autoreplace.json",
  Blacklist: "blacklist.json",
};

export const FOLDER_NAME = {
  Clipboard: "clipboard",
  Favorites: "favorites",
};

export const MENU_TYPE = {
  None: 0,
  Context: 1,
  Main: 2,
};

export const FOLDER_NAME_MAP: Record<number, string> = {
  0: FOLDER_NAME.Clipboard,
  1: FOLDER_NAME.Favorites,
};

export const DIR_DATA = "data";

export const FILE_EXT = {
  PNG: "png",
  TXT: "txt",
};
