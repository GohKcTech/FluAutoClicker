export type MacroActionType = "mouse" | "move" | "keyboard" | "sleep";

export type MacroUiAction = {
    id: number;
    type: MacroActionType;
    name: string;
    details: string;
    icon: string;
};

export type MacroCapabilities = {
    supported_mouse_buttons: string[];
    recording_supported: boolean;
    recording_reason?: string | null;
    pick_delay_ms: number;
};

export type MacroRecordingOptions = {
    record_mouse_clicks: boolean;
    record_mouse_moves: boolean;
    record_keyboard: boolean;
    record_delays: boolean;
    record_click_position: boolean;
};

export type MacroRepeatState = {
    actions: number;
    repeat_mode: string;
};

export type MacroBackendAction = {
    id?: unknown;
    config?: {
        type?: MacroActionType;
        button?: string;
        action?: string;
        position?: string | null;
        x?: number;
        y?: number;
        style?: string;
        key?: string;
        text?: string;
        modifiers?: string;
        duration_ms?: number;
    };
};

export type MacroMouseDraft = {
    button: string;
    action: "press" | "hold";
    positionMode: "current" | "custom";
    x: string;
    y: string;
    durationMs: number;
};

export type MacroMoveDraft = {
    x: string;
    y: string;
    style: "instant" | "smooth";
};

export type MacroKeyboardDraft = {
    action: "press" | "hold";
    key: string;
    durationMs: number;
};

export type MacroSleepDraft = {
    durationMs: string;
};

export type MacroActionDraft = MacroMouseDraft | MacroMoveDraft | MacroKeyboardDraft | MacroSleepDraft;
