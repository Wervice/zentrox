import { Button } from "@/components/ui/button.jsx";
import {
 HardDriveIcon,
 AppWindow,
 Package2,
 Loader2,
 CircleX,
 SearchIcon,
 TrashIcon,
 Paintbrush2,
 CircleCheck,
} from "lucide-react";
import { useEffect, useState, useRef } from "react";
import "./table.css";
import "./scroll.css";
import Spinner from "@/components/ui/Spinner.jsx";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
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
import Page from "@/components/ui/PageWrapper";
import { useToast } from "@/components/ui/use-toast";
import fetchURLPrefix from "@/lib/fetchPrefix";

function Packages() {
 const { toast } = useToast();
 const [packagePopUpConfig, setPackagePopUp] = useState({
  visible: false,
  mode: "",
  packageName: "",
 });
 var packageSudoPasswordInput = useRef();
 const [installedPackages, setInstalledPackages] = useState([]);
 const [otherPackages, setOtherPackages] = useState([]);
 const [autoRemovePackages, setAutoRemovePackages] = useState([]);
 const [visible, setVisibility] = useState(false);
 const [packageSearchValue, setPackageSearchValue] = useState("");
 const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] =
  useState("default");
 useEffect(() => fetchPackageList(), []);
 const [packagePopUpButtonState, setPackagePopUpButtonState] =
  useState("default");


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
   installedPackages.length + otherPackages.length !==
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
      <Input
       type="password"
       placeholder="Sudo password"
       ref={sudoPasswordInput}
      />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Close</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          setClearAutoRemoveButtonState("working");
          fetch(fetchURLPrefix + "/api/clearAutoRemove", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            sudoPassword: sudoPasswordInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) {
            setAutoRemovePackages([]);
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
     name="Installed packages"
     value={installedPackages.length}
     Icon={<HardDriveIcon className="h-5 w-5 inline-block" />}
     Info="Packages that are installed on your system. This includes apps."
    />
    <StatCard
     name="Available packages"
     value={otherPackages.length}
     Icon={<Package2 className="h-5 w-5 inline-block" />}
     Info="Packages including apps, that are not installed on your system but listed in your package manager."
    />
    <StatCard
     name="Unused packages"
     value={autoRemovePackages.length}
     Icon={<TrashIcon className="h-5 w-5 inline-block" />}
     Info="Packages that are not required by the system anymore"
    />

    <br />
    <div className="h-fit">
     <Input
      placeholder="e.g. Apache, Nginx or Nextcloud"
      onChange={(e) => {
       setPackageSearchValue(e.target.value);
      }}
      className="mt-2 w-[300px] w-max-[75vw] inline-block"
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

export default Packages;
