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
import Drives from "./Drives";
import Vault from "./Vault";
import Server from "./Server";
import Logs from "./Logs";
import Media from "./Media";
import Processes from "./Processes";
import Endpoints from "./Endpoints";
import Cron from "./Cron";
import { Checkbox } from "@/components/ui/checkbox";
import InfoButton from "@/components/ui/InfoButton";
import {
  ActivityIcon,
  BellDotIcon,
  BellIcon,
  BrickWallIcon,
  ClockIcon,
  CornerDownRight,
  EthernetPortIcon,
  FoldersIcon,
  HardDriveIcon,
  HouseIcon,
  ListIcon,
  MusicIcon,
  PackageIcon,
  ServerIcon,
  SidebarCloseIcon,
  SidebarOpenIcon,
  VaultIcon,
  XIcon,
} from "lucide-react";
import { Details } from "@/components/ui/Details";
import { Switch } from "@/components/ui/switch";
const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
  console.warn(
    "Fetch URL Prefix is enabled\nThis feature is meant for development only and may break the interface if left enabled.\nYou may be running a Non-Release version of Zentrox. Please look at your running Zentrox' log and check if Auth is enabled.\nIf it is not enabled, stop the program.",
  );
}

function SideBar({ children }) {
  var def =
    typeof window !== "undefined"
      ? localStorage.getItem("sidebarExpanded") == null
        ? true
        : localStorage.getItem("sidebarExpanded") == "true"
      : true;
  const [expanded, setExpanded] = useState(def);

  return (
    <>
      <span
        className={
          !expanded
            ? "border-r border-r-neutral-800 p-4 overflow-hidden"
            : "hidden"
        }
      >
        <img
          src="zentrox_dark_emblem.svg"
          className="w-6 h-6 inline-block mb-2"
        />
        <SidebarOpenIcon
          onClick={() => {
            setExpanded(true);
            localStorage.setItem("sidebarExpanded", true);
          }}
          className="h-5 w-5 cursor-pointer duration-200 opacity-75 hover:opacity-100 mb-2"
        />
        <NotificationBell />
      </span>

      <span
        className={
          "border-r border-r-neutral-800 overflow-hidden " +
          (expanded ? "w-[300px]" : "hidden")
        }
      >
        <span className="w-full max-w-full whitespace-nowrap p-3 flex items-center border-b border-neutral-800 bg-white/5">
          <span className="w-full font-medium text-lg flex items-center">
            <Account />
            <span className="ml-2 font-semibold select-none">Zentrox</span>
          </span>
          <NotificationBell />
          <SidebarCloseIcon
            onClick={() => {
              setExpanded(false);
              localStorage.setItem("sidebarExpanded", false);
            }}
            className="h-5 w-5 cursor-pointer duration-200 opacity-75 hover:opacity-100 ml-1"
          />
        </span>
        <span className="p-2 block h-full h-max-full overflow-y-scroll no-scroll">
          {children}
        </span>
      </span>
    </>
  );
}

function SideBarEntry({
  children,
  activeTab,
  setActiveTab,
  sub = false,
  name,
}) {
  const active = activeTab == name;

  return (
    <>
      <span
        className={
          "p-2 pb-1 pt-1 text-white cursor-pointer select-none flex items-center " +
          (!sub ? "font-semibold " : "ml-3 font-medium opacity-90 ") +
          (active && "underline")
        }
        onClick={() => setActiveTab(name)}
      >
        {!sub || (
          <CornerDownRight className="h-4 mr-1 opacity-50 inline-block" />
        )}
        {children}
      </span>
    </>
  );
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
  }, []);

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
          <Avatar className="cursor-pointer select-none">
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
          <DropdownMenuLabel>My account</DropdownMenuLabel>
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
  const SuggestedIcon = unreadNotifications ? BellDotIcon : BellIcon;

  return (
    <>
      <SuggestedIcon
        className={
          "h-5 w-5 cursor-pointer duration-200 opacity-75 hover:opacity-100" +
          (unreadNotifications ? " text-blue-500 opacity-100" : "")
        }
        onClick={() => {
          readNotifications();
          setExpanded(true);
        }}
      />

      <Dialog open={expanded} onOpenChange={setExpanded}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Notifications</DialogTitle>
            <DialogDescription>
              Zentrox yields notifications to inform you about errors, warnings
              or success messages.
            </DialogDescription>
          </DialogHeader>
          <p>
            <span className="flex items-center">
              <Switch
                className="mr-2"
                defaultChecked={
                  typeof window != "undefined"
                    ? localStorage.getItem("enableSysNotification") == "true"
                    : false
                }
                onCheckedChange={(e) => {
                  setSysNotification(e);
                }}
              />{" "}
              Enable browser notifications{" "}
              <InfoButton
                className="ml-1"
                title={"Enable browser notifications"}
                info={
                  "For every notification yielded by Zentrox, you will also get a notification in your browser."
                }
              />
            </span>

            <span className="block w-full p-1 max-h-[250px] overflow-y-scroll no-scroll">
              {notifications.length == 0 && (
                <span className="opacity-75 pt-2 w-full text-center block select-none">
                  No notifications
                </span>
              )}

              {notifications.map((x) => {
                return (
                  <span className="p-2 m-1 block rounded-lg border-neutral-800 border relative">
                    <XIcon
                      className="absolute top-2 right-2 opacity-50 hover:opacity-75 transition-all duration-200 cursor-pointer"
                      onClick={() => {
                        deleteNotification(x[1]);
                      }}
                    />
                    <span className="block max-w-[calc(100%-2rem)]">
                      {x[0]}
                    </span>
                  </span>
                );
              })}
            </span>
          </p>
        </DialogContent>
      </Dialog>
    </>
  );
}

export default function Dashboard() {
  const [activeTab, setActiveTab] = useState("Overview");

  function PageToShow() {
    if (activeTab == "Overview") {
      return <Overview />;
    } else if (activeTab == "Packages") {
      return <Packages />;
    } else if (activeTab == "Firewall") {
      return <Firewall />;
    } else if (activeTab == "Files") {
      return <Files />;
    } else if (activeTab == "Drives") {
      return <Drives />;
    } else if (activeTab == "Vault") {
      return <Vault />;
    } else if (activeTab == "Server") {
      return <Server />;
    } else if (activeTab == "Logs") {
      return <Logs />;
    } else if (activeTab == "Media") {
      return <Media />;
    } else if (activeTab == "Endpoints") {
      return <Endpoints />;
    } else if (activeTab == "Processes") {
      return <Processes />;
    } else if (activeTab == "Cronjobs") {
      return <Cron />;
    }
  }

  return (
    <main className="h-screen w-screen overflow-hidden p-0 m-0 flex transition-opacity">
      <Toaster />
      <SideBar>
        <SideBarEntry
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          name="Overview"
        >
          <HouseIcon className="w-4 inline-block mr-1" />
          Overview
        </SideBarEntry>
        <Details
          rememberState={true}
          name={"sideBarApplications"}
          open
          title={"Applications"}
        >
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Packages"
            sub
          >
            <PackageIcon className="w-4 inline-block mr-1" />
            Packages
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Processes"
            sub
          >
            <ActivityIcon className="w-4 inline-block mr-1" />
            Processes
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Cronjobs"
            sub
          >
            <ClockIcon className="w-4 inline-block mr-1" />
            Cronjobs
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Logs"
            sub
          >
            <ListIcon className="w-4 inline-block mr-1" />
            Logs
          </SideBarEntry>
        </Details>
        <Details
          rememberState={true}
          name={"sideBarNetworking"}
          open
          title={"Networking"}
        >
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Firewall"
            sub
          >
            <BrickWallIcon className="w-4 inline-block mr-1" />
            Firewall
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Endpoints"
            sub
          >
            <EthernetPortIcon className="w-4 inline-block mr-1" />
            Endpoints
          </SideBarEntry>
        </Details>
        <Details
          rememberState={true}
          name={"sideBarStorage"}
          open
          title={"Storage"}
        >
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Drives"
            sub
          >
            <HardDriveIcon className="w-4 inline-block mr-1" />
            Drives
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Files"
            sub
          >
            <FoldersIcon className="w-4 inline-block mr-1" />
            Files
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Vault"
            sub
          >
            <VaultIcon className="w-4 inline-block mr-1" />
            Vault
          </SideBarEntry>
          <SideBarEntry
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            name="Media"
            sub
          >
            <MusicIcon className="w-4 inline-block mr-1" />
            Media
          </SideBarEntry>
        </Details>
        <SideBarEntry
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          name="Server"
        >
          <ServerIcon className="w-4 inline-block mr-1" />
          Server
        </SideBarEntry>
      </SideBar>
      <PageToShow />
    </main>
  );
}
