import { invoke } from "@tauri-apps/api/core";

export type PlatformCapabilities = {
    window_acrylic: boolean;
    system_startup: boolean;
    global_hotkeys: boolean;
    wayland: boolean;
    os?: string;
};

const fallbackCapabilities: PlatformCapabilities = {
    window_acrylic: false,
    system_startup: false,
    global_hotkeys: true,
    wayland: false,
    os: undefined,
};

let cachedCapabilities: PlatformCapabilities | null = null;

export async function getPlatformCapabilities(): Promise<PlatformCapabilities> {
    if (cachedCapabilities) {
        return cachedCapabilities;
    }

    try {
        cachedCapabilities = await invoke<PlatformCapabilities>("get_platform_capabilities");
    } catch (error) {
        console.warn("Failed to load platform capabilities", error);
        cachedCapabilities = fallbackCapabilities;
    }

    return cachedCapabilities;
}
