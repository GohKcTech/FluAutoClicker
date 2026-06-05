import { invoke } from "@tauri-apps/api/core";
import { notify } from "../notifications";
import { macroState } from "./state";
import type {
    MacroActionDraft,
    MacroActionType,
    MacroBackendAction,
    MacroKeyboardDraft,
    MacroMoveDraft,
    MacroMouseDraft,
    MacroSleepDraft,
    MacroScrollDraft,
    MacroUiAction,
} from "./types";

export function normalizeKey(key: string): string {
    const map: Record<string, string> = {
        backspace: "backspace",
        esc: "escape",
        pgup: "pageup",
        pgdn: "pagedown",
        del: "delete",
        ins: "insert",
    };

    return map[key] || key;
}

export function capitalize(value: string) {
    return value.charAt(0).toUpperCase() + value.slice(1);
}

export function parseKeyboardCombo(raw: string) {
    const tokens = raw
        .split("+")
        .map((value) => value.trim().toLowerCase())
        .filter(Boolean);

    const modifiers: string[] = [];
    let key = "a";

    tokens.forEach((token) => {
        if (["ctrl", "shift", "alt", "win"].includes(token)) {
            if (!modifiers.includes(token)) {
                modifiers.push(token);
            }
            return;
        }

        key = normalizeKey(token);
    });

    if (tokens.length === 1 && ["ctrl", "shift", "alt", "win"].includes(tokens[0])) {
        key = "a";
    }

    return { key, modifiers };
}

function parseModifierList(value: unknown): string[] {
    if (Array.isArray(value)) {
        return value.map(String).map((part) => part.trim().toLowerCase()).filter(Boolean);
    }

    return String(value || "")
        .split(/[+,]/)
        .map((part) => part.trim().toLowerCase())
        .filter((part) => part && part !== "none");
}

export function getHoldMs(action: unknown): number | null {
    if (typeof action === "string" && action.startsWith("hold_")) {
        return Number.parseInt(action.replace("hold_", ""), 10);
    }

    if (action && typeof action === "object" && "hold" in action) {
        const hold = (action as { hold?: { duration_ms?: unknown } }).hold;
        const duration = Number(hold?.duration_ms);
        return Number.isFinite(duration) ? duration : null;
    }

    return null;
}

export async function captureCursorPosition(
    pickButton: HTMLElement,
    xInputId: string,
    yInputId: string
) {
    const delayMs = Math.max(0, Number(macroState.capabilities.pick_delay_ms || 5000));
    const originalText = pickButton.innerHTML;
    const button = pickButton as HTMLButtonElement;
    let countdownTimer: number | null = null;

    button.disabled = true;
    if (delayMs > 0) {
        let remainingSeconds = Math.ceil(delayMs / 1000);
        const renderCountdown = () => {
            pickButton.innerHTML = `<span class="icon">&#58633;</span> PICK ${remainingSeconds}`;
        };
        renderCountdown();
        countdownTimer = window.setInterval(() => {
            remainingSeconds -= 1;
            if (remainingSeconds > 0) {
                renderCountdown();
            } else {
                window.clearInterval(countdownTimer!);
                countdownTimer = null;
            }
        }, 1000);
        notify(`Move the cursor. Position will be captured in ${Math.round(delayMs / 1000)}s.`, "info", delayMs + 600);
    } else {
        pickButton.innerHTML = '<span class="icon">&#58633;</span> PICK';
    }

    try {
        const position = await invoke<{ x: number; y: number }>("pick_cursor_position", {
            delayMs,
            delay_ms: delayMs,
        });

        const xInput = document.getElementById(xInputId) as HTMLInputElement | null;
        const yInput = document.getElementById(yInputId) as HTMLInputElement | null;
        if (xInput) {
            xInput.value = String(position.x);
            xInput.dispatchEvent(new Event("input", { bubbles: true }));
        }
        if (yInput) {
            yInput.value = String(position.y);
            yInput.dispatchEvent(new Event("input", { bubbles: true }));
        }

        notify(`Captured position ${position.x}, ${position.y}`, "success", 2200);
    } catch (error) {
        notify(error instanceof Error ? error.message : String(error), "error", 3200);
    } finally {
        if (countdownTimer !== null) {
            window.clearInterval(countdownTimer);
        }
        button.disabled = false;
        pickButton.innerHTML = originalText;
    }
}

export function fromBackendAction(item: MacroBackendAction): MacroUiAction | null {
    if (!item || typeof item !== "object") {
        return null;
    }

    const id = Number(item.id || 0);
    const config = item.config || {};
    const type = config.type;
    if (!id || !type) {
        return null;
    }

    if (type === "mouse") {
        const button = String(config.button || "left");
        const action = config.action || "press";
        const holdMs = getHoldMs(action);
        const position = typeof config.position === "string" ? config.position : null;

        let name = `${capitalize(button)} Click`;
        if (holdMs) {
            name = `${capitalize(button)} Hold`;
        } else if (action === "down") {
            name = `${capitalize(button)} Down`;
        } else if (action === "up") {
            name = `${capitalize(button)} Up`;
        }

        return {
            id,
            type,
            name,
            details: `${position ? `At ${position}` : "At current position"}${holdMs ? ` for ${holdMs}ms` : ""}`,
            icon: "&#58633;",
        };
    }

    if (type === "move") {
        const x = Number(config.x || 0);
        const y = Number(config.y || 0);
        const style = String(config.style || "instant");
        return {
            id,
            type,
            name: "Move Cursor",
            details: `To ${x}, ${y} (${style})`,
            icon: "&#57987;",
        };
    }

    if (type === "keyboard") {
        const text = typeof config.text === "string" && config.text.length > 0 ? config.text : "";
        const key = String(text || config.key || "a").toUpperCase();
        const action = config.action || "press";
        const holdMs = getHoldMs(action);
        const detailKeys = [...parseModifierList(config.modifiers), key].filter(Boolean);
        const combo = detailKeys.join(" + ");

        let name = `Key Press`;
        if (holdMs) {
            name = `Key Hold`;
        } else if (action === "down") {
            name = `Key Down`;
        } else if (action === "up") {
            name = `Key Up`;
        }

        return {
            id,
            type,
            name,
            details: `${combo}${holdMs ? ` for ${holdMs}ms` : ""}`,
            detailKeys,
            detailSuffix: holdMs ? `for ${holdMs}ms` : "",
            icon: "&#57988;",
        };
    }

    if (type === "sleep") {
        const durationMs = Number(config.duration_ms || 0);
        return {
            id,
            type,
            name: "Sleep",
            details: `Wait for ${durationMs}ms`,
            icon: "&#57824;",
        };
    }

    if (type === "scroll") {
        const clicks = Number(config.clicks || 0);
        const direction = clicks < 0 ? "Down" : "Up";
        return {
            id,
            type,
            name: "Mouse Scroll",
            details: `Scroll ${direction} (${Math.abs(clicks)} step${Math.abs(clicks) !== 1 ? "s" : ""})`,
            icon: "&#59030;",
        };
    }

    return null;
}

export function toBackendConfig(type: MacroActionType, draft: MacroActionDraft): Record<string, unknown> | null {
    if (type === "mouse") {
        const mouseDraft = draft as MacroMouseDraft;
        return {
            type: "mouse",
            button: String(mouseDraft.button || "left").toLowerCase(),
            action: mouseDraft.action === "hold"
                ? { hold: { duration_ms: Math.max(1, Number(mouseDraft.durationMs || 100)) } }
                : (mouseDraft.action === "down" || mouseDraft.action === "up" ? mouseDraft.action : "press"),
            position: mouseDraft.positionMode === "current" ? null : [Number(mouseDraft.x || 0), Number(mouseDraft.y || 0)],
        };
    }

    if (type === "move") {
        const moveDraft = draft as MacroMoveDraft;
        return {
            type: "move",
            x: Number(moveDraft.x || 0),
            y: Number(moveDraft.y || 0),
            style: moveDraft.style === "linear"
                ? { linear: { duration_ms: 300 } }
                : (moveDraft.style === "smooth"
                    ? { smooth: { path: [], duration_ms: 300 } }
                    : "instant"),
        };
    }

    if (type === "keyboard") {
        const keyboardDraft = draft as MacroKeyboardDraft;
        const parsed = parseKeyboardCombo(String(keyboardDraft.key || "A"));
        return {
            type: "keyboard",
            key: parsed.key,
            modifiers: parsed.modifiers,
            action: keyboardDraft.action === "hold"
                ? { hold: { duration_ms: Math.max(1, Number(keyboardDraft.durationMs || 50)) } }
                : (keyboardDraft.action === "down" || keyboardDraft.action === "up" ? keyboardDraft.action : "press"),
        };
    }

    if (type === "sleep") {
        const sleepDraft = draft as MacroSleepDraft;
        return {
            type: "sleep",
            duration_ms: Math.max(1, Number(sleepDraft.durationMs || 500)),
        };
    }

    if (type === "scroll") {
        const scrollDraft = draft as MacroScrollDraft;
        return {
            type: "scroll",
            clicks: Number(scrollDraft.clicks || 0),
        };
    }

    return null;
}
