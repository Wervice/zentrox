"use client";

import { QRCodeSVG } from "qrcode.react";
import { Checkbox } from "@/components/ui/checkbox.jsx";
import { Switch } from "@/components/ui/switch.jsx";
import { Button } from "@/components/ui/button.jsx";
import { CalendarButton } from "@/components/ui/calendar.jsx";
import { DataTable } from "@/components/ui/dataTable.jsx";
import {
 ComputerIcon,
 CpuIcon,
 Disc2,
 List,
 MemoryStickIcon,
 MonitorIcon,
 Network,
 PieChart,
 Plug,
 TableIcon,
 Thermometer,
 WatchIcon,
 Share2,
 HardDriveIcon,
 AppWindow,
 Package2,
 Loader2,
 CircleX,
 SearchIcon,
 TrashIcon,
 Paintbrush2,
 Plus,
 Ban,
 CircleCheck,
 BrickWall,
 ArrowUpFromDot,
 ArrowDownToDot,
 Shield,
 RepeatIcon,
 LayoutPanelTopIcon,
 TagIcon,
 MapPinIcon,
 WeightIcon,
 UserIcon,
 MountainIcon,
 PieChartIcon,
 KeyIcon,
 LockIcon,
 LockOpenIcon,
 FolderIcon,
 UploadIcon,
 FileIcon,
 ArrowUp,
 LoaderIcon,
 DeleteIcon,
 PenLineIcon,
 ClockIcon,
 Table,
 CogIcon,
 QrCodeIcon,
 PlusIcon,
} from "lucide-react";
import { useEffect, useState, useRef } from "react";
import { Label } from "@/components/ui/label";
import { SideWayBarChart } from "@/components/ui/Charts.jsx";
import "./table.css";
import "./scroll.css";
import Spinner from "@/components/ui/Spinner.jsx";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { toast, useToast } from "@/components/ui/use-toast";
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
 Select,
 SelectContent,
 SelectGroup,
 SelectItem,
 SelectTrigger,
 SelectValue,
} from "@/components/ui/select";
import {
 AlertDialog,
 AlertDialogAction,
 AlertDialogCancel,
 AlertDialogContent,
 AlertDialogDescription,
 AlertDialogFooter,
 AlertDialogHeader,
 AlertDialogTitle,
 AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import FileView from "@/components/ui/fileview";
import "./scroll.css";
import {
 ContextMenu,
 ContextMenuContent,
 ContextMenuItem,
 ContextMenuTrigger,
} from "@/components/ui/context-menu";
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

// const fetchURLPrefix = "";
const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
 console.warn(
  "Fetch URL Prefix is enabled\nThis feature is meant for development only and may break the interface if left enabled.\nYou may be running a Non-Release version of Zentrox. Please look at your running Zentrox' log and check if Auth is enabled.\nIf it is not enabled, stop the program.",
 );
}

/**
 * @param {string} value to check
 * @description Returns the string or "N/A" when the string is not defined*/
function na(value) {
 if (typeof value === "undefined" || value === null) return "N/A";
 return value;
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

function Page({ name, children, className, ...props }) {
 return (
  <div
   className={"w-full h-full flex-grow overflow-y-auto text-white " + className}
   {...props}
  >
   <div className="p-4 h-full">
    <h2 className="text-3xl font-bold">{name}</h2>
    {children}
   </div>
  </div>
 );
}

function Overview() {
 function overviewFetch() {
  // Check localStorage for cached CPU, RAM, Disk, and Device Information
  const cachedCpuUsage = localStorage.getItem("cpuUsagePercent");
  const cachedRamUsage = localStorage.getItem("ramUsagePercent");
  const cachedDiskUsage = localStorage.getItem("diskUsagePercent");
  const cachedDeviceInformation = localStorage.getItem("deviceInformation");

  // If cached values exist, use them to update the state
  if (cachedCpuUsage) setCpuUssagePercent(JSON.parse(cachedCpuUsage));
  if (cachedRamUsage) setRamUssagePercent(JSON.parse(cachedRamUsage));
  if (cachedDiskUsage) setDiskUssagePercent(JSON.parse(cachedDiskUsage));
  if (cachedDeviceInformation)
   setDeviceInformation(JSON.parse(cachedDeviceInformation));

  // Fetch new data for CPU usage
  fetch(fetchURLPrefix + "/api/cpuPercent", {
   method: "GET",
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setCpuUssagePercent(json["p"]);
     localStorage.setItem("cpuUsagePercent", JSON.stringify(json["p"])); // Cache CPU usage
    });
   }
  });

  // Fetch new data for RAM usage
  fetch(fetchURLPrefix + "/api/ramPercent", {
   method: "GET",
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setRamUssagePercent(json["p"]);
     localStorage.setItem("ramUsagePercent", JSON.stringify(json["p"])); // Cache RAM usage
    });
   }
  });

  // Fetch new data for Disk usage
  fetch(fetchURLPrefix + "/api/diskPercent", {
   method: "GET",
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setDiskUssagePercent(json["p"]);
     localStorage.setItem("diskUsagePercent", JSON.stringify(json["p"])); // Cache Disk usage
    });
   }
  });

  // Fetch new data for Device Information
  fetch(fetchURLPrefix + "/api/deviceInformation", {
   method: "GET",
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setDeviceInformation(json);
     localStorage.setItem("deviceInformation", JSON.stringify(json)); // Cache Device Information
    });
   }
  });
 }

 function ftpStatusFetch() {
  fetch(fetchURLPrefix + "/api/fetchFTPconfig", {
   method: "GET",
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setFtpEnabled(json["enabled"]);
    });
   }
  });
 }

 const [cpuUssagePercent, setCpuUssagePercent] = useState(0);
 const [ramUssagePercent, setRamUssagePercent] = useState(0);
 const [diskUssagePercent, setDiskUssagePercent] = useState(0);
 const [deviceInformation, setDeviceInformation] = useState({
  os_name: "",
  power_supply: "",
  zentrox_pid: "",
  process_number: "",
  hostname: "",
  uptime: "",
  temperature: "",
 });
 const [ftpEnabled, setFtpEnabled] = useState(false);
 var enableFtpSudoPasswordInput = useRef("");

 useEffect(() => {
  const interval = setInterval(() => {
   overviewFetch();
  }, 5000);

  return () => clearInterval(interval);
 }, []);

 useEffect(() => {
  overviewFetch();
  ftpStatusFetch();
 }, []);

 return (
  <Page name="Overview" className="align-top">
   <div className="h-72">
    <div className="inline-block w-72 h-full align-top p-4 rounded-2xl border border-neutral-700 m-2">
     <Label className="text-lg">
      <PieChart className="inline-block h-6 pr-1" /> Resources
     </Label>
     <br />
     <Label className="text-neutral-700">
      <CpuIcon className="inline-block h-6 pb-1 pr-1" />
      Processor
     </Label>
     <br />
     <SideWayBarChart percentage={cpuUssagePercent} />
     <Label className="text-neutral-700">
      <MemoryStickIcon className="inline-block h-6 pb-1 pr-1" />
      Memory
     </Label>
     <br />
     <SideWayBarChart percentage={ramUssagePercent} />
     <Label className="text-neutral-700">
      <Disc2 className="inline-block h-6 pb-1 pr-1" />
      Disk
     </Label>
     <br />
     <SideWayBarChart percentage={diskUssagePercent} />
    </div>
    <div className="inline-block align-top w-fit h-full p-4 rounded-2xl border border-neutral-700 m-2">
     <Label className="text-lg">
      <MonitorIcon className="inline-block h-7 pr-1" /> System information
     </Label>
     <br />
     <table>
      <tbody>
       <tr>
        <td>
         <ComputerIcon className="inline-block h-6 pb-1 pr-1" /> Operating
         System
        </td>
        <td>{deviceInformation["os_name"].replaceAll(/\(.*\)/g, "")}</td>
       </tr>

       <tr>
        <td>
         <Network className="inline-block h-6 pb-1 pr-1" /> Hostname{" "}
         <InfoButton
          title="Hostname"
          info="The name of your computer in your local network"
         />
        </td>
        <td>{deviceInformation["hostname"]}</td>
       </tr>
       <tr>
        <td>
         <Thermometer className="inline-block h-6 pb-1 pr-1" /> Temperature{" "}
         <InfoButton
          title="Temperature"
          info="The temperature of your computer's CPU"
         />
        </td>
        <td>
         {deviceInformation["temperature"] === null
          ? "No data"
          : deviceInformation["temperature"]}
        </td>
       </tr>
       <tr>
        <td>
         <WatchIcon className="inline-block h-6 pb-1 pr-1" /> Uptime{" "}
         <InfoButton
          title="Uptime"
          info="The time your computer is running since the last boot"
         />
        </td>
        <td>{deviceInformation["uptime"]}</td>
       </tr>
       <tr>
        <td>
         <Plug className="inline-block h-6 pb-1 pr-1" /> Power Supply
        </td>
        <td>{deviceInformation["power_supply"]}</td>
       </tr>
       <tr>
        <td>
         <List className="inline-block h-6 pb-1 pr-1" /> Active Processes{" "}
         <InfoButton
          title="Active Processes"
          info="The number of currently running processes on your system"
         />
        </td>
        <td>{deviceInformation["process_number"]}</td>
       </tr>
       <tr>
        <td>
         <TableIcon className="inline-block h-6 pb-1 pr-1" /> Zentrox PID{" "}
         <InfoButton
          title="Zentrox PID"
          info="The process ID of the currently running Zentrox instance"
         />
        </td>
        <td>{deviceInformation["zentrox_pid"]}</td>
       </tr>
      </tbody>
     </table>
    </div>
   </div>
  </Page>
 );
}

function Packages() {
 const { toast } = useToast();
 const [packagePopUpConfig, setPackagePopUp] = useState({
  visible: false,
  mode: "",
  packageName: "",
 });
 var packageSudoPasswordInput = useRef();
 const [installedPackages, setInstalledPackages] = useState([]);
 const [installedApps, setInstalledApps] = useState([]);
 const [otherPackages, setOtherPackages] = useState([]);
 const [autoRemovePackages, setAutoRemovePackages] = useState([]);
 const [visible, setVisibility] = useState(false);
 const [packageSearchValue, setPackageSearchValue] = useState("");
 const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] =
  useState("default");
 useEffect(() => fetchPackageList(), []);
 const [packagePopUpButtonState, setPackagePopUpButtonState] =
  useState("default");

 useEffect(() => {
  const storedApps = localStorage.getItem("installedApps");
  if (storedApps) {
   setInstalledApps(JSON.parse(storedApps));

   setVisibility(true);
  }
 }, []);

 useEffect(() => {
  if (installedApps.length > 0) {
   localStorage.setItem("installedApps", JSON.stringify(installedApps));
  }
  setVisibility(true);
 }, [installedApps]);

 // Effect for managing installedPackages from localStorage
 useEffect(() => {
  const storedPackages = localStorage.getItem("installedPackages");
  if (storedPackages) {
   setInstalledPackages(JSON.parse(storedPackages));

   setVisibility(true);
  }
 }, []);

 useEffect(() => {
  if (installedPackages.length > 0) {
   localStorage.setItem("installedPackages", JSON.stringify(installedPackages));

   setVisibility(true);
  }
 }, [installedPackages]);

 // Effect for managing otherPackages from localStorage
 useEffect(() => {
  const storedOtherPackages = localStorage.getItem("otherPackages");
  if (storedOtherPackages) {
   setOtherPackages(JSON.parse(storedOtherPackages));

   setVisibility(true);
  }
 }, []);

 useEffect(() => {
  if (otherPackages.length > 0) {
   localStorage.setItem("otherPackages", JSON.stringify(otherPackages));

   setVisibility(true);
  }
 }, [otherPackages]);

 function fetchPackageList() {
  if (
   installedPackages.length + installedApps.length + otherPackages.length !==
   0
  )
   return;
  fetch(fetchURLPrefix + "/api/packageDatabase", {
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setInstalledPackages(Array.from(json["packages"]));
     setInstalledApps(Array.from(json["apps"]));
     setOtherPackages(Array.from(json["others"]));
     setVisibility(true);
    });
   } else {
    toast({
     title: "Package Database Error",
     message: "Zentrox failed to retrieve a list of packages",
    });
    setVisibility(false);
   }
  });

  fetch(fetchURLPrefix + "/api/packageDatabaseAutoremove", {
   headers: {
    "Content-Type": "application/json",
   },
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setAutoRemovePackages(json["packages"]);
    });
   } else {
    toast({
     title: "Package Database Error",
     message: "Zentrox failed to retrieve a list of packages",
    });
    setVisibility(false);
   }
  });
 }

 function installPackage(packageName) {
  fetch(fetchURLPrefix + "/api/installPackage", {
   method: "POST",
   headers: {
    "Content-Type": "application/json",
   },
   body: JSON.stringify({
    packageName: packageName,
    sudoPassword: packageSudoPasswordInput.current.value,
   }),
  }).then((res) => {
   setPackagePopUp({
    visible: false,
    packageName: "",
    mode: "install",
   });
   setPackagePopUpButtonState("default");
   if (!res.ok) {
    toast({
     title: "Failed to install package",
     description: "Zentrox failed to install a package on your system.",
    });
   } else {
    setOtherPackages(
     otherPackages.filter((entry) => {
      if (entry.split(".")[0] === packageName) return false;
      return true;
     }),
    );
    setInstalledPackages([packageName, ...installedPackages]);
   }
  });
 }

 function removePackage(packageName) {
  fetch(fetchURLPrefix + "/api/removePackage", {
   method: "POST",
   headers: {
    "Content-Type": "application/json",
   },
   body: JSON.stringify({
    packageName: packageName,
    sudoPassword: packageSudoPasswordInput.current.value,
   }),
  }).then((res) => {
   setPackagePopUp({
    visible: false,
    packageName: "",
    mode: "remove",
   });
   setPackagePopUpButtonState("default");
   if (!res.ok) {
    toast({
     title: "Failed to remove package",
     description: "Zentrox failed to remove a package from your system.",
    });
   } else {
    setInstalledPackages(
     installedPackages.filter((entry) => {
      if (entry.split(".")[0] === packageName) return false;
      return true;
     }),
    );
    setOtherPackages([packageName, ...otherPackages]);
   }
  });
 }

 /**
  * @param {string} mode
  * @param {function} stateFn
  * @param {string} packageName */
 function packageActionPopUp(packageName, mode) {
  setPackagePopUp({
   visible: true,
   mode: mode,
   packageName,
  });
  setPackagePopUpButtonState("default");
 }

 function PackageBox({ packageName, task }) {
  const [buttonState, _] = useState("default");

  return (
   <div
    className="inline-block p-4 m-2 w-72 h-24 border border-neutral-600 md:w-52 text-white rounded-sm align-top relative"
    title={packageName}
   >
    <span className="block mb-1">
     {packageName.length > 20
      ? packageName.substring(0, 17) + "..."
      : packageName}
    </span>
    <Button
     className="block right-2 bottom-2 absolute"
     variant={task == "remove" ? "destructive" : "default"}
     onClick={() => {
      packageActionPopUp(packageName, task);
     }}
    >
     {(function () {
      if (task === "remove" && buttonState === "default") {
       return "Remove";
      } else if (task === "install" && buttonState === "default") {
       return "Install";
      } else if (buttonState === "working") {
       return (
        <>
         <Loader2 className="h-4 w-4 inline-block animate-spin" /> Working
        </>
       );
      } else if (buttonState === "failed") {
       return (
        <>
         <CircleX className="h-4 w-4 inline-block text-red-900" /> Failed
        </>
       );
      } else if (buttonState === "done") {
       return (
        <>
         <CircleCheck className="w-4 h-4 inline-block text-green-800" /> Done
        </>
       );
      }
     })()}
    </Button>
   </div>
  );
 }

 function AutoRemoveButon() {
  var sudoPasswordInput = useRef();
  if (clearAutoRemoveButtonState === "default") {
   return (
    <Dialog>
     <DialogTrigger asChild>
      <Button className="inline">
       <Paintbrush2 className="h-4 w-4 inline-block" /> Autoremove
      </Button>
     </DialogTrigger>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>Autoremove Packages</DialogTitle>
       <DialogDescription>
        Autoremove removes packages that are not requried by the system anymore
        according to your package manager.
        <br />
        It requires your sudo password.
       </DialogDescription>
      </DialogHeader>
      <Input type="password" title="Sudo password" ref={sudoPasswordInput} />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Close</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          setClearAutoRemoveButtonState("working");
          fetch("/api/clearAutoRemove", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            sudoPassword: sudoPasswordInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) {
            res.json().then((json) => {
             setAutoRemovePackages(json["packages"]);
            });
           } else {
            toast({
             title: "Failed to autoremove packages",
             description:
              "Zentrox failed to remove not needed packages from your system.",
            });
           }
           setClearAutoRemoveButtonState("default");
          });
         }}
        >
         Proceed
        </Button>
       </DialogClose>
      </DialogFooter>
     </DialogContent>
    </Dialog>
   );
  } else {
   return (
    <Button
     className="inline"
     onClick={() => {
      setClearAutoRemoveButtonState("working");
      fetch("/api/clearAutoRemove").then((res) => {
       if (res.ok) {
        res.json().then((json) => {
         setAutoRemovePackages(json["packages"]);
        });
        setClearAutoRemoveButtonState("default");
       }
      });
     }}
    >
     <Loader2 className="h-4 w-4 inline-block animate-spin" /> Working
    </Button>
   );
  }
 }

 if (visible) {
  if (packageSearchValue.length > 2) {
   var PackageView = (
    <>
     {installedPackages
      .filter((pkg) => {
       if (pkg == "Available") return false;
       if (pkg.length === 0) return false;
       if (pkg.includes(packageSearchValue)) {
        return true;
       }
       return false;
      })
      .sort((pkg) => {
       if (pkg == packageSearchValue) return -1;
       return +1;
      })
      .map((pkg, i) => {
       return (
        <PackageBox
         packageName={pkg.split(".")[0]}
         task="remove"
         key={i}
        ></PackageBox>
       );
      })}
     {otherPackages
      .filter((pkg) => {
       if (pkg == "Available") return false;
       if (pkg.length === 0) return false;
       if (pkg.includes(packageSearchValue)) {
        return true;
       }
       return false;
      })
      .sort((pkg) => {
       if (pkg == packageSearchValue) return -1;
       return +1;
      })
      .map((pkg, i) => {
       return (
        <PackageBox
         packageName={pkg.split(".")[0]}
         task="install"
         key={i}
        ></PackageBox>
       );
      })}
    </>
   );
  } else {
   var PackageView = (
    <>
     <div className="p-auto">
      <SearchIcon className="w-16 h-16 m-auto mt-8 text-neutral-600" />
      <br />
      <h3 className="text-xl text-neutral-600 m-auto text-center">
       Search for package to install or uninstall
      </h3>
     </div>
    </>
   );
  }
  return (
   <Page name="Packages">
    <Dialog open={packagePopUpConfig.visible} onOpenChange={setPackagePopUp}>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>
        {packagePopUpConfig.mode == "install" ? "Install" : "Remove"} package?
       </DialogTitle>
       <DialogDescription>
        Do you really want to remove {packagePopUpConfig.packageName}?
        <br />
        Please enter your sudo password to proceed.
       </DialogDescription>
      </DialogHeader>
      <Input
       type="password"
       placeholder="Password"
       ref={packageSudoPasswordInput}
      />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Close</Button>
       </DialogClose>
       <Button
        variant={
         packagePopUpConfig.mode == "install" ? "default" : "destructive"
        }
        onClick={(e) => {
         setPackagePopUpButtonState("working");
         if (packagePopUpConfig.mode == "install") {
          installPackage(packagePopUpConfig.packageName);
         } else {
          removePackage(packagePopUpConfig.packageName);
         }
        }}
       >
        {packagePopUpButtonState == "default" ? (
         <></>
        ) : (
         <Spinner visible={true} />
        )}
        {packagePopUpConfig.mode == "install"
         ? packagePopUpButtonState == "default"
           ? "Install Package"
           : "Installing Package"
         : packagePopUpButtonState == "default"
           ? "Remove Package"
           : "Removing Package"}
       </Button>
      </DialogFooter>
     </DialogContent>
    </Dialog>

    <StatCard
     name="Installed Packages"
     value={installedPackages.length}
     Icon={<HardDriveIcon className="h-5 w-5 inline-block" />}
     Info="Packages that are installed on your system. This includes apps."
    />
    <StatCard
     name="Installed Apps"
     value={installedApps.length}
     Icon={<AppWindow className="h-5 w-5 inline-block" />}
     Info="Packages that have a graphical interface and are installed on your system."
    />
    <StatCard
     name="Other Packages"
     value={otherPackages.length}
     Icon={<Package2 className="h-5 w-5 inline-block" />}
     Info="Packages including apps, that are not installed on your system but listed in your package manager."
    />
    <StatCard
     name="Autoremove Packages"
     value={autoRemovePackages.length}
     Icon={<TrashIcon className="h-5 w-5 inline-block" />}
     Info="Packages that are not required by the system anymore"
    />

    <br />
    <div className="h-fit">
     <Input
      placeholder="Search for package"
      onChange={(e) => {
       setPackageSearchValue(e.target.value);
      }}
      className="mt-2 inline-block"
     />{" "}
     <AutoRemoveButon />
    </div>
    <br />
    {PackageView}
   </Page>
  );
 } else {
  return (
   <div className="p-auto pt-5">
    <Loader2 className="animate-spin m-auto w-20 h-20" />
   </div>
  );
 }
}

function Firewall() {
 const [rules, setRules] = useState([]);
 const [fireWallEnabled, setFireWallEnabled] = useState(false);
 const [newRuleAction, setNewRuleAction] = useState("allow");
 const [preventRefetch, setPreventRefetch] = useState(false);
 const [sudoPassword, setSudoPassword] = useState("");
 const [sudoDialogOpen, setSudoDialogOpen] = useState(true);
 var newRuleTo = useRef("");
 var newRuleFrom = useRef("");
 var sudoPasswordInput = useRef("");
 const { toast } = useToast();

 function fetchFireWallInformation(password = sudoPassword) {
  fetch(fetchURLPrefix + "/api/fireWallInformation", {
   headers: {
    "Content-Type": "application/json",
   },
   method: "POST",
   body: JSON.stringify({
    sudoPassword: password,
   }),
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setRules(json["rules"]);
     setFireWallEnabled(json["enabled"]);
    });
   } else {
    toast({
     title: "Wrong Sudo Password",
     description: "Zentrox failed to validate your sudo password",
    });
   }
  });
 }

 function RuleView() {
  if (fireWallEnabled) {
   return (
    <div className="max-h-full min-h-fit overflow-y-scroll overflow-x-hidden w-fit no-scroll">
     <table className="pt-2 firewall block">
      <tbody>
       <tr
        className="w-fit animate-fadein"
        style={{
         animationDuration: `100ms`,
        }}
       >
        <td>
         <ArrowUpFromDot className="w-4 h-4 pb-0.5 inline" /> To
        </td>
        <td>
         <ArrowDownToDot className="w-4 h-4 pb-0.5 inline" /> From
        </td>
        <td>
         <Shield className="w-4 h-4 pb-0.5 inline" /> Action
        </td>
        <td></td>
       </tr>
       {rules.map((rule, i) => {
        return (
         <tr
          key={i}
          className="w-fit animate-fadein"
          style={{
           animationDuration: `${i > 6 ? 600 : i * 100}ms`,
          }}
         >
          <td>{rule.to.replaceAll("(v6)", "IPv6")}</td>
          <td>{rule.from.replaceAll("(v6)", "IPv6")}</td>
          <td>
           {rule.action === "DENY" ? (
            <>
             <Ban className="h-4 w-4 inline-block text-red-500 pr-1" />
             Deny
            </>
           ) : (
            <>
             <CircleCheck className="h-4 w-4 inline-block text-green-500 pr-1" />
             Allow
            </>
           )}
          </td>
          <td>
           <AlertDialog>
            <AlertDialogTrigger asChild>
             <Button className="bg-transparent text-white p-0 m-0 hover:bg-red-500/20 active:bg-red-500/30 w-12">
              <TrashIcon />
             </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
             <AlertDialogHeader>
              <AlertDialogTitle>Delete Rule</AlertDialogTitle>
              <AlertDialogDescription>
               Do you really want to remove this rule? This action can not be
               undone.
              </AlertDialogDescription>
             </AlertDialogHeader>
             <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
               onClick={() => {
                fetch(
                 fetchURLPrefix + "/api/deleteFireWallRule/" + rule.index,
                 {
                  method: "POST",
                  headers: {
                   "Content-Type": "application/json",
                  },
                  body: JSON.stringify({
                   sudoPassword: sudoPassword,
                  }),
                 },
                ).then((res) => {
                 if (!res.ok) {
                  toast({
                   title: "Failed to delete firewall rule",
                   description: `Zentrox failed to delete rule ${rule.index}.`,
                  });
                 } else {
                  fetchFireWallInformation();
                 }
                });
               }}
              >
               Proceed
              </AlertDialogAction>
             </AlertDialogFooter>
            </AlertDialogContent>
           </AlertDialog>
          </td>
         </tr>
        );
       })}
      </tbody>
     </table>
    </div>
   );
  } else {
   return (
    <span className="align-middle p-2 block">
     <BrickWall className="w-8 h-8 inline text-neutral-600" /> Firewall is
     disabled
    </span>
   );
  }
 }

 return (
  <>
   <Dialog
    open={sudoPassword == "" && sudoDialogOpen}
    onOpenChange={setSudoDialogOpen}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Sudo Password</DialogTitle>
      <DialogDescription className="text-white">
       To view the current state of your firewall, please enter your sudo
       password. The password will be saved as long as you are viewing the
       firewall tab. You will have to re-input it again if you leave the the
       firewall tab.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      placeholder="Sudo password"
      ref={sudoPasswordInput}
     />
     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
       <Button
        onClick={() => {
         setSudoPassword(sudoPasswordInput.current.value);
         fetchFireWallInformation(sudoPasswordInput.current.value);
        }}
       >
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <Toaster />
   <Page name="Firewall">
    <div className="w-64">
     <div>
      <Dialog>
       <DialogTrigger>
        <Button className="mr-1">
         <Plus className="h-6 w-6 inline" />
         New Rule
        </Button>
       </DialogTrigger>
       <DialogContent>
        <DialogHeader>
         <DialogTitle>New Firewall Rule</DialogTitle>
         <DialogDescription>
          You can create a new rule that applies to your firewall.
         </DialogDescription>
         <label htmlFor="ruleTo">
          <ArrowUpFromDot className="w-4 h-4 inline" /> To
         </label>
         <Input id="ruleTo" placeholder="Port" ref={newRuleTo} />
         <label htmlFor="ruleFrom">
          <ArrowDownToDot className="w-4 h-4 inline" /> From
         </label>
         <Input id="ruleFrom" placeholder="IP Address" ref={newRuleFrom} />
         <label htmlFor="ruleAction">
          <Shield className="w-4 h-4 inline" /> Action
         </label>
         <Select
          value={newRuleAction}
          onValueChange={(e) => {
           setPreventRefetch(true);
           setNewRuleAction(e);
          }}
         >
          <SelectTrigger className="w-[180px]">
           <SelectValue placeholder="Select an action" />
          </SelectTrigger>
          <SelectContent>
           <SelectGroup>
            <SelectItem value="allow">
             <CircleCheck className="w-4 h-4 inline mr-1 text-green-500" />{" "}
             Allow
            </SelectItem>
            <SelectItem value="deny">
             <Ban className="w-4 h-4 inline mr-1 text-red-500" /> Deny
            </SelectItem>
           </SelectGroup>
          </SelectContent>
         </Select>
         <DialogFooter>
          <DialogClose asChild>
           <Button variant="outline">Cancel</Button>
          </DialogClose>
          <DialogClose asChild>
           <Button
            onClick={() => {
             setPreventRefetch(false);
             if (
              newRuleTo.current.value.length === 0 ||
              typeof newRuleTo.current.value === "undefined"
             ) {
              toast({
               title: "Can not create firewall rule",
               description:
                "Zentrox can not create a rule with the provided details.",
              });
              return;
             }
             fetch(
              fetchURLPrefix +
               "/api/newFireWallRule/" +
               encodeURIComponent(
                newRuleFrom.current.value.length === 0
                 ? "any"
                 : newRuleFrom.current.value,
               ) +
               "/" +
               encodeURIComponent(newRuleTo.current.value) +
               "/" +
               encodeURIComponent(newRuleAction),
              {
               method: "POST",
               headers: {
                "Content-Type": "application/json",
               },
               body: JSON.stringify({
                sudoPassword: sudoPassword,
               }),
              },
             ).then((res) => {
              if (res.ok) {
               fetchFireWallInformation();
              } else {
               res.json().then((json) => {
                if (json["msg"] !== undefined) {
                 toast({
                  title: "Failed to create new rule",
                  description:
                   "Zentrox failed to create new firewall rule: " + json["msg"],
                 });
                } else {
                 toast({
                  title: "Failed to create new rule",
                  description: "Zentrox failed to create a new firewall rule",
                 });
                }
               });
              }
             });
            }}
           >
            Create
           </Button>
          </DialogClose>
         </DialogFooter>
        </DialogHeader>
       </DialogContent>
      </Dialog>
      <Switch
       onClick={(e) => {
        e.target.disabled = true;
        fetch(fetchURLPrefix + "/api/switchUfw/" + !fireWallEnabled, {
         method: "POST",
         headers: {
          "Content-Type": "application/json",
         },
         body: JSON.stringify({
          sudoPassword: sudoPassword,
         }),
        }).then((res) => {
         if (res.ok) {
          setFireWallEnabled(!fireWallEnabled);
         } else {
          toast({
           title: "Failed to apply firewall configuration",
           description: "Zentrox failed to change the state of the firewall.",
          });
         }
         e.target.disabled = false;
         fetchFireWallInformation();
        });
       }}
       value={fireWallEnabled ? "on" : "off"}
       checked={fireWallEnabled}
       title="Enable Firewall"
       className="ml-1"
      />
     </div>
     <RuleView />
    </div>
   </Page>
  </>
 );
}

function Files() {
 return (
  <>
   <Page name="Files">
    <FileView className=""></FileView>
   </Page>
  </>
 );
}

function Storage() {
 const { toast } = useToast();
 const [drivesList, setDrivesList] = useState([]);
 const [driveInformation, setDriveInformation] = useState({
  drives: {
   model: "N/A",
   path: "N/A",
   owner: "N/A",
   mountpoint: "",
   size: 0,
  },
  ussage: [],
 });
 const [currentDrive, setCurrentDrive] = useState([]);
 const [driveInformationDialogOpen, setDriveInformationDialogOpen] =
  useState(false);

 useEffect(() => {
  fetchDrivesList();
 }, []);

 function fetchDrivesList() {
  fetch(fetchURLPrefix + "/api/driveList").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setDrivesList(json["drives"]);
    });
   } else {
    toast({
     title: "Failed to fetch drives list",
     description:
      "Zentrox failed to fetch a list of all connected storage mediums.",
    });
   }
  });
 }

 function showDriveDetails(driveName) {
  setDriveInformationDialogOpen(true);
  setCurrentDrive(driveName);
  fetch(
   fetchURLPrefix + "/api/driveInformation/" + encodeURIComponent(driveName),
  ).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setDriveInformation(json);
    });
   } else {
    toast({
     title: "Failed to fetch drive informaiton",
     description: "Zentrox failed to fetch drive details",
    });
   }
  });
 }

 function DriveEntry({ entry, inset = 0 }) {
  var children = <></>;
  if (entry.children != null) {
   children = entry.children.map((entry) => {
    return <DriveEntry entry={entry} inset={inset + 1} />;
   });
  }

  return (
   <>
    {" "}
    <span
     className="w-full p-4 bg-transparent border border-neutral-800 border-x-transparent block cursor-default select-none hover:bg-neutral-800 hover:transition-bg hover:duration-400 duration-200 animate-fadein focus:bg-neutral-800 focus:duration-50"
     style={{
      paddingLeft: 16 + inset * 10,
     }}
     onClick={() => {
      showDriveDetails(entry.name);
     }}
    >
     {(function (entry) {
      if (entry.name.startsWith("loop")) {
       return (
        <RepeatIcon className="inline-block h-6 w-6 pr-1 text-neutral-700" />
       );
      } else if (inset != 0) {
       return <LayoutPanelTopIcon className="inline-block h-6 w-6 pr-1" />;
      } else {
       return <HardDriveIcon className="inline-block h-6 w-6 pr-1" />;
      }
     })(entry)}{" "}
     {entry.name}
    </span>
    {children}
   </>
  );
 }

 /**
  * @param {number} bytes
  * @description Converts the unit Bytes into a higher unit and add a unit symbol*
  * @returns {string} */
 function bytesUnitToOther(bytes) {
  if (bytes >= 1024 * 1024 * 1024) {
   return Math.round(bytes / (1024 * 1024 * 1024)) + " GB";
  } else if (bytes >= 1024 * 1024) {
   return Math.round(bytes / (1024 * 1024)) + " MB";
  } else if (bytes >= 1024) {
   return Math.round(bytes / 1024) + " KB";
  } else {
   return bytes + " B";
  }
 }

 var driveCapacity = "N/A";
 var drive;
 for (drive of driveInformation.ussage) {
  if (
   drive[0] === driveInformation.drives.mountpoint ||
   drive[0] == driveInformation.drives.path
  ) {
   driveCapacity = drive[4] + "%";
  }
 }

 return (
  <>
   <Toaster />
   <Dialog
    open={driveInformationDialogOpen}
    onOpenChange={setDriveInformationDialogOpen}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>{currentDrive}</DialogTitle>
     </DialogHeader>
     <DialogDescription className="text-white">
      <b className="block mb-1">
       <TagIcon className="w-4 h-4 inline" /> Model
      </b>
      {na(driveInformation.drives.model)}
      <br />
      <b className="block mb-1">
       <MapPinIcon className="w-4 h-4 inline" /> Path
      </b>
      {na(driveInformation.drives.path)} <br />
      <b className="block mb-1">
       <WeightIcon className="w-4 h-4 inline" /> Size
      </b>
      {na(bytesUnitToOther(driveInformation.drives.size))} <br />
      <b className="block mb-1">
       <UserIcon className="w-4 h-4 inline" /> Owner
      </b>
      {na(driveInformation.drives.owner)} <br />
      <b className="block mb-1">
       <MountainIcon className="w-4 h-4 inline" /> Mountpoint
      </b>
      {na(driveInformation.drives.mountpoint)}
      <b className="block mb-1">
       <PieChartIcon className="w-4 h-4 inline" /> Ussage (Capacity)
      </b>
      {na(driveCapacity)}
     </DialogDescription>
     <DialogFooter>
      <DialogClose asChild>
       <Button>Close</Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <Page name="Storage">
    <div
     className="rounded-xl m-2 overflow-hidden overflow-y-scroll border-2 border-neutral-800"
     style={{ maxHeight: "calc(100vh - 180px)" }}
    >
     {drivesList
      .sort((a) => {
       if (a.name.includes("loop")) return 1;
       return -1;
      })
      .map((entry) => {
       return <DriveEntry entry={entry} />;
      })}
    </div>
   </Page>
  </>
 );
}

function Vault() {
 var vaultEncryptionKey = useRef();
 var vaultKeyDecryptModal = useRef();
 var uploadInput = useRef();
 var newDirectoryInput = useRef();
 var renamingModalInput = useRef();

 const { toast } = useToast();
 const [decryptKeyModalVisible, setDecryptKeyModalVisibility] = useState(false);
 const [decryptModalCallback, setDecryptModalCallback] = useState(() => {});
 const [currentVaultPath, setCurrentVaultPath] = useState("");
 const [currentVaultContents, setCurrentVaultContents] = useState([]);
 const [vaultSessionKey, setVaultSessionKey] = useState("");
 const [uploadButton, setUploadButton] = useState("default");
 const [deletionModalVisible, setDeletionModalVisible] = useState(false);
 const [renamingModalVisible, setRenamingModalVisible] = useState(false);
 const [currentVaultFileRename, setCurrentVaultFileRename] = useState("");
 const [currentVaultFileDelete, setCurrentVaultFileDelete] = useState("");

 const [vaultConfigModalOpen, setVaultConfigModalOpen] = useState(false);
 const [vaultState, setVaultState] = useState("locked");

 useEffect(() => {
  if (vaultState == "locked") {
   fetch("/api/isVaultConfigured").then((r) => {
    if (r.ok) {
     r.text().then((t) => {
      console.log(t);
      if (t === "0") {
       setVaultState("unconfigured");
      }
     });
    } else {
     setVaultState("unconfigured");
    }
   });
  }
 }, [setVaultState]);

 function parentDir(path) {
  if (!path.endsWith("/")) path += "/";
  var parsedPath = path.split("/");
  parsedPath.pop();
  parsedPath.pop();
  var parentPath = parsedPath.join("/") + "/";
  if (parentPath === "/") parentPath = "";
  return parentPath;
 }

 function vaultTree(key = "") {
  var vaultKey;
  if (key === "") vaultKey = vaultKeyDecryptModal.current.value;
  else vaultKey = key;
  fetch(fetchURLPrefix + "/api/vaultTree", {
   method: "POST",
   headers: {
    "Content-Type": "application/json",
   },
   body: JSON.stringify({
    key: vaultKey,
   }),
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setCurrentVaultContents(json.fs);
     setVaultSessionKey(vaultKey);
     setVaultState("unlocked");
    });
   } else {
    if (res.status === 403) {
     toast({
      title: "Failed to authenticate",
      description: "Zentrox was unable to validate your key",
     });
    }
   }
  });
 }

 function requestRename() {
  setRenamingModalVisible(true);
 }

 function requestDeletion() {
  setDeletionModalVisible(true);
 }

 function isDirectChild(entry, currentVaultPath) {
  // Remove trailing `/` from entry if it exists
  entry = entry.endsWith("/") ? entry.slice(0, -1) : entry;

  console.log(currentVaultPath);

  // Check if the entry starts with the currentVaultPath
  if (!entry.startsWith(currentVaultPath)) return false;

  // Get the remaining part of the entry after currentVaultPath
  let remainingPath = entry.slice(currentVaultPath.length);

  console.log(remainingPath);

  // Check if the remaining path contains any `/`
  return !remainingPath.includes("/");
 }

 return (
  <Page name="Vault">
   <Toaster />
   <Dialog
    open={decryptKeyModalVisible}
    onOpenChange={setDecryptKeyModalVisibility}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Unlock Vault</DialogTitle>
      <DialogDescription>Please enter your vault key.</DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      placeholder="Current key"
      ref={vaultKeyDecryptModal}
     />
     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
       <Button
        onClick={() => {
         vaultTree();
        }}
       >
        <KeyIcon className="w-4 h-4 pr-1" /> Unlock
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <Dialog open={vaultConfigModalOpen} onOpenChange={setVaultConfigModalOpen}>
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Setup Vault</DialogTitle>
      <DialogDescription>
       Please enter a strong and secure password to configure vault. You need
       this password to view and upload files to vault.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      id="vaultEncryptionKey"
      ref={vaultEncryptionKey}
      placeholder="Key"
      className="inline"
     />

     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
       <Button
        variant="destructive"
        className="inline-block mb-1"
        onClick={() => {
         /** @type {string}*/
         var key = vaultEncryptionKey.current.value;
         if (key.length === 0) {
          toast({
           title: "Missing new key",
           description: "You need to input a new vault key",
          });
          return;
         }
         fetch(fetchURLPrefix + "/api/vaultConfigure", {
          method: "POST",
          headers: {
           "Content-Type": "application/json",
          },
          body: JSON.stringify({
           key: key,
          }),
         }).then((res) => {
          if (res.ok) {
           res.json().then((json) => {
            if (json.code === "no_decrypt_key") {
             noDecryptKeyModal();
            } else {
             toast({
              title: "Finished Vault configuration",
              description: "Vault has been configured and is ready for use",
             });
             setVaultState("locked");
            }
           });
          } else {
           if (res.status === 400) {
            toast({
             title: "Bad Request",
             description:
              "The data you provided was incorrect. The server responded with error 400.",
            });
           } else {
            toast({
             title: "Server Error " + res.status,
             description:
              "The server responded with an HTTP error of " + res.status * ".",
            });
           }
          }
         });
        }}
       >
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <div>
    <Button
     onClick={
      vaultState !== "unlocked"
       ? () => {
          location.reload();
         }
       : () => {
          setCurrentVaultContents([]);
          setCurrentVaultPath("");
          setVaultSessionKey("");
          setVaultState("locked");
         }
     }
     variant="destructive"
     className={
      vaultState !== "unlocked" ? "bg-blue-500 hover:bg-blue-600" : "mr-1"
     }
    >
     <LockIcon className="w-4 h-4 inline-block mr-1" />{" "}
     {vaultState !== "unlocked" ? "Reload" : "Exit"}
    </Button>
    <Dialog>
     <DialogTrigger asChild>
      <Button className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}>
       <FolderIcon className="w-4 h-4 inline-block mr-1" /> New Directory
      </Button>
     </DialogTrigger>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>New Directory</DialogTitle>
       <DialogDescription>Create a new directory.</DialogDescription>
      </DialogHeader>
      <Input type="text" ref={newDirectoryInput} placeholder="Name" />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Close</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          if (
           newDirectoryInput.current.value.includes("/") ||
           newDirectoryInput.current.value.includes(" ")
          ) {
           toast({
            title: "Illegal name",
            description: "A file name may not include slashes or spaces.",
           });
           return;
          }
          if (newDirectoryInput.current.value.length > 64) {
           toast({
            title: "Filename too long",
            description: "A filename can not be longer than 64 characters.",
           });
           return;
          }
          fetch(fetchURLPrefix + "/api/vaultNewFolder", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            key: vaultSessionKey,
            folder_name:
             currentVaultPath + "/" + newDirectoryInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) {
            vaultTree(vaultSessionKey);
           } else {
            toast({
             title: "Failed to create new directory",
             description: `Vault could not create a new directory ${newDirectoryInput.current.value} in ${currentVaultPath}`,
            });
           }
          });
         }}
        >
         <FolderIcon className="w-4 h-4 inline-block mr-1" /> Create
        </Button>
       </DialogClose>
      </DialogFooter>
     </DialogContent>
    </Dialog>
    <Dialog open={renamingModalVisible} onOpenChange={setRenamingModalVisible}>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>Rename File</DialogTitle>
       <DialogDescription>Rename a file</DialogDescription>
      </DialogHeader>
      <Input type="text" ref={renamingModalInput} placeholder="New Name" />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Cancel</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          fetch(fetchURLPrefix + "/api/renameVaultFile", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            key: vaultSessionKey,
            path: currentVaultFileRename,
            newName: currentVaultPath + "/" + renamingModalInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) vaultTree(vaultSessionKey);
           else
            toast({
             title: "Failed to rename file",
             description: "Zentrox failed to rename a file.",
            });
          });
         }}
        >
         <PenLineIcon className="w-4 h-4 inline-block mr-1" /> Rename
        </Button>
       </DialogClose>
      </DialogFooter>
     </DialogContent>
    </Dialog>
    <AlertDialog
     open={deletionModalVisible}
     onOpenChange={setDeletionModalVisible}
    >
     <AlertDialogContent>
      <AlertDialogTitle>Delete File</AlertDialogTitle>
      <AlertDialogDescription>
       Do you really want to delete{" "}
       {currentVaultFileDelete.length > 64
        ? currentVaultFileDelete.substring(0, 61) + "..."
        : currentVaultFileDelete}
       ?<br />
       This action can not be undone.
      </AlertDialogDescription>
      <AlertDialogFooter>
       <AlertDialogCancel>Cancel</AlertDialogCancel>
       <AlertDialogAction
        onClick={() => {
         fetch(fetchURLPrefix + "/api/deleteVaultFile", {
          method: "POST",
          headers: {
           "Content-Type": "application/json",
          },
          body: JSON.stringify({
           key: vaultSessionKey,
           deletePath: currentVaultFileDelete,
          }),
         }).then((res) => {
          if (res.ok) vaultTree(vaultSessionKey);
          else
           toast({
            title: "Failed to delete file",
            description: "Zentrox failed to delete a file.",
           });
         });
        }}
       >
        Delete
       </AlertDialogAction>
      </AlertDialogFooter>
     </AlertDialogContent>
    </AlertDialog>
    <Button
     className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}
     onClick={() => {
      uploadInput.current.click();
     }}
    >
     {uploadButton === "default" ? (
      <UploadIcon className="w-4 h-4 inline-block mr-1" />
     ) : (
      <LoaderIcon className="animate-spin h-4 w-4 inline mr-1" />
     )}{" "}
     Upload File
    </Button>
    <input
     type="file"
     ref={uploadInput}
     onInput={() => {
      if (event.target.files.length > 0) {
       setUploadButton("loading");
       var fileForSubmit = uploadInput.current.files[0];
       if (fileForSubmit.size >= 1024 * 1024 * 1024 * 10) {
        toast({
         title: "File to big",
         description: "The file you provided was larger than 10GB",
        });
       }
       var formData = new FormData();
       formData.append("file", fileForSubmit);
       formData.append("path", currentVaultPath);
       formData.append("key", vaultSessionKey);
       fetch(fetchURLPrefix + "/upload/vault", {
        method: "POST",
        body: formData,
       }).then((res) => {
        uploadInput.current.value = "";
        if (res.ok) {
         vaultTree(vaultSessionKey);
         setUploadButton("default");
        } else {
         setUploadButton("default");
         toast({
          title: "Failed to upload file",
          description: "Zentrox failed to upload the file you provided",
         });
        }
       });
      }
     }}
     hidden
    />
    <Button
     className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}
     onClick={() => {
      setCurrentVaultPath(parentDir(currentVaultPath));
     }}
    >
     <ArrowUp className="w-4 h-4 inline-block mr-1" /> Up
    </Button>
   </div>
   <div
    className={`no-scroll h-fit rounded-xl mt-2 overflow-hidden overflow-y-scroll no-scroll`}
    style={{
     minHeight: "fit-content",
     maxHeight: "calc(100vh - 220px)",
    }}
   >
    {vaultState == "locked" ? (
     <span className="h-fit">
      <div className="text-center text-2xl opacity-50">
       <LockIcon className="m-auto h-52 w-52" />
       Vault Is Locked
      </div>
      <Button
       className="m-auto block mt-4"
       onClick={() => {
        setDecryptKeyModalVisibility(true);
       }}
      >
       Unlock Vault
      </Button>
     </span>
    ) : vaultState == "unconfigured" ? (
     <span className="h-fit">
      <div className="text-center text-2xl opacity-50">
       <CogIcon className="m-auto h-52 w-52" />
       Vault Needs To Be Configured
      </div>
      <Button
       className="m-auto block mt-4"
       onClick={() => {
        setVaultConfigModalOpen(true);
       }}
      >
       Setup Vault
      </Button>
     </span>
    ) : (
     ""
    )}
    {
     /*
      * @param {string} entry*/
     currentVaultContents
      .filter((entry) => {
       return isDirectChild(entry, currentVaultPath);
      })
      .map((entry) => {
       var type = "";
       if (entry.endsWith("/")) {
        type = "folder";
       } else {
        type = "file";
       }
       return (
        <ContextMenu>
         <ContextMenuContent>
          <ContextMenuItem
           onClick={() => {
            setCurrentVaultFileDelete(entry);
            requestDeletion(entry);
           }}
          >
           <DeleteIcon className="w-4 h-4 inline-block mr-1" /> Delete
          </ContextMenuItem>
          <ContextMenuItem
           onClick={() => {
            setCurrentVaultFileRename(entry);
            requestRename(entry);
           }}
          >
           <PenLineIcon className="w-4 h-4 inline-block mr-1" /> Rename
          </ContextMenuItem>
         </ContextMenuContent>
         <ContextMenuTrigger>
          <span
           className="w-full p-4 bg-transparent block cursor-default select-none hover:bg-neutral-900 hover:transition-bg hover:duration-300 focus:outline-blue-500 focus:duration-50"
           onClick={
            type === "folder"
             ? () => {
                setCurrentVaultPath(entry);
               }
             : (e) => {
                e.target.classList.add("animate-pulse");
                e.target.classList.add("duration-300");

                e.target.classList.remove("duration-200");
                fetch(fetchURLPrefix + "/api/vaultFileDownload", {
                 method: "POST",
                 headers: {
                  "Content-Type": "application/json",
                 },
                 body: JSON.stringify({
                  key: vaultSessionKey,
                  path: entry,
                 }),
                }).then((res) => {
                 e.target.classList.remove("animate-pulse");
                 e.target.classList.remove("duration-300");

                 e.target.classList.add("duration-200");
                 if (res.ok) {
                  res.blob().then((blob) => {
                   var url = window.URL.createObjectURL(blob);
                   var a = document.createElement("a");
                   a.href = url;
                   a.download = entry;
                   document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
                   a.click();
                   a.remove();
                  });
                 } else {
                  toast({
                   title: "File download error",
                  });
                 }
                });
               }
           }
          >
           {type === "folder" ? (
            <FolderIcon className="w-6 h-6 inline-block mr-1" fill="white" />
           ) : (
            <FileIcon className="w-6 h-6 inline-block mr-1" />
           )}{" "}
           {type === "folder"
            ? entry.split("/").at(-2)
            : entry.split("/").at(-1)}
          </span>
         </ContextMenuTrigger>
        </ContextMenu>
       );
      })
    }
   </div>
  </Page>
 );
}

function Sharing() {
 var ftpUserNameInput = useRef();
 var ftpPassWordInput = useRef();
 var ftpRootInput = useRef();
 var ftpApplySudoPasswordInput = useRef();
 var tlsCertFileInput = useRef();

 const [ftpConfig, setFtpConfig] = useState({
  enabled: false,
  ftpUserName: "",
  ftpLocalRoot: "",
 });
 const [ftpCheckBoxChecked, setFtpCheckBoxChecked] = useState(false);
 const [certNames, setCertName] = useState({
  tls: "TLS Certificate",
 });

 const fetchData = () => {
  fetch(fetchURLPrefix + "/api/fetchFTPconfig").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setFtpConfig(json);
     setFtpCheckBoxChecked(json.enabled);
     ftpUserNameInput.current.value = json.ftpUserUsername;
     ftpRootInput.current.value = json.ftpLocalRoot;
    });
   } else {
    toast({
     title: "Failed to fetch FTP configuration",
     description: "Zentrox failed to fetch the current FTP configuration",
    });
   }
  });

  fetch("/api/certNames").then((res) => {
   res.json().then((j) => {
    setCertName(j);
   });
  });
 };

 useEffect(fetchData, []);

 return (
  <Page name="Sharing">
   <h1 className="text-xl">
    Certificates{" "}
    <InfoButton
     title={"Certificates"}
     info={
      <>
       Zentrox automatically generates self signed certificates to provide an
       encrypted connection.
       <br />
       This connection is not protected from Man-In-The-Middle attacks, which is
       why it is recommended to use a SSL certificate by a Certificate
       Authority.
      </>
     }
    />
   </h1>
   <h2 className="font-semibold">TLS</h2>
   <input
    type="file"
    ref={tlsCertFileInput}
    hidden={true}
    onChange={() => {
     var fileForSubmit = tlsCertFileInput.current.files[0];
     if (fileForSubmit.size >= 1024 * 1024 * 1024 * 1) {
      toast({
       title: "File to big",
       description: "The file you provided was larger than 1GB",
      });
     }

     var fileName = tlsCertFileInput.current.files[0].name;

     if (fileName.split(".").reverse()[0].toLowerCase() != "pem") {
      toast({
       title: "Not a pem file",
       description: "Zentrox can only use pem certificates.",
      });
     }

     var formData = new FormData();
     formData.append("file", fileForSubmit);
     fetch(fetchURLPrefix + "/upload/tls", {
      method: "POST",
      body: formData,
     }).then((res) => {
      if (res.ok) {
       setCertName({
        tls: fileName,
       });
       tlsCertFileInput.current.value = "";
       toast({
        title: "Upload finished",
        description:
         "Zentrox successfully uploaded the new certificate. You need to manually restart Zentrox to start using the new certificate.",
        duration: 200000,
       });
      } else {
       toast({
        title: "Failed to upload TLS certificate",
        description:
         "Zentrox failed to upload the TLS certificate you provided",
       });
      }
     });
    }}
   />
   <Button
    onClick={() => {
     tlsCertFileInput.current.click();
    }}
    className="mr-1"
   >
    Upload
   </Button>{" "}
   <span className="text-neutral-600">{certNames.tls}</span>
   <h1 className="text-xl pt-3">
    FTP{" "}
    <InfoButton
     title={"File Transfer Protocol"}
     info={
      <>
       The FTP Protocol is used to transfer files. <br />
       Zentrox automatically encrypts the trafic using the provided
       certificates.
       <br />
      </>
     }
    />
   </h1>
   <Checkbox
    onClick={() => {
     setFtpCheckBoxChecked(!ftpCheckBoxChecked);
     console.log(ftpCheckBoxChecked);
    }}
    id="ftpUserEnabledCheckBox"
    className="ml-1"
    checked={ftpCheckBoxChecked}
   />{" "}
   <label htmlFor="ftpUserEnabledCheckBox">Enabled</label>
   <br />
   <Label className="block p-1">Username</Label>
   <Input
    type="text"
    ref={ftpUserNameInput}
    placeholder="Username"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Username"}
    info={
     <>
      The FTP username is the username that is used to connect to your FTP
      server. By Default it is: <code>ftp_zentrox</code>
     </>
    }
   />
   <br />
   <Label className="block p-1">Password</Label>
   <Input
    type="password"
    ref={ftpPassWordInput}
    placeholder="Password"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Password"}
    info={
     <>
      The FTP password is the password that is used to connect to your FTP
      server. <br />
      You should change it to prevent getting hacked.
      <br />
      By Default it is: <code>change_me</code>
     </>
    }
   />
   <br />
   <Label className="block p-1">Root Directory</Label>
   <Input
    type="text"
    ref={ftpRootInput}
    placeholder="FTP Root Directory"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Root"}
    info={
     <>
      The FTP root directory is the directory that a connected FTP user can
      access. By Default it is: <code>/</code>
     </>
    }
   />
   <br />
   <Dialog>
    <DialogTrigger>
     <Button>Apply</Button>
    </DialogTrigger>
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Elevated privileges</DialogTitle>
      <DialogDescription>
       Zentrox requires your sudo password for this action.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      ref={ftpApplySudoPasswordInput}
      placeholder="Password"
     />
     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Close</Button>
      </DialogClose>
      <DialogClose
       onClick={() => {
        fetch(fetchURLPrefix + "/api/updateFTPConfig", {
         method: "POST",
         headers: {
          "Content-Type": "application/json",
         },
         body: JSON.stringify({
          enableDisable: false,
          enableFTP: ftpCheckBoxChecked,
          ftpUserUsername: ftpUserNameInput.current.value,
          ftpLocalRoot: ftpRootInput.current.value,
          ftpUserPassword: ftpPassWordInput.current.value,
          sudoPassword: ftpApplySudoPasswordInput.current.value,
         }),
        }).then((res) => {
         if (res.ok) {
          toast({
           title: "FTP server updated",
           description: "Zentrox updated your FTP server configuration",
          });
         } else {
          toast({
           title: "FTP server error",
           description: "Failed to update FTP server configuration",
          });
         }
        });
       }}
       asChild
      >
       <Button>
        <KeyIcon className="w-4 h-4 inline-block mr-1" />
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
  </Page>
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
      className="inline-table float-right cursor-pointer"
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

function Logs() {
 const [selectedLog, setSelectedLog] = useState("");
 const [sudoPasswordModal, setSudoPasswordModal] = useState(true);
 const [sudoPassword, setSudoPassword] = useState("");
 const [logRefreshKeyValue, setLogRefreshKeyValue] = useState(0);
 const [tableFilter, setTableFilter] = useState("");
 const [since, setSince] = useState(
  (function () {
   let currentDate = new Date();
   currentDate.setHours(currentDate.getHours() - 4);
   return currentDate;
  })(),
 );

 const [until, setUntil] = useState(
  (function () {
   let currentDate = new Date();
   currentDate.setHours(currentDate.getHours());
   return currentDate;
  })(),
 );
 let sudoPasswordInput = useRef();
 let searchInput = useRef();

 const LogTable = ({ log, logFilter }) => {
  const [logEntries, setLogEntries] = useState([["", "", "", ""]]);
  const [logInfo, setLogInfo] = useState("");
  const [logMessageColor, setLogMessageColor] = useState("white");

  const Td = ({ children, orientation = "center", className }) => {
   return (
    <td
     className={
      "p-2 hover:bg-neutral-900 rounded cursor-default text-" +
      orientation +
      " " +
      className
     }
    >
     {children}
    </td>
   );
  };

  const fetchLog = (log, sudo, since, until) => {
   setLogInfo(`Fetching for logs`);
   setLogMessageColor("blue");
   fetch(
    fetchURLPrefix +
     "/api/logs/" +
     log +
     "/" +
     new Date(since).getTime() +
     "/" +
     new Date(until).getTime(),
    {
     method: "POST",
     body: JSON.stringify({
      sudoPassword: sudo,
     }),
     headers: {
      "Content-Type": "application/json",
     },
    },
   ).then((res) => {
    if (res.ok) {
     setLogInfo(`Viewing log`);
     setLogMessageColor("green");
     res.json().then((j) => {
      setLogEntries(j.logs);
     });
    } else {
     setLogEntries([["", "", "", ""]]);
     setLogInfo(`Failed to fetch log`);
     setLogMessageColor("red");
    }
   });
  };

  useEffect(() => {
   if (sudoPassword != "") {
    fetchLog(log, sudoPassword, since, until);
   } else {
    setLogInfo(`Missing sudo password`);
    setLogMessageColor("yellow");
   }
  }, []);

  function formatTimestamp(timestamp) {
   if (parseInt(timestamp) < 1000 || timestamp == "") {
    return "";
   }

   // Convert the timestamp string to an integer
   const date = new Date(parseInt(Math.floor(timestamp / 1000))); // Multiply by 1000 to convert seconds to milliseconds

   // Format the date to a human-readable format
   const formattedDate = date.toLocaleString("en-GB", {
    year: "2-digit",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: true,
   });

   return formattedDate;
  }
  return (
   <>
    <span className={`m-2 mt-1 mb-1 block text-${logMessageColor}-500`}>
     {logInfo}
    </span>
    <div className="overflow-scroll no-scroll">
     <table className="mt-2">
      <tr>
       <Td className="font-bold">Time</Td>
       <Td className="font-bold">Application</Td>
       <Td className="font-bold">Message</Td>
      </tr>
      {logEntries
       .filter((e) => {
        if (tableFilter.length != 0) {
         if (tableFilter.startsWith("application: ")) {
          return (
           e[2].toLowerCase() ===
           tableFilter.split("application: ")[1].toLowerCase()
          );
         } else if (tableFilter.startsWith("time: ")) {
          const stamp = Number(e[0]);
          const passedStamp = Number(tableFilter.split("time: ")[1]) * 1000;
          return Math.abs(passedStamp - stamp) < 10000;
         } else if (tableFilter.startsWith("priority: ")) {
          const passedPrio = tableFilter.split("priority: ")[1];
          return e[4] === passedPrio;
         } else if (tableFilter.startsWith("excluded_application: ")) {
          return (
           e[2].toLowerCase() ==
           tableFilter.split("excluded_application: ")[1].toLowerCase()
          );
         } else {
          return (
           e[2].toLowerCase().includes(tableFilter.toLowerCase()) ||
           e[3].toLowerCase().includes(tableFilter.toLowerCase()) ||
           e[4] == tableFilter
          );
         }
        } else {
         return true;
        }
       })
       .reverse()
       .map((logEntry) => {
        return (
         <tr>
          <Td orientation="left">{formatTimestamp(logEntry[0])}</Td>
          <Td orientation="left">{logEntry[2]}</Td>
          <Td
           orientation="left"
           className={(function () {
            var level = logEntry[4];
            if (level == "7") {
             return "text-white/50"; // Verbose
            } else if (level == "6") {
             return "text-blue-400";
            } else if (level == "5") {
             return "text-lime-300";
            } else if (level == "4") {
             return "text-orange-400";
            } else if (level == "3") {
             return "text-red-500";
            } else if (level == "2") {
             return "text-rose-600";
            } else if (level == "1") {
             return "text-purple-500 text-bold";
            } else if (level == "0") {
             return "text-black bg-red-500 text-bold hover:bg-red-500"; // Emergency
            }
           })()}
          >
           {logEntry[3]}
          </Td>
         </tr>
        );
       })}
     </table>
    </div>
   </>
  );
 };

 const CurrentTimer = () => {
  const [timeFormat, setTimeFormat] = useState("unix");
  const [time, setTime] = useState(currentTime());

  const formats = ["unix", "human.dot", "human.slash", "human.dash"];

  useEffect(() => {
   if (localStorage.getItem("logsTimeFormat") != undefined) {
    setTimeFormat(localStorage.getItem("logsTimeFormat"));
   }
  }, []);

  function currentTime() {
   const z = (v) => {
    if (v < 10) {
     return "0" + v;
    } else {
     return v;
    }
   };

   if (timeFormat == "unix") {
    return new Date().getTime();
   } else if (timeFormat.startsWith("human.")) {
    let date = new Date();
    let day = z(date.getDate());
    let month = z(date.getMonth() + 1);
    let year = date.getFullYear();
    let minute = z(date.getMinutes());
    let hour = z(date.getHours());
    let second = z(date.getSeconds());

    if (timeFormat == "human.dot") {
     return `${day}.${month}.${year} ${hour}:${minute}:${second}`; // Used in post soviet countries
    } else if (timeFormat == "human.slash") {
     return `${month}/${day}/${year} ${hour}:${minute}:${second}`;
    } else if (timeFormat == "human.dash") {
     return `${month}-${day}-${year} ${hour}:${minute}:${second}`;
    }
   }
  }

  useEffect(() => {
   const interval = setInterval(() => {
    setTime(currentTime());
   }, 250);

   return () => clearInterval(interval);
  }, []);

  return (
   <span
    onClick={() => {
     let currentFormatIndex = formats.indexOf(timeFormat);
     if (currentFormatIndex > formats.length - 2) {
      currentFormatIndex = 0;
     } else {
      currentFormatIndex++;
     }
     localStorage.setItem("logsTimeFormat", formats[currentFormatIndex]);
     setTimeFormat(formats[currentFormatIndex]);
     console.log(timeFormat);
    }}
    className="cursor-pointer inline-flex"
   >
    {currentTime()}
   </span>
  );
 };

 return (
  <Page name="Logs">
   <Dialog open={sudoPasswordModal} onOpenChange={setSudoPasswordModal}>
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Sudo password</DialogTitle>
      <DialogDescription>
       Please enter your sudo password to view your log files.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      placeholder="Sudo password"
      ref={sudoPasswordInput}
     />

     <DialogFooter>
      <DialogClose>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose>
       <Button
        onClick={() => {
         setSudoPassword(sudoPasswordInput.current.value);
         setSelectedLog("messages");
        }}
       >
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>

   <span className="m-1 block"></span>
   <Button
    onClick={() => {
     setLogRefreshKeyValue(new Date().getTime());
    }}
    className="mr-1"
    variant="secondary"
   >
    Refresh
   </Button>
   <Input
    type="text"
    placeholder="Search"
    ref={searchInput}
    className="inline-flex mr-1 ml-1"
    onKeyPress={(event) => {
     if (event.key == "Enter") {
      setTableFilter(searchInput.current.value);
     }
    }}
   />
   <InfoButton
    title="Search filters"
    info={
     <>
      <p>
       You can use search filters to query system logs for specific entries.
       <br /> This is done by writing{" "}
       <code className="p-1 rounded bg-white/10 m-1">prefix: query</code>
       <br />
       The following prefixes are supported:
       <br />
       <ul className="list-dice">
        <li>application</li>
        <li>time</li>
        <li>priority</li>
       </ul>
      </p>
      <p>Or simply enter a keyword to search for a message</p>
     </>
    }
   />

   <CalendarButton
    placeholder="Since"
    className="mr-1 ml-1"
    onValueChange={setSince}
    confirmMode={true}
   />
   <CalendarButton
    placeholder="Until"
    className="mr-1"
    onValueChange={setUntil}
    confirmMode={true}
   />
   <Dialog>
    <DialogTrigger asChild>
     <Button
      variant="secondary"
      className="mr-1"
      title="Show QR code for Alerts app"
     >
      Alerts App
     </Button>
    </DialogTrigger>

    <DialogContent>
     <DialogHeader>
      <DialogTitle>Alerts App</DialogTitle>
      <DialogDescription>
       Zentrox Alerts lets you view current system statistics and logs.
      </DialogDescription>
      <QRCodeSVG
       value={"https://" + location.host + "/alerts"}
       className="w-48 h-48"
      />
      <DialogFooter>
       <DialogClose asChild>
        <Button>Close</Button>
       </DialogClose>
      </DialogFooter>
     </DialogHeader>
    </DialogContent>
   </Dialog>
   <CurrentTimer />
   <br />
   <LogTable
    log={selectedLog}
    refreshKey={logRefreshKeyValue}
    tableFilter={tableFilter}
   />
  </Page>
 );
}

function Media() {
 const [mediaEnabled, setMediaEnabled] = useState(false);
 const [locations, setLocations] = useState([["", "", true]]);
 const [fetchedLocations, setFetchedLocations] = useState(false);

 useEffect(() => {
  if (!fetchedLocations) {
   fetch(`${fetchURLPrefix}/api/getVideoSourceList`).then((res) => {
    if (res.ok) {
     res.json().then((j) => setLocations(j.locations));
    } else {
     toast({
      title: "Failed to fetch video location list",
      description: "Zentrox failed to fetch the list of video locations",
     });
    }
   });
   setFetchedLocations(true);
  }
 }, [fetchedLocations]);

 const updateList = () => {
  fetch(`${fetchURLPrefix}/api/updateVideoSourceList`, {
   method: "POST",
   headers: {
    "Content-Type": "application/json",
   },
   body: JSON.stringify({
    locations,
   }),
  });
 };

 return (
  <Page name="Media">
   <Toaster />
   <div className="flex items-center">
    <Checkbox
     onValueChange={setMediaEnabled}
     value={mediaEnabled}
     className="inline-block mr-1 align-middle"
    />{" "}
    Enabled
   </div>

   <DataTable entries={locations} onEntriesChange={setLocations}>
    <Button className="ml-1" onClick={() => updateList()}>
     Apply
    </Button>
   </DataTable>
  </Page>
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
