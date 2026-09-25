import { describe, it } from "node:test";
import assert from "node:assert";
import { intervalToMilliseconds, parseTimingSeconds } from "./interval.ts";

describe("intervalToMilliseconds", () => {
    it("preserves decimal seconds for interval settings persistence", () => {
        assert.strictEqual(parseTimingSeconds("0.9"), 0.9);
        assert.strictEqual(parseTimingSeconds("-0.9"), 0);
        assert.strictEqual(parseTimingSeconds("invalid"), 0);
    });

    it("should convert 0.9 seconds to 900 ms", () => {
        const result = intervalToMilliseconds("0", "0", "0.9", "0");
        assert.strictEqual(result, 900);
    });

    it("should convert 1.234 seconds + 5 ms to 1239 ms", () => {
        const result = intervalToMilliseconds("0", "0", "1.234", "5");
        assert.strictEqual(result, 1239);
    });

    it("should handle integer seconds", () => {
        const result = intervalToMilliseconds("0", "0", "2", "500");
        assert.strictEqual(result, 2500);
    });

    it("should handle additive hours, minutes, seconds, and milliseconds", () => {
        const result = intervalToMilliseconds("1", "30", "15", "250");
        assert.strictEqual(result, 5415250);
    });

    it("should handle decimal rounding for fractional seconds", () => {
        const result = intervalToMilliseconds("0", "0", "0.999", "0");
        assert.strictEqual(result, 999);
    });

    it("should handle invalid/negative seconds as 0", () => {
        assert.strictEqual(intervalToMilliseconds("0", "0", "", "0"), 0);
        assert.strictEqual(intervalToMilliseconds("0", "0", "abc", "0"), 0);
        assert.strictEqual(intervalToMilliseconds("0", "0", "-0.9", "0"), 0);
    });

    it("should handle invalid/negative hours and minutes", () => {
        const result = intervalToMilliseconds("-1", "0", "1", "0");
        assert.strictEqual(result, 1000);
    });

    it("should handle empty strings for all fields", () => {
        const result = intervalToMilliseconds("", "", "", "");
        assert.strictEqual(result, 0);
    });

    it("should handle zero values properly", () => {
        const result = intervalToMilliseconds("0", "0", "0", "0");
        assert.strictEqual(result, 0);
    });
});
