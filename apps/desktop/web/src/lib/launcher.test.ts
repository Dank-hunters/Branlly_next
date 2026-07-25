import { describe, expect, it } from "vitest";
import { addDiscovered, filterDiscovered, radialPositions } from "./launcher";

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
	it("searches the complete catalogue by name, identifier and target", () => {
		const apps = [
			"Discord",
			"Opera GX",
			"Visual Studio",
			"Éditeur",
			"Firefox",
			"Terminal",
		].map((name, index) => ({
			id: `app-${index}`,
			name,
			icon: null,
			launch: {
				identifier: index === 2 ? "devenv.exe" : name.toLowerCase(),
				arguments: [],
			},
		}));
		expect(filterDiscovered(apps, "")).toHaveLength(6);
		expect(filterDiscovered(apps, "opera").map((app) => app.name)).toEqual([
			"Opera GX",
		]);
		expect(filterDiscovered(apps, "editeur").map((app) => app.name)).toEqual([
			"Éditeur",
		]);
		expect(filterDiscovered(apps, "devenv").map((app) => app.name)).toEqual([
			"Visual Studio",
		]);
		expect(filterDiscovered(apps, "absent")).toEqual([]);
	});

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
