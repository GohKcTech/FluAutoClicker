import { invoke } from "@tauri-apps/api/core";
import { notify } from "./notifications";

type InvokeOptions<T> = {
    fallback?: T;
    notifyOnError?: boolean;
    errorMessage?: string;
};

export function formatInvokeError(error: unknown): string {
    if (error instanceof Error) return error.message;
    if (typeof error === "string") return error;
    try {
        return JSON.stringify(error);
    } catch {
        return "Unknown error";
    }
}

export async function safeInvoke<T>(
    command: string,
    args?: Record<string, unknown>,
    options: InvokeOptions<T> = {},
): Promise<T> {
    try {
        return await invoke<T>(command, args);
    } catch (error) {
        const detail = formatInvokeError(error);
        const message = options.errorMessage ? `${options.errorMessage}: ${detail}` : detail;
        console.error(`Tauri command failed: ${command}`, error);

        if (options.notifyOnError) {
            notify(message, "error", 2800);
        }

        if ("fallback" in options) {
            return options.fallback as T;
        }

        throw new Error(message);
    }
}
