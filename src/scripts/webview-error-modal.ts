import { t } from "./i18n";

type WebviewErrorSource = "runtime" | "promise" | "webview";

type WebviewErrorDetail = {
    source: WebviewErrorSource;
    title?: string;
    message: string;
    raw: string;
};

let lastErrorText = "";
let isInitialized = false;

function stringifyError(error: unknown): string {
    if (error instanceof Error) {
        return [error.name ? `${error.name}: ${error.message}` : error.message, error.stack]
            .filter(Boolean)
            .join("\n\n");
    }

    if (typeof error === "string") {
        return error;
    }

    try {
        return JSON.stringify(error, null, 2);
    } catch {
        return String(error);
    }
}

function ensureErrorModal(): HTMLElement {
    const existing = document.getElementById("webview-error-modal");
    if (existing) {
        return existing;
    }

    const modal = document.createElement("div");
    modal.id = "webview-error-modal";
    modal.className = "drawer-overlay startup-modal webview-error-modal";
    modal.style.display = "none";
    modal.innerHTML = `
      <div class="modal-content">
        <div class="startup-focus">
          <span class="icon startup-focus-icon">&#58485;</span>
        </div>

        <div class="startup-copy-block">
          <span class="startup-modal-kicker" id="webview-error-kicker">${t("webview.app_notice", "app notice")}</span>
          <strong><em id="webview-error-title">${t("webview.attention", "Something needs attention")}</em></strong>
          <p id="webview-error-message">${t("webview.default_message", "The app ran into a problem. You can close this window and try the action again.")}</p>
        </div>

        <div class="webview-error-copy-wrap">
          <div class="webview-error-copy-head">
            <span>${t("webview.details_for_support", "Details for support")}</span>
            <button id="webview-error-copy-btn" type="button">${t("webview.copy", "Copy")}</button>
          </div>
          <textarea id="webview-error-text" readonly spellcheck="false"></textarea>
        </div>

        <div class="startup-modal-actions">
          <button id="webview-error-close-btn" class="startup-modal-button secondary" type="button">${t("webview.close", "Close")}</button>
        </div>
      </div>
    `;
    document.body.appendChild(modal);
    bindErrorModal(modal);
    return modal;
}

function bindErrorModal(modal: HTMLElement) {
    const closeButton = modal.querySelector<HTMLButtonElement>("#webview-error-close-btn");
    const copyButton = modal.querySelector<HTMLButtonElement>("#webview-error-copy-btn");
    const textarea = modal.querySelector<HTMLTextAreaElement>("#webview-error-text");

    closeButton?.addEventListener("click", () => hideWebviewErrorModal());
    copyButton?.addEventListener("click", () => {
        void copyErrorText(textarea, copyButton);
    });
}

async function copyErrorText(textarea: HTMLTextAreaElement | null, button: HTMLButtonElement) {
    const text = textarea?.value || lastErrorText;
    if (!text) return;

    try {
        await navigator.clipboard.writeText(text);
    } catch {
        textarea?.focus();
        textarea?.select();
        document.execCommand("copy");
    }

    const previousText = button.textContent || t("webview.copy", "Copy");
    button.textContent = t("webview.copied", "Copied");
    window.setTimeout(() => {
        button.textContent = previousText;
    }, 1100);
}

function showModal(modal: HTMLElement) {
    modal.style.display = "flex";
    document.getElementById("content")?.classList.add("blurred");
    requestAnimationFrame(() => {
        modal.classList.add("active");
    });
}

function hideWebviewErrorModal() {
    const modal = document.getElementById("webview-error-modal");
    if (!modal) return;

    modal.classList.remove("active");
    window.setTimeout(() => {
        modal.style.display = "none";
        document.getElementById("content")?.classList.remove("blurred");
    }, 220);
}

function sourceLabel(source: WebviewErrorSource): string {
    if (source === "promise") return t("webview.source_background", "background action");
    if (source === "webview") return t("webview.source_window", "window issue");
    return t("webview.source_app", "app issue");
}

function fallbackMessage(source: WebviewErrorSource): string {
    if (source === "webview") {
        return t("webview.fallback_webview", "This window could not be opened. Close this message and try again.");
    }

    if (source === "promise") {
        return t("webview.fallback_promise", "A background action did not finish correctly. Close this message and try the action again.");
    }

    return t("webview.fallback_runtime", "The app ran into a problem. Close this message and try the action again.");
}

export function showWebviewErrorModal(detail: WebviewErrorDetail) {
    const modal = ensureErrorModal();
    const title = modal.querySelector<HTMLElement>("#webview-error-title");
    const kicker = modal.querySelector<HTMLElement>("#webview-error-kicker");
    const message = modal.querySelector<HTMLElement>("#webview-error-message");
    const textarea = modal.querySelector<HTMLTextAreaElement>("#webview-error-text");
    const copyButton = modal.querySelector<HTMLButtonElement>("#webview-error-copy-btn");

    lastErrorText = detail.raw || detail.message;

    if (kicker) kicker.textContent = sourceLabel(detail.source);
    if (title) title.textContent = detail.title || t("webview.attention", "Something needs attention");
    if (message) message.textContent = detail.message || fallbackMessage(detail.source);
    if (textarea) textarea.value = lastErrorText;
    if (copyButton) copyButton.textContent = t("webview.copy", "Copy");

    showModal(modal);
    window.setTimeout(() => textarea?.focus(), 50);
}

export function initWebviewErrorModal() {
    ensureErrorModal();
    if (isInitialized) return;
    isInitialized = true;

    window.addEventListener("error", (event) => {
        const raw = stringifyError(event.error || event.message);
        showWebviewErrorModal({
            source: "runtime",
            title: t("webview.error_title", "The app hit a problem"),
            message: t("webview.error_message", "Something inside this window stopped working. Close this message and try again."),
            raw,
        });
    });

    window.addEventListener("unhandledrejection", (event) => {
        const raw = stringifyError(event.reason);
        showWebviewErrorModal({
            source: "promise",
            title: t("webview.promise_title", "Action did not finish"),
            message: t("webview.promise_message", "A background action could not be completed. Close this message and try again."),
            raw,
        });
    });
}

export function showWebviewCreationError(label: string, error: unknown) {
    const raw = stringifyError(error);
    showWebviewErrorModal({
        source: "webview",
        title: t("webview.open_error_title", "{label} could not open", { label }),
        message: t("webview.open_error_message", "The {label} window did not open. Close this message and try again.", { label }),
        raw,
    });
}
