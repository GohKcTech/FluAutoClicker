import type { MacroCapabilities, MacroRecordingOptions, MacroUiAction } from "./types";

type MacroState = {
    actions: MacroUiAction[];
    currentPlayingActionId: number | null;
    capabilities: MacroCapabilities;
    recordingOptions: MacroRecordingOptions;
};

const defaultState: MacroState = {
    actions: [],
    currentPlayingActionId: null,
    capabilities: {
        supported_mouse_buttons: ["left", "middle", "right"],
        recording_supported: true,
        recording_reason: null,
        pick_delay_ms: 5000,
    },
    recordingOptions: {
        record_mouse_clicks: true,
        record_mouse_moves: true,
        record_keyboard: true,
        record_delays: true,
        record_click_position: true,
    },
};

export const macroState: MacroState = defaultState;
