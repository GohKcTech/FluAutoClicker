import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyWindowControlIcons, WINDOW_CONTROL_ICONS_CHANGED_EVENT } from "../window-control-icons";

const appWindow = getCurrentWindow();

async function refreshWindowControlIcons() {
    try {
        applyWindowControlIcons({ maximized: await appWindow.isMaximized() });
    } catch {
        applyWindowControlIcons();
    }
}

export function initWindowControls() {
    void refreshWindowControlIcons();
    window.addEventListener("resize", () => {
        void refreshWindowControlIcons();
    });
    window.addEventListener(WINDOW_CONTROL_ICONS_CHANGED_EVENT, () => {
        void refreshWindowControlIcons();
    });
    window.addEventListener("flu:settings-applied", () => {
        void refreshWindowControlIcons();
    });

    document.getElementById("minimize-btn")?.addEventListener("click", () => {
        void invoke("minimize_main_window").catch((error) => {
            console.error("Failed to minimize window", error);
            void appWindow.minimize();
        });
    });

    document.getElementById("maximize-btn")?.addEventListener("click", async () => {
        try {
            const isMaximized = await appWindow.isMaximized();
            if (isMaximized) {
                await appWindow.unmaximize();
            } else {
                await appWindow.maximize();
            }
            await refreshWindowControlIcons();
        } catch (error) {
            console.error("Failed to toggle maximize state", error);
        }
    });

    document.getElementById("close-btn")?.addEventListener("click", () => {
        void appWindow.close();
    });
}
