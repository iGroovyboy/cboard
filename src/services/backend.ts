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

export const getFilesData = async (): Promise<FileEntry[]> => {
  console.log("...fetching...");

  return new Promise(async (resolve, reject) => {
    try {
      let data = await readDir(DIR_DATA, {
        dir: BaseDirectory.AppLocalData,
        recursive: true,
      });
      console.log("finished fetching", data);

      if (!data || !data[Folder.Clipboard] || !data[Folder.Favorites]) {
        resolve([]);
      }

      // keep only folders
      data = data.filter((f) => "children" in f);

      for (const [f, folder] of Object.entries(data)) {
        for (const [c, file] of Object.entries(
          (folder as ClipboardFolder).children
        )) {
          data[f].children[c].folder = folder.name;

          let contents;
          if (getFileTypeByFilename(file.path) === FILE_EXT.TXT) {
            // TODO: mb something faster?
            contents = truncateString(await readTextFile(file.path));
          } else {
            contents = convertFileSrc(file.path);
          }

          data[f].children[c].contents = contents;
          data[f].children[c].extension = getFileTypeByFilename(file.path);
        }

        data[f].children.sort((a, b) => {
          const nameA = a.name.toUpperCase();
          const nameB = b.name.toUpperCase();

          if (nameA > nameB) {
            return -1;
          }
          if (nameA < nameB) {
            return 1;
          }
          return 0;
        });
      }

      resolve(data);
    } catch (e) {
      reject(new Error(e));
    }
  });
};
