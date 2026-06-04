import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type RuntimeHotkeys = {
    toggle_start_stop: string;
    pick_position: string;
    toggle_macro_recording: string;
};

const MODIFIER_KEYS = new Set(["control", "shift", "alt", "meta"]);

const CODE_TO_SHORTCUT: Record<string, string> = {
    Space: "Space",
    Tab: "Tab",
    Enter: "Enter",
    Escape: "Escape",
    Backspace: "Backspace",
    Delete: "Delete",
    Insert: "Insert",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    CapsLock: "CapsLock",
    PrintScreen: "PrintScreen",
    ScrollLock: "ScrollLock",
    Pause: "Pause",
    Backquote: "`",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
};

function normalizeMainKey(event: KeyboardEvent): string | null {
    const key = event.key ?? "";
    if (!key || key === "Dead") return null;

    const lower = key.toLowerCase();
    if (MODIFIER_KEYS.has(lower)) {
        return null;
    }

    const code = event.code ?? "";
    if (/^Key[A-Z]$/i.test(code)) return code.slice(3).toUpperCase();
    if (/^Digit[0-9]$/i.test(code)) return code.slice(5);
    if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(code)) return code.toUpperCase();
    if (/^Numpad[0-9]$/i.test(code)) return code;
    if (code.startsWith("Numpad")) return code;
    if (CODE_TO_SHORTCUT[code]) return CODE_TO_SHORTCUT[code];

    if (key === " ") return "Space";
    if (key.length === 1 && /^[\x20-\x7E]$/.test(key)) return key.toUpperCase();
    return key.replace(/\s+/g, "");
}

function buildShortcut(event: KeyboardEvent): string | null {
    const main = normalizeMainKey(event);
    if (!main) return null;

    const parts: string[] = [];
    if (event.ctrlKey) parts.push("Ctrl");
    if (event.shiftKey) parts.push("Shift");
    if (event.altKey) parts.push("Alt");
    if (event.metaKey) parts.push("Win");
    parts.push(main);
    return parts.join("+");
}

export function initLocalShortcuts() {
    let hotkeys: RuntimeHotkeys = {
        toggle_start_stop: "F6",
        pick_position: "Ctrl+P",
        toggle_macro_recording: "Ctrl+Shift+R",
    };

    const updateHotkeys = async () => {
        try {
            hotkeys = await invoke<RuntimeHotkeys>("get_hotkeys");
        } catch (error) {
            console.error("Failed to fetch hotkeys for local intercept", error);
        }
    };

    void updateHotkeys();

    void listen("settings-applied", () => {
        void updateHotkeys();
    });

    const isCapturing = (): boolean => {
        if (document.activeElement?.tagName === "INPUT" || document.activeElement?.tagName === "TEXTAREA") {
            return true;
        }
        if (document.querySelector(".listening") || document.querySelector(".kb-record-btn.running")) {
            return true;
        }
        return false;
    };

    window.addEventListener("keydown", (event) => {
        if (isCapturing()) return;

        const shortcut = buildShortcut(event);
        if (!shortcut) return;

        if (shortcut === hotkeys.toggle_start_stop) {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById("start-btn")?.click();
        } else if (shortcut === hotkeys.toggle_macro_recording) {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById("macro-record-btn")?.click();
        } else if (shortcut === hotkeys.pick_position) {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById("pick-btn")?.click();
        }
    }, true);
}
