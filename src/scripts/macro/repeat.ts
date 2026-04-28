import { invoke } from "@tauri-apps/api/core";
import { createSlideIndicator, updateIndicator } from "../utils";

export function applyRepeatModeToUi(repeatMode: string) {
    const repeatToggle = document.getElementById("macro-repeat-toggle");
    const finiteModeToggle = document.getElementById("macro-finite-mode-toggle");
    const countInput = document.getElementById("macro-repeat-count") as HTMLInputElement | null;

    if (!repeatToggle || !finiteModeToggle || !countInput) {
        return;
    }

    if (repeatMode === "infinite") {
        setActiveButton(repeatToggle, "infinite");
    } else if (repeatMode.startsWith("finite_times_")) {
        const count = Number.parseInt(repeatMode.replace("finite_times_", ""), 10);
        countInput.value = Number.isFinite(count) && count > 0 ? String(count) : "1";
        setActiveButton(repeatToggle, "finite");
        setActiveButton(finiteModeToggle, "times");
    } else if (repeatMode.startsWith("finite_seconds_")) {
        const durationMs = Number.parseInt(repeatMode.replace("finite_seconds_", ""), 10);
        const seconds = Math.max(1, Math.round((Number.isFinite(durationMs) ? durationMs : 1000) / 1000));
        countInput.value = String(seconds);
        setActiveButton(repeatToggle, "finite");
        setActiveButton(finiteModeToggle, "seconds");
    }

    applyMacroRepeatExpandableState();
    refreshMacroRepeatIndicators();
}

export function initRepeatSettingsListeners() {
    const repeatToggle = document.getElementById("macro-repeat-toggle");
    const finiteModeToggle = document.getElementById("macro-finite-mode-toggle");
    const countInput = document.getElementById("macro-repeat-count") as HTMLInputElement | null;

    const syncRepeatSettings = async () => {
        const mode = repeatToggle?.querySelector(".toggle-option.active")?.getAttribute("data-value") || "infinite";
        const finiteMode = finiteModeToggle?.querySelector(".toggle-option.active")?.getAttribute("data-value") || "times";
        const count = Math.max(1, Number.parseInt(countInput?.value || "1", 10) || 1);

        if (mode === "infinite") {
            await invoke("set_macro_repeat_mode", { mode: "infinite" });
            return;
        }

        if (finiteMode === "seconds") {
            await invoke("set_macro_repeat_mode", { mode: "finite_seconds" });
            const durationMs = count * 1000;
            await invoke("set_macro_repeat_duration", { durationMs, duration_ms: durationMs });
            return;
        }

        await invoke("set_macro_repeat_mode", { mode: "finite_times" });
        await invoke("set_macro_repeat_count", { count });
    };

    repeatToggle?.querySelectorAll(".toggle-option").forEach((button) => {
        button.addEventListener("click", () => {
            setActiveButton(repeatToggle, (button as HTMLElement).dataset.value || "infinite");
            applyMacroRepeatExpandableState();
            refreshMacroRepeatIndicators();
            void syncRepeatSettings().catch((error) => {
                console.error("Failed to sync macro repeat mode", error);
            });
        });
    });

    finiteModeToggle?.querySelectorAll(".toggle-option").forEach((button) => {
        button.addEventListener("click", () => {
            setActiveButton(finiteModeToggle, (button as HTMLElement).dataset.value || "times");
            refreshMacroRepeatIndicators();
            void syncRepeatSettings().catch((error) => {
                console.error("Failed to sync macro finite mode", error);
            });
        });
    });

    countInput?.addEventListener("input", () => {
        void syncRepeatSettings().catch((error) => {
            console.error("Failed to sync macro repeat count", error);
        });
    });
}

function setActiveButton(container: HTMLElement, value: string) {
    let activeButton: HTMLElement | null = null;

    container.querySelectorAll<HTMLElement>(".toggle-option, .multi-btn").forEach((button) => {
        const isActive = button.dataset.value === value;
        button.classList.toggle("active", isActive);
        if (isActive) {
            activeButton = button;
        }
    });

    if (activeButton) {
        updateToggleIndicator(container, activeButton);
    }
}

function applyMacroRepeatExpandableState() {
    const repeatToggle = document.getElementById("macro-repeat-toggle");
    const activeValue = repeatToggle
        ?.querySelector<HTMLElement>(".toggle-option.active")
        ?.dataset.value;

    document.getElementById("macro-repeat-finite-section")?.classList.toggle(
        "expanded",
        activeValue === "finite"
    );
}

function refreshMacroRepeatIndicators() {
    refreshToggleIndicator("macro-repeat-toggle");
    refreshToggleIndicator("macro-finite-mode-toggle");

    requestAnimationFrame(() => {
        refreshToggleIndicator("macro-repeat-toggle");
        refreshToggleIndicator("macro-finite-mode-toggle");
    });
}

function refreshToggleIndicator(containerId: string) {
    const container = document.getElementById(containerId);
    const activeButton = container?.querySelector<HTMLElement>(".toggle-option.active, .multi-btn.active");
    if (container && activeButton) {
        updateToggleIndicator(container, activeButton);
    }
}

function updateToggleIndicator(container: HTMLElement, activeButton: HTMLElement) {
    if (container.querySelector(".slide-indicator")) {
        updateIndicator(container, activeButton);
        return;
    }

    createSlideIndicator(container, activeButton, true);
}
