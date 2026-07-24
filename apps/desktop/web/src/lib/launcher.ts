export type LaunchItem = {
	id: string;
	kind: "application" | "routine";
	name: string;
	icon: string | null;
	order: number;
	platform: "linux" | "windows" | null;
	launch:
		| { kind: "application"; identifier: string; arguments: string[] }
		| { kind: "routine"; routine_id: string };
};
export type DiscoveredApplication = {
	id: string;
	name: string;
	icon: string | null;
	launch: { identifier: string; arguments: string[] };
};
export type RadialPosition = { angle: number; ring: number };
export function radialPositions(count: number): RadialPosition[] {
	if (!count) return [];
	const inner = Math.min(count, 8);
	return Array.from({ length: count }, (_, index) => {
		const ring = index < inner ? 0 : 1;
		const onRing = ring ? count - inner : inner;
		const offset = ring ? index - inner : index;
		return { ring, angle: -90 + (offset * 360) / onRing };
	});
}
export function addDiscovered(
	items: LaunchItem[],
	application: DiscoveredApplication,
): LaunchItem[] {
	if (
		items.some(
			(item) =>
				item.kind === "application" &&
				item.launch.kind === "application" &&
				item.launch.identifier === application.launch.identifier &&
				item.launch.arguments.join("\0") ===
					application.launch.arguments.join("\0"),
		)
	)
		return items;
	return [
		...items,
		{
			id: application.id,
			kind: "application",
			name: application.name,
			icon: application.icon,
			order: items.length,
			platform: null,
			launch: { kind: "application", ...application.launch },
		},
	];
}
