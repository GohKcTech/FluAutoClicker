import { safeInvoke } from "./invoke";

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
    const percent = ((val - min) / (max - min)) * 100;
    slider.style.background = `linear-gradient(to right, var(--accent-dim) ${percent}%, rgba(11, 11, 11, 1) ${percent}%)`;
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
