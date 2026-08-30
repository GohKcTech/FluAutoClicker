export function parseTimingSeconds(seconds: string): number {
    const parsedSeconds = Number.parseFloat(seconds);
    return Number.isFinite(parsedSeconds) && parsedSeconds >= 0 ? parsedSeconds : 0;
}

export function intervalToMilliseconds(
    hours: string,
    minutes: string,
    seconds: string,
    milliseconds: string,
): number {
    const h = Math.max(0, Number.parseInt(hours, 10) || 0);
    const m = Math.max(0, Number.parseInt(minutes, 10) || 0);
    const ms = Math.max(0, Number.parseInt(milliseconds, 10) || 0);

    const s = Math.round(parseTimingSeconds(seconds) * 1000);

    return h * 3_600_000 + m * 60_000 + s + ms;
}
