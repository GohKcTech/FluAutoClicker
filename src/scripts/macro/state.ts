import type { MacroCapabilities, MacroRecordingOptions, MacroUiAction, MacroBackendAction } from "./types";

type MacroState = {
    actions: MacroUiAction[];
    rawActions: MacroBackendAction[];
    currentPlayingActionId: number | null;
    capabilities: MacroCapabilities;
    recordingOptions: MacroRecordingOptions;
};

const defaultState: MacroState = {
    actions: [],
    rawActions: [],
    currentPlayingActionId: null,
    capabilities: {
        supported_mouse_buttons: ["left", "middle", "right"],
        recording_supported: true,
        recording_reason: null,
        pick_delay_ms: 5000,
    },
    recordingOptions: {
        record_mouse_clicks: true,
        record_mouse_moves: "instant",
        record_keyboard: true,
        record_delays: true,
        record_click_position: true,
        record_live_preview: true,
    },
};

export const macroState: MacroState = defaultState;
