import { maybeShowStartupUinputPermissionModal } from "./uinput-permissions";

const FRONTEND_STATE_KEY = "flu-frontend-state";
const WELCOME_ACK_KEY = "first_welcome_v25_ack";
const BETA_ACK_KEY = "beta_warning_v25_ack";

type FrontendState = Record<string, unknown>;

function asFrontendState(value: unknown): FrontendState {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return {};
    }
    return value as FrontendState;
}

function readFrontendState(): FrontendState {
    try {
        return asFrontendState(JSON.parse(localStorage.getItem(FRONTEND_STATE_KEY) || "{}"));
    } catch {
        return {};
    }
}

function writeFrontendState(state: FrontendState) {
    localStorage.setItem(FRONTEND_STATE_KEY, JSON.stringify(state));
    window.dispatchEvent(new CustomEvent("flu:settings-changed"));
}

function markAcknowledged(key: string) {
    const state = readFrontendState();
    state[key] = true;
    writeFrontendState(state);
}

function showModal(modal: HTMLElement | null) {
    if (!modal) return;

    modal.style.display = "flex";
    document.getElementById("content")?.classList.add("blurred");
    requestAnimationFrame(() => {
        modal.classList.add("active");
    });
}

function hideModal(modal: HTMLElement | null, onHidden?: () => void) {
    if (!modal) return;

    modal.classList.remove("active");
    window.setTimeout(() => {
        modal.style.display = "none";
        document.getElementById("content")?.classList.remove("blurred");
        onHidden?.();
    }, 220);
}

function bindDismiss(buttonId: string, modalId: string, stateKey: string, onHidden?: () => void) {
    const button = document.getElementById(buttonId);
    const modal = document.getElementById(modalId);
    if (!button || !modal) return;

    button.addEventListener("click", () => {
        markAcknowledged(stateKey);
        hideModal(modal, onHidden);
    });
}

export function initStartupModals() {
    const state = readFrontendState();
    const welcomeModal = document.getElementById("first-welcome-modal");
    const betaModal = document.getElementById("beta-warning-modal");

    const showBetaIfNeeded = () => {
        if (!readFrontendState()[BETA_ACK_KEY]) {
            showModal(betaModal);
        } else {
            void maybeShowStartupUinputPermissionModal();
        }
    };

    bindDismiss("first-welcome-continue-btn", "first-welcome-modal", WELCOME_ACK_KEY, showBetaIfNeeded);
    bindDismiss("beta-warning-confirm-btn", "beta-warning-modal", BETA_ACK_KEY, () => {
        void maybeShowStartupUinputPermissionModal();
    });

    if (!state[WELCOME_ACK_KEY]) {
        window.setTimeout(() => showModal(welcomeModal), 350);
        return;
    }

    if (!state[BETA_ACK_KEY]) {
        window.setTimeout(() => showModal(betaModal), 350);
        return;
    }

    void maybeShowStartupUinputPermissionModal();
}
