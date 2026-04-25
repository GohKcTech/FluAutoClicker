import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { syncAllKeyboardSettings } from "../keyboard";
import { syncAllMouseSettings } from "../mouse";
import { notify } from "../notifications";
import { updateTabStates } from "../ui";

type RunningPayload = {
    running?: boolean;
};

type MacroStatusPayload = {
    state?: string;
    error?: string;
};

type SupportedTab = "mouse" | "keyboard" | "macro";

function getStartButton() {
    return document.getElementById("start-btn");
}

function setStartButtonState(isRunning: boolean) {
    const startButton = getStartButton();
    const label = startButton?.querySelector(".start-label");
    if (!startButton || !label) {
        return;
    }

    if (isRunning) {
        startButton.classList.add("running");
        label.textContent = "STOP";
        startButton.style.backgroundColor = "var(--action-active-bg)";
        startButton.style.color = "var(--action-active-text)";
        return;
    }

    startButton.classList.remove("running");
    label.textContent = "START";
    startButton.style.backgroundColor = "";
    startButton.style.color = "";
}

function getActiveTab(): SupportedTab {
    const activeTab = document.querySelector<HTMLElement>(".mode-tabs .tab.active");
    const tab = activeTab?.dataset.tab;

    if (tab === "keyboard" || tab === "macro") {
        return tab;
    }

    return "mouse";
}

function getMouseIntervalMs(): number {
    const hours = Number.parseInt((document.getElementById("mouse-hours") as HTMLInputElement | null)?.value || "0", 10) || 0;
    const minutes = Number.parseInt((document.getElementById("mouse-minutes") as HTMLInputElement | null)?.value || "0", 10) || 0;
    const seconds = Number.parseInt((document.getElementById("mouse-seconds") as HTMLInputElement | null)?.value || "0", 10) || 0;
    const milliseconds = Number.parseInt((document.getElementById("mouse-ms") as HTMLInputElement | null)?.value || "0", 10) || 0;

    return hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds;
}

function showTimingWarningModal() {
    const modal = document.getElementById("timing-warning-modal");
    const handle = document.getElementById("timing-unlock-handle");
    if (!modal || !handle) {
        return;
    }

    modal.style.display = "flex";
    requestAnimationFrame(() => {
        modal.classList.add("active");
    });
    window.setTimeout(() => handle.focus(), 50);
    document.getElementById("content")?.classList.add("blurred");
}

async function toggleMouseClicker() {
    await invoke("toggle_clicker");
}

async function handleStartButtonClick() {
    const startButton = getStartButton();
    if (!startButton) {
        return;
    }

    const activeTab = getActiveTab();
    const isAlreadyRunning = startButton.classList.contains("running");

    if (activeTab === "keyboard") {
        if (!isAlreadyRunning) {
            await syncAllKeyboardSettings();
        }

        const isRunning = await invoke<boolean>("toggle_keyboard_clicker");
        setStartButtonState(Boolean(isRunning));
        updateTabStates();
        return;
    }

    if (activeTab === "macro") {
        try {
            const isRunning = await invoke<boolean>("toggle_macro_player");
            setStartButtonState(Boolean(isRunning));
        } catch (error) {
            console.error("Failed to toggle macro player", error);
            notify(error instanceof Error ? error.message : String(error), "error", 3200);
        }

        updateTabStates();
        return;
    }

    if (!isAlreadyRunning && getMouseIntervalMs() <= 3) {
        showTimingWarningModal();
        return;
    }

    await syncAllMouseSettings();
    await toggleMouseClicker();
    updateTabStates();
}

export function initStartStopControls() {
    const startButton = getStartButton();
    if (startButton) {
        startButton.addEventListener("click", () => {
            void handleStartButtonClick();
        });
    }

    void listen<RunningPayload>("status-changed", (event) => {
        setStartButtonState(Boolean(event.payload.running));
        updateTabStates();
    });

    void listen<RunningPayload>("keyboard-status-changed", (event) => {
        setStartButtonState(Boolean(event.payload.running));
        updateTabStates();
    });

    void listen<MacroStatusPayload>("macro-status-changed", (event) => {
        const state = String(event.payload?.state || "stopped");
        setStartButtonState(state === "playing");
        updateTabStates();

        if (state === "error" && event.payload?.error) {
            notify(String(event.payload.error), "error", 3600);
        }
    });

    return toggleMouseClicker;
}
