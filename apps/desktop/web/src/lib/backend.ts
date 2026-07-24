import { Channel, invoke } from "@tauri-apps/api/core";
import type { DiscoveredApplication, LaunchItem } from "./launcher";

export type Mood = "sleepy" | "neutral" | "curious" | "happy" | "irritated";

export interface PlatformCapabilities {
	canListWindows: boolean;
	canFocusWindows: boolean;
	canPositionOverlay: boolean;
	canFollowPointer: boolean;
	canQueryNetwork: boolean;
	canQueryBluetooth: boolean;
	canDiscoverApplications: boolean;
}

export interface BootstrapStatus {
	model: string;
	mood: Mood;
	energy: number;
	capabilities: PlatformCapabilities;
	ollamaAvailable: boolean;
}

export type InvokeBackend = <T>(
	command: string,
	args?: Record<string, unknown>,
) => Promise<T>;

export interface WindowInfo {
	id: string;
	title: string;
	applicationId: string | null;
	processId: number | null;
}

export interface DeviceInfo {
	id: string;
	name: string;
	connected: boolean;
	paired: boolean;
}

export interface SystemSnapshot {
	network: "offline" | "local" | "online" | "unknown";
	bluetoothDevices: DeviceInfo[];
	connectedDevices: DeviceInfo[];
}

export interface WikiResult {
	title: string;
	description: string;
	url: string;
}

export type ChatEvent =
	| { type: "delta"; payload: string }
	| { type: "complete" }
	| { type: "error"; payload: string };

export const PREVIEW_STATUS: BootstrapStatus = {
	model: "qwen2.5:3b",
	mood: "neutral",
	energy: 65,
	ollamaAvailable: false,
	capabilities: {
		canListWindows: false,
		canFocusWindows: false,
		canPositionOverlay: false,
		canFollowPointer: false,
		canQueryNetwork: false,
		canQueryBluetooth: false,
		canDiscoverApplications: false,
	},
};

export async function fetchBootstrapStatus(
	invokeBackend: InvokeBackend = invoke,
): Promise<BootstrapStatus> {
	const status = await invokeBackend<BootstrapStatus>("bootstrap_status");
	if (
		!status.model.trim() ||
		!Number.isInteger(status.energy) ||
		status.energy < 0 ||
		status.energy > 100
	) {
		throw new Error("Backend returned an invalid Branlly status");
	}
	return status;
}

export function validateChatMessage(message: string): string {
	const normalized = message.trim();
	if (!normalized || [...normalized].length > 4_000) {
		throw new RangeError(
			"Le message doit contenir entre 1 et 4000 caractères.",
		);
	}
	return normalized;
}

export async function sendChat(
	message: string,
	receive: (event: ChatEvent) => void,
): Promise<void> {
	const channel = new Channel<ChatEvent>();
	channel.onmessage = receive;
	await invoke("chat", {
		message: validateChatMessage(message),
		onEvent: channel,
	});
}

export async function cancelChat(): Promise<void> {
	await invoke("cancel_chat");
}

export async function listOpenWindows(): Promise<WindowInfo[]> {
	return invoke<WindowInfo[]>("list_windows");
}

export async function focusOpenWindow(id: string): Promise<void> {
	await invoke("focus_window", { id });
}

export async function closeOpenWindow(id: string): Promise<void> {
	await invoke("close_window", { id });
}

export async function getSystemSnapshot(): Promise<SystemSnapshot> {
	return invoke<SystemSnapshot>("system_snapshot");
}

export async function launchShortcut(id: string): Promise<void> {
	await invoke("launch_shortcut", { id });
}

export const listLaunchItems = () => invoke<LaunchItem[]>("list_launch_items");
export const discoverApplications = () =>
	invoke<DiscoveredApplication[]>("discover_applications");
export const saveLaunchItems = (items: LaunchItem[]) =>
	invoke<LaunchItem[]>("save_launch_items", { items });
export const launchItem = (id: string) => invoke<void>("launch_item", { id });

export async function searchWikipedia(query: string): Promise<WikiResult[]> {
	return invoke<WikiResult[]>("wiki_search", { query });
}

export async function getPointerPosition(): Promise<{ x: number; y: number }> {
	return invoke<{ x: number; y: number }>("pointer_position");
}

export async function cleanupTemporaryFiles(): Promise<{
	removedEntries: number;
}> {
	return invoke<{ removedEntries: number }>("cleanup_temp");
}

export function isTauriRuntime(target: object = globalThis): boolean {
	return "__TAURI_INTERNALS__" in target;
}
