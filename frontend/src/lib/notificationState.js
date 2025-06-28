import { create } from "zustand";
import { v4 as uuidv4, v4 } from "uuid";

/**
 * Custom hook to manage universal notifications using zustand
 * Use notify to create a new notification with message body and unique UUID:
 * ```javascript
 * notify("This is a message", "1111-1111-1111-1111")
 * ```
 *
 * Use deleteNotification to delete a notification by UUID:
 * ```javascript
 * deleteNotification("1111-1111-1111-1111") // Deletes a notification
 * ```
 *
 * notifications is an array of notifications structured as follows:
 * ```javascript
 * [
 *	[
 *	 body,
 *	 uuid
 *	],...
 * ]
 * ```
 * */
const useNotification = create((set) => ({
  notifications: [],
  unreadNotifications: false,
  sysNotifications:
    typeof window !== "undefined"
      ? localStorage.getItem("enableSysNotification") === "true"
      : false,
  notify: (message) => {
    set((prevState) => {
      if (prevState.sysNotifications) {
        Notification.requestPermission();
        new Notification("Zentrox notification", {
          body: message,
          actions: [],
        });
      }
      return {
        notifications: [
          [message, v4(), Date.now()],
          ...prevState.notifications,
        ],
        unreadNotifications: true,
      };
    });
  },
  deleteNotification: (uuid) =>
    set((prevState) => ({
      notifications: prevState.notifications.filter((e) => e[1] !== uuid),
    })),
  readNotifications: () => {
    set((prevState) => ({
      notifications: prevState.notifications,
      unreadNotifications: false,
    }));
  },
  setSysNotification: (e) => {
    if (typeof window !== "undefined") {
      localStorage.setItem("enableSysNotification", e);
    }
    set(() => ({
      sysNotifications: e,
    }));
  },
}));

export default useNotification;
