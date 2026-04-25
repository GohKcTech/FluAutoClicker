import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export function initCpsTestWindow() {
    document.getElementById("cps-test-btn")?.addEventListener("click", () => {
        new WebviewWindow("cps-test", {
            url: "cps.html",
            title: "CPS Test",
            width: 400,
            height: 500,
            resizable: false,
            decorations: false,
            transparent: true,
            center: true,
        });
    });
}

export function initDrawerLaunchers() {
    document.getElementById("jiggler-btn")?.addEventListener("click", async (event) => {
        event.stopPropagation();
        const { openDrawer } = await import("../drawer");
        openDrawer("section-jiggler", "Mouse Jiggler", "&#57987;");
    });

    document.getElementById("multithread-btn")?.addEventListener("click", async (event) => {
        event.stopPropagation();
        const { openDrawer } = await import("../drawer");
        openDrawer("section-multithread", "Multi Threading", "&#58548;");
    });
}
