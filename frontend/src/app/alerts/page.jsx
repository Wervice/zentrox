"use client";

import { useEffect, useRef, useState } from "react";
import {
 ThermometerIcon,
 ComputerIcon,
 ClockIcon,
 BatteryIcon,
 CpuIcon,
 MemoryStickIcon,
 FileIcon,
 CircleAlertIcon,
 Loader,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
 Dialog,
 DialogFooter,
 DialogHeader,
 DialogContent,
 DialogTitle,
 DialogClose,
} from "@/components/ui/dialog";

function NavigationBar({ onPageChange, activePage, pages = [] }) {
 return (
  <div className="fixed bottom-0 w-full block text-center bg-zinc-950">
   {pages.map((e) => {
    return (
     <span
      className={
       "w-1/3 inline-block m-3 p-3 rounded-xl text-center text-white cursor-pointer transition-colors duration-100 " +
       (activePage == e ? "bg-zinc-900" : "")
      }
      onClick={() => {
       onPageChange(e);
      }}
     >
      {e}
     </span>
    );
   })}
  </div>
 );
}

function Page({ children, title }) {
 return (
  <div className="h-full block w-full overflow-scroll">
   <h1 className="p-2 text-3xl font-bold">
    {title}{" "}
    <Button
     className="bg-transparent hover:bg-white/10 text-white hover:text-white fixed top-1 right-1"
     onClick={() => {
      fetch("/logout", {
       method: "POST",
      }).then(() => {
       location.href = "/";
      });
     }}
    >
     Logout
    </Button>
   </h1>{" "}
   {children}
  </div>
 );
}

function Log() {
 const [sudoPassword, setSudoPassword] = useState("");
 const [since, setSince] = useState(Date.now() - 1000 * 60 * 60); // Since one hour ago
 const [until, setUntil] = useState(Date.now()); // Until now
 const [log, setLog] = useState([]);
 const [errorMessage, setErrorMessage] = useState("");
 const [activeLogMessage, setActiveLogMessage] = useState([]);
 const [activeLogMessagePopupVisible, setActiveLogMessagePopupVisible] =
  useState(false);
 const [filterString, setFilterString] = useState("");
 var sudoPasswordInputReference = useRef();
 var searchInput = useRef();

 useEffect(() => {
  if (sudoPassword !== "") {
   fetch("/api/logs/messages/" + since + "/" + until, {
    method: "POST",
    body: JSON.stringify({
     sudoPassword: sudoPassword,
    }),
    headers: {
     "Content-Type": "application/json",
    },
   }).then((res) => {
    if (res.ok) {
     res.json().then((j) => {
      setLog(j.logs);
     });
    } else {
     setErrorMessage("Failed to fetch log entries");
    }
   });
  }
 }, [sudoPassword, since, until]);

 return (
  <Page title="Log">
   <div hidden={sudoPassword !== ""} className="p-2">
    <h2 className="font-semibold text-xl block mb-2">Sudo Password</h2>
    <small className="block">
     Zentrox requires your sudo password to fully read system logs.
     <br />
     Please enter it bellow.
    </small>
    <Input
     type="password"
     placeholder="password"
     className="mb-2 mr-1"
     ref={sudoPasswordInputReference}
    />
    <Button
     onClick={() => {
      setSudoPassword(sudoPasswordInputReference.current.value);
     }}
    >
     Confirm
    </Button>
   </div>
   <div hidden={sudoPassword === ""} className="h-full p-2">
    <span className="text-red-500 block">{errorMessage}</span>
    <Button
     className="bg-transparent border-0 text-white active:bg-white/10 hover:bg-transparent left-2 fixed"
     onClick={() => {
      setSince(since - 1000 * 60 * 60);
      setUntil(until - 1000 * 60 * 60);
     }}
    >
     -1h
    </Button>
    <Button
     className="bg-transparent border-0 text-white active:bg-white/10 hover:bg-transparent right-2 fixed"
     onClick={() => {
      setSince(since + 1000 * 60 * 60);
      setUntil(until + 1000 * 60 * 60);
     }}
    >
     +1h
    </Button>
    <div
     className="mt-10 overflow-scroll"
     style={{
      maxHeight: "calc(100vh - 250px)",
     }}
    >
     {log.length === 0 ? (
      <div className="text-center font-bold text-2xl">No Logs</div>
     ) : (
      <></>
     )}
     {log
      .filter((e) => {
       if (filterString === "") {
        return true;
       }
       if (e[3].includes(filterString) || e[2].includes(filterString)) {
        return true;
       }
       return false;
      })
      .map((e) => {
       const application = e[2];
       const message = e[3];
       const priority = e[4];
       const colors = {
        0: "purple",
        1: "red",
        2: "orange",
        3: "yellow",
        4: "blue",
        5: "green",
        6: "neutral",
        7: "neutral",
       };
       return (
        <div
         className={
          "p-2 rounded mb-2 bg-neutral-900 cursor-pointer text-" +
          colors[priority] +
          "-500"
         }
         onClick={() => {
          setActiveLogMessage(e);
          setActiveLogMessagePopupVisible(true);
         }}
        >
         <strong>{application}</strong>
         <br />
         <span className="text-neutral-600">{message.substr(0, 40)}...</span>
        </div>
       );
      })}
    </div>
   </div>
   <Dialog
    open={activeLogMessagePopupVisible}
    onOpenChange={setActiveLogMessagePopupVisible}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>{activeLogMessage[2]}</DialogTitle>
     </DialogHeader>
     <span className="mb-1 text-white/60">
      Time:{" "}
      {activeLogMessage[0] !== undefined
       ? new Date(activeLogMessage[0] / 1000).toLocaleString()
       : ""}
     </span>
     <span className="w-full max-w-screen-sm">{activeLogMessage[3]}</span>
     <DialogFooter>
      <DialogClose asChild>
       <Button>Close</Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
  </Page>
 );
}

function Home() {
 const fetching = (
  <span className="pr-1 pl-1 rounded bg-white/10 text-white">
   <Loader className="animate-spin w-4 h-4 inline-block mr-1" /> Fetching
  </span>
 );
 const [hostname, setHostname] = useState(fetching);
 const [uptime, setUptime] = useState(fetching);
 const [temperature, setTemperature] = useState(fetching);
 const [powersupply, setPowersupply] = useState(fetching);
 const [cpuPercentage, setCpuPercentage] = useState(fetching);
 const [memoryPercentage, setMemoryPercentage] = useState(fetching);
 const [storagePercentage, setStoragePercentage] = useState(fetching);
 const [statusWarning, setStatusWarning] = useState("");

 const [warnings, setWarnings] = useState([]);

 const Td = function ({ children }) {
  return <td className="inline-table mr-2">{children}</td>;
 };

 useEffect(() => {
  if ("serviceWorker" in navigator) {
   window.addEventListener("load", () => {
    navigator.serviceWorker
     .register("/worker.js")
     .then((registration) => {
      console.log("ServiceWorker registered: ", registration);
     })
     .catch((error) => {
      console.log("ServiceWorker registration failed: ", error);
     });
   });
  }
 }, []);

 function fetchData() {
  fetch("/api/deviceInformation").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setHostname(json.hostname);
     setUptime(json.uptime);
     setTemperature(json.temperature);
     setPowersupply(json.power_supply);

     // Remove temperature warning if temperature is below or equal to 70°C
     setWarnings((prevWarnings) =>
      prevWarnings.filter(
       (warning) => warning !== "System temperature is above 70°C",
      ),
     );

     if (json.temperature != "No data") {
      if (Number(json.temperature.split("°C")[0]) > 70) {
       setWarnings((prevWarnings) => {
        if (!prevWarnings.includes("System temperature is above 70°C")) {
         return prevWarnings.concat("System temperature is above 70°C");
        }
        return prevWarnings;
       });
      }
     }

     // Remove battery warning if power supply is not discharging
     setWarnings((prevWarnings) =>
      prevWarnings.filter((warning) => warning !== "Battery is discharging"),
     );

     if (json.power_supply.includes("Discharging")) {
      setWarnings((prevWarnings) => {
       if (!prevWarnings.includes("Battery is discharging")) {
        return prevWarnings.concat("Battery is discharging");
       }
       return prevWarnings;
      });
     }
    });
   } else {
    setStatusWarning(`Failed to fetch ${res.status}`);
   }
  });

  fetch("/api/cpuPercent").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setCpuPercentage(Number(json.p));

     // Remove CPU warning if usage is below or equal to 90%
     setWarnings((prevWarnings) =>
      prevWarnings.filter((warning) => warning !== "CPU Ussage is above 90%"),
     );

     if (Number(json.p) > 90) {
      setWarnings((prevWarnings) => {
       if (!prevWarnings.includes("CPU Ussage is above 90%")) {
        return prevWarnings.concat("CPU Ussage is above 90%");
       }
       return prevWarnings;
      });
     }
    });
   } else {
    setStatusWarning(`Failed to fetch ${res.status}`);
   }
  });

  fetch("/api/ramPercent").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setMemoryPercentage(Number(json.p));

     // Remove memory warning if usage is below or equal to 80%
     setWarnings((prevWarnings) =>
      prevWarnings.filter(
       (warning) => warning !== "Memory Ussage is above 80%",
      ),
     );

     if (Number(json.p) > 80) {
      setWarnings((prevWarnings) => {
       if (!prevWarnings.includes("Memory Ussage is above 80%")) {
        return prevWarnings.concat("Memory Ussage is above 80%");
       }
       return prevWarnings;
      });
     }
    });
   } else {
    setStatusWarning(`Failed to fetch ${res.status}`);
   }
  });

  fetch("/api/diskPercent").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setStoragePercentage(Number(json.p));

     // Remove storage warning if usage is below or equal to 70%
     setWarnings((prevWarnings) =>
      prevWarnings.filter(
       (warning) => warning !== "Storage Ussage is above 80%",
      ),
     );

     if (Number(json.p) > 80) {
      setWarnings((prevWarnings) => {
       if (!prevWarnings.includes("Storage Ussage is above 80%")) {
        return prevWarnings.concat("Storage Ussage is above 80%");
       }
       return prevWarnings;
      });
     }
    });
   } else {
    setStatusWarning(`Failed to fetch ${res.status}`);
   }
  });
 }

 useEffect(() => {
  fetchData();
 }, [
  hostname,
  uptime,
  temperature,
  powersupply,
  setPowersupply,
  setTemperature,
  setUptime,
  setHostname,
 ]);

 useEffect(() => {
  let i = setInterval(fetchData, 4000);
  return () => clearInterval(i);
 }, [
  hostname,
  uptime,
  temperature,
  powersupply,
  setPowersupply,
  setTemperature,
  setUptime,
  setHostname,
 ]);

 return (
  <Page title="Home">
   <div
    className="bg-red-500/20 border-1 border-red-500 rounded p-2 m-2"
    hidden={warnings.length === 0}
   >
    <CircleAlertIcon className="w-4 h-4 mr-2 inline-block" /> Warning
    <br />
    {warnings.map((e) => {
     return (
      <>
       {e}
       <br />
      </>
     );
    })}
   </div>

   <div className="m-2 border-1 border-green-500/20 p-2 bg-green-500/5 rounded">
    <span className="text-red-500">{statusWarning}</span>
    <table>
     <tr>
      <Td>
       <ComputerIcon className="w-4 h-4 inline-block mr-1" />
       Hostname
      </Td>
      <Td>{hostname}</Td>
     </tr>
     <tr>
      <Td>
       <ThermometerIcon className="w-4 h-4 inline-block mr-1" />
       CPU Temp
      </Td>
      <Td>{temperature}</Td>
     </tr>
     <tr>
      <Td>
       <ClockIcon className="w-4 h-4 inline-block mr-1" />
       Uptime
      </Td>
      <Td>{uptime}</Td>
     </tr>
     <tr>
      <Td>
       <BatteryIcon className="w-4 h-4 inline-block mr-1" />
       Power
      </Td>
      <Td>{powersupply}</Td>
     </tr>
    </table>
   </div>

   <div className="m-2 border-1 border-yellow-500/20 p-2 bg-yellow-500/5 rounded">
    <span className="text-red-500">{statusWarning}</span>
    <table>
     <tr>
      <Td>
       <CpuIcon className="w-4 h-4 inline-block mr-1" />
       CPU Ussage
      </Td>
      <Td>
       {typeof cpuPercentage === "number"
        ? Math.round(cpuPercentage)
        : fetching}
       %
      </Td>
     </tr>
     <tr>
      <Td>
       <MemoryStickIcon className="w-4 h-4 inline-block mr-1" />
       Memory Ussage
      </Td>
      <Td>
       {typeof memoryPercentage === "number"
        ? Math.round(memoryPercentage)
        : fetching}
       %
      </Td>
     </tr>
     <tr>
      <Td>
       <FileIcon className="w-4 h-4 inline-block mr-1" />
       Storage Ussage
      </Td>
      <Td>
       {typeof storagePercentage === "number"
        ? Math.round(storagePercentage)
        : fetching}
       %
      </Td>
     </tr>
    </table>
   </div>
  </Page>
 );
}

export default function Overview() {
 const [activePage, setActivePage] = useState("home");

 return (
  <div>
   <link rel="manifest" href="/alerts/manifest.json" />
   {activePage === "home" && <Home />}
   {activePage === "log" && <Log />}
   <NavigationBar
    onPageChange={setActivePage}
    activePage={activePage}
    pages={["home", "log"]}
   />
  </div>
 );
}
