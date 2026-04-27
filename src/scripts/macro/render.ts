import { invoke } from "@tauri-apps/api/core";
import { setKeyBadgeContent } from "../key-badges";
import { fromBackendAction } from "./backend";
import { macroState } from "./state";
import type { MacroBackendAction } from "./types";

function getListContainer() {
    return document.getElementById("macro-list-container");
}

function appendHighlightedDetails(container: Element, text: string) {
    const parts = text.split(/(\d+(?:\.\d+)?(?:\s*ms)?)/gi).filter(Boolean);

    parts.forEach((part) => {
        const isNumber = /^\d+(?:\.\d+)?(?:\s*ms)?$/i.test(part.trim());
        const element = document.createElement("span");
        element.className = isNumber ? "action-detail-number" : "action-detail-text";
        element.textContent = part;
        container.appendChild(element);
    });
}

function getActionSummary(action: (typeof macroState.actions)[number]) {
    if (action.type === "sleep") {
        return action.details;
    }

    if (action.type === "move") {
        return `Move ${action.details.charAt(0).toLowerCase()}${action.details.slice(1).replace(" (instant)", "")}`;
    }

    if (action.type === "mouse") {
        const actionLabel = action.name.includes("Hold")
            ? action.name.replace("Hold", "hold")
            : action.name.replace("Click", "click");
        return `${actionLabel} ${action.details.charAt(0).toLowerCase()}${action.details.slice(1)}`;
    }

    if (action.type === "keyboard") {
        return action.detailSuffix ? `${action.name.replace("Key ", "")} ${action.details}` : `${action.name.replace("Key ", "")} ${action.details}`;
    }

    return `${action.name} ${action.details}`.trim();
}

function createActionElement(action: (typeof macroState.actions)[number], animate = false) {
    const item = document.createElement("div");
    item.className = "macro-action-item";
    item.setAttribute("data-action-id", String(action.id));
    if (animate) {
        item.classList.add("adding");
    }
    if (macroState.currentPlayingActionId === action.id) {
        item.classList.add("playing");
    }

    item.innerHTML = `
        <div class="action-icon"><span class="icon">${action.icon}</span></div>
        <div class="action-info">
            <span class="action-details"></span>
        </div>
        <button class="action-remove" data-id="${action.id}">
            <span class="icon">&#57742;</span>
        </button>
    `;

    const details = item.querySelector(".action-details");
    if (details && action.detailKeys?.length) {
        details.classList.add("action-details-keys");
        const prefix = document.createElement("span");
        prefix.className = "action-detail-text";
        prefix.textContent = `${action.name.replace("Key ", "")} `;
        details.appendChild(prefix);

        const combo = document.createElement("span");
        combo.className = "key-badge-combo";
        setKeyBadgeContent(combo, action.detailKeys);
        details.appendChild(combo);

        if (action.detailSuffix) {
            const suffix = document.createElement("span");
            suffix.className = "action-detail-suffix";
            appendHighlightedDetails(suffix, action.detailSuffix);
            details.appendChild(suffix);
        }
    } else if (details) {
        appendHighlightedDetails(details, getActionSummary(action));
    }

    item.querySelector(".action-remove")?.addEventListener("click", () => {
        item.classList.add("removing");
        window.setTimeout(async () => {
            try {
                await invoke("remove_macro_action", { actionId: action.id, action_id: action.id });
                macroState.actions = macroState.actions.filter((entry) => entry.id !== action.id);
                renderActions();
            } catch (error) {
                console.error("Failed to remove macro action", error);
                await loadActionsFromBackend();
            }
        }, 300);
    });

    if (animate) {
        item.addEventListener("animationend", () => item.classList.remove("adding"), { once: true });
    }

    return item;
}

export function renderActions(options: { animateNew?: boolean } = {}) {
    const listContainer = getListContainer();
    if (!listContainer) {
        return;
    }

    if (macroState.actions.length === 0) {
        listContainer.innerHTML = `
            <div class="macro-list-empty">
                <span class="icon" style="font-size: 32px; color: var(--accent); margin-bottom: 12px; opacity: 0.6;">&#58964;</span>
                <span style="letter-spacing: 1px; text-transform: uppercase; font-size: 11px; font-weight: 700; color: var(--text); opacity: 0.9;">No actions recorded yet</span>
            </div>
        `;
        return;
    }

    listContainer.querySelector(".macro-list-empty")?.remove();
    const existingIds = new Set(
        Array.from(listContainer.querySelectorAll<HTMLElement>(".macro-action-item"))
            .map((element) => element.getAttribute("data-action-id") || "")
    );
    const liveIds = new Set(macroState.actions.map((action) => String(action.id)));
    listContainer.querySelectorAll<HTMLElement>(".macro-action-item").forEach((element) => {
        const id = element.getAttribute("data-action-id") || "";
        if (!liveIds.has(id)) {
            element.remove();
        }
    });

    let addedNewItem = false;
    const lastActionId = macroState.actions[macroState.actions.length - 1]?.id ?? null;
    macroState.actions.forEach((action) => {
        const existing = listContainer.querySelector<HTMLElement>(`.macro-action-item[data-action-id="${action.id}"]`);
        const shouldAnimate = Boolean(options.animateNew && !existingIds.has(String(action.id)) && action.id === lastActionId);
        const item = existing || createActionElement(action, shouldAnimate);
        addedNewItem ||= !existing;
        listContainer.appendChild(item);
    });

    if (options.animateNew && addedNewItem) {
        window.requestAnimationFrame(() => {
            listContainer.scrollTo({ top: listContainer.scrollHeight, behavior: "smooth" });
        });
    }
}

export function updateCurrentActionHighlight() {
    const listContainer = getListContainer();
    if (!listContainer) {
        return;
    }

    listContainer.querySelectorAll(".macro-action-item").forEach((element) => {
        const item = element as HTMLElement;
        const id = Number(item.getAttribute("data-action-id"));
        const isPlaying = macroState.currentPlayingActionId !== null && id === macroState.currentPlayingActionId;
        item.classList.toggle("playing", isPlaying);
        if (isPlaying) {
            item.scrollIntoView({ block: "nearest", behavior: "smooth" });
        }
    });
}

export async function loadActionsFromBackend(options: { animateNew?: boolean } = {}) {
    const rawActions = await invoke<MacroBackendAction[]>("get_macro_actions");
    macroState.actions = rawActions
        .map((item) => fromBackendAction(item))
        .filter((item): item is NonNullable<typeof item> => Boolean(item));
    renderActions(options);
}
