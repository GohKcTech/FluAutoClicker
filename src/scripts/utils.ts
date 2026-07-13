import { safeInvoke } from "./invoke";
import { t } from "./i18n";

export function createSlideIndicator(container: Element, activeBtn: Element, noTransition = false) {
    let indicator = container.querySelector('.slide-indicator') as HTMLElement;
    if (!indicator) {
        indicator = document.createElement('div');
        indicator.classList.add('slide-indicator');
        if (noTransition) indicator.classList.add('no-transition');
        container.insertBefore(indicator, container.firstChild);
    }
    updateIndicator(container, activeBtn, indicator);
    
    if (noTransition) {
        
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                indicator.classList.remove('no-transition');
            });
        });
    }
    return indicator;
}

export function updateIndicator(container: Element, activeBtn: Element, indicator?: HTMLElement) {
    if (!indicator) indicator = container.querySelector('.slide-indicator') as HTMLElement;
    if (!indicator || !activeBtn) return;

    
    const left = (activeBtn as HTMLElement).offsetLeft;
    const width = (activeBtn as HTMLElement).offsetWidth;

    indicator.style.left = `${left}px`;
    indicator.style.width = `${width}px`;
}

export function updateSliderFill(slider: HTMLInputElement) {
    const min = parseFloat(slider.min) || 0;
    const max = parseFloat(slider.max) || 1000;
    const val = parseFloat(slider.value) || 0;
    const ratio = max === min ? 0 : Math.min(1, Math.max(0, (val - min) / (max - min)));
    const thumbWidth = 16;
    const fillPx = slider.clientWidth > thumbWidth
        ? thumbWidth / 2 + ratio * (slider.clientWidth - thumbWidth)
        : ratio * 100;
    const fillStop = slider.clientWidth > thumbWidth ? `${fillPx}px` : `${fillPx}%`;

    slider.style.background = `linear-gradient(to right, var(--accent-dim) ${fillStop}, rgba(11, 11, 11, 1) ${fillStop})`;
}

function getCssHighlightColor() {
    const dummy = document.createElement('div');
    dummy.style.color = 'Highlight'; 
    dummy.style.display = 'none';
    document.body.appendChild(dummy);
    const color = getComputedStyle(dummy).color;
    document.body.removeChild(dummy);
    
    const rgbMatch = color.match(/\d+/g);
    if (rgbMatch && rgbMatch.length >= 3) {
        return '#' + rgbMatch.slice(0,3).map(x => parseInt(x).toString(16).padStart(2, '0')).join('');
    }
    return '#77B6DD';
}

export async function getSystemAccentColor() {
    return safeInvoke<string>("get_system_accent_color", undefined, {
        fallback: getCssHighlightColor(),
    });
}

export async function toggleWebviewDevtools() {
    await safeInvoke<void>("plugin:webview|internal_toggle_devtools");
}

export function isBetaBuild() {
    return __APP_VERSION__.toLowerCase().includes("beta");
}

export async function isWebviewDevtoolsAvailable() {
    if (!isBetaBuild()) return false;

    const capabilities = await safeInvoke<{ webview_devtools?: boolean }>(
        "get_platform_capabilities",
        undefined,
        { fallback: {} },
    );
    return Boolean(capabilities.webview_devtools);
}

export function formatDuration(ms: number): string {
    if (ms >= 1000) {
        return t("duration.s", undefined, { value: (ms / 1000).toFixed(2) });
    }
    return t("duration.ms", undefined, { value: String(ms) });
}

