export type WindowControlIconStyle = "fluent" | "classic";

export const WINDOW_CONTROL_ICONS_STORAGE_KEY = "flu-window-control-icons";
export const WINDOW_CONTROL_ICONS_CHANGED_EVENT = "flu:window-control-icons-changed";

const FLUENT_GLYPHS = {
    minimize: String.fromCharCode(57627),
    maximize: String.fromCharCode(57618),
    unmaximize: String.fromCharCode(57626),
};

const CLASSIC_ICONS = {
    minimize: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="4" y1="12" x2="20" y2="12"></line></svg>',
    maximize: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><rect x="5" y="5" width="14" height="14" rx="1"></rect></svg>',
    close: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>',
};

function asWindowControlIconStyle(value: string | null): WindowControlIconStyle {
    return value === "classic" ? "classic" : "fluent";
}

export function getPreferredWindowControlIconStyle(): WindowControlIconStyle {
    return asWindowControlIconStyle(localStorage.getItem(WINDOW_CONTROL_ICONS_STORAGE_KEY));
}

export function setPreferredWindowControlIconStyle(style: WindowControlIconStyle) {
    localStorage.setItem(WINDOW_CONTROL_ICONS_STORAGE_KEY, style);
    window.dispatchEvent(new CustomEvent(WINDOW_CONTROL_ICONS_CHANGED_EVENT, { detail: style }));
}

function fluentGlyph(glyph: string) {
    return `<span class="window-control-glyph" aria-hidden="true">${glyph}</span>`;
}

function setButtonIcon(id: string, html: string, title: string) {
    const button = document.getElementById(id);
    if (!button) return;

    button.innerHTML = html;
    button.setAttribute("title", title);
    button.setAttribute("aria-label", title);
}

export function applyWindowControlIcons(options: { maximized?: boolean } = {}) {
    const style = getPreferredWindowControlIconStyle();
    document.documentElement.dataset.windowControlIcons = style;

    if (style === "classic") {
        setButtonIcon("minimize-btn", CLASSIC_ICONS.minimize, "Minimize");
        setButtonIcon("maximize-btn", CLASSIC_ICONS.maximize, "Maximize");
        setButtonIcon("close-btn", CLASSIC_ICONS.close, "Close");
        return;
    }

    const maximized = options.maximized === true;
    setButtonIcon("minimize-btn", fluentGlyph(FLUENT_GLYPHS.minimize), "Minimize");
    setButtonIcon(
        "maximize-btn",
        fluentGlyph(maximized ? FLUENT_GLYPHS.unmaximize : FLUENT_GLYPHS.maximize),
        maximized ? "Unmaximize" : "Maximize",
    );
    setButtonIcon("close-btn", CLASSIC_ICONS.close, "Close");
}
