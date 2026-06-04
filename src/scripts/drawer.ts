import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { notify } from "./notifications";
import { ensureUinputPermissionsForFeature } from "./uinput-permissions";
import { updateSliderFill } from "./utils";

const DRAWER_FOCUSABLE_SELECTOR = [
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    'a[href]',
    '[tabindex]:not([tabindex="-1"])',
].join(',');

let drawerReturnFocus: HTMLElement | null = null;

function isVisibleElement(element: HTMLElement) {
    return Boolean(element.offsetWidth || element.offsetHeight || element.getClientRects().length);
}

function getDrawerFocusableElements(drawer: HTMLElement) {
    return Array.from(drawer.querySelectorAll<HTMLElement>(DRAWER_FOCUSABLE_SELECTOR))
        .filter((element) => !element.hasAttribute("inert") && isVisibleElement(element));
}

function setDrawerAccessibilityState(drawer: HTMLElement, isOpen: boolean) {
    drawer.toggleAttribute("inert", !isOpen);
    drawer.setAttribute("aria-hidden", isOpen ? "false" : "true");
}

function rememberDrawerReturnFocus(drawer: HTMLElement) {
    const activeElement = document.activeElement;
    if (activeElement instanceof HTMLElement && !drawer.contains(activeElement)) {
        drawerReturnFocus = activeElement;
    }
}

function restoreDrawerReturnFocus(drawer: HTMLElement) {
    const activeElement = document.activeElement;
    if (activeElement instanceof HTMLElement && drawer.contains(activeElement)) {
        activeElement.blur();
    }

    if (drawerReturnFocus?.isConnected) {
        drawerReturnFocus.focus({ preventScroll: true });
    }
    drawerReturnFocus = null;
}

export function openDrawer(sectionId: string, title: string, icon?: string) {
    const drawer = document.getElementById('bottom-drawer');
    const drawerOverlay = document.getElementById('drawer-overlay');
    const drawerTitleText = document.getElementById('drawer-title-text');
    const drawerTitleIcon = document.getElementById('drawer-title-icon');
    const drawerSections = document.querySelectorAll('.drawer-section');

    if (!drawer || !drawerOverlay || !drawerTitleText) return;

    rememberDrawerReturnFocus(drawer);
    setDrawerAccessibilityState(drawer, true);
    
    drawerTitleText.textContent = title;
    if (drawerTitleIcon) {
        drawerTitleIcon.innerHTML = icon || '';
        drawerTitleIcon.style.display = icon ? 'flex' : 'none';
    }
    
    drawerSections.forEach(s => s.classList.remove('active'));
    const target = document.getElementById(sectionId);
    if (target) target.classList.add('active');
    
    drawer.classList.add('active');
    drawerOverlay.classList.add('active');
    document.getElementById('content')?.classList.add('blurred');
    
    
    setTimeout(() => refreshDrawerUi(target), 50);
}

export function closeDrawer() {
    const drawer = document.getElementById('bottom-drawer');
    const drawerOverlay = document.getElementById('drawer-overlay');
    drawer?.classList.remove('active');
    drawerOverlay?.classList.remove('active');
    if (drawer) {
        restoreDrawerReturnFocus(drawer);
        setDrawerAccessibilityState(drawer, false);
    }
    setTimeout(() => {
        document.getElementById('content')?.classList.remove('blurred');
    }, 300);
}

function refreshDrawerUi(section?: HTMLElement | null) {
    const rows = document.querySelectorAll('.multi-button-row');
    rows.forEach(row => {
        const indicator = row.querySelector('.slide-indicator') as HTMLElement;
        const activeBtn = row.querySelector('.multi-btn.active') as HTMLElement;
        if (indicator && activeBtn) {
            indicator.style.width = `${activeBtn.offsetWidth}px`;
            indicator.style.left = `${activeBtn.offsetLeft}px`;
        }
    });

    section?.querySelectorAll('.interval-slider').forEach(slider => {
        updateSliderFill(slider as HTMLInputElement);
    });
}

function bindHelpLink(elementId: string, url: string, label: string) {
    const element = document.getElementById(elementId);
    if (!element) return;

    const openResource = async () => {
        try {
            await openUrl(url);
        } catch (error) {
            console.error(`Failed to open ${label}`, error);
            notify(`Could not open ${label}.`, "error", 2600);
        }
    };

    element.addEventListener("click", () => {
        void openResource();
    });

    element.addEventListener("keydown", (event) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        void openResource();
    });
}

function bindDrawerLink(elementId: string, sectionId: string, title: string, icon?: string) {
    const element = document.getElementById(elementId);
    if (!element) return;

    const openTarget = () => {
        openDrawer(sectionId, title, icon);
    };

    element.addEventListener("click", openTarget);
    element.addEventListener("keydown", (event) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        openTarget();
    });
}

export function initDrawer() {
    const drawerOverlay = document.getElementById('drawer-overlay');
    const drawer = document.getElementById('bottom-drawer');
    const drawerClose = document.getElementById('drawer-close-btn');

    if (drawer) {
        setDrawerAccessibilityState(drawer, drawer.classList.contains('active'));

        drawer.addEventListener('keydown', (event) => {
            if (event.key !== 'Tab' || !drawer.classList.contains('active')) return;

            const focusableElements = getDrawerFocusableElements(drawer);
            if (focusableElements.length === 0) {
                event.preventDefault();
                return;
            }

            const firstElement = focusableElements[0];
            const lastElement = focusableElements[focusableElements.length - 1];
            const activeElement = document.activeElement;

            if (event.shiftKey && activeElement === firstElement) {
                event.preventDefault();
                lastElement.focus({ preventScroll: true });
            } else if (!event.shiftKey && activeElement === lastElement) {
                event.preventDefault();
                firstElement.focus({ preventScroll: true });
            }
        });
    }

    drawerClose?.addEventListener('click', closeDrawer);
    drawerOverlay?.addEventListener('click', closeDrawer);

    document.getElementById('settings-btn')?.addEventListener('click', () => openDrawer('section-settings', 'Settings', '&#xe154;'));
    document.getElementById('hotkeys-btn')?.addEventListener('click', () => openDrawer('section-hotkeys', 'Hotkeys', '&#xe284;'));
    document.getElementById('profiles-btn')?.addEventListener('click', () => openDrawer('section-profiles', 'Profiles System', '&#xe36b;'));
    document.getElementById('help-btn')?.addEventListener('click', () => openDrawer('section-help', 'Help & Resources', '&#xe47b;'));
    bindHelpLink('help-discord-link', 'https://agzes.github.io/go/to/discord', 'Discord');
    bindHelpLink('help-github-link', 'https://github.com/Agzes/FluAutoClicker', 'GitHub');
    bindDrawerLink('help-updates-link', 'section-updates', 'Updates', '&#xe47b;');

    return { openDrawer, closeDrawer };
}


export function initJiggler() {
    
    const jigglerToggle = document.getElementById('jiggler-toggle');
    if (jigglerToggle) {
        jigglerToggle.addEventListener('click', async () => {
            
            if (!(await ensureUinputPermissionsForFeature())) {
                return;
            }

            const newState = await invoke("toggle_jiggler") as boolean;
            jigglerToggle.classList.toggle('active', newState);

            
            const mainBtn = document.getElementById('jiggler-btn');
            mainBtn?.classList.toggle('active', newState);
        });
    }

    
    const patternRow = document.getElementById('jiggler-pattern-row');
    if (patternRow) {
        const buttons = patternRow.querySelectorAll('.multi-btn');
        const ozoneHelp = document.getElementById('jiggler-ozone-help');
        const syncOzoneHelp = () => {
            const activePattern = (patternRow.querySelector('.multi-btn.active') as HTMLElement | null)?.dataset.pattern;
            ozoneHelp?.classList.toggle('visible', activePattern === 'ozn');
        };
        syncOzoneHelp();
        buttons.forEach(btn => {
            btn.addEventListener('click', async () => {
                buttons.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                syncOzoneHelp();
                
                
                const indicator = patternRow.querySelector('.slide-indicator') as HTMLElement;
                const activeBtn = patternRow.querySelector('.multi-btn.active') as HTMLElement;
                if (indicator && activeBtn) {
                    indicator.style.width = `${activeBtn.offsetWidth}px`;
                    indicator.style.left = `${activeBtn.offsetLeft}px`;
                }
                
                
                const pattern = (btn as HTMLElement).dataset.pattern;
                if (pattern) {
                    await invoke("set_jiggler_pattern", { pattern });
                }
            });
        });
    }

    
    const intervalSlider = document.getElementById('jiggler-slider') as HTMLInputElement;
    if (intervalSlider) {
        intervalSlider.addEventListener('input', () => {
            const value = parseInt(intervalSlider.value);
            const valueDisplay = intervalSlider.parentElement?.querySelector('.visual-slider-value');
            if (valueDisplay) {
                valueDisplay.textContent = `${value}s`;
            }
            
            invoke("set_jiggler_interval", { interval: value * 1000 });
        });
    }

    
    const distanceSlider = document.getElementById('jiggler-distance-slider') as HTMLInputElement;
    if (distanceSlider) {
        distanceSlider.addEventListener('input', () => {
            const value = parseInt(distanceSlider.value);
            const valueDisplay = distanceSlider.parentElement?.querySelector('.visual-slider-value');
            if (valueDisplay) {
                valueDisplay.textContent = `${value}px`;
            }
            invoke("set_jiggler_distance", { distance: value });
        });
    }

    void listen("jiggler-status-changed", (event) => {
        const payload = event.payload as { active?: boolean };
        const active = Boolean(payload?.active);
        jigglerToggle?.classList.toggle('active', active);
        document.getElementById('jiggler-btn')?.classList.toggle('active', active);
    });
}

export function initTimingModal(onConfirm: () => void) {
    const modal = document.getElementById('timing-warning-modal');
    const track = document.getElementById('timing-unlock-track');
    const progress = document.getElementById('timing-unlock-progress');
    const text = track?.querySelector('.unlock-slider-text');
    const handle = document.getElementById('timing-unlock-handle');
    const cancel = document.getElementById('timing-cancel-btn');

    if (modal && track && handle && cancel) {
        let isDragging = false;
        let startX = 0;
        let currentX = 0;
        let maxDelta = 0;
        const handleTransition = 'transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), background 0.35s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.35s ease';
        const handleDragTransition = 'transform 0s linear, background 0.35s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.35s ease';
        const progressTransition = 'width 0.3s cubic-bezier(0.16, 1, 0.3, 1), background 0.35s cubic-bezier(0.16, 1, 0.3, 1)';
        const progressDragTransition = 'width 0s linear, background 0.35s cubic-bezier(0.16, 1, 0.3, 1)';

        const resetHandle = () => {
            isDragging = false;
            currentX = 0;
            handle.style.transform = 'translateX(0)';
            handle.style.transition = handleTransition;
            if (progress) {
                progress.style.width = '0';
                progress.style.transition = progressTransition;
            }
            if (text) text.textContent = 'Slide to confirm';
            track.classList.remove('unlocked');
        };

        const startDragging = (clientX: number) => {
            isDragging = true;
            maxDelta = track.clientWidth - handle.clientWidth - 8;
            startX = clientX - currentX;
            handle.style.transition = handleDragTransition;
            if (progress) progress.style.transition = progressDragTransition;
        };

        const moveDragging = (clientX: number) => {
            if (!isDragging) return;
            
            let delta = clientX - startX;
            if (delta < 0) delta = 0;
            if (delta > maxDelta) delta = maxDelta;
            
            currentX = delta;
            handle.style.transform = `translateX(${delta}px)`;
            if (progress) {
                progress.style.width = `${delta + 21}px`; 
            }

            const iconEl = document.getElementById('timing-unlock-icon');
            if (delta >= maxDelta * 0.98) {
                track.classList.add('unlocked');
                if (text) text.textContent = 'Release to confirm';
                if (iconEl) iconEl.innerHTML = '&#58544;';
            } else {
                track.classList.remove('unlocked');
                if (text) text.textContent = 'Slide to confirm';
                if (iconEl) iconEl.innerHTML = '&#58591;';
            }
        };

        handle.addEventListener('mousedown', (e) => startDragging(e.clientX));
        window.addEventListener('mousemove', (e) => moveDragging(e.clientX));
        window.addEventListener('mouseup', () => {
            if (!isDragging) return;
            if (currentX >= maxDelta * 0.98) {
                modal.classList.remove('active');
                setTimeout(() => {
                    modal.style.display = 'none';
                    document.getElementById('content')?.classList.remove('blurred');
                }, 300);
                onConfirm();
                setTimeout(resetHandle, 300);
            } else {
                resetHandle();
            }
            isDragging = false;
        });

        
        handle.addEventListener('touchstart', (e) => startDragging(e.touches[0].clientX), { passive: true });
        window.addEventListener('touchmove', (e) => moveDragging(e.touches[0].clientX), { passive: true });
        window.addEventListener('touchend', () => {
            if (!isDragging) return;
            if (currentX >= maxDelta * 0.98) {
                modal.classList.remove('active');
                setTimeout(() => {
                    modal.style.display = 'none';
                    document.getElementById('content')?.classList.remove('blurred');
                }, 300);
                onConfirm();
                setTimeout(resetHandle, 300);
            } else {
                resetHandle();
            }
            isDragging = false;
        });

        cancel.addEventListener('click', () => {
            modal.classList.remove('active');
            setTimeout(() => {
                modal.style.display = 'none';
                document.getElementById('content')?.classList.remove('blurred');
            }, 300);
            resetHandle();
        });
    }
}

