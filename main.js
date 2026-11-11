const { app, BrowserWindow, Menu, protocol, net, session, shell, ipcMain } = require("electron");
const path = require("path");
const fs = require("fs");
const { pathToFileURL } = require("url");

const MEDIA_EXT = /\.(png|jpe?g|gif|webp|mp4|webm)$/i;

protocol.registerSchemesAsPrivileged([
  { scheme: "app", privileges: { standard: true, secure: true, supportFetchAPI: true, stream: true } },
]);

const memesDir = () => {
  const dir = path.join(app.getPath("userData"), "memes");
  fs.mkdirSync(dir, { recursive: true });
  return dir;
};

// Sert l'appli (app://local/...) et le dossier de memes de l'utilisateur (app://memes/...)
function registerProtocol() {
  protocol.handle("app", (req) => {
    const url = new URL(req.url);
    const root = url.host === "memes" ? memesDir() : app.getAppPath();
    const file = path.normalize(path.join(root, decodeURIComponent(url.pathname)));
    if (!file.startsWith(root)) return new Response("Forbidden", { status: 403 });
    return net.fetch(pathToFileURL(file).toString());
  });
}

function createWindow() {
  const win = new BrowserWindow({
    width: 520, height: 780, minWidth: 420, minHeight: 600,
    title: "CharlieKirkPomodoro",
    webPreferences: { preload: path.join(__dirname, "preload.js"), contextIsolation: true },
  });
  win.loadURL("app://local/src/index.html");
}

function buildMenu() {
  Menu.setApplicationMenu(Menu.buildFromTemplate([
    { label: "Fichier", submenu: [
      { label: "Ouvrir le dossier des memes", click: () => shell.openPath(memesDir()) },
      { type: "separator" },
      { role: "quit", label: "Quitter" },
    ] },
    { role: "viewMenu", label: "Affichage" },
  ]));
}

ipcMain.handle("list-memes", () =>
  fs.readdirSync(memesDir()).filter((f) => MEDIA_EXT.test(f)).map((f) => `app://memes/${encodeURIComponent(f)}`));
ipcMain.handle("open-memes-dir", () => shell.openPath(memesDir()));

app.whenReady().then(() => {
  registerProtocol();
  session.defaultSession.setPermissionRequestHandler((_wc, permission, cb) => cb(permission === "media"));
  session.defaultSession.setPermissionCheckHandler((_wc, permission) => permission === "media");
  buildMenu();
  createWindow();
  app.on("activate", () => BrowserWindow.getAllWindows().length === 0 && createWindow());
});

app.on("window-all-closed", () => app.quit());
