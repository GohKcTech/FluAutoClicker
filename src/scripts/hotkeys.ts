import { invoke } from "@tauri-apps/api/core";
import { notify } from "./notifications";
import { getPlatformCapabilities } from "./platform-capabilities";

type HotkeyAction =
    | "toggle_start_stop"
    | "pick_position";

type RuntimeHotkeys = {
    toggle_start_stop: string;
    pick_position: string;
};

const ACTIONS: HotkeyAction[] = [
    "toggle_start_stop",
    "pick_position",
];

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

function isHotkeyAction(value: string | undefined): value is HotkeyAction {
    return Boolean(value && ACTIONS.includes(value as HotkeyAction));
}

function getHotkeyValue(hotkeys: RuntimeHotkeys, action: HotkeyAction): string {
    return hotkeys[action];
}

function normalizeDisplayPart(part: string): string {
    const trimmed = part.trim();
    if (!trimmed) return "";

    if (/^key[a-z]$/i.test(trimmed)) return trimmed.slice(3).toUpperCase();
    if (/^digit[0-9]$/i.test(trimmed)) return trimmed.slice(5);
    if (/^arrow(up|down|left|right)$/i.test(trimmed)) return trimmed.replace(/^arrow/i, "");

    if (trimmed === "Escape") return "Esc";

    const lower = trimmed.toLowerCase();
    if (lower === "ctrl" || lower === "control") return "Ctrl";
    if (lower === "shift") return "Shift";
    if (lower === "alt") return "Alt";
    if (lower === "meta" || lower === "cmd" || lower === "win" || lower === "super") return "Win";
    if (trimmed.length === 1) return trimmed.toUpperCase();
    return trimmed;
}

function formatHotkeyDisplay(value: string): string {
    return value
        .split("+")
        .map((part) => normalizeDisplayPart(part.trim()))
        .join(" + ");
}

function updateStartHotkeyLabel(hotkeys: RuntimeHotkeys) {
    const label = document.querySelector<HTMLElement>(".start-hotkey");
    if (!label) return;

    label.textContent = formatHotkeyDisplay(hotkeys.toggle_start_stop);
}

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

function fallbackHotkeys(): RuntimeHotkeys {
    return {
        toggle_start_stop: "F6",
        pick_position: "Ctrl+P",
    };
}

async function fetchHotkeys(): Promise<RuntimeHotkeys> {
    try {
        return await invoke<RuntimeHotkeys>("get_hotkeys");
    } catch {
        return fallbackHotkeys();
    }
}

export async function initHotkeysEditor() {
    const rows = Array.from(document.querySelectorAll<HTMLElement>(".hotkey-list-item[data-hotkey-action]"));
    if (!rows.length) return;
    const capabilities = await getPlatformCapabilities();
    const hotkeysAvailable = capabilities.global_hotkeys;

    const rowByAction = new Map<HotkeyAction, HTMLElement>();
    rows.forEach((row) => {
        const action = row.dataset.hotkeyAction;
        if (isHotkeyAction(action)) {
            rowByAction.set(action, row);
        }
    });

    let hotkeys = await fetchHotkeys();
    let activeAction: HotkeyAction | null = null;
    let activeRow: HTMLElement | null = null;
    let activeBadge: HTMLElement | null = null;
    let activePreviousRaw = "";
    let hotkeysSuspendedForCapture = false;

    const render = () => {
        ACTIONS.forEach((action) => {
            const row = rowByAction.get(action);
            const badge = row?.querySelector<HTMLElement>("[data-hotkey-value]");
            if (!row || !badge) return;

            const raw = getHotkeyValue(hotkeys, action);
            badge.dataset.rawHotkey = raw;
            badge.textContent = hotkeysAvailable ? formatHotkeyDisplay(raw) : "Unavailable";
            badge.title = hotkeysAvailable ? "Click to change" : "Global hotkeys are unavailable on Wayland";
            row.classList.toggle("disabled", !hotkeysAvailable);
            row.setAttribute("aria-disabled", hotkeysAvailable ? "false" : "true");
        });

        if (hotkeysAvailable) {
            updateStartHotkeyLabel(hotkeys);
        } else {
            const label = document.querySelector<HTMLElement>(".start-hotkey");
            if (label) label.textContent = "BETA";
        }
    };

    const clearCaptureState = async (restoreBadge: boolean, resumeHotkeys = true) => {
        if (activeRow) activeRow.classList.remove("listening");
        if (activeBadge) {
            activeBadge.classList.remove("listening");
            if (restoreBadge) {
                activeBadge.textContent = formatHotkeyDisplay(activePreviousRaw);
            }
        }

        window.removeEventListener("keydown", onGlobalKeydown, true);
        window.removeEventListener("mousedown", onGlobalPointerDown, true);

        activeAction = null;
        activeRow = null;
        activeBadge = null;
        activePreviousRaw = "";

        if (hotkeysSuspendedForCapture && resumeHotkeys) {
            try {
                await invoke("resume_hotkeys");
                hotkeysSuspendedForCapture = false;
            } catch (error) {
                const message = error instanceof Error ? error.message : String(error);
                notify(message, "error", 2600);
            }
        }
    };

    const onGlobalPointerDown = (event: MouseEvent) => {
        if (!activeRow) return;
        const target = event.target as Node | null;
        if (target && activeRow.contains(target)) return;
        void clearCaptureState(true);
    };

    const onGlobalKeydown = async (event: KeyboardEvent) => {
        if (!activeAction || !activeBadge) return;
        if (event.repeat) return;

        event.preventDefault();
        event.stopPropagation();

        if (event.key === "Escape") {
            await clearCaptureState(true);
            return;
        }

        const shortcut = buildShortcut(event);
        if (!shortcut) return;

        try {
            const updated = await invoke<RuntimeHotkeys>("set_hotkey", {
                action: activeAction,
                shortcut,
            });
            hotkeys = updated;
            render();
            await clearCaptureState(false);
            notify("Hotkey updated", "success", 1800);
        } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            notify(message, "error", 2600);
            await clearCaptureState(true);
        }
    };

    const startCapture = async (row: HTMLElement, action: HotkeyAction) => {
        if (!hotkeysAvailable) {
            notify("Global hotkeys are unavailable on Wayland", "info", 2400);
            return;
        }

        const badge = row.querySelector<HTMLElement>("[data-hotkey-value]");
        if (!badge) return;

        if (activeRow && activeRow !== row) {
            await clearCaptureState(true, false);
        }

        if (activeRow === row) return;

        if (!hotkeysSuspendedForCapture) {
            try {
                await invoke("suspend_hotkeys");
                hotkeysSuspendedForCapture = true;
            } catch (error) {
                const message = error instanceof Error ? error.message : String(error);
                notify(message, "error", 2600);
                return;
            }
        }

        activeAction = action;
        activeRow = row;
        activeBadge = badge;
        activePreviousRaw = getHotkeyValue(hotkeys, action);

        row.classList.add("listening");
        badge.classList.add("listening");
        badge.textContent = "Press keys...";

        window.addEventListener("keydown", onGlobalKeydown, true);
        window.addEventListener("mousedown", onGlobalPointerDown, true);
    };

    rowByAction.forEach((row, action) => {
        const badge = row.querySelector<HTMLElement>("[data-hotkey-value]");
        if (!badge) return;

        row.addEventListener("click", () => {
            void startCapture(row, action);
        });
        badge.addEventListener("click", (event) => {
            event.stopPropagation();
            void startCapture(row, action);
        });
    });

    render();
}
