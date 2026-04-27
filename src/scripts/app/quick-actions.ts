import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { showWebviewCreationError } from "../webview-error-modal";

export function initCpsTestWindow() {
    document.getElementById("cps-test-btn")?.addEventListener("click", () => {
        const cpsWindow = new WebviewWindow("cps-test", {
            url: "cps.html",
            title: "CPS Test",
            width: 400,
            height: 500,
            resizable: false,
            decorations: false,
            transparent: true,
            center: true,
        });

        void cpsWindow.once("tauri://error", (event) => {
            showWebviewCreationError("CPS Test", event.payload);
        });
    });
}

export function initDrawerLaunchers() {
    document.getElementById("jiggler-btn")?.addEventListener("click", async (event) => {
        event.stopPropagation();
        const { openDrawer } = await import("../drawer");
        openDrawer("section-jiggler", "Mouse Jiggler", "&#57987;");
    });

    document.getElementById("multithread-btn")?.addEventListener("click", async (event) => {
        event.stopPropagation();
        const { openDrawer } = await import("../drawer");
        openDrawer("section-multithread", "Multi Threading", "&#58548;");
    });
}
