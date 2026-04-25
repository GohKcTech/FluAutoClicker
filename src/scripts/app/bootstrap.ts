import { initDrawer, initJiggler, initMultithreading, initTimingModal } from "../drawer";
import { initHotkeysEditor } from "../hotkeys";
import { initKeyboard } from "../keyboard";
import { initMacro } from "../macro";
import { initMouseSettings } from "../mouse";
import { initProfiles } from "../profiles";
import { loadPersistedSettings, initSettingsPersistence } from "../settings-persistence";
import { initStartupModals } from "../startup-modals";
import { initTheme } from "../theme";
import { initInputs } from "../ui";
import { initUpdateChecks } from "../update-check";
import { initWindowEffects } from "../window-effects";
import { initCpsTestWindow, initDrawerLaunchers } from "./quick-actions";
import { initStartStopControls } from "./start-stop";
import { initTabAndToggleUi } from "./toggle-groups";
import { initWindowControls } from "./window-controls";
import { initKeyboardPartyMode } from "./easter-egg";
import { initNativeShellGuards } from "./native-shell-guards";

export async function bootstrapApp() {
    initNativeShellGuards();

    await loadPersistedSettings();
    await initWindowEffects();

    initKeyboard();
    initDrawer();
    initTheme();
    initInputs();
    initMacro();
    initMouseSettings();
    initJiggler();
    initMultithreading();
    void initHotkeysEditor();
    void initSettingsPersistence();
    void initProfiles();

    initCpsTestWindow();
    const toggleMouseClicker = initStartStopControls();
    initTimingModal(toggleMouseClicker);
    initWindowControls();
    initTabAndToggleUi();
    initDrawerLaunchers();
    initKeyboardPartyMode();
    initStartupModals();
    void initUpdateChecks();
}
