import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

export function initWindowControls() {
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
        } catch (error) {
            console.error("Failed to toggle maximize state", error);
        }
    });

    document.getElementById("close-btn")?.addEventListener("click", () => {
        void appWindow.close();
    });
}
