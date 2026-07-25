import { describe, expect, it } from "vitest";
import { addDiscovered, filterDiscovered, isBrowser, launchUrlArgument, radialPositions } from "./launcher";

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

	it("recognizes browsers and accepts only web URLs", () => {
		const browser = { id: "opera", name: "Opera GX", icon: null, launch: { identifier: "C:\\Apps\\opera.exe", arguments: [] } };
		expect(isBrowser(browser)).toBe(true);
		expect(launchUrlArgument("https://branlly.test/music")).toBe("https://branlly.test/music");
		expect(launchUrlArgument("file:///secret")).toBeNull();
		expect(addDiscovered([], browser, "https://branlly.test")[0].launch).toEqual({ kind: "application", identifier: browser.launch.identifier, arguments: ["https://branlly.test"] });
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
