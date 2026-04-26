const BLOCKED_CTRL_KEYS = new Set([
    "f",
    "g",
    "j",
    "l",
    "n",
    "o",
    "p",
    "r",
    "s",
    "u",
    "w",
]);

function hasModifier(event: KeyboardEvent) {
    return event.ctrlKey || event.metaKey;
}

function shouldBlockBrowserShortcut(event: KeyboardEvent) {
    if (event.key === "F5") return true;
    if (event.key === "F3") return true;
    if (event.key === "F7") return true;
    if (event.altKey && event.key === "ArrowLeft") return true;
    if (event.altKey && event.key === "ArrowRight") return true;
    if (!hasModifier(event)) return false;

    const key = event.key.toLowerCase();
    if (key === "f5") return true;
    if (key === "+" || key === "-" || key === "=" || key === "0") return true;
    if (event.shiftKey && (key === "i" || key === "j" || key === "c")) return true;
    return BLOCKED_CTRL_KEYS.has(key);
}

export function initNativeShellGuards() {
    window.addEventListener(
        "keydown",
        (event) => {
            if (!shouldBlockBrowserShortcut(event)) return;
            event.preventDefault();
        },
        true
    );

    window.addEventListener(
        "contextmenu",
        (event) => {
            event.preventDefault();
        },
        true
    );

    window.addEventListener("dragover", (event) => event.preventDefault(), true);
    window.addEventListener("drop", (event) => event.preventDefault(), true);
}
