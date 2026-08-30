export function intervalToMilliseconds(
    hours: string,
    minutes: string,
    seconds: string,
    milliseconds: string,
): number {
    const h = Math.max(0, Number.parseInt(hours, 10) || 0);
    const m = Math.max(0, Number.parseInt(minutes, 10) || 0);
    const ms = Math.max(0, Number.parseInt(milliseconds, 10) || 0);

    const parsedSeconds = Number.parseFloat(seconds);
    const s = Number.isFinite(parsedSeconds) && parsedSeconds >= 0
        ? Math.round(parsedSeconds * 1000)
        : 0;

    return h * 3_600_000 + m * 60_000 + s + ms;
}
