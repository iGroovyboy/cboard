import { readDir, BaseDirectory, readTextFile } from "@tauri-apps/api/fs";
import { DIR_DATA, FOLDER_NAME_MAP, Folder } from "../common/constants";
import { ClipboardFolder } from "../common/interfaces";

const invoke = window.__TAURI__.invoke;

export const folderDeleteAll = (contextMenuFolder: number) => {
  invoke("deleteAllByFolder", { folder: FOLDER_NAME_MAP[contextMenuFolder] });
};

export const quit = () => invoke("quit");

export const getFilesData = () => {
  return new Promise(async (resolve, reject) => {
    try {
      const data = await readDir(DIR_DATA, {
        dir: BaseDirectory.AppLocalData,
        recursive: true,
      });
      console.log("FETCHED RAW", data);

      if (!data || !data[Folder.Clipboard] || !data[Folder.Favorites]) {
        resolve([]);
      }

      // TODO: mb move to backend
      for (const [f, folder] of Object.entries(data)) {
        for (const [c, file] of Object.entries(
          (folder as ClipboardFolder).children
        )) {
          const contents = await readTextFile(file.path);
          data[f].children[c].contents = contents;
          data[f].children[c].folder = folder.name;
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
