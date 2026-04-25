import { invoke } from "@tauri-apps/api/core";
import { updateSliderFill, updateIndicator } from "./utils";

function formatThreadsLabel(raw: number | string): string {
    const count = Number(raw) || 0;
    return `${count} ${count === 1 ? 'Thread' : 'Threads'}`;
}

export function initInputs() {
    function updateCPS() {
        const h = parseInt((document.getElementById('mouse-hours') as HTMLInputElement)?.value) || 0;
        const m = parseInt((document.getElementById('mouse-minutes') as HTMLInputElement)?.value) || 0;
        const s = parseInt((document.getElementById('mouse-seconds') as HTMLInputElement)?.value) || 0;
        const ms = parseInt((document.getElementById('mouse-ms') as HTMLInputElement)?.value) || 0;
        const totalMs = h * 3600000 + m * 60000 + s * 1000 + ms;
        const isInfinite = totalMs < 3;
        const cpsEl = document.querySelector('.start-cps');
        if (cpsEl) {
            let cps = 0;
            if (!isInfinite) {
                cps = 1000 / totalMs;
                cpsEl.textContent = `~ ${cps.toFixed(1)} CPS`;
            } else {
                cpsEl.textContent = `~ \u221E CPS`;
                cps = 10000;
            }
            invoke("set_cps", { cps: Math.round(cps) });
        }
    }

    ['mouse-hours', 'mouse-minutes', 'mouse-seconds', 'mouse-ms', 'mouse-variation'].forEach(id => {
        document.getElementById(id)?.addEventListener('input', updateCPS);
    });
    
    
    updateCPS();

    
    const mouseSlider = document.getElementById('mouse-slider') as HTMLInputElement;
    const mouseMs = document.getElementById('mouse-ms') as HTMLInputElement;
    if (mouseSlider && mouseMs) {
        mouseSlider.addEventListener('input', () => {
            mouseMs.value = mouseSlider.value;
            updateCPS();
            updateSliderFill(mouseSlider);
        });
        mouseMs.addEventListener('input', () => {
            const val = parseInt(mouseMs.value) || 0;
            mouseSlider.value = Math.min(val, 1000).toString();
            updateCPS();
            updateSliderFill(mouseSlider);
        });
        updateSliderFill(mouseSlider);
    }

    
    const kbSlider = document.getElementById('kb-slider') as HTMLInputElement;
    const kbMs = document.getElementById('kb-ms') as HTMLInputElement;
    const kbVariation = document.getElementById('kb-variation') as HTMLInputElement;
    if (kbSlider && kbMs) {
        function updateKbCps() {
            const h = parseInt((document.getElementById('kb-hours') as HTMLInputElement)?.value) || 0;
            const m = parseInt((document.getElementById('kb-minutes') as HTMLInputElement)?.value) || 0;
            const s = parseInt((document.getElementById('kb-seconds') as HTMLInputElement)?.value) || 0;
            const ms = parseInt((document.getElementById('kb-ms') as HTMLInputElement)?.value) || 0;
            const totalMs = h * 3600000 + m * 60000 + s * 1000 + ms;
            const cps = totalMs > 0 ? Math.round(1000 / totalMs) : 10000;
            invoke("set_keyboard_cps", { cps });
            
            const variation = parseInt(kbVariation?.value) || 0;
            invoke("set_keyboard_variation_ms", { variation });
        }

        kbSlider.addEventListener('input', () => {
            kbMs.value = kbSlider.value;
            updateSliderFill(kbSlider);
            updateKbCps();
        });
        kbMs.addEventListener('input', () => {
            const val = parseInt(kbMs.value) || 0;
            kbSlider.value = Math.min(val, 1000).toString();
            updateSliderFill(kbSlider);
            updateKbCps();
        });
        
        if (kbVariation) {
            kbVariation.addEventListener('input', () => {
                const variation = parseInt(kbVariation.value) || 0;
                invoke("set_keyboard_variation_ms", { variation });
            });
        }
        updateKbCps();
    }

    
    const jigglerSlider = document.getElementById('jiggler-slider') as HTMLInputElement;
    if (jigglerSlider) {
        jigglerSlider.addEventListener('input', () => {
            const valEl = jigglerSlider.previousElementSibling?.querySelector('.visual-slider-value');
            if (valEl) valEl.textContent = jigglerSlider.value + 's';
            updateSliderFill(jigglerSlider);
            
            invoke("set_jiggler_interval", { interval: parseInt(jigglerSlider.value) * 1000 });
        });
        updateSliderFill(jigglerSlider);
    }

    const jigglerDistanceSlider = document.getElementById('jiggler-distance-slider') as HTMLInputElement;
    if (jigglerDistanceSlider) {
        jigglerDistanceSlider.addEventListener('input', () => {
            const valEl = jigglerDistanceSlider.parentElement?.querySelector('.visual-slider-value');
            if (valEl) valEl.textContent = jigglerDistanceSlider.value + 'px';
            updateSliderFill(jigglerDistanceSlider);
            invoke("set_jiggler_distance", { distance: parseInt(jigglerDistanceSlider.value) });
        });
        updateSliderFill(jigglerDistanceSlider);
    }

    const multithreadSlider = document.getElementById('multithread-slider') as HTMLInputElement;
    if (multithreadSlider) {
        multithreadSlider.addEventListener('input', () => {
            const valEl = multithreadSlider.parentElement?.querySelector('.visual-slider-value');
            if (valEl) valEl.textContent = formatThreadsLabel(multithreadSlider.value);
            updateSliderFill(multithreadSlider);
            invoke("set_threads_count", { count: parseInt(multithreadSlider.value) });
        });
        const valEl = multithreadSlider.parentElement?.querySelector('.visual-slider-value');
        if (valEl) valEl.textContent = formatThreadsLabel(multithreadSlider.value);
        updateSliderFill(multithreadSlider);
    }

    
    const numInputs = document.querySelectorAll('input[type="number"]');
    numInputs.forEach(input => {
        const inputEl = input as HTMLInputElement;
        inputEl.addEventListener('focus', () => {
            if (inputEl.value === '0') {
                inputEl.value = '';
            } else {
                inputEl.select();
            }
        });
        inputEl.addEventListener('blur', () => {
            if (inputEl.value === '') {
                inputEl.value = '0';
                inputEl.dispatchEvent(new Event('input'));
            }
        });
        inputEl.addEventListener('wheel', (e: any) => {
            e.preventDefault();
            const step = parseFloat(inputEl.getAttribute('step') || '1');
            const val = parseFloat(inputEl.value) || 0;
            inputEl.value = (e.deltaY < 0 ? val + step : Math.max(0, val - step)).toString();
            inputEl.dispatchEvent(new Event('input'));
        }, { passive: false });
    });

    updateCPS();
}

export function initTabs(updateIndicatorFn: typeof updateIndicator, createSlideIndicatorFn: any) {
    const tabContainer = document.querySelector('.mode-tabs');
    const tabs = document.querySelectorAll('.tab');
    const sections = document.querySelectorAll('.content-section');

    if (tabContainer) {
        const activeTab = tabContainer.querySelector('.tab.active');
        if (activeTab) createSlideIndicatorFn(tabContainer, activeTab, true);
    }

    tabs.forEach(tab => {
        tab.addEventListener('click', async () => {
            const targetTab = tab.getAttribute('data-tab');

            
            const startBtn = document.getElementById('start-btn');
            const isRunning = startBtn?.classList.contains('running');
            const activeTabEl = document.querySelector('.mode-tabs .tab.active');
            const activeTabText = activeTabEl?.textContent?.trim().toLowerCase();

            
            if (isRunning && activeTabText && ['mouse', 'keyboard', 'macro'].includes(activeTabText)) {
                if (targetTab !== activeTabText) {
                    
                    if (activeTabEl) {
                        activeTabEl.classList.remove('lock-flash');
                        void (activeTabEl as HTMLElement).offsetWidth;
                        activeTabEl.classList.add('lock-flash');
                        setTimeout(() => {
                            activeTabEl.classList.remove('lock-flash');
                        }, 400);
                    }
                    return;
                }
            }

            
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            if (tabContainer) updateIndicatorFn(tabContainer, tab);

            const featureBar = document.getElementById('mouse-feature-bar');
            if (featureBar) {
                featureBar.style.display = targetTab === 'mouse' ? 'flex' : 'none';
            }

            sections.forEach(section => {
                section.classList.remove('active');
                if (section.id === `${targetTab}-section`) {
                    section.classList.add('active');
                    requestAnimationFrame(() => {
                        section.querySelectorAll('.toggle-row, .multi-button-row').forEach(row => {
                            const activeBtn = row.querySelector('.active');
                            if (activeBtn) {
                                updateIndicatorFn(row, activeBtn);
                            }
                        });
                    });
                }
            });
        });
    });
}


export function updateTabStates() {
    const startBtn = document.getElementById('start-btn');
    const isRunning = startBtn?.classList.contains('running');
    const activeTabEl = document.querySelector('.mode-tabs .tab.active');
    const activeTabText = activeTabEl?.textContent?.trim().toLowerCase();

    const tabs = document.querySelectorAll('.tab');
    tabs.forEach(tab => {
        const tabName = tab.getAttribute('data-tab');
        if (isRunning && activeTabText && ['mouse', 'keyboard', 'macro'].includes(activeTabText)) {
            
            if (tabName !== activeTabText) {
                tab.classList.add('disabled');
            } else {
                tab.classList.remove('disabled');
            }
        } else {
            
            tab.classList.remove('disabled');
        }
    });
}
