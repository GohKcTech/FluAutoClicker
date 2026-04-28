import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { t } from "../i18n";
import { isWebviewDevtoolsAvailable, toggleWebviewDevtools } from "../utils";
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

    void isWebviewDevtoolsAvailable().then((available) => {
        if (!available || document.getElementById("webview-devtools-btn")) {
            return;
        }

        const cpsTestBtn = document.getElementById("cps-test-btn");
        if (!cpsTestBtn?.parentElement) return;

        const webviewDevtoolsBtn = document.createElement("div");
        webviewDevtoolsBtn.className = "settings-list-item";
        webviewDevtoolsBtn.id = "webview-devtools-btn";
        webviewDevtoolsBtn.style.cursor = "pointer";
        webviewDevtoolsBtn.innerHTML = `
            <div class="settings-item-icon"><span class="icon">&#57866;</span></div>
            <div class="settings-item-content">
                <span class="settings-item-title"></span>
                <span class="settings-item-desc"></span>
            </div>
            <span class="icon settings-item-arrow">&#57529;</span>
        `;

        const title = webviewDevtoolsBtn.querySelector(".settings-item-title");
        const desc = webviewDevtoolsBtn.querySelector(".settings-item-desc");
        if (title) title.textContent = t("settings.webview_devtools.title", "Webview DevTools");
        if (desc) desc.textContent = t("settings.webview_devtools.desc", "Open inspector for the current webview");

        webviewDevtoolsBtn.addEventListener("click", () => {
            void toggleWebviewDevtools().catch((error) => {
                console.error("Failed to toggle webview devtools", error);
            });
        });

        cpsTestBtn.insertAdjacentElement("afterend", webviewDevtoolsBtn);
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
