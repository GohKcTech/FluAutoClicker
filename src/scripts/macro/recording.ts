import { invoke } from "@tauri-apps/api/core";
import { createSlideIndicator, updateIndicator } from "../utils";
import { notify } from "../notifications";
import { macroState } from "./state";
import type { MacroRecordingOptions } from "./types";

const recordingOptionRows: Array<[keyof MacroRecordingOptions, string]> = [
    ["record_mouse_clicks", "macro-record-mouse-clicks-toggle"],
    ["record_mouse_moves", "macro-record-mouse-moves-toggle"],
    ["record_keyboard", "macro-record-keyboard-toggle"],
    ["record_delays", "macro-record-delays-toggle"],
    ["record_click_position", "macro-record-click-position-toggle"],
];

export function setMacroRecordingState(recordButton: HTMLElement | null, isRecording: boolean) {
    if (!recordButton) {
        return;
    }

    recordButton.classList.toggle("active", isRecording);
    const label = recordButton.querySelector("span:not(.icon)");
    if (label) {
        label.textContent = isRecording ? "STOP" : "RECORD";
    }
}

export function syncRecordingOptionsToUi() {
    recordingOptionRows.forEach(([key, id]) => {
        const row = document.getElementById(id);
        if (!row) {
            return;
        }

        const value = macroState.recordingOptions[key] ? "on" : "off";
        const target = row.querySelector<HTMLElement>(`.toggle-option[data-value="${value}"]`);
        row.querySelectorAll(".toggle-option").forEach((button) => {
            button.classList.toggle("active", button.getAttribute("data-value") === value);
        });

        if (target) {
            updateIndicator(row, target);
        }
    });
}

async function persistRecordingOptions() {
    try {
        const optionsJson = JSON.stringify(macroState.recordingOptions);
        macroState.recordingOptions = await invoke<MacroRecordingOptions>("set_macro_recording_options", {
            optionsJson,
            options_json: optionsJson,
        });
        syncRecordingOptionsToUi();
        notify("Recording filters updated", "success", 1600);
    } catch (error) {
        notify(error instanceof Error ? error.message : String(error), "error", 3200);
    }
}

export function applyRecordAvailability(recordButton: HTMLElement | null) {
    if (!recordButton) {
        return;
    }

    const button = recordButton as HTMLButtonElement;
    button.disabled = !macroState.capabilities.recording_supported;
    button.title = macroState.capabilities.recording_supported
        ? "Record live mouse and keyboard actions"
        : macroState.capabilities.recording_reason || "Live macro recording is unavailable";
    button.style.opacity = macroState.capabilities.recording_supported ? "" : "0.55";
    button.style.cursor = macroState.capabilities.recording_supported ? "" : "not-allowed";
}

export function initRecordingSettingsUi() {
    recordingOptionRows.forEach(([key, id]) => {
        const row = document.getElementById(id);
        if (!row) {
            return;
        }

        const activeButton = row.querySelector<HTMLElement>(".toggle-option.active");
        if (activeButton) {
            createSlideIndicator(row, activeButton, true);
        }

        row.querySelectorAll<HTMLElement>(".toggle-option").forEach((button) => {
            button.addEventListener("click", () => {
                const nextValue = button.dataset.value === "on";
                if (macroState.recordingOptions[key] === nextValue) {
                    return;
                }

                macroState.recordingOptions = {
                    ...macroState.recordingOptions,
                    [key]: nextValue,
                };
                syncRecordingOptionsToUi();
                void persistRecordingOptions();
            });
        });
    });
}
