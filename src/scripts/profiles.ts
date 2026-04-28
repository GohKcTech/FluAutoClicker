import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { formatInvokeError, safeInvoke } from "./invoke";
import { notify } from "./notifications";
import { applyPersistedConfig, persistCurrentSettings, type AppConfigFile } from "./settings-persistence";

type ProfileFile = {
    version: number;
    name: string;
    data: AppConfigFile;
};

let selectedProfile = "default";
let activeProfile = "default";

function getList() {
    return document.getElementById("profiles-list");
}

function getNameInput() {
    return document.getElementById("profile-name-input") as HTMLInputElement | null;
}

function timestampForFile() {
    return new Date().toISOString().replace(/[:.]/g, "-");
}

function safeFilePart(value: string) {
    return value.replace(/[^a-z0-9_-]+/gi, "_") || "profile";
}

async function saveJson(filename: string, payload: unknown) {
    const path = await save({
        defaultPath: filename,
        filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return false;

    await invoke("save_export_file", {
        path,
        contents: JSON.stringify(payload, null, 2),
    });
    return true;
}

function readJsonFile(onRead: (payload: unknown) => void | Promise<void>) {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.style.display = "none";
    input.addEventListener("change", async () => {
        const file = input.files?.[0];
        input.remove();
        if (!file) return;

        try {
            await onRead(JSON.parse(await file.text()));
        } catch (error) {
            notify(formatInvokeError(error), "error", 3200);
        }
    }, { once: true });
    document.body.appendChild(input);
    input.click();
}

async function refreshProfiles(preferredProfile?: string) {
    const list = getList();
    const activeLabel = document.getElementById("profiles-active-name");
    const nameInput = getNameInput();
    if (!list) return;

    const [profiles, config] = await Promise.all([
        safeInvoke<string[]>("list_profiles_cmd", undefined, {
            fallback: ["default"],
            notifyOnError: true,
            errorMessage: "Could not load profiles",
        }),
        safeInvoke<Pick<AppConfigFile, "active_profile">>("load_app_config", undefined, {
            fallback: { active_profile: "default" },
            notifyOnError: true,
            errorMessage: "Could not load active profile",
        }),
    ]);

    activeProfile = config.active_profile || "default";
    selectedProfile = preferredProfile || activeProfile;
    if (!profiles.includes(selectedProfile)) {
        selectedProfile = profiles.includes(activeProfile) ? activeProfile : profiles[0] || "default";
    }

    list.innerHTML = "";
    profiles.forEach((profile) => {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "settings-list-item profile-list-item";
        button.classList.toggle("active", profile === activeProfile);
        button.classList.toggle("selected", profile === selectedProfile && profile !== activeProfile);
        button.dataset.profileName = profile;
        button.innerHTML = `
            <div class="settings-item-icon"><span class="icon">&#58218;</span></div>
            <div class="settings-item-content">
                <span class="settings-item-title">${profile}</span>
                <span class="settings-item-desc">${profile === activeProfile ? "Active profile" : "Click to load"}</span>
            </div>
            ${profile === activeProfile ? '<span class="profile-status-badge">ACTIVE</span>' : ""}
        `;
        button.addEventListener("click", () => {
            void loadProfile(profile);
        });
        list.appendChild(button);
    });

    if (activeLabel) {
        activeLabel.textContent = activeProfile;
    }
    if (nameInput && !nameInput.matches(":focus")) {
        nameInput.value = selectedProfile;
    }
    syncProfileActionState();
}

async function saveProfile() {
    const name = getNameInput()?.value.trim() || selectedProfile || "default";
    const currentConfig = await persistCurrentSettings();
    const updated = await safeInvoke("save_profile_cmd", {
        name,
        config: currentConfig,
    }, { notifyOnError: true, errorMessage: "Could not save profile" });
    applyPersistedConfig(updated as any);
    await refreshProfiles(name);
    notify(`Profile "${name}" saved`, "success", 1800);
}

async function loadProfile(name: string) {
    const updated = await safeInvoke("load_profile_cmd", { name }, {
        notifyOnError: true,
        errorMessage: "Could not load profile",
    });
    selectedProfile = name;
    activeProfile = name;
    applyPersistedConfig(updated as any);
    await refreshProfiles(name);
    notify(`Profile "${name}" loaded`, "success", 1800);
}

async function renameProfile() {
    const nextName = getNameInput()?.value.trim() || "";
    const previousName = selectedProfile;
    await safeInvoke("rename_profile_cmd", {
        oldName: previousName,
        newName: nextName,
    }, {
        notifyOnError: true,
        errorMessage: "Could not rename profile",
    });
    selectedProfile = nextName;
    await refreshProfiles(nextName);
    notify(`Profile renamed to "${nextName}"`, "success", 1800);
}

async function deleteProfile() {
    const profile = selectedProfile;
    await safeInvoke("delete_profile_cmd", { name: profile }, {
        notifyOnError: true,
        errorMessage: "Could not delete profile",
    });
    const config = await safeInvoke<Pick<AppConfigFile, "active_profile">>("load_app_config", undefined, {
        fallback: { active_profile: "default" },
    });
    selectedProfile = "default";
    activeProfile = "default";
    applyPersistedConfig(config as any);
    await refreshProfiles("default");
    notify(`Profile "${profile}" deleted`, "info", 1800);
}

async function exportProfile() {
    const profileName = getNameInput()?.value.trim() || selectedProfile || activeProfile || "default";
    const profile = profileName === activeProfile
        ? {
            version: 1,
            name: profileName,
            data: {
                ...(await persistCurrentSettings()),
                active_profile: profileName,
            },
        } satisfies ProfileFile
        : await invoke<ProfileFile>("export_profile_cmd", { name: profileName });
    const saved = await saveJson(
        `fluautoclicker-profile-${safeFilePart(profile.name)}-${timestampForFile()}.json`,
        profile
    );
    if (saved) notify(`Profile "${profile.name}" exported`, "success", 1800);
}

function importProfile() {
    readJsonFile(async (payload) => {
        const updated = await invoke<AppConfigFile>("import_profile_cmd", { profile: payload });
        selectedProfile = updated.active_profile || "default";
        activeProfile = selectedProfile;
        applyPersistedConfig(updated);
        await refreshProfiles(selectedProfile);
        notify(`Profile "${selectedProfile}" imported`, "success", 2200);
    });
}

function syncProfileActionState() {
    const renameBtn = document.getElementById("profile-rename-btn") as HTMLButtonElement | null;
    const deleteBtn = document.getElementById("profile-delete-btn") as HTMLButtonElement | null;
    const isDefault = selectedProfile === "default";

    for (const button of [renameBtn, deleteBtn]) {
        if (!button) continue;
        button.disabled = isDefault;
        button.classList.toggle("disabled", isDefault);
        button.title = isDefault ? "The default profile cannot be changed by this action." : "";
    }
}

export function initProfiles() {
    const saveBtn = document.getElementById("profile-save-btn");
    const renameBtn = document.getElementById("profile-rename-btn");
    const deleteBtn = document.getElementById("profile-delete-btn");
    const reloadBtn = document.getElementById("profile-reload-btn");
    const exportBtn = document.getElementById("profile-export-btn");
    const importBtn = document.getElementById("profile-import-btn");
    const nameInput = getNameInput();
    if (!saveBtn || !renameBtn || !deleteBtn || !reloadBtn || !nameInput) return;

    saveBtn.addEventListener("click", () => {
        void saveProfile().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });
    renameBtn.addEventListener("click", () => {
        if ((renameBtn as HTMLButtonElement).disabled) return;
        void renameProfile().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });
    deleteBtn.addEventListener("click", () => {
        if ((deleteBtn as HTMLButtonElement).disabled) return;
        void deleteProfile().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });
    reloadBtn.addEventListener("click", () => {
        void refreshProfiles().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });
    exportBtn?.addEventListener("click", () => {
        void exportProfile().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });
    importBtn?.addEventListener("click", () => {
        importProfile();
    });
    nameInput.addEventListener("input", () => {
        selectedProfile = nameInput.value.trim() || activeProfile;
        syncProfileActionState();
    });
    nameInput.addEventListener("keydown", (event) => {
        if (event.key !== "Enter") return;
        event.preventDefault();
        void saveProfile().catch((error) => {
            notify(formatInvokeError(error), "error", 2800);
        });
    });

    void listen("profiles-updated", (event) => {
        const payload = event.payload as { active_profile?: string };
        void refreshProfiles(payload?.active_profile || selectedProfile);
    });

    void refreshProfiles().catch((error) => {
        notify(formatInvokeError(error), "error", 2800);
    });
}
