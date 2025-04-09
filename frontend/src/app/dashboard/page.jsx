"use client";

import { Button } from "@/components/ui/button.jsx";
import { useEffect, useState, useRef } from "react";
import "./table.css";
import "./scroll.css";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { toast } from "@/components/ui/use-toast";
import useNotification from "@/lib/notificationState";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import "./scroll.css";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useCallback } from "react";
import Overview from "./Overview";
import Packages from "./Packages";
import Firewall from "./Firewall";
import Files from "./Files";
import Storage from "./Storage";
import Vault from "./Vault";
import Server from "./Server";
import Logs from "./Logs";
import Media from "./Media";
import Networking from "./Networking";
import { Checkbox } from "@/components/ui/checkbox";
import {
  BellDotIcon,
  BellIcon,
  BrickWall,
  ChartBar,
  CircleDot,
  DiscIcon,
  FileIcon,
  HardDriveIcon,
  LockIcon,
  LogsIcon,
  NetworkIcon,
  PackageIcon,
  ServerIcon,
  XIcon,
} from "lucide-react";
import InfoButton from "@/components/ui/InfoButton";
const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
  console.warn(
    "Fetch URL Prefix is enabled\nThis feature is meant for development only and may break the interface if left enabled.\nYou may be running a Non-Release version of Zentrox. Please look at your running Zentrox' log and check if Auth is enabled.\nIf it is not enabled, stop the program.",
  );
}

function TopBar({ children }) {
  return (
    <nav className="bg-transparent text-neutral-100 p-3 border-neutral-900 border-b font-semibold text-xl flex items-center animate-fadein duration-300">
      {children}
    </nav>
  );
}

function TabButton({ onClick, isDefault, isActive, children, icon }) {
  const [isOnloadDefault, setOnloadDefault] = useState(isDefault);
  const [smallTopBar, setSmallTopBar] = useState(false);

  useEffect(() => {
    setSmallTopBar(window.innerWidth < 1000);
  }, []);

  if (isOnloadDefault || isActive) {
    var style =
      "mr-2 ml-2 text-lg hover:bg-neutral-900 text-white bg-neutral-900 hover:bg-neutral-800 hover:text-neutral-100";
  } else {
    var style =
      "bg-transparent mr-2 ml-2 text-lg hover:bg-neutral-800 hover:text-neutral-200 text-neutral-400";
  }
  if (isOnloadDefault) {
    onClick();
    setOnloadDefault(false);
  }
  if (!smallTopBar) {
    return (
      <Button
        className={style}
        onClick={() => {
          onClick();
        }}
      >
        {children}
      </Button>
    );
  } else {
    return (
      <Button
        className={style}
        onClick={() => {
          onClick();
        }}
      >
        {icon}
      </Button>
    );
  }
}

function Account() {
  const [account, setAccount] = useState({ username: "" });
  const [usernameWarningVisible, setUsernameWarningVisible] = useState(false);
  const [accountDetailsDialogOpen, setAccountDetailsOpen] = useState(false);
  const [passwordWarningVisible, setPasswordWarningVisible] = useState(false);
  const [powerOffDialogOpen, setPowerOffDialogOpen] = useState(false);
  const [reloadTrigger, setReloadTrigger] = useState(0);
  const [canUpdateCredentials, setCanUpdateCredentials] = useState(false);
  const [otpEnabled, setOtpEnabled] = useState(false);
  const { deleteNotification, notify, notifications } = useNotification();

  const sudoPasswordInput = useRef(null);
  const accountUsernameInput = useRef(null);
  const accountPasswordInput = useRef(null);
  const profilePictureUploadInput = useRef(null);

  useEffect(() => {
    if (account.username == "") {
      fetch(fetchURLPrefix + "/api/useOtp", { method: "POST" }).then((r) => {
        if (r.ok) {
          r.json().then((j) => {
            setOtpEnabled(j.used);
          });
        }
      });
    }
  });

  function fetchOtpInformation() {
    if (account.username == "") {
      fetch(fetchURLPrefix + "/api/accountDetails", {
        method: "POST",
      }).then((r) => {
        if (r.ok) {
          r.json().then((j) => {
            setAccount(j);
          });
        } else {
          toast({
            title: "Failed to fetch account details",
          });
        }
      });
    }
  }

  useEffect(() => {
    fetchOtpInformation();
  }, []);

  // Callbacks to handle state updates
  const handleEditDetailsClick = useCallback(() => {
    setAccountDetailsOpen(true);
  }, []);

  const handleLogoutClick = useCallback(() => {
    fetch("/logout", { method: "POST" }).then(() => {
      location.href = "/";
    });
  }, []);

  const handlePowerOffClick = useCallback(() => {
    setPowerOffDialogOpen(true);
  }, []);

  const updateCredentials = () => {
    const username = accountUsernameInput.current?.value;
    const password = accountPasswordInput.current?.value;

    setAccount({
      username: username,
    });

    fetch(fetchURLPrefix + "/api/updateAccountDetails", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    }).then((res) => {
      if (res.ok) {
        notify("Account details updated");
        toast({
          title: "Account details updated",
          description: "Your account details have been updated",
        });
      } else {
        notify("Failed to updated account details");
        toast({
          title: "Failed to update account details",
          description: "Your account details have not been updated",
        });
      }
    });
  };

  const handlePowerOffConfirm = useCallback(() => {
    fetch(fetchURLPrefix + "/api/powerOff", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sudoPassword: sudoPasswordInput.current?.value }),
    }).then((res) => {
      if (!res.ok) {
        notify("Power off failed");
        toast({ title: "Power off failed" });
      }
    });
  }, []);

  return (
    <>
      <Dialog
        open={accountDetailsDialogOpen}
        onOpenChange={setAccountDetailsOpen}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Account</DialogTitle>
            <DialogDescription>Edit your account details.</DialogDescription>
          </DialogHeader>
          <div>
            <strong className="block ml-1 mb-1">Credentials</strong>
            <div
              className="mb-1 text-sm text-red-500 ml-1"
              hidden={!usernameWarningVisible}
            >
              A username may not be shorter than 5 characters.
            </div>

            <Input
              placeholder="Username"
              className="mb-2"
              ref={accountUsernameInput}
              defaultValue={account.username}
              disabled={account.username === ""}
              onChange={() => {
                setUsernameWarningVisible(
                  accountUsernameInput.current?.value.length < 5,
                );
                if (
                  accountUsernameInput.current?.value != account.username ||
                  accountPasswordInput.current?.value !== ""
                ) {
                  setCanUpdateCredentials(true);
                }
              }}
            />
            <div
              className="mb-1 text-sm text-red-500 ml-1"
              hidden={!passwordWarningVisible}
            >
              The password may not be shorter than 15 characters.
            </div>
            <Input
              placeholder="Password"
              className="mb-2"
              type="password"
              ref={accountPasswordInput}
              disabled={account.username === ""}
              onChange={() => {
                setPasswordWarningVisible(
                  accountPasswordInput.current?.value.length < 15 &&
                    accountPasswordInput?.current.value.length > 0,
                );
                if (
                  accountUsernameInput.current?.value != account.username ||
                  accountPasswordInput.current?.value !== ""
                ) {
                  setCanUpdateCredentials(true);
                }
              }}
            />
            <Button
              variant="outline"
              className={"block mb-2 " + (canUpdateCredentials ? "" : "hidden")}
              disabled={passwordWarningVisible || usernameWarningVisible}
              onClick={updateCredentials}
            >
              Update credentials
            </Button>
            <strong className="block ml-1 mb-1">2FA</strong>
            <div className="flex items-center">
              <Checkbox
                className="inline-block mr-1 ml-1"
                defaultChecked={otpEnabled}
                onCheckedChange={(e) => {
                  fetch(fetchURLPrefix + "/api/updateOtp/" + e)
                    .then((r) => {
                      if (r.ok) {
                        setOtpEnabled(e);
                        if (!e) {
                          toast({
                            title: "Updated 2FA status",
                          });
                        } else {
                          r.text().then((t) => {
                            toast({
                              title: "Updated 2FA status",
                              description: (
                                <>
                                  Your new 2FA token is <code>{t}</code>{" "}
                                  <Button
                                    className="mt-2 border !border-black/20"
                                    onClick={() => {
                                      window.navigator.clipboard
                                        .writeText(t)
                                        .then(() => {
                                          toast({
                                            title: "Copied to clipboard",
                                          });
                                        });
                                    }}
                                  >
                                    Copy to clipboard
                                  </Button>
                                </>
                              ),
                              duration: 120000,
                            });
                          });
                        }
                      } else {
                        toast({
                          title: "Failed to update 2FA status",
                        });
                      }
                    })
                    .catch(() => {
                      toast({
                        title: "Failed to update 2FA status",
                      });
                    });
                }}
              />{" "}
              <label className="inline-block mr-1">Enable 2FA</label>
            </div>
            <small className="block ml-1 text-white/60">
              Two-factor authentication (2FA) uses a One-Time-Pad to generate a
              unique code every 30 seconds. To use 2FA, you need to securely
              store a code in an authenticator application. When you disable
              2FA, Zentrox will automatically remove your current OTP token and
              generate a new one when you re-enable OTP. You can not view your
              token after enabling OTP anymore.
            </small>
            <input
              type="file"
              ref={profilePictureUploadInput}
              onChange={() => {
                var fileForSubmit = profilePictureUploadInput.current.files[0];
                if (fileForSubmit.size >= 1024 * 1024) {
                  toast({
                    title: "File to big",
                    description: "The file you provided was larger than 1MB",
                  });
                }
                var formData = new FormData();
                formData.append("file", fileForSubmit);
                fetch(fetchURLPrefix + "/api/uploadProfilePicture", {
                  method: "POST",
                  body: formData,
                }).then((res) => {
                  profilePictureUploadInput.current.value = "";
                  if (res.ok) {
                    setReloadTrigger(Date.now());
                  } else {
                    toast({
                      title: "Failed to upload profile picture",
                      description:
                        "Zentrox failed to upload the file you provided",
                    });
                  }
                });
              }}
              hidden
            />
            <strong className="block ml-1 mb-1">Profile</strong>
            <Button
              className="w-fit ml-1"
              onClick={() => {
                profilePictureUploadInput.current.click();
              }}
            >
              Upload profile picture
            </Button>
          </div>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Close</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={powerOffDialogOpen}
        onOpenChange={setPowerOffDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Power off</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to power off your machine? Zentrox cannot
              reboot it automatically. Please enter your sudo password to do so:
              <br />
              <br />
              <Input
                type="password"
                placeholder="Sudo password"
                className="w-full"
                ref={sudoPasswordInput}
              />
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handlePowerOffConfirm}>
              Power off
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <DropdownMenu modal={false}>
        <DropdownMenuTrigger asChild>
          <Avatar className="block float-right cursor-pointer ml-2">
            <AvatarImage
              src={`${fetchURLPrefix}/api/profilePicture?reload=${reloadTrigger}`}
            />
            <AvatarFallback>
              {account.username != ""
                ? account.username[0]?.toUpperCase()
                : "A"}
            </AvatarFallback>
          </Avatar>
        </DropdownMenuTrigger>

        <DropdownMenuContent>
          <DropdownMenuLabel>My Account</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={handleEditDetailsClick}>
            Edit details
          </DropdownMenuItem>
          <DropdownMenuItem onClick={handleLogoutClick}>
            Logout
          </DropdownMenuItem>

          <DropdownMenuSeparator></DropdownMenuSeparator>
          <DropdownMenuLabel>Machine</DropdownMenuLabel>
          <DropdownMenuItem onClick={handlePowerOffClick}>
            Power Off
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  );
}

function NotificationBell() {
  const {
    deleteNotification,
    notify,
    notifications,
    unreadNotifications,
    readNotifications,
    setSysNotification,
  } = useNotification();
  const [expanded, setExpanded] = useState(false);
  const [closing, setClosing] = useState(false);

  function openList() {
    setExpanded(true);
  }

  function closeList() {
    setClosing(true);
    setTimeout(() => {
      setExpanded(false);
      setClosing(false);
    }, 150 - 20);
  }

  const SuggestedIcon = unreadNotifications ? BellDotIcon : BellIcon;

  return (
    <>
      <SuggestedIcon
        className={
          "inline-block ml-auto mr-2 h-4 w-4 cursor-pointer" +
          (unreadNotifications ? " text-orange-500" : "")
        }
        onClick={() => {
          readNotifications();
          if (expanded) {
            closeList();
          } else {
            openList();
          }
        }}
      />
      <span
        className={
          "w-screen h-screen top-0 left-0 fixed z-10" +
          (expanded ? " block" : " hidden")
        }
        onClick={closeList}
        onContextMenu={closeList}
      ></span>
      <span
        className={
          "w-64 h-full fixed top-[76px] right-0 bg-zinc-950 border-l-zinc-900 border-l z-20 duration-150 p-2 max-h-full overflow-scroll " +
          (!closing
            ? "fade-in-0 animate-in"
            : "fade-out-0 animate-out right-[-20px]") +
          (expanded ? " block" : " hidden")
        }
      >
        <strong className="flex items-center w-full">
          <BellIcon className="inline-block mr-1 ml-1 h-5 w-5" />
          Notifications
        </strong>
        <span className="flex items-center h-fit ml-1 font-normal">
          <Checkbox
            className="mr-1"
            defaultChecked={
              typeof window !== "undefined"
                ? localStorage.getItem("enableSysNotification") === "true"
                : false
            }
            onCheckedChange={(e) => {
              if (e) {
                Notification.requestPermission();
              }
              setSysNotification(e);
            }}
          />{" "}
          <label>
            Push notifications{" "}
            <InfoButton
              title={"Push notifications"}
              info={
                <>
                  Enable push notifications to get notifications when this tab
                  is not active or your browser window is minimized. Zentrox
                  will only send copies of the notifications that are written
                  into this notification bar to your push notifications. You can
                  disable this feature at any time.
                </>
              }
            />
          </label>
        </span>
        {notifications.length === 0 && (
          <span className="opacity-50">No notifications</span>
        )}
        {notifications.map((e, k) => {
          const deleteE = () => deleteNotification(e[1]);
          return (
            <span
              key={"notbell" + k}
              className="relative block w-full p-2 pt-5 mb-2 h-fit whitespace-pre-wrap overflow-ellipsis overflow-hidden rounded border-zinc-900 border hover:border-zinc-800 select-none cursor-pointer duration-200 transition-all text-sm font-normal"
            >
              {e[0]}
              <XIcon
                className="absolute top-1 right-1 h-4 w-4 opacity-75 hover:opacity-100 duration-150 transition-all cursor-pointer"
                onClick={deleteE}
              />
            </span>
          );
        })}
      </span>
    </>
  );
}

export default function Dashboard() {
  const [activeTab, setActiveTab] = useState("Overview");
  const [smallTopBar, setSmallTopBar] = useState(false);

  useEffect(() => {
    setSmallTopBar(window.innerWidth < 1000);
  }, []);

  function PageToShow() {
    if (activeTab == "Overview") {
      return <Overview />;
    } else if (activeTab == "Packages") {
      return <Packages />;
    } else if (activeTab == "Firewall") {
      return <Firewall />;
    } else if (activeTab == "Files") {
      return <Files />;
    } else if (activeTab == "Storage") {
      return <Storage />;
    } else if (activeTab == "Vault") {
      return <Vault />;
    } else if (activeTab == "Server") {
      return <Server />;
    } else if (activeTab == "Logs") {
      return <Logs />;
    } else if (activeTab == "Media") {
      return <Media />;
    } else if (activeTab == "Networking") {
      return <Networking />;
    }
  }

  return (
    <main className="h-screen w-screen overflow-hidden p-0 m-0 flex flex-col transition-opacity">
      <Toaster />
      <TopBar>
        <span
          className={
            !smallTopBar
              ? "p-2 pl-4 pr-4 border border-neutral-700 cursor-pointer rounded transition-all content-center inline-block text-lg font-normal"
              : "hidden"
          }
          onClick={() => {
            window.open("https://github.com/wervice/zentrox");
          }}
        >
          <img
            src="zentrox_dark_emblem.svg"
            className="inline-block pb-0.5 w-5 h-5"
            alt="Zentrox Logo"
          />{" "}
          Zentrox
        </span>{" "}
        <TabButton
          onClick={() => {
            setActiveTab("Overview");
          }}
          isDefault={true}
          isActive={activeTab == "Overview"}
          icon={<ChartBar />}
        >
          Overview
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Packages");
          }}
          isDefault={false}
          isActive={activeTab == "Packages"}
          icon={<PackageIcon />}
        >
          Packages
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Logs");
          }}
          isDefault={false}
          isActive={activeTab == "Logs"}
          icon={<LogsIcon />}
        >
          Logs
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Firewall");
          }}
          isDefault={false}
          isActive={activeTab == "Firewall"}
          icon={<BrickWall />}
        >
          Firewall
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Networking");
          }}
          isDefault={false}
          isActive={activeTab == "Networking"}
          icon={<NetworkIcon />}
        >
          Networking
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Files");
          }}
          isDefault={false}
          isActive={activeTab == "Files"}
          icon={<FileIcon />}
        >
          Files
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Storage");
          }}
          isDefault={false}
          isActive={activeTab == "Storage"}
          icon={<HardDriveIcon />}
        >
          Storage
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Vault");
          }}
          isDefault={false}
          isActive={activeTab == "Vault"}
          icon={<LockIcon />}
        >
          Vault
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Server");
          }}
          isDefault={false}
          isActive={activeTab == "Server"}
          icon={<ServerIcon />}
        >
          Server
        </TabButton>
        <TabButton
          onClick={() => {
            setActiveTab("Media");
          }}
          isDefault={false}
          isActive={activeTab == "Media"}
          icon={<DiscIcon />}
        >
          Media
        </TabButton>
        <NotificationBell />
        <Account />
      </TopBar>
      <PageToShow />
    </main>
  );
}
