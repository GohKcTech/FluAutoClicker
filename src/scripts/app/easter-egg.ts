function randomHexColor() {
    return `#${Math.floor(Math.random() * 16_777_215)
        .toString(16)
        .padStart(6, "0")}`;
}

export function initKeyboardPartyMode() {
    let originalAccent = "";
    let originalAccentDim = "";
    let partyTimeoutId: number | null = null;
    let partyClickCount = 0;
    let waveTimeoutIds: number[] = [];

    const clearWaveTimeouts = () => {
        waveTimeoutIds.forEach((timeoutId) => window.clearTimeout(timeoutId));
        waveTimeoutIds = [];
    };

    const resetWaveState = (keys: HTMLElement[]) => {
        keys.forEach((key) => {
            key.classList.remove("party-wave");
            key.style.removeProperty("--party-wave-bg");
            key.style.removeProperty("--party-wave-filter");
        });
    };

    document.addEventListener("click", (event) => {
        const target = event.target as HTMLElement | null;
        if (!target?.classList.contains("kb-tsu")) {
            return;
        }

        event.stopPropagation();
        partyClickCount += 1;

        if (partyClickCount < 3) {
            target.classList.remove("pre-flash");
            void target.offsetWidth;
            target.classList.add("pre-flash");
            return;
        }

        partyClickCount = 0;
        if (partyTimeoutId) {
            window.clearTimeout(partyTimeoutId);
        }
        clearWaveTimeouts();

        if (!originalAccent) {
            originalAccent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim();
            originalAccentDim = getComputedStyle(document.documentElement).getPropertyValue("--accent-dim").trim();
        }

        const accentColor = randomHexColor();
        document.documentElement.style.setProperty("--accent", accentColor);
        document.documentElement.style.setProperty("--accent-dim", `${accentColor}66`);

        const triggerKey = document.querySelector(".kb-tsu") as HTMLElement | null;
        if (triggerKey) {
            triggerKey.classList.remove("party-active");
            void triggerKey.offsetWidth;
            triggerKey.classList.add("party-active");
        }

        const keys = Array.from(document.querySelectorAll<HTMLElement>(".kb-key:not(.kb-tsu)"));
        resetWaveState(keys);

        void document.body.offsetWidth;

        keys.forEach((key, index) => {
            const timeoutId = window.setTimeout(() => {
                key.classList.add("party-wave");
                key.style.setProperty("--party-wave-bg", `${accentColor}33`);
                key.style.setProperty("--party-wave-filter", "brightness(1.25)");
            }, index * 12);
            waveTimeoutIds.push(timeoutId);
        });

        const waveDurationMs = keys.length * 12 + 600;
        partyTimeoutId = window.setTimeout(() => {
            document.documentElement.style.setProperty("--accent", originalAccent);
            document.documentElement.style.setProperty("--accent-dim", originalAccentDim);

            clearWaveTimeouts();
            resetWaveState(keys);

            window.setTimeout(() => {
                triggerKey?.classList.remove("party-active");
                partyTimeoutId = null;
            }, 600);
        }, waveDurationMs + 3_000);
    });
}
