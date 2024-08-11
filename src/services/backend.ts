import {
  BaseDirectory,
  exists,
  FileEntry,
  readDir,
  readTextFile,
  writeTextFile,
} from "@tauri-apps/api/fs";
import {
  DIR_DATA,
  FILE_EXT,
  Folder,
  FOLDER_NAME_MAP,
} from "../common/constants";
import { ClipboardFolder } from "../common/interfaces";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { getFileTypeByFilename, truncateString } from "../common/helpers";

const invoke = window.__TAURI__.invoke;

export const folderDeleteAll = (contextMenuFolder: number) => {
  invoke("delete_all_by_folder", {
    folder: FOLDER_NAME_MAP[contextMenuFolder],
  });
};

export const quit = () => invoke("quit");

export const getFile = async (filename: string) => {
  const dir = BaseDirectory.AppLocalData;
  let path = DIR_DATA + "/" + filename;

  if (await exists(path, { dir: dir })) {
    try {
      return await readTextFile(path, { dir: dir });
    } catch (e) {
      console.error(e);
    }
  } else {
    try {
      await writeTextFile(path, "", { dir: dir });
      return "";
    } catch (e) {
      console.error(e);
    }
  }
};

export const saveTextFile = async (filename: string, contents: string) => {
  const dir = BaseDirectory.AppLocalData;
  let path = DIR_DATA + "/" + filename;

  try {
    await writeTextFile(path, contents, { dir: dir });
    return true;
  } catch (e) {
    console.error(e);
  }
};
