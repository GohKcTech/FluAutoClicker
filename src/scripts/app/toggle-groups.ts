import { invoke } from "@tauri-apps/api/core";
import { createSlideIndicator, updateIndicator } from "../utils";
import { initTabs } from "../ui";

type ExpandableConfig = {
    toggleId: string;
    sectionId: string;
    activeValue: string;
};

type ExtremeSelection = {
    button: HTMLElement | null;
    row: HTMLElement | null;
};

const expandableConfigs: ExpandableConfig[] = [
    { toggleId: "press-hold-toggle", sectionId: "mouse-hold-duration-section", activeValue: "hold" },
    { toggleId: "kb-mode-toggle", sectionId: "kb-hold-duration-section", activeValue: "hold" },
    { toggleId: "repeat-toggle", sectionId: "mouse-repeat-finite-section", activeValue: "finite" },
    { toggleId: "kb-repeat-toggle", sectionId: "kb-repeat-finite-section", activeValue: "finite" },
    { toggleId: "position-toggle", sectionId: "mouse-coord-section", activeValue: "custom" },
    { toggleId: "macro-repeat-toggle", sectionId: "macro-repeat-finite-section", activeValue: "finite" },
];

function isMultithreadSupported() {
    return document.documentElement.dataset.multithreadSupported !== "0";
}

function updateAllIndicators() {
    document.querySelectorAll(".toggle-row, .multi-button-row").forEach((row) => {
        const activeButton = row.querySelector<HTMLElement>(".toggle-option.active, .multi-btn.active");
        if (activeButton) {
            updateIndicator(row, activeButton);
        }
    });
}

function applyExpandableState(rowId: string, value: string | null) {
    const sectionIdByRow: Record<string, string> = {
        "press-hold-toggle": "mouse-hold-duration-section",
        "kb-mode-toggle": "kb-hold-duration-section",
        "repeat-toggle": "mouse-repeat-finite-section",
        "kb-repeat-toggle": "kb-repeat-finite-section",
        "position-toggle": "mouse-coord-section",
        "macro-repeat-toggle": "macro-repeat-finite-section",
    };

    const expectedValueByRow: Record<string, string> = {
        "press-hold-toggle": "hold",
        "kb-mode-toggle": "hold",
        "repeat-toggle": "finite",
        "kb-repeat-toggle": "finite",
        "position-toggle": "custom",
        "macro-repeat-toggle": "finite",
    };

    const sectionId = sectionIdByRow[rowId];
    const expectedValue = expectedValueByRow[rowId];
    if (!sectionId || !expectedValue) {
        return;
    }

    document.getElementById(sectionId)?.classList.toggle("expanded", value === expectedValue);
}

function initExpandableStates() {
    expandableConfigs.forEach(({ toggleId, sectionId, activeValue }) => {
        const toggle = document.getElementById(toggleId);
        const section = document.getElementById(sectionId);
        const activeButton = toggle?.querySelector<HTMLElement>(".toggle-option.active");
        const value = activeButton?.dataset.value || null;
        section?.classList.toggle("expanded", value === activeValue);
    });
}

function applyMultithreadPreset(button: HTMLElement, row: Element) {
    if (row.id !== "multithread-mode-row" || !isMultithreadSupported()) {
        return;
    }

    const label = (button.innerText || "").toLowerCase();
    let threads = 4;
    if (label.includes("eco")) {
        threads = 2;
    }
    if (label.includes("extreme")) {
        threads = 16;
    }

    const slider = document.getElementById("multithread-slider") as HTMLInputElement | null;
    if (slider) {
        slider.value = String(threads);
        slider.dispatchEvent(new Event("input", { bubbles: true }));
        return;
    }

    void invoke("set_threads_count", { count: threads });
}

function openExtremeWarningModal(selection: ExtremeSelection) {
    const modal = document.getElementById("extreme-warning-modal");
    const input = document.getElementById("extreme-captcha-input") as HTMLInputElement | null;
    const confirmButton = document.getElementById("extreme-confirm-btn") as HTMLButtonElement | null;
    if (!modal || !input || !confirmButton) {
        return;
    }

    modal.style.display = "flex";
    requestAnimationFrame(() => {
        modal.classList.add("active");
    });

    input.value = "";
    confirmButton.disabled = true;
    confirmButton.style.background = "rgba(255, 193, 7, 0.05)";
    confirmButton.style.color = "rgba(255, 193, 7, 0.4)";
    confirmButton.style.cursor = "not-allowed";
    window.setTimeout(() => input.focus(), 50);

    selection.button = selection.button;
    selection.row = selection.row;
    document.getElementById("content")?.classList.add("blurred");
}

function closeExtremeWarningModal() {
    const modal = document.getElementById("extreme-warning-modal");
    if (!modal) {
        return;
    }

    modal.classList.remove("active");
    window.setTimeout(() => {
        modal.style.display = "none";
        document.getElementById("content")?.classList.remove("blurred");
    }, 300);
}

function setExtremeConfirmState(enabled: boolean) {
    const confirmButton = document.getElementById("extreme-confirm-btn") as HTMLButtonElement | null;
    if (!confirmButton) {
        return;
    }

    confirmButton.disabled = !enabled;
    confirmButton.style.background = enabled ? "rgba(255, 193, 7, 0.15)" : "rgba(255, 193, 7, 0.05)";
    confirmButton.style.color = enabled ? "#FFC107" : "rgba(255, 193, 7, 0.4)";
    confirmButton.style.cursor = enabled ? "pointer" : "not-allowed";
}

function initExtremeWarningModal(pendingSelection: ExtremeSelection) {
    const input = document.getElementById("extreme-captcha-input") as HTMLInputElement | null;
    const confirmButton = document.getElementById("extreme-confirm-btn") as HTMLButtonElement | null;

    input?.addEventListener("input", () => {
        setExtremeConfirmState(input.value.trim().toUpperCase() === "I AGREE");
    });

    input?.addEventListener("keydown", (event) => {
        if (event.key === "Enter" && confirmButton && !confirmButton.disabled) {
            confirmButton.click();
        }
    });

    document.getElementById("extreme-cancel-btn")?.addEventListener("click", () => {
        closeExtremeWarningModal();
    });

    if (!confirmButton) {
        return;
    }

    confirmButton.addEventListener("click", () => {
        if (confirmButton.disabled) {
            return;
        }

        closeExtremeWarningModal();
        window.setTimeout(() => {
            if (!pendingSelection.button || !pendingSelection.row) {
                return;
            }

            pendingSelection.row
                .querySelectorAll(".toggle-option, .multi-btn")
                .forEach((button) => button.classList.remove("active"));

            pendingSelection.button.classList.add("active");
            updateIndicator(pendingSelection.row, pendingSelection.button);

            if (pendingSelection.row.id === "multithread-mode-row" && isMultithreadSupported()) {
                const slider = document.getElementById("multithread-slider") as HTMLInputElement | null;
                if (slider) {
                    slider.value = "16";
                    slider.dispatchEvent(new Event("input", { bubbles: true }));
                } else {
                    void invoke("set_threads_count", { count: 16 });
                }
            }

            pendingSelection.button = null;
            pendingSelection.row = null;
        }, 300);
    });
}

function initSlidableRows() {
    const pendingExtremeSelection: ExtremeSelection = {
        button: null,
        row: null,
    };

    document.querySelectorAll(".toggle-row, .multi-button-row").forEach((row) => {
        const buttons = row.querySelectorAll<HTMLElement>(".toggle-option, .multi-btn");
        const activeButton = row.querySelector<HTMLElement>(".toggle-option.active, .multi-btn.active");
        if (activeButton) {
            createSlideIndicator(row, activeButton, true);
        }

        buttons.forEach((button) => {
            button.addEventListener("click", () => {
                if (button.classList.contains("active")) {
                    return;
                }

                if (button.dataset.mode === "extreme") {
                    if (row.id === "multithread-mode-row" && !isMultithreadSupported()) {
                        return;
                    }

                    pendingExtremeSelection.button = button;
                    pendingExtremeSelection.row = row as HTMLElement;
                    openExtremeWarningModal(pendingExtremeSelection);
                    return;
                }

                buttons.forEach((entry) => entry.classList.remove("active"));
                button.classList.add("active");
                updateIndicator(row, button);
                applyMultithreadPreset(button, row);
                applyExpandableState(row.id, button.dataset.value || null);
            });
        });
    });

    initExpandableStates();
    initExtremeWarningModal(pendingExtremeSelection);
}

export function initTabAndToggleUi() {
    window.addEventListener("resize", updateAllIndicators);

    requestAnimationFrame(() => {
        initTabs(updateIndicator, createSlideIndicator);
        initSlidableRows();
    });
}
