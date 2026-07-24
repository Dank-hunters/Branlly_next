import { describe, expect, it } from "vitest";
import { addDiscovered, radialPositions } from "./launcher";

describe("launcher layout", () => {
	it.each([1, 4, 8, 12, 16])(
		"lays out %s items without duplicate positions",
		(count) => {
			const positions = radialPositions(count);
			expect(positions).toHaveLength(count);
			expect(
				new Set(positions.map((item) => `${item.ring}:${item.angle}`)).size,
			).toBe(count);
		},
	);
	it("keeps order and prevents exact duplicates", () => {
		const app = {
			id: "firefox",
			name: "Firefox",
			icon: null,
			launch: { identifier: "firefox", arguments: [] },
		};
		expect(addDiscovered(addDiscovered([], app), app)).toHaveLength(1);
	});
});
