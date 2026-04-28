import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyWindowControlIcons, WINDOW_CONTROL_ICONS_CHANGED_EVENT } from "../window-control-icons";

const appWindow = getCurrentWindow();
export const ALWAYS_ON_TOP_STORAGE_KEY = "flu-always-on-top";
export const ALWAYS_ON_TOP_CHANGED_EVENT = "flu:always-on-top-changed";

function readStoredAlwaysOnTop() {
    return localStorage.getItem(ALWAYS_ON_TOP_STORAGE_KEY) === "true";
}

export async function setAlwaysOnTopPreference(active: boolean) {
    await appWindow.setAlwaysOnTop(active);
    localStorage.setItem(ALWAYS_ON_TOP_STORAGE_KEY, String(active));
    window.dispatchEvent(new CustomEvent(ALWAYS_ON_TOP_CHANGED_EVENT, { detail: active }));
}

async function syncAlwaysOnTopFromWindow() {
    try {
        localStorage.setItem(ALWAYS_ON_TOP_STORAGE_KEY, String(await appWindow.isAlwaysOnTop()));
    } catch {
        localStorage.setItem(ALWAYS_ON_TOP_STORAGE_KEY, String(readStoredAlwaysOnTop()));
    }
}

async function applyStoredAlwaysOnTop() {
    const active = readStoredAlwaysOnTop();
    try {
        await appWindow.setAlwaysOnTop(active);
    } catch (error) {
        console.error("Failed to apply always on top preference", error);
    }
}

async function refreshWindowControlIcons() {
    try {
        applyWindowControlIcons({ maximized: await appWindow.isMaximized() });
    } catch {
        applyWindowControlIcons();
    }
}

export function initWindowControls() {
    void refreshWindowControlIcons();
    void applyStoredAlwaysOnTop();
    window.addEventListener("resize", () => {
        void refreshWindowControlIcons();
    });
    window.addEventListener(WINDOW_CONTROL_ICONS_CHANGED_EVENT, () => {
        void refreshWindowControlIcons();
    });
    window.addEventListener("flu:settings-applied", () => {
        void refreshWindowControlIcons();
        void applyStoredAlwaysOnTop();
    });
    window.addEventListener(ALWAYS_ON_TOP_CHANGED_EVENT, () => {
        void syncAlwaysOnTopFromWindow();
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
