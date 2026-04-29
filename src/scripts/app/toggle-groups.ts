import { createSlideIndicator, updateIndicator } from "../utils";
import { initTabs } from "../ui";

type ExpandableConfig = {
    toggleId: string;
    sectionId: string;
    activeValue: string;
};

const expandableConfigs: ExpandableConfig[] = [
    { toggleId: "press-hold-toggle", sectionId: "mouse-hold-duration-section", activeValue: "hold" },
    { toggleId: "kb-mode-toggle", sectionId: "kb-hold-duration-section", activeValue: "hold" },
    { toggleId: "repeat-toggle", sectionId: "mouse-repeat-finite-section", activeValue: "finite" },
    { toggleId: "kb-repeat-toggle", sectionId: "kb-repeat-finite-section", activeValue: "finite" },
    { toggleId: "position-toggle", sectionId: "mouse-coord-section", activeValue: "custom" },
    { toggleId: "macro-repeat-toggle", sectionId: "macro-repeat-finite-section", activeValue: "finite" },
];

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

function initSlidableRows() {
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

                buttons.forEach((entry) => entry.classList.remove("active"));
                button.classList.add("active");
                updateIndicator(row, button);
                applyExpandableState(row.id, button.dataset.value || null);
            });
        });
    });

    initExpandableStates();
}

export function initTabAndToggleUi() {
    window.addEventListener("resize", updateAllIndicators);

    requestAnimationFrame(() => {
        initTabs(updateIndicator, createSlideIndicator);
        initSlidableRows();
    });
}
