const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("pomodoro", {
  listMemes: () => ipcRenderer.invoke("list-memes"),
  openMemesDir: () => ipcRenderer.invoke("open-memes-dir"),
});
