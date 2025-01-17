"use client";

import { QRCodeSVG } from "qrcode.react";
import { Checkbox } from "@/components/ui/checkbox.jsx";
import { Button } from "@/components/ui/button.jsx";
import { CalendarButton } from "@/components/ui/calendar.jsx";
import { DataTable } from "@/components/ui/dataTable.jsx";
import { Link } from "lucide-react";
import { useEffect, useState, useRef } from "react";
import { Label } from "@/components/ui/label";
import "./table.css";
import "./scroll.css";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { toast } from "@/components/ui/use-toast";
import InfoButton from "@/components/ui/InfoButton.jsx";
import {
 Dialog,
 DialogContent,
 DialogDescription,
 DialogHeader,
 DialogTitle,
 DialogTrigger,
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
import Sharing from "./Sharing";
import Logs from "./Logs";
import Media from "./Media";

// const fetchURLPrefix = "";
const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
 console.warn(
  "Fetch URL Prefix is enabled\nThis feature is meant for development only and may break the interface if left enabled.\nYou may be running a Non-Release version of Zentrox. Please look at your running Zentrox' log and check if Auth is enabled.\nIf it is not enabled, stop the program.",
 );
}

function TopBar({ children }) {
 return (
  <nav className="bg-transparent text-neutral-100 p-3 border-neutral-900 border-b font-semibold text-xl">
   {children}
  </nav>
 );
}

function TabButton({ onClick, isDefault, isActive, children }) {
 const [isOnloadDefault, setOnloadDefault] = useState(isDefault);
 if (isOnloadDefault || isActive) {
  var style =
   "mr-2 ml-2 text-lg hover:bg-neutral-900 text-white bg-neutral-900 hover:bg-neutral-800 hover:text-neutral-100 focus:outline outline-2 outline-offset-2";
 } else {
  var style =
   "bg-transparent mr-2 ml-2 text-lg hover:bg-neutral-800 hover:text-neutral-200 text-neutral-400 focus:outline outline-2 outline-offset-2";
 }
 if (isOnloadDefault) {
  onClick();
  setOnloadDefault(false);
 }
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
}

function Account() {
 const [account, setAccount] = useState({ username: "" });
 const [usernameWarningVisible, setUsernameWarningVisible] = useState(false);
 const [accountDetailsDialogOpen, setAccountDetailsOpen] = useState(false);
 const [passwordWarningVisible, setPasswordWarningVisible] = useState(false);
 const [powerOffDialogOpen, setPowerOffDialogOpen] = useState(false);
 const [reloadTrigger, setReloadTrigger] = useState(0);

 const sudoPasswordInput = useRef(null);
 const accountUsernameInput = useRef(null);
 const accountPasswordInput = useRef(null);
 const profilePictureUploadInput = useRef(null);

 useEffect(() => {
  if (account.username == "") {
   fetch("/api/accountDetails", {
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
 }, [account]);

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

 const handleApplyClick = () => {
  const username = accountUsernameInput.current?.value;
  const password = accountPasswordInput.current?.value;

  setAccount({
   username: username,
  });

  fetch("/api/updateAccountDetails", {
   method: "POST",
   headers: { "Content-Type": "application/json" },
   body: JSON.stringify({ username, password }),
  }).then((res) => {
   if (res.ok) {
    toast({
     title: "Account details updated",
     description: "Your account details have been updated",
    });
   } else {
    toast({
     title: "Failed to update account details",
     description: "Your account details have not been updated",
    });
   }
  });
 };

 const handlePowerOffConfirm = useCallback(() => {
  fetch("/api/powerOff", {
   method: "POST",
   headers: { "Content-Type": "application/json" },
   body: JSON.stringify({ sudoPassword: sudoPasswordInput.current?.value }),
  }).then((res) => {
   if (!res.ok) {
    toast({ title: "Power Off failed" });
   }
  });
 }, []);

 return (
  <>
   <Dialog open={accountDetailsDialogOpen} onOpenChange={setAccountDetailsOpen}>
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Account</DialogTitle>
      <DialogDescription>Edit your account details.</DialogDescription>
     </DialogHeader>
     <span className="p-1 text-red-500" hidden={!usernameWarningVisible}>
      A username may not be shorter than 5 characters.
     </span>
     Username
     <Input
      placeholder="Username"
      ref={accountUsernameInput}
      defaultValue={account.username}
      disabled={account.username === ""}
      onKeyPress={() => {
       setUsernameWarningVisible(
        accountUsernameInput.current?.value.length < 5,
       );
      }}
     />
     <span className="p-1 text-red-500" hidden={!passwordWarningVisible}>
      A password may not be shorter than 10 characters.
     </span>
     <Input
      placeholder="Password"
      type="password"
      ref={accountPasswordInput}
      disabled={account.username === ""}
      onKeyPress={() => {
       setPasswordWarningVisible(
        accountPasswordInput.current?.value.length < 10,
       );
      }}
     />
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
          description: "Zentrox failed to upload the file you provided",
         });
        }
       });
      }}
      hidden
     />
     <Button
      className="w-fit"
      onClick={() => {
       profilePictureUploadInput.current.click();
      }}
     >
      Upload profile picture
     </Button>
     <DialogFooter>
      <DialogClose>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose>
       <Button onClick={handleApplyClick}>Apply</Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>

   <AlertDialog open={powerOffDialogOpen} onOpenChange={setPowerOffDialogOpen}>
    <AlertDialogContent>
     <AlertDialogHeader>
      <AlertDialogTitle>Power Off</AlertDialogTitle>
      <AlertDialogDescription>
       Do you really want to power off your machine? Zentrox cannot reboot it
       automatically. Please enter your sudo password to do so:
       <br />
       <br />
       <Input
        type="password"
        placeholder="Sudo Password"
        ref={sudoPasswordInput}
       />
      </AlertDialogDescription>
     </AlertDialogHeader>
     <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction onClick={handlePowerOffConfirm}>
       Power Off
      </AlertDialogAction>
     </AlertDialogFooter>
    </AlertDialogContent>
   </AlertDialog>

   <DropdownMenu>
    <DropdownMenuTrigger asChild>
     <Avatar
      className="block float-right cursor-pointer"
      onClick={handleEditDetailsClick}
     >
      <AvatarImage src={`/api/profilePicture?reload=${reloadTrigger}`} />
      <AvatarFallback>
       {account.username != "" ? account.username[0]?.toUpperCase() : "A"}
      </AvatarFallback>
     </Avatar>
    </DropdownMenuTrigger>

    <DropdownMenuContent>
     <DropdownMenuLabel>My Account</DropdownMenuLabel>
     <DropdownMenuSeparator />
     <DropdownMenuItem onClick={handleEditDetailsClick}>
      Edit details
     </DropdownMenuItem>
     <DropdownMenuItem onClick={handleLogoutClick}>Logout</DropdownMenuItem>

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

export default function Dashboard() {
 const [activeTab, setActiveTab] = useState("Overview");

 function PageToShow() {
  if (activeTab == "Overview") {
   return Overview();
  } else if (activeTab == "Packages") {
   return Packages();
  } else if (activeTab == "Firewall") {
   return Firewall();
  } else if (activeTab == "Files") {
   return Files();
  } else if (activeTab == "Storage") {
   return Storage();
  } else if (activeTab == "Vault") {
   return Vault();
  } else if (activeTab == "Sharing") {
   return Sharing();
  } else if (activeTab == "Logs") {
   return Logs();
  } else if (activeTab == "Media") {
   return Media();
  }
 }

 return (
  <main className="h-screen w-screen overflow-hidden p-0 m-0 flex flex-col transition-opacity">
   <Toaster />
   <TopBar>
    <span
     className="p-2 pl-4 pr-4 border border-neutral-700 cursor-pointer rounded transition-all content-center inline-block text-lg font-normal"
     onClick={() => {
      window.open("https://github.com/wervice/zentrox");
     }}
    >
     <img
      src="zentrox_dark.svg"
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
    >
     Overview
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Packages");
     }}
     isDefault={false}
     isActive={activeTab == "Packages"}
    >
     Packages
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Firewall");
     }}
     isDefault={false}
     isActive={activeTab == "Firewall"}
    >
     Firewall
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Files");
     }}
     isDefault={false}
     isActive={activeTab == "Files"}
    >
     Files
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Storage");
     }}
     isDefault={false}
     isActive={activeTab == "Storage"}
    >
     Storage
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Vault");
     }}
     isDefault={false}
     isActive={activeTab == "Vault"}
    >
     Vault
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Sharing");
     }}
     isDefault={false}
     isActive={activeTab == "Sharing"}
    >
     Sharing
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Logs");
     }}
     isDefault={false}
     isActive={activeTab == "Logs"}
    >
     Logs
    </TabButton>
    <TabButton
     onClick={() => {
      setActiveTab("Media");
     }}
     isDefault={false}
     isActive={activeTab == "Media"}
    >
     Media
    </TabButton>
    <Account />
   </TopBar>
   <PageToShow />
  </main>
 );
}
