import { getCurrentWindow } from "@tauri-apps/api/window";
import { createSlideIndicator, updateIndicator } from "./utils";
import { initWebviewErrorModal } from "./webview-error-modal";
import { initWindowEffects } from "./window-effects";

const appWindow = getCurrentWindow();

window.addEventListener("DOMContentLoaded", async () => {
    initWebviewErrorModal();
    await initWindowEffects();

    const closeBtn = document.getElementById('close-btn');
    const minimizeBtn = document.getElementById('minimize-btn');
    const modeRow = document.getElementById('cps-mode-row');
    const timeRow = document.getElementById('cps-time-row');
    const clickArea = document.getElementById('cps-click-area');
    const mainDisplay = document.getElementById('cps-main-display');
    const maxDisplay = document.getElementById('cps-max-display');
    const minDisplay = document.getElementById('cps-min-display');
    const statusHint = document.getElementById('cps-status-hint');
    const resultText = document.getElementById('cps-result-text');
    const retryBtn = document.getElementById('cps-retry-btn');

    
    let mode: 'click' | 'hold' = 'click';
    let clickCount = 0;
    let isRunning = false;
    let isFinished = false;
    let startTime = 0;
    let duration = 5;
    let timer: any = null;
    let facts: string[] = ["Loading facts..."];
    let lastCps = 0;

    
    let intervals: number[] = [];
    let lastClickTime = 0;

    
    let holdStartTime = 0;
    let isHolding = false;
    let totalHoldTime = 0;

    
    function calcStats() {
        if (intervals.length === 0) return { median: 0, min: 0, max: 0, accuracy: 0 };

        const sorted = [...intervals].sort((a, b) => a - b);
        const min = sorted[0];
        const max = sorted[sorted.length - 1];

        
        const mid = Math.floor(sorted.length / 2);
        const median = sorted.length % 2 !== 0
            ? sorted[mid]
            : (sorted[mid - 1] + sorted[mid]) / 2;

        
        const mean = intervals.reduce((s, v) => s + v, 0) / intervals.length;
        const variance = intervals.reduce((s, v) => s + (v - mean) ** 2, 0) / intervals.length;
        const stddev = Math.sqrt(variance);
        
        const cv = mean > 0 ? (stddev / mean) * 100 : 0;
        const accuracy = Math.max(0, Math.min(100, 100 - cv));

        return { median, min, max, accuracy };
    }

    function formatMs(ms: number): string {
        if (ms < 10) return `${ms.toFixed(1)}ms`;
        if (ms < 1000) return `${Math.round(ms)}ms`;
        return `${(ms / 1000).toFixed(2)}s`;
    }

    
    function updateDisplayLoop() {
        if (!isFinished && mainDisplay) {
            if (isRunning) {
                if (mode === 'click') {
                    const stats = calcStats();
                    mainDisplay.textContent = clickCount.toString();
                    if (maxDisplay) maxDisplay.textContent = `MAX ${formatMs(stats.max)}`;
                    if (minDisplay) minDisplay.textContent = `MIN ${formatMs(stats.min)}`;
                } else {
                    
                    if (isHolding) {
                        const currentHold = Date.now() - holdStartTime;
                        mainDisplay.textContent = `${currentHold}ms`;
                    } else {
                        mainDisplay.textContent = totalHoldTime > 0 ? `${totalHoldTime}ms` : "HOLD!";
                    }
                    if (maxDisplay) maxDisplay.textContent = '';
                    if (minDisplay) minDisplay.textContent = '';
                }
            }
            requestAnimationFrame(updateDisplayLoop);
        }
    }

    function showRandomFact() {
        if (facts.length > 0) {
            const fact = facts[Math.floor(Math.random() * facts.length)];
            if (resultText) resultText.textContent = fact;
        }
    }

    try {
        const response = await fetch('/assets/data/facts.json');
        facts = await response.json();
        showRandomFact();
    } catch (e) {
        facts = ["FluAutoClicker: High-performance."];
        showRandomFact();
    }

    closeBtn?.addEventListener('click', () => appWindow.close());
    minimizeBtn?.addEventListener('click', () => appWindow.minimize());

    function applyTheme() {
        const mode_ = localStorage.getItem('flu-theme-mode') || 'solid';
        const color = localStorage.getItem('flu-theme-color') || '#77B6DD';
        document.body.classList.toggle('theme-rainbow', mode_ === 'rainbow');
        if (mode_ !== 'rainbow') {
            document.documentElement.style.setProperty('--accent', color);
            const dimColor = color.length === 7 ? `${color}66` : color;
            document.documentElement.style.setProperty('--accent-dim', dimColor);
        }
    }

    window.addEventListener('storage', (e) => {
        if (e.key?.startsWith('flu-theme')) applyTheme();
    });
    applyTheme();

    
    if (modeRow) {
        const activeBtn = modeRow.querySelector('.multi-btn.active');
        if (activeBtn) createSlideIndicator(modeRow, activeBtn as HTMLElement, true);

        modeRow.querySelectorAll('.multi-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                if (isRunning || isFinished) return;
                modeRow.querySelectorAll('.multi-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                updateIndicator(modeRow, btn as HTMLElement);
                mode = (btn.getAttribute('data-value') || 'click') as 'click' | 'hold';
                resetTest();
            });
        });
    }

    
    if (timeRow) {
        const activeBtn = timeRow.querySelector('.multi-btn.active');
        if (activeBtn) createSlideIndicator(timeRow, activeBtn as HTMLElement, true);

        timeRow.querySelectorAll('.multi-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                if (isRunning || isFinished) return;
                timeRow.querySelectorAll('.multi-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                updateIndicator(timeRow, btn as HTMLElement);
                duration = parseInt(btn.getAttribute('data-value') || '5');
                resetTest();
            });
        });
    }

    
    function startClickTest() {
        clickCount = 1;
        isRunning = true;
        isFinished = false;
        startTime = Date.now();
        lastClickTime = Date.now();
        intervals = [];
        if (mainDisplay) mainDisplay.textContent = "1";
        if (maxDisplay) maxDisplay.textContent = '—';
        if (minDisplay) minDisplay.textContent = '—';
        if (statusHint) statusHint.textContent = `Time left: ${duration}s`;
        if (retryBtn) retryBtn.classList.remove('ready');

        requestAnimationFrame(updateDisplayLoop);

        timer = setInterval(() => {
            const elapsed = (Date.now() - startTime) / 1000;
            const remaining = Math.max(0, duration - elapsed);
            lastCps = clickCount / elapsed;

            const stats = calcStats();
            if (statusHint) {
                const accText = intervals.length >= 2
                    ? `Accuracy: ~${stats.accuracy.toFixed(1)}%`
                    : `Time left: ${remaining.toFixed(1)}s`;
                statusHint.textContent = `${accText} • ${lastCps.toFixed(1)} CPS`;
            }

            if (lastCps > 15) {
                document.body.classList.add('high-performance-mode');
            }

            if (remaining <= 0) {
                endClickTest();
            }
        }, 100);
    }

    function endClickTest() {
        isRunning = false;
        isFinished = true;
        clearInterval(timer);
        const finalCps = clickCount / duration;
        const stats = calcStats();
        if (mainDisplay) mainDisplay.textContent = `${finalCps.toFixed(1)} CPS`;
        if (maxDisplay) maxDisplay.textContent = `MAX ${formatMs(stats.max)}`;
        if (minDisplay) minDisplay.textContent = `MIN ${formatMs(stats.min)}`;
        if (statusHint) {
            const accText = intervals.length >= 2
                ? `Accuracy: ~${stats.accuracy.toFixed(1)}%`
                : 'No interval data';
            statusHint.textContent = `${accText} • ${clickCount} clicks in ${duration}s`;
        }

        showRandomFact();
        if (retryBtn) retryBtn.classList.add('ready');
        document.body.classList.remove('high-performance-mode');
    }

    
    function startHoldTest() {
        isRunning = true;
        isFinished = false;
        totalHoldTime = 0;
        startTime = Date.now();
        if (mainDisplay) mainDisplay.textContent = "HOLD!";
        if (statusHint) statusHint.textContent = `Hold the button • ${duration}s timer`;
        if (retryBtn) retryBtn.classList.remove('ready');

        requestAnimationFrame(updateDisplayLoop);

        timer = setInterval(() => {
            const elapsed = (Date.now() - startTime) / 1000;
            const remaining = Math.max(0, duration - elapsed);
            const holdFraction = totalHoldTime / 1000 / Math.max(elapsed, 0.001);
            lastCps = holdFraction;

            if (statusHint) {
                const holdSecs = (totalHoldTime / 1000).toFixed(2);
                statusHint.textContent = `Time left: ${remaining.toFixed(1)}s • Held: ${holdSecs}s (${(holdFraction * 100).toFixed(1)}%)`;
            }

            if (remaining <= 0) {
                endHoldTest();
            }
        }, 100);
    }

    function endHoldTest() {
        if (isHolding) {
            totalHoldTime += Date.now() - holdStartTime;
            isHolding = false;
        }
        isRunning = false;
        isFinished = true;
        clearInterval(timer);

        const totalHoldSecs = totalHoldTime / 1000;
        const holdPercent = (totalHoldTime / 1000 / duration) * 100;
        if (mainDisplay) mainDisplay.textContent = `${holdPercent.toFixed(1)}%`;
        if (statusHint) statusHint.textContent = `Held ${totalHoldSecs.toFixed(2)}s out of ${duration}s`;

        showRandomFact();
        if (retryBtn) retryBtn.classList.add('ready');
        document.body.classList.remove('high-performance-mode');
    }

    
    function resetTest() {
        isRunning = false;
        isFinished = false;
        isHolding = false;
        clearInterval(timer);
        clickCount = 0;
        totalHoldTime = 0;
        lastCps = 0;
        intervals = [];
        lastClickTime = 0;
        if (mainDisplay) mainDisplay.textContent = mode === 'click' ? "CLICK!" : "HOLD!";
        if (maxDisplay) maxDisplay.textContent = '—';
        if (minDisplay) minDisplay.textContent = '—';
        if (statusHint) statusHint.textContent = mode === 'click'
            ? `READY TO START • ${duration}S`
            : `READY TO HOLD • ${duration}S`;
        showRandomFact();
        if (retryBtn) retryBtn.classList.remove('ready');
        document.body.classList.remove('high-performance-mode');
    }

    
    clickArea?.addEventListener('pointerdown', (e) => {
        if (isFinished) return;

        if (mode === 'click') {
            if (!isRunning && clickCount === 0) {
                startClickTest();
            } else if (isRunning) {
                const now = Date.now();
                const interval = now - lastClickTime;
                intervals.push(interval);
                lastClickTime = now;
                clickCount++;
            }

            if (lastCps <= 15) {
                clickArea.classList.remove('clicking');
                void clickArea.offsetWidth;
                clickArea.classList.add('clicking');
                setTimeout(() => clickArea.classList.remove('clicking'), 100);

                const ripple = document.createElement('div');
                ripple.className = 'cps-ripple';
                const rect = clickArea.getBoundingClientRect();
                ripple.style.left = `${e.clientX - rect.left - 60}px`;
                ripple.style.top = `${e.clientY - rect.top - 60}px`;
                clickArea.appendChild(ripple);
                setTimeout(() => ripple.remove(), 500);
            }
        } else {
            if (!isRunning && !isFinished) {
                startHoldTest();
            }
            if (isRunning) {
                isHolding = true;
                holdStartTime = Date.now();
            }
        }
    });

    clickArea?.addEventListener('pointerup', () => {
        if (mode === 'hold' && isHolding && isRunning) {
            totalHoldTime += Date.now() - holdStartTime;
            isHolding = false;
        }
    });

    clickArea?.addEventListener('pointercancel', () => {
        if (mode === 'hold' && isHolding && isRunning) {
            totalHoldTime += Date.now() - holdStartTime;
            isHolding = false;
        }
    });

    clickArea?.addEventListener('pointerleave', () => {
        if (mode === 'hold' && isHolding && isRunning) {
            totalHoldTime += Date.now() - holdStartTime;
            isHolding = false;
        }
    });

    retryBtn?.addEventListener('click', resetTest);
    resetTest();
});
