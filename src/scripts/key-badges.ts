export function formatKeyLabel(value: string): string {
    const normalized = value.trim();
    if (!normalized) {
        return "";
    }

    const lower = normalized.toLowerCase();
    const labels: Record<string, string> = {
        ctrl: "Ctrl",
        shift: "Shift",
        alt: "Alt",
        win: "Win",
        escape: "Esc",
        capslock: "Caps",
        pageup: "PgUp",
        pagedown: "PgDn",
        delete: "Del",
        insert: "Ins",
        space: "Space",
    };

    return labels[lower] || (normalized.length === 1 ? normalized.toUpperCase() : normalized);
}

export function setKeyBadgeContent(container: Element, keys: string[]) {
    removeLegacyTextNodes(container);
    container.classList.toggle("empty", keys.length === 0);

    const visibleKeys = keys.map(formatKeyLabel).filter(Boolean);
    if (visibleKeys.length === 0) {
        showEmptyStateAfterBadges(container);
        return;
    }

    removeEmptyState(container, true);

    const existingSlots = Array.from(container.querySelectorAll<HTMLElement>(".key-badge-slot"))
        .filter((slot) => !slot.classList.contains("key-badge-removing"));

    visibleKeys.forEach((key, index) => {
        const slot = existingSlots[index] || createKeyBadgeSlot(key);
        updateKeyBadgeSlot(slot, key);

        if (!slot.parentElement) {
            container.appendChild(slot);
        }
    });

    existingSlots.slice(visibleKeys.length).forEach(removeWithAnimation);
}

function showEmptyStateAfterBadges(container: Element) {
    const badges = Array.from(container.querySelectorAll<HTMLElement>(".key-badge-slot"));
    removeKeyBadges(container);

    const delay = badges.length > 0 ? 130 : 0;
    window.setTimeout(() => {
        if (container.querySelector(".key-badge-slot")) {
            return;
        }

        if (!container.querySelector(".key-badge-empty")) {
            const empty = document.createElement("span");
            empty.className = "key-badge-empty";
            empty.textContent = "None";
            container.appendChild(empty);
        }
    }, delay);
}

function createKeyBadgeSlot(key: string): HTMLElement {
    const slot = document.createElement("span");
    slot.className = "key-badge-slot";
    slot.dataset.key = key;

    const badge = document.createElement("span");
    badge.className = "key-badge";

    const label = document.createElement("span");
    label.className = "key-badge-label";
    label.textContent = key;
    badge.appendChild(label);
    slot.appendChild(badge);

    return slot;
}

function updateKeyBadgeSlot(slot: HTMLElement, key: string) {
    if (slot.dataset.key === key) {
        return;
    }

    slot.dataset.key = key;
    const label = slot.querySelector<HTMLElement>(".key-badge-label");
    if (label) {
        label.textContent = key;
    }

    const badge = slot.querySelector<HTMLElement>(".key-badge");
    if (badge) {
        badge.classList.remove("key-badge-updated");
        void badge.offsetWidth;
        badge.classList.add("key-badge-updated");
    }
}

function removeLegacyTextNodes(container: Element) {
    container.querySelectorAll(".kb-modifiers, .kb-main-key").forEach((node) => node.remove());
}

function removeKeyBadges(container: Element) {
    container.querySelectorAll<HTMLElement>(".key-badge-slot").forEach(removeWithAnimation);
}

function removeEmptyState(container: Element, immediate = false) {
    container.querySelectorAll<HTMLElement>(".key-badge-empty").forEach((empty) => {
        if (immediate) {
            empty.remove();
            return;
        }

        empty.classList.add("key-badge-empty-removing");
        window.setTimeout(() => empty.remove(), 140);
    });
}

function removeWithAnimation(slot: HTMLElement) {
    if (slot.classList.contains("key-badge-removing")) {
        return;
    }

    slot.classList.add("key-badge-removing");
    window.setTimeout(() => slot.remove(), 130);
}
