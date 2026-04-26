import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { emitSettingsChanged } from "./settings-persistence";

const UPDATE_FEED_URL = "https://raw.githubusercontent.com/Agzes/FluAutoClicker/next/docs/updates/latest-beta.json";
const GITHUB_PROJECT_URL = "https://github.com/Agzes/FluAutoClicker";
const GITHUB_RELEASES_URL = `${GITHUB_PROJECT_URL}/releases`;
const AUTO_CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
const UPDATE_SNOOZE_MS = 12 * 60 * 60 * 1000;
const MIN_AUTO_RETRY_DELAY_MS = 30 * 1000;
const LAST_CHECK_STORAGE_KEY = "flu-update-last-checked-at";
const FRONTEND_STATE_STORAGE_KEY = "flu-frontend-state";
const UPDATE_SNOOZE_STORAGE_KEY = "update_snoozed_until_by_version";
const BUILD_VERSION = __APP_VERSION__;

type UpdateManifest = {
  version: string;
  notes?: string;
  pub_date?: string;
  github_url?: string;
};

type UpdateStatus = "idle" | "checking" | "available" | "up-to-date" | "error";

let currentVersion = "0.0.0";
let lastManifest: UpdateManifest | null = null;
let isChecking = false;
let autoCheckTimer: number | null = null;

type FrontendState = Record<string, unknown>;

function normalizeVersion(version: string): string {
  return version.trim().replace(/^v/i, "");
}

function parseVersion(version: string) {
  const [coreAndPre] = normalizeVersion(version).split("+");
  const [core, prerelease = ""] = coreAndPre.split("-", 2);

  return {
    core: core.split(".").map((part) => {
      const parsed = Number.parseInt(part, 10);
      return Number.isFinite(parsed) ? parsed : 0;
    }),
    prerelease,
  };
}

function comparePrerelease(left: string, right: string): number {
  if (!left && !right) return 0;
  if (!left) return 1;
  if (!right) return -1;

  const leftParts = left.split(".");
  const rightParts = right.split(".");
  const maxLength = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < maxLength; index += 1) {
    const leftPart = leftParts[index];
    const rightPart = rightParts[index];

    if (leftPart === rightPart) continue;
    if (leftPart === undefined) return -1;
    if (rightPart === undefined) return 1;

    const leftNumber = Number.parseInt(leftPart, 10);
    const rightNumber = Number.parseInt(rightPart, 10);
    const leftIsNumber = String(leftNumber) === leftPart;
    const rightIsNumber = String(rightNumber) === rightPart;

    if (leftIsNumber && rightIsNumber) {
      return leftNumber === rightNumber ? 0 : leftNumber > rightNumber ? 1 : -1;
    }

    if (leftIsNumber) return -1;
    if (rightIsNumber) return 1;

    return leftPart.localeCompare(rightPart);
  }

  return 0;
}

function compareVersions(left: string, right: string): number {
  const parsedLeft = parseVersion(left);
  const parsedRight = parseVersion(right);
  const maxLength = Math.max(parsedLeft.core.length, parsedRight.core.length);

  for (let index = 0; index < maxLength; index += 1) {
    const leftPart = parsedLeft.core[index] ?? 0;
    const rightPart = parsedRight.core[index] ?? 0;

    if (leftPart === rightPart) continue;
    return leftPart > rightPart ? 1 : -1;
  }

  return comparePrerelease(parsedLeft.prerelease, parsedRight.prerelease);
}

function releaseUrl(manifest?: UpdateManifest | null): string {
  const remoteUrl = manifest?.github_url?.trim();
  return remoteUrl && remoteUrl.length > 0 ? remoteUrl : GITHUB_RELEASES_URL;
}

function setVersionTag() {
  const versionTag = document.getElementById("app-version-tag");
  if (versionTag) {
    versionTag.textContent = `v${currentVersion}`;
  }

  const currentVersionLabel = document.getElementById("updates-current-version");
  if (currentVersionLabel) {
    currentVersionLabel.textContent = `v${currentVersion}`;
  }
}

async function resolveCurrentVersion(): Promise<string> {
  try {
    return normalizeVersion(await getVersion());
  } catch (error) {
    console.warn("Failed to read Tauri app version, using build version", error);
    return normalizeVersion(BUILD_VERSION);
  }
}

function setChannelPill() {
  const channelLabel = document.getElementById("updates-current-channel");
  if (channelLabel) {
    channelLabel.textContent = "Current installed beta build. Feed: GitHub raw manifest.";
  }

  const feedBadge = document.getElementById("updates-feed-badge");
  if (feedBadge) {
    feedBadge.textContent = "latest-beta.json";
  }
}

function formatPublishedDate(value?: string): string {
  if (!value) return "Published date is not provided by the manifest.";

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return `Published: ${value}`;
  }

  return `Published: ${parsed.toLocaleString()}`;
}

function updateLatestBuildDetails(manifest?: UpdateManifest | null) {
  const latestVersion = document.getElementById("updates-latest-version");
  const releaseNotes = document.getElementById("updates-release-notes");

  if (!latestVersion || !releaseNotes) return;

  if (!manifest) {
    latestVersion.textContent = "Latest published build is not loaded yet";
    releaseNotes.textContent = "Run a check to fetch version, date, and release notes from the manifest.";
    return;
  }

  latestVersion.textContent = `Latest published build: v${normalizeVersion(manifest.version)}`;
  releaseNotes.textContent = manifest.notes?.trim()
    ? `${manifest.notes.trim()} ${formatPublishedDate(manifest.pub_date)}`
    : formatPublishedDate(manifest.pub_date);
}

function renderStatus(status: UpdateStatus, title: string, description: string) {
  const panel = document.getElementById("update-status-panel");
  const titleElement = document.getElementById("updates-status-title");
  const descElement = document.getElementById("updates-status-desc");
  const iconElement = document.getElementById("updates-status-icon");
  const checkButton = document.getElementById("update-check-btn") as HTMLButtonElement | null;
  const checkButtonLabel = document.getElementById("update-check-btn-label");

  panel?.setAttribute("data-update-status", status);

  if (titleElement) {
    titleElement.textContent = title;
  }

  if (descElement) {
    descElement.textContent = description;
  }

  if (iconElement) {
    iconElement.innerHTML =
      status === "available" ? "&#58437;" :
      status === "up-to-date" ? "&#58544;" :
      status === "error" ? "&#58485;" :
      status === "checking" ? "&#58546;" :
      "&#58437;";
  }

  if (checkButton) {
    checkButton.disabled = status === "checking";
  }

  if (checkButtonLabel) {
    checkButtonLabel.textContent = status === "checking" ? "CHECKING" : "CHECK";
  }
}

function readLastCheckAt(): number {
  const raw = localStorage.getItem(LAST_CHECK_STORAGE_KEY);
  const parsed = Number.parseInt(raw || "", 10);
  return Number.isFinite(parsed) ? parsed : 0;
}

function writeLastCheckAt(timestamp: number) {
  localStorage.setItem(LAST_CHECK_STORAGE_KEY, String(timestamp));
  emitSettingsChanged();
}

function asRecord(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return {};
  }
  return value as Record<string, unknown>;
}

function readFrontendState(): FrontendState {
  try {
    return asRecord(JSON.parse(localStorage.getItem(FRONTEND_STATE_STORAGE_KEY) || "{}"));
  } catch {
    return {};
  }
}

function writeFrontendState(state: FrontendState) {
  localStorage.setItem(FRONTEND_STATE_STORAGE_KEY, JSON.stringify(state));
  emitSettingsChanged();
}

function readUpdateSnoozes(): Record<string, number> {
  const rawSnoozes = asRecord(readFrontendState()[UPDATE_SNOOZE_STORAGE_KEY]);
  const snoozes: Record<string, number> = {};

  Object.entries(rawSnoozes).forEach(([version, value]) => {
    const timestamp = typeof value === "number" ? value : Number.parseInt(String(value), 10);
    if (Number.isFinite(timestamp)) {
      snoozes[version] = timestamp;
    }
  });

  return snoozes;
}

function isUpdateSnoozed(version: string) {
  const snoozedUntil = readUpdateSnoozes()[version] || 0;
  return snoozedUntil > Date.now();
}

function snoozeUpdateNotification(version: string) {
  const state = readFrontendState();
  const snoozes = readUpdateSnoozes();
  snoozes[version] = Date.now() + UPDATE_SNOOZE_MS;
  state[UPDATE_SNOOZE_STORAGE_KEY] = snoozes;
  writeFrontendState(state);
}

function shouldAutoCheck(): boolean {
  const lastCheckAt = readLastCheckAt();
  if (lastCheckAt <= 0) return true;
  return Date.now() - lastCheckAt >= AUTO_CHECK_INTERVAL_MS;
}

function nextAutoCheckDelay(): number {
  const lastCheckAt = readLastCheckAt();
  if (lastCheckAt <= 0) return AUTO_CHECK_INTERVAL_MS;
  return Math.max(MIN_AUTO_RETRY_DELAY_MS, AUTO_CHECK_INTERVAL_MS - (Date.now() - lastCheckAt));
}

async function openGitHubReleases() {
  await openUrl(releaseUrl(lastManifest));
}

function showModal(modal: HTMLElement | null) {
  if (!modal) return;

  modal.style.display = "flex";
  document.getElementById("content")?.classList.add("blurred");
  requestAnimationFrame(() => {
    modal.classList.add("active");
  });
}

function hideModal(modal: HTMLElement | null) {
  if (!modal) return;

  modal.classList.remove("active");
  window.setTimeout(() => {
    modal.style.display = "none";
    document.getElementById("content")?.classList.remove("blurred");
  }, 220);
}

function updateModalText(manifest: UpdateManifest) {
  const version = normalizeVersion(manifest.version);
  const versionLabel = document.getElementById("update-modal-version");
  const notesLabel = document.getElementById("update-modal-notes");
  const currentLabel = document.getElementById("update-modal-current");
  const publishedLabel = document.getElementById("update-modal-published");

  if (versionLabel) {
    versionLabel.textContent = `v${version}`;
  }

  if (notesLabel) {
    notesLabel.textContent = manifest.notes?.trim() || "A newer FluAutoClicker build is available.";
  }

  if (currentLabel) {
    currentLabel.textContent = `v${currentVersion} -> v${version}`;
  }

  if (publishedLabel) {
    publishedLabel.textContent = formatPublishedDate(manifest.pub_date);
  }
}

function showUpdateAvailableNotification(manifest: UpdateManifest, force = false) {
  const version = normalizeVersion(manifest.version);
  if (!force && isUpdateSnoozed(version)) return;

  updateModalText(manifest);
  showModal(document.getElementById("update-available-modal"));
}

async function fetchManifest(): Promise<UpdateManifest> {
  const response = await fetch(UPDATE_FEED_URL, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(`Unexpected status ${response.status}`);
  }

  const manifest = (await response.json()) as UpdateManifest;
  if (!manifest.version || !manifest.version.trim()) {
    throw new Error("Manifest does not contain a version.");
  }

  return manifest;
}

async function checkForUpdates(manual = false) {
  if (isChecking) return;
  isChecking = true;
  let checkSucceeded = false;
  let finalStatus: UpdateStatus = "checking";
  let finalTitle = "Checking manifest...";
  let finalDescription = "Fetching the hosted static JSON file and comparing versions.";
  renderStatus(finalStatus, finalTitle, finalDescription);

  try {
    const manifest = await fetchManifest();
    checkSucceeded = true;
    lastManifest = manifest;
    updateLatestBuildDetails(manifest);

    if (compareVersions(currentVersion, manifest.version) < 0) {
      finalStatus = "available";
      finalTitle = `Update v${normalizeVersion(manifest.version)} is available`;
      finalDescription = "A newer build was found in the hosted manifest. Use the GitHub button to open the release page.";
      renderStatus(finalStatus, finalTitle, finalDescription);

      showUpdateAvailableNotification(manifest, manual);

      return;
    }

    finalStatus = "up-to-date";
    finalTitle = "You are up to date";
    finalDescription = `Installed version v${currentVersion} matches the newest build from the manifest.`;
    renderStatus(finalStatus, finalTitle, finalDescription);
  } catch (error) {
    console.warn("Failed to check update manifest", error);
    finalStatus = "error";
    finalTitle = "Manifest is unavailable";
    finalDescription = "The version file could not be loaded right now. You can still open GitHub Releases manually.";
    renderStatus(finalStatus, finalTitle, finalDescription);
    updateLatestBuildDetails(lastManifest);
  } finally {
    isChecking = false;
    if (checkSucceeded) {
      writeLastCheckAt(Date.now());
    }
    renderStatus(finalStatus, finalTitle, finalDescription);
  }
}

function scheduleNextAutoCheck() {
  if (autoCheckTimer !== null) {
    window.clearTimeout(autoCheckTimer);
  }

  autoCheckTimer = window.setTimeout(() => {
    void checkForUpdates(false).finally(scheduleNextAutoCheck);
  }, nextAutoCheckDelay());
}

function bindActions() {
  const checkButton = document.getElementById("update-check-btn");
  const openGithubButton = document.getElementById("update-open-github-btn");
  const updateModal = document.getElementById("update-available-modal");
  const updateModalOpenButton = document.getElementById("update-modal-open-btn");
  const updateModalLaterButton = document.getElementById("update-modal-later-btn");

  checkButton?.addEventListener("click", () => {
    void checkForUpdates(true);
  });

  openGithubButton?.addEventListener("click", () => {
    void openGitHubReleases();
  });

  updateModalOpenButton?.addEventListener("click", () => {
    hideModal(updateModal);
    void openGitHubReleases();
  });

  updateModalLaterButton?.addEventListener("click", () => {
    if (lastManifest) {
      snoozeUpdateNotification(normalizeVersion(lastManifest.version));
    }
    hideModal(updateModal);
  });

  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible" && shouldAutoCheck()) {
      void checkForUpdates(false).finally(scheduleNextAutoCheck);
    }
  });
}

export async function initUpdateChecks() {
  currentVersion = normalizeVersion(BUILD_VERSION);
  setVersionTag();

  currentVersion = await resolveCurrentVersion();
  setVersionTag();
  setChannelPill();
  bindActions();
  updateLatestBuildDetails(null);
  renderStatus(
    "idle",
    "Version checks are ready",
    "Checks compare your installed version with the hosted static release manifest."
  );

  if (shouldAutoCheck()) {
    void checkForUpdates(false).finally(scheduleNextAutoCheck);
  } else {
    scheduleNextAutoCheck();
  }
}
