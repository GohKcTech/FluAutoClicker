import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { notify } from "./notifications";
import { } from "./utils";

function formatThreadsLabel(raw: number | string): string {
    const count = Number(raw) || 0;
    return `${count} ${count === 1 ? 'Thread' : 'Threads'}`;
}

export function openDrawer(sectionId: string, title: string, icon?: string) {
    const drawer = document.getElementById('bottom-drawer');
    const drawerOverlay = document.getElementById('drawer-overlay');
    const drawerTitleText = document.getElementById('drawer-title-text');
    const drawerTitleIcon = document.getElementById('drawer-title-icon');
    const drawerSections = document.querySelectorAll('.drawer-section');

    if (!drawer || !drawerOverlay || !drawerTitleText) return;
    
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
    
    
    setTimeout(refreshIndicators, 50);
}

export function closeDrawer() {
    const drawer = document.getElementById('bottom-drawer');
    const drawerOverlay = document.getElementById('drawer-overlay');
    drawer?.classList.remove('active');
    drawerOverlay?.classList.remove('active');
    setTimeout(() => {
        document.getElementById('content')?.classList.remove('blurred');
    }, 300);
}

function refreshIndicators() {
    
    const rows = document.querySelectorAll('.multi-button-row');
    rows.forEach(row => {
        const indicator = row.querySelector('.slide-indicator') as HTMLElement;
        const activeBtn = row.querySelector('.multi-btn.active') as HTMLElement;
        if (indicator && activeBtn) {
            indicator.style.width = `${activeBtn.offsetWidth}px`;
            indicator.style.left = `${activeBtn.offsetLeft}px`;
        }
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
    const drawerClose = document.getElementById('drawer-close-btn');

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
            
            try {
                const hasPermission = await invoke("check_uinput_permissions") as boolean;
                if (!hasPermission) {
                    showUinputPermissionModal();
                    return;
                }
            } catch (e) {
                
                showUinputPermissionModal();
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
        buttons.forEach(btn => {
            btn.addEventListener('click', async () => {
                buttons.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                
                
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

export async function initMultithreading() {
    const toggle = document.getElementById('multithread-toggle');
    const featureBtn = document.getElementById('multithread-btn');
    const slider = document.getElementById('multithread-slider') as HTMLInputElement | null;
    const modeRow = document.getElementById('multithread-mode-row') as HTMLElement | null;
    const section = document.getElementById('section-multithread');

    if (!toggle) return;

    const syncUi = (active: boolean, threads: number) => {
        toggle.classList.toggle('active', active);
        featureBtn?.classList.toggle('active', active);

        if (slider) {
            const clamped = Math.max(1, Math.min(16, threads || 1));
            slider.value = String(clamped);
            const valueEl = slider.parentElement?.querySelector('.visual-slider-value');
            if (valueEl) valueEl.textContent = formatThreadsLabel(clamped);
        }
    };

    const setSupportedFlag = (supported: boolean) => {
        document.documentElement.dataset.multithreadSupported = supported ? '1' : '0';
    };

    const applyUnsupportedUi = () => {
        setSupportedFlag(false);
        syncUi(false, 1);
        toggle.style.opacity = '0.45';
        toggle.style.cursor = 'not-allowed';
        toggle.setAttribute('title', 'Linux-only feature');
        featureBtn?.classList.remove('active');
        featureBtn?.setAttribute('title', 'Multi-Instance is available only on Linux for now');

        if (slider) {
            slider.disabled = true;
            slider.style.opacity = '0.45';
        }
        if (modeRow) {
            modeRow.style.pointerEvents = 'none';
            modeRow.style.opacity = '0.45';
        }

        if (section && !document.getElementById('multithread-platform-note')) {
            const note = document.createElement('div');
            note.id = 'multithread-platform-note';
            note.className = 'settings-list-item';
            note.style.pointerEvents = 'none';
            note.style.background = 'rgba(255, 193, 7, 0.05)';
            note.style.borderTopColor = 'rgba(255, 193, 7, 0.15)';
            note.style.margin = '2px 0 8px';
            note.style.padding = '8px 10px';
            note.style.alignItems = 'flex-start';
            note.style.gap = '8px';
            note.innerHTML = `
                <span class="icon" style="font-size: 14px; color: #FFC107; margin-top: 1px;">&#58485;</span>
                <span style="font-size: 10px; color: #FFC107; line-height: 1.5; opacity: 0.9;">Multi-Instance is temporarily disabled on Windows and is available only on Linux.</span>
            `;
            section.querySelector('.settings-items-list')?.prepend(note);
        }
    };

    try {
        const state = await invoke("get_multithread_state") as { supported?: boolean; active?: boolean; threads?: number };
        const supported = Boolean(state.supported);
        if (!supported) {
            applyUnsupportedUi();
            return;
        }
        setSupportedFlag(true);
        syncUi(Boolean(state.active), Number(state.threads || 1));
    } catch (e) {
        console.warn("Failed to load multithread state", e);
        applyUnsupportedUi();
        return;
    }

    toggle.addEventListener('click', async () => {
        try {
            const active = await invoke("toggle_multithread") as boolean;
            const threads = slider ? (parseInt(slider.value, 10) || 1) : 1;
            syncUi(active, threads);
        } catch (e) {
            console.error("Failed to toggle multithread", e);
        }
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

        const resetHandle = () => {
            isDragging = false;
            currentX = 0;
            handle.style.transform = 'translateX(0)';
            handle.style.transition = 'transform 0.3s cubic-bezier(0.16, 1, 0.3, 1)';
            if (progress) {
                progress.style.width = '0';
                progress.style.transition = 'width 0.3s cubic-bezier(0.16, 1, 0.3, 1)';
            }
            if (text) text.textContent = 'Slide to confirm';
            track.classList.remove('unlocked');
        };

        const startDragging = (clientX: number) => {
            isDragging = true;
            maxDelta = track.clientWidth - handle.clientWidth - 8;
            startX = clientX - currentX;
            handle.style.transition = 'none';
            if (progress) progress.style.transition = 'none';
        };

        const moveDragging = (clientX: number) => {
            if (!isDragging) return;
            
            let delta = clientX - startX;
            if (delta < 0) delta = 0;
            if (delta > maxDelta) delta = maxDelta;
            
            currentX = delta;
            handle.style.transform = `translateX(${delta}px)`;
            if (progress) {
                progress.style.width = `${delta + 20}px`; 
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


function showUinputPermissionModal() {
    const modal = document.getElementById('uinput-permission-modal');
    const grantBtn = document.getElementById('uinput-grant-btn');
    const cancelBtn = document.getElementById('uinput-modal-cancel-btn');
    const closeBtn = document.getElementById('uinput-modal-close-btn');
    const statusText = document.getElementById('uinput-status');

    if (!modal) return;

    modal.style.display = 'flex';
    requestAnimationFrame(() => {
        modal.classList.add('active');
    });
    document.getElementById('content')?.classList.add('blurred');

    if (statusText) {
        statusText.textContent = '';
    }

    
    const handleGrantPermission = async () => {
        if (grantBtn) {
            (grantBtn as HTMLButtonElement).disabled = true;
            grantBtn.style.opacity = '0.5';
        }

        if (statusText) {
            statusText.textContent = 'Requesting permission...';
            statusText.style.color = 'var(--text-dim)';
        }

        try {
            const result = await invoke("request_uinput_permissions") as boolean;
            
            if (result && statusText) {
                statusText.textContent = '✓ Permission granted! You can now use Mouse Jiggler.';
                statusText.style.color = 'var(--accent)';
                
                
                setTimeout(() => {
                    hideUinputPermissionModal();
                    
                    const jigglerToggle = document.getElementById('jiggler-toggle');
                    if (jigglerToggle) {
                        jigglerToggle.click();
                    }
                }, 1500);
            }
        } catch (error) {
            if (statusText) {
                statusText.textContent = `✗ Failed: ${error}`;
                statusText.style.color = '#ff4444';
            }
            if (grantBtn) {
                (grantBtn as HTMLButtonElement).disabled = false;
                grantBtn.style.opacity = '1';
            }
        }
    };

    
    if (grantBtn) {
        grantBtn.onclick = handleGrantPermission;
    }

    const closeModal = () => {
        hideUinputPermissionModal();
    };

    if (cancelBtn) {
        cancelBtn.onclick = closeModal;
    }
    if (closeBtn) {
        closeBtn.onclick = closeModal;
    }
}


function hideUinputPermissionModal() {
    const modal = document.getElementById('uinput-permission-modal');
    if (!modal) return;

    modal.classList.remove('active');
    setTimeout(() => {
        modal.style.display = 'none';
        document.getElementById('content')?.classList.remove('blurred');
    }, 300);
}
