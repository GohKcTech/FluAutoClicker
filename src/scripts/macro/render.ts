import { invoke } from "@tauri-apps/api/core";
import { fromBackendAction } from "./backend";
import { macroState } from "./state";
import type { MacroBackendAction } from "./types";

function getListContainer() {
    return document.getElementById("macro-list-container");
}

export function renderActions() {
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

    listContainer.innerHTML = "";
    macroState.actions.forEach((action) => {
        const item = document.createElement("div");
        item.className = "macro-action-item";
        item.setAttribute("data-action-id", String(action.id));
        if (macroState.currentPlayingActionId === action.id) {
            item.classList.add("playing");
        }

        item.innerHTML = `
            <div class="action-icon"><span class="icon">${action.icon}</span></div>
            <div class="action-info">
                <span class="action-name">${action.name}</span>
                <span class="action-details">${action.details}</span>
            </div>
            <button class="action-remove" data-id="${action.id}">
                <span class="icon">&#57742;</span>
            </button>
        `;

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

        listContainer.appendChild(item);
    });
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

export async function loadActionsFromBackend() {
    const rawActions = await invoke<MacroBackendAction[]>("get_macro_actions");
    macroState.actions = rawActions
        .map((item) => fromBackendAction(item))
        .filter((item): item is NonNullable<typeof item> => Boolean(item));
    renderActions();
}
