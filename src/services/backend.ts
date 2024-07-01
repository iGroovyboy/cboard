import { FOLDER_NAME_MAP } from "../common/constants";

const invoke = window.__TAURI__.invoke;

export const folderDeleteAll = (contextMenuFolder: number) => {
  invoke("deleteAllByFolder", { folder: FOLDER_NAME_MAP[contextMenuFolder] });
};

export const quit = () => invoke("quit");
