import { updateIndicator } from "../utils";
import { captureCursorPosition, normalizeKey } from "./backend";
import { setKeyBadgeContent } from "../key-badges";
import { macroState } from "./state";
import type {
    MacroActionDraft,
    MacroActionType,
    MacroKeyboardDraft,
    MacroMouseDraft,
} from "./types";

const MODIFIER_LABELS = ["ctrl", "alt", "win", "shift"];

function isModifierLabel(label: string): boolean {
    return MODIFIER_LABELS.includes(label);
}

function normalizeKeyboardLabel(label: string): string {
    const value = label.trim().toLowerCase();
    const map: Record<string, string> = {
        "←": "backspace",
        esc: "escape",
        caps: "capslock",
        pgup: "pageup",
        pgdn: "pagedown",
        del: "delete",
        ins: "insert",
    };

    return map[value] || normalizeKey(value);
}

function formatKeyboardCombo(modifiers: string[], mainKey = ""): string {
    const combo = [...modifiers];
    if (mainKey && !modifiers.includes(mainKey)) {
        combo.push(mainKey);
    }

    return combo.join("+") || "a";
}

export function applyMouseButtonSupport(container: HTMLElement) {
    const allowedButtons = new Set(macroState.capabilities.supported_mouse_buttons);
    const row = container.querySelector("#cfg-mouse-btn");
    if (!row) {
        return;
    }

    row.querySelectorAll<HTMLElement>(".multi-btn").forEach((button) => {
        const value = button.dataset.value || "";
        const supported = allowedButtons.has(value);
        const target = button as HTMLButtonElement;
        target.disabled = !supported;
        target.title = supported ? "" : `Mouse button "${value}" is not supported on this system`;
        target.style.opacity = supported ? "" : "0.4";
        target.style.cursor = supported ? "" : "not-allowed";
    });

    const activeButton = row.querySelector<HTMLElement>(".multi-btn.active");
    if (activeButton && (activeButton as HTMLButtonElement).disabled) {
        activeButton.classList.remove("active");
        const fallback = row.querySelector<HTMLElement>('.multi-btn[data-value="left"]');
        fallback?.classList.add("active");
        if (fallback) {
            updateIndicator(row, fallback);
        }
    }
}

function initNumericInputs(container: HTMLElement) {
    container.querySelectorAll("input[type='number']").forEach((input) => {
        const inputElement = input as HTMLInputElement;
        inputElement.addEventListener("focus", () => {
            if (inputElement.value === "0") {
                inputElement.value = "";
            } else {
                inputElement.select();
            }
        });

        inputElement.addEventListener("blur", () => {
            if (inputElement.value !== "") {
                return;
            }

            inputElement.value = "0";
            inputElement.dispatchEvent(new Event("input"));
        });

        inputElement.addEventListener(
            "wheel",
            (event: WheelEvent) => {
                event.preventDefault();
                const step = Number.parseFloat(inputElement.getAttribute("step") || "1");
                const value = Number.parseFloat(inputElement.value) || 0;
                inputElement.value = (event.deltaY < 0 ? value + step : Math.max(0, value - step)).toString();
                inputElement.dispatchEvent(new Event("input"));
            },
            { passive: false }
        );
    });
}

function initToggleRows(container: HTMLElement) {
    container.querySelectorAll(".toggle-row, .multi-button-row").forEach((row) => {
        row.addEventListener("click", (event) => {
            const button = (event.target as HTMLElement).closest(".toggle-option, .multi-btn") as HTMLElement | null;
            if (!button) {
                return;
            }

            row.querySelectorAll(".toggle-option, .multi-btn").forEach((entry) => entry.classList.remove("active"));
            button.classList.add("active");
            updateIndicator(row as HTMLElement, button);

            const value = button.dataset.value;
            if (row.id === "cfg-mouse-action" || row.id === "cfg-kb-action") {
                row.parentElement?.querySelector(".expandable-content")?.classList.toggle("expanded", value === "hold");
            }

            if (row.id === "cfg-mouse-pos-toggle") {
                document.getElementById("cfg-mouse-coord-section")?.classList.toggle("expanded", value === "custom");
            }
        });
    });
}

function initKeyboardConfigUi() {
    const recordButton = document.getElementById("cfg-kb-record-btn");
    const keyInput = document.getElementById("cfg-kb-key") as HTMLInputElement | null;
    const modifiersDisplay = document.getElementById("cfg-kb-modifiers-display");
    const mainKeyDisplay = document.getElementById("cfg-kb-main-key-display");
    const comboDisplay = document.querySelector("#macro-config-content .kb-selected-combo");
    const slidingContainer = document.getElementById("cfg-kb-sliding-container");
    const numpadBack = document.getElementById("cfg-kb-numpad-back");
    const keyboardContainer = slidingContainer?.parentElement?.parentElement?.parentElement as HTMLElement | null;

    let activeModifiers: string[] = [];
    let activeMainKey = "";
    let activeMainElement: HTMLElement | null = null;
    let isLocalRecording = false;

    keyboardContainer?.querySelectorAll<HTMLElement>(".kb-tsu").forEach((key) => {
        key.textContent = ":)";
    });

    const updateKeyDisplay = (mainKey = "") => {
        if (mainKey) {
            activeMainKey = mainKey;
        }

        const result = formatKeyboardCombo(activeModifiers, activeMainKey);

        if (keyInput) {
            keyInput.value = result;
        }
        if (comboDisplay || (modifiersDisplay && mainKeyDisplay)) {
            setKeyBadgeContent(
                comboDisplay || modifiersDisplay!.parentElement || modifiersDisplay!,
                [...activeModifiers, activeMainKey || ""].filter(Boolean)
            );
        }
    };

    numpadBack?.addEventListener("click", () => {
        slidingContainer?.classList.remove("numpad-active");
    });

    recordButton?.addEventListener("click", () => {
        isLocalRecording = !isLocalRecording;
        recordButton.textContent = isLocalRecording ? "Recording..." : "Click to Record";
        recordButton.classList.toggle("active", isLocalRecording);

        if (!isLocalRecording) {
            return;
        }

        const handleKey = (event: KeyboardEvent) => {
            event.preventDefault();
            activeModifiers = [];
            if (event.ctrlKey) {
                activeModifiers.push("ctrl");
            }
            if (event.shiftKey) {
                activeModifiers.push("shift");
            }
            if (event.altKey) {
                activeModifiers.push("alt");
            }
            if (event.metaKey) {
                activeModifiers.push("win");
            }

            const keyMap: Record<string, string> = {
                " ": "space",
                enter: "enter",
                tab: "tab",
                backspace: "backspace",
                escape: "escape",
                capslock: "capslock",
                arrowup: "up",
                arrowdown: "down",
                arrowleft: "left",
                arrowright: "right",
                pageup: "pageup",
                pagedown: "pagedown",
                delete: "delete",
                insert: "insert",
                home: "home",
                end: "end",
            };
            const key = keyMap[event.key.toLowerCase()] || normalizeKeyboardLabel(event.key);
            const isModifier = ["control", "shift", "alt", "meta"].includes(key);
            updateKeyDisplay(isModifier ? "" : key);

            isLocalRecording = false;
            recordButton.textContent = "Click to Record";
            recordButton.classList.remove("active");
            window.removeEventListener("keydown", handleKey);
        };

        window.addEventListener("keydown", handleKey);
    });

    keyboardContainer?.querySelectorAll(".kb-key").forEach((button) => {
        button.addEventListener("click", () => {
            const key = button as HTMLElement;
            const label = key.textContent?.trim().toLowerCase() || "";
            const value = normalizeKeyboardLabel(label);

            if (key.classList.contains("kb-tsu")) {
                return;
            }

            if (key.classList.contains("kb-menu")) {
                slidingContainer?.classList.toggle("numpad-active");
                return;
            }

            if (key.id === "cfg-kb-numpad-back") {
                return;
            }

            if (isModifierLabel(label)) {
                if (activeModifiers.includes(label)) {
                    activeModifiers = activeModifiers.filter((entry) => entry !== label);
                    key.classList.remove("active");
                } else {
                    activeModifiers.push(label);
                    key.classList.add("active");
                }

                updateKeyDisplay();
                return;
            }

            if (activeMainElement && activeMainElement === key) {
                activeMainElement.classList.remove("active");
                activeMainElement = null;
                activeMainKey = "";
            } else {
                activeMainElement?.classList.remove("active");
                activeMainElement = key;
                key.classList.add("active");
                activeMainKey = value;
            }
            updateKeyDisplay();
        });
    });
}

function initSleepConfigUi() {
    const sleepInput = document.getElementById("cfg-sleep-ms") as HTMLInputElement | null;
    const sleepHint = document.getElementById("cfg-sleep-hint");

    sleepInput?.addEventListener("input", () => {
        if (!sleepHint) {
            return;
        }

        const seconds = (Number.parseInt(sleepInput.value, 10) || 0) / 1000;
        sleepHint.textContent = `${seconds.toFixed(3)} seconds pause`;
    });
}

export function setupConfigListeners(type: MacroActionType, container: HTMLElement) {
    initToggleRows(container);
    initNumericInputs(container);

    if (type === "mouse") {
        document.getElementById("cfg-mouse-pick-btn")?.addEventListener("click", async (event) => {
            await captureCursorPosition(event.currentTarget as HTMLElement, "cfg-mouse-x", "cfg-mouse-y");
        });
        return;
    }

    if (type === "move") {
        document.getElementById("cfg-move-pick-btn")?.addEventListener("click", async (event) => {
            await captureCursorPosition(event.currentTarget as HTMLElement, "cfg-move-x", "cfg-move-y");
        });
        return;
    }

    if (type === "keyboard") {
        initKeyboardConfigUi();
        return;
    }

    if (type === "sleep") {
        initSleepConfigUi();
    }
}

export function generateConfigUi(type: MacroActionType): string {
    switch (type) {
        case "mouse":
            return `
                <div class="section-row">
                    <div class="multi-button-row" id="cfg-mouse-btn" style="margin: 0;">
                        <div class="slide-indicator" style="width: 33.333%; left: 0%;"></div>
                        <button class="multi-btn active" data-value="left">Left</button>
                        <button class="multi-btn" data-value="middle">Middle</button>
                        <button class="multi-btn" data-value="right">Right</button>
                    </div>
                </div>

                <div class="section-row">
                    <div class="toggle-row" id="cfg-mouse-action" style="margin: 0;">
                        <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                        <button class="toggle-option active" data-value="press">Press</button>
                        <button class="toggle-option" data-value="hold">Hold</button>
                    </div>

                    <div class="expandable-content" id="cfg-mouse-duration-container">
                        <div class="expandable-inner">
                            <div class="finite-inputs-row">
                                <div class="coord-input-box" style="flex: 2;">
                                    <input type="number" value="100" id="cfg-mouse-duration" min="1">
                                </div>
                                <div class="toggle-row" id="cfg-mouse-hold-mode-toggle" style="flex: 3; margin: 0;">
                                    <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                                    <button class="toggle-option active" data-value="ms">ms</button>
                                    <button class="toggle-option" data-value="s">sec</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="section-row">
                    <div class="toggle-row" id="cfg-mouse-pos-toggle" style="margin: 0;">
                        <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                        <button class="toggle-option active" data-value="current">Current position</button>
                        <button class="toggle-option" data-value="custom">Choose a place</button>
                    </div>

                    <div class="expandable-content" id="cfg-mouse-coord-section">
                        <div class="expandable-inner">
                            <div class="coord-row-inner">
                                <div class="coord-input-box">
                                    <input type="number" value="0" id="cfg-mouse-x" min="0">
                                </div>
                                <div class="coord-input-box">
                                    <input type="number" value="0" id="cfg-mouse-y" min="0">
                                </div>
                                <button class="pick-btn" id="cfg-mouse-pick-btn">
                                    <span class="icon" style="margin-right: 6px; font-size: 14px;">&#58633;</span>
                                    PICK
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        case "move":
            return `
                <div class="section-row">
                    <div class="coord-row-inner">
                        <div class="coord-input-box">
                            <input type="number" value="0" id="cfg-move-x" min="0">
                        </div>
                        <div class="coord-input-box">
                            <input type="number" value="0" id="cfg-move-y" min="0">
                        </div>
                        <button class="pick-btn" id="cfg-move-pick-btn">
                            <span class="icon" style="margin-right: 6px; font-size: 14px;">&#58633;</span>
                            PICK
                        </button>
                    </div>
                </div>

                <div class="section-row">
                    <div class="toggle-row" id="cfg-move-style" style="margin: 0;">
                        <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                        <button class="toggle-option active" data-value="instant">Instant</button>
                        <button class="toggle-option" data-value="smooth">Smooth (WIP)</button>
                    </div>
                </div>
            `;
        case "keyboard":
            return `
                <div class="section-row">
                    <div class="toggle-row" id="cfg-kb-action" style="margin: 0;">
                        <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                        <button class="toggle-option active" data-value="press">Press</button>
                        <button class="toggle-option" data-value="hold">Hold</button>
                    </div>

                    <div class="expandable-content" id="cfg-kb-duration-container">
                        <div class="expandable-inner">
                            <div class="finite-inputs-row">
                                <div class="coord-input-box" style="flex: 2;">
                                    <input type="number" value="100" id="cfg-kb-duration" min="1">
                                </div>
                                <div class="toggle-row" id="cfg-kb-hold-mode-toggle" style="flex: 3; margin: 0;">
                                    <div class="slide-indicator" style="width: 50%; left: 0%;"></div>
                                    <button class="toggle-option active" data-value="ms">ms</button>
                                    <button class="toggle-option" data-value="s">sec</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="section-row keyboard-section-row">
                    <div class="kb-viewport">
                        <div class="kb-container" id="cfg-kb-sliding-container">
                            <div class="kb-main-view">
                                <div class="virtual-keyboard">
                                    <div class="kb-row">
                                        <button class="kb-key kb-esc">ESC</button><button class="kb-key kb-fn">F1</button><button class="kb-key kb-fn">F2</button><button class="kb-key kb-fn">F3</button><button class="kb-key kb-fn">F4</button><button class="kb-key kb-fn">F5</button><button class="kb-key kb-fn">F6</button><button class="kb-key kb-fn">F7</button><button class="kb-key kb-fn">F8</button><button class="kb-key kb-fn">F9</button><button class="kb-key kb-fn">F10</button><button class="kb-key kb-fn">F11</button><button class="kb-key kb-fn">F12</button>
                                    </div>
                                    <div class="kb-row">
                                        <button class="kb-key">\`</button><button class="kb-key">1</button><button class="kb-key">2</button><button class="kb-key">3</button><button class="kb-key">4</button><button class="kb-key">5</button><button class="kb-key">6</button><button class="kb-key">7</button><button class="kb-key">8</button><button class="kb-key">9</button><button class="kb-key">0</button><button class="kb-key">-</button><button class="kb-key">=</button><button class="kb-key kb-backspace">←</button>
                                    </div>
                                    <div class="kb-row">
                                        <button class="kb-key kb-tab">Tab</button><button class="kb-key">Q</button><button class="kb-key">W</button><button class="kb-key">E</button><button class="kb-key">R</button><button class="kb-key">T</button><button class="kb-key">Y</button><button class="kb-key">U</button><button class="kb-key">I</button><button class="kb-key">O</button><button class="kb-key">P</button><button class="kb-key">[</button><button class="kb-key">]</button><button class="kb-key kb-bslash">\\</button>
                                    </div>
                                    <div class="kb-row">
                                        <button class="kb-key kb-caps">Caps</button><button class="kb-key">A</button><button class="kb-key">S</button><button class="kb-key">D</button><button class="kb-key">F</button><button class="kb-key">G</button><button class="kb-key">H</button><button class="kb-key">J</button><button class="kb-key">K</button><button class="kb-key">L</button><button class="kb-key">;</button><button class="kb-key">'</button><button class="kb-key kb-enter">Enter</button>
                                    </div>
                                    <div class="kb-row">
                                        <button class="kb-key kb-shift">Shift</button><button class="kb-key">Z</button><button class="kb-key">X</button><button class="kb-key">C</button><button class="kb-key">V</button><button class="kb-key">B</button><button class="kb-key">N</button><button class="kb-key">M</button><button class="kb-key">,</button><button class="kb-key">.</button><button class="kb-key">/</button><button class="kb-key kb-shift">Shift</button>
                                    </div>
                                    <div class="kb-row">
                                        <button class="kb-key">Ctrl</button><button class="kb-key kb-mod">Win</button><button class="kb-key">Alt</button><button class="kb-key kb-space">Space</button><button class="kb-key">Alt</button><button class="kb-key kb-tsu" style="cursor: default; opacity: 0.5;">TSU</button><button class="kb-key">Ctrl</button><button class="kb-key kb-menu">≡</button>
                                    </div>
                                </div>
                            </div>
                            <div class="kb-numpad-view">
                                <div class="kb-numpad-content">
                                    <div class="kb-row"><button class="kb-key">Home</button><button class="kb-key">PgUp</button><button class="kb-key">PgDn</button></div>
                                    <div class="kb-row"><button class="kb-key">7</button><button class="kb-key">8</button><button class="kb-key">9</button><button class="kb-key">Num</button></div>
                                    <div class="kb-row"><button class="kb-key">4</button><button class="kb-key">5</button><button class="kb-key">6</button><button class="kb-key">Ins</button></div>
                                    <div class="kb-row"><button class="kb-key">1</button><button class="kb-key">2</button><button class="kb-key">3</button><button class="kb-key">Del</button></div>
                                    <div class="kb-row"><button class="kb-key">0</button><button class="kb-key">.</button><button class="kb-key">/</button><button class="kb-key">End</button></div>
                                    <div class="kb-row"><button class="kb-key kb-back-btn" id="cfg-kb-numpad-back">BACK</button><button class="kb-key">*</button><button class="kb-key">-</button></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="section-row">
                    <div class="kb-status-row">
                        <span class="kb-selected-combo">
                            <span class="kb-modifiers" id="cfg-kb-modifiers-display"></span><span class="kb-main-key" id="cfg-kb-main-key-display">...</span>
                        </span>
                        <button class="kb-record-btn" id="cfg-kb-record-btn">Click to Record</button>
                        <input type="hidden" value="a" id="cfg-kb-key">
                    </div>
                </div>
            `;
        case "sleep":
            return `
                <div class="section-row">
                    <div class="time-inputs-row">
                        <div class="time-input-box" style="flex: 1;">
                            <input type="number" value="500" id="cfg-sleep-ms" min="1">
                        </div>
                    </div>
                    <div class="time-labels-row">
                        <span class="time-label" id="cfg-sleep-hint">0.500 seconds pause</span>
                    </div>
                </div>
            `;
    }
}

export function gatherConfig(type: MacroActionType): MacroActionDraft {
    if (type === "mouse") {
        const durationValue = Number.parseInt((document.getElementById("cfg-mouse-duration") as HTMLInputElement | null)?.value || "100", 10);
        const durationUnit =
            document
                .getElementById("cfg-mouse-hold-mode-toggle")
                ?.querySelector(".active")
                ?.getAttribute("data-value") || "ms";

        const draft: MacroMouseDraft = {
            button:
                document.getElementById("cfg-mouse-btn")?.querySelector(".active")?.getAttribute("data-value") || "left",
            action:
                (document.getElementById("cfg-mouse-action")?.querySelector(".active")?.getAttribute("data-value") as "press" | "hold") || "press",
            positionMode:
                (document.getElementById("cfg-mouse-pos-toggle")?.querySelector(".active")?.getAttribute("data-value") as "current" | "custom") || "current",
            x: (document.getElementById("cfg-mouse-x") as HTMLInputElement | null)?.value || "0",
            y: (document.getElementById("cfg-mouse-y") as HTMLInputElement | null)?.value || "0",
            durationMs: durationUnit === "s" ? durationValue * 1000 : durationValue,
        };

        return draft;
    }

    if (type === "move") {
        return {
            x: (document.getElementById("cfg-move-x") as HTMLInputElement | null)?.value || "0",
            y: (document.getElementById("cfg-move-y") as HTMLInputElement | null)?.value || "0",
            style:
                (document.getElementById("cfg-move-style")?.querySelector(".active")?.getAttribute("data-value") as "instant" | "smooth") || "instant",
        };
    }

    if (type === "keyboard") {
        const durationValue = Number.parseInt((document.getElementById("cfg-kb-duration") as HTMLInputElement | null)?.value || "50", 10);
        const durationUnit =
            document
                .getElementById("cfg-kb-hold-mode-toggle")
                ?.querySelector(".active")
                ?.getAttribute("data-value") || "ms";

        const draft: MacroKeyboardDraft = {
            action:
                (document.getElementById("cfg-kb-action")?.querySelector(".active")?.getAttribute("data-value") as "press" | "hold") || "press",
            key: (document.getElementById("cfg-kb-key") as HTMLInputElement | null)?.value || "A",
            durationMs: durationUnit === "s" ? durationValue * 1000 : durationValue,
        };

        return draft;
    }

    return {
        durationMs: (document.getElementById("cfg-sleep-ms") as HTMLInputElement | null)?.value || "500",
    };
}
