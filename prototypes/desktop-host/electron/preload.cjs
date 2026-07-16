const { ipcRenderer } = require("electron");

ipcRenderer.once("orchestra-port", (event) => {
  setTimeout(() => window.postMessage({ type: "orchestra-port" }, "*", event.ports), 0);
});

window.addEventListener("message", (event) => {
  if (typeof event.data?.orchestraPrototypeResult === "string") {
    ipcRenderer.send("orchestra-prototype-result", event.data.orchestraPrototypeResult);
  }
});
