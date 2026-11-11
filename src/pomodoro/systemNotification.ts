import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

/**
 * Asks for permission once, so the first phase end can already notify.
 * Called when the timer starts, i.e. on a user action.
 */
export async function prepareSystemNotifications(): Promise<void> {
  try {
    if (!(await isPermissionGranted())) await requestPermission();
  } catch {
    // Without notifications the in-app toast still tells the user.
  }
}

/** Shows a system notification if the user allowed them; otherwise nothing. */
export async function notifySystem(title: string, body: string): Promise<void> {
  try {
    if (await isPermissionGranted()) sendNotification({ title, body });
  } catch {
    // Not available (e.g. in the browser preview); the toast is enough.
  }
}
