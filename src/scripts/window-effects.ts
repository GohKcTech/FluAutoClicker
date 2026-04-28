import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const ACRYLIC_STORAGE_KEY = "flu-window-acrylic";
let acrylicEnabledState = false;
let acrylicNativeReady = false;
const currentWindow = getCurrentWindow();

export function isAcrylicPreferred() {
    return localStorage.getItem(ACRYLIC_STORAGE_KEY) === "true";
}

export function setAcrylicPreferred(enabled: boolean) {
    localStorage.setItem(ACRYLIC_STORAGE_KEY, enabled ? "true" : "false");
}

export async function applyWindowEffects(enabled: boolean) {
    let acrylicApplied = false;
    const focused = document.hasFocus();

    try {
        acrylicApplied = await invoke<boolean>("set_window_acrylic", { enabled, focused });
    } catch (error) {
        console.warn("Failed to update acrylic window effect", error);
    }

    const active = enabled && acrylicApplied;
    acrylicEnabledState = enabled;
    acrylicNativeReady = active;
    document.body.classList.toggle("window-acrylic", active);
    document.documentElement.dataset.windowEffect = active ? "acrylic" : "default";

    return active;
}

export function applyWindowFocusState(focused: boolean) {
    document.body.classList.toggle("window-unfocused", !focused);
    document.documentElement.dataset.windowFocused = focused ? "1" : "0";
}

async function syncWindowFocusEffect(focused: boolean) {
    if (!acrylicEnabledState || !acrylicNativeReady) {
        return;
    }

    try {
        await invoke("set_window_acrylic", { enabled: true, focused });
    } catch (error) {
        console.warn("Failed to sync acrylic focus state", error);
    }
}

export function initWindowFocusState() {
    applyWindowFocusState(true);

    void currentWindow.onFocusChanged(({ payload: focused }) => {
        applyWindowFocusState(focused);
        void syncWindowFocusEffect(focused);
    });

    void currentWindow.onMoved(async () => {
        const focused = await currentWindow.isFocused();
        applyWindowFocusState(focused);
        void syncWindowFocusEffect(focused);
    });
}

export async function initWindowEffects() {
    initWindowFocusState();
    window.addEventListener("flu:settings-applied", () => {
        void applyWindowEffects(isAcrylicPreferred());
    });
    return applyWindowEffects(isAcrylicPreferred());
}
