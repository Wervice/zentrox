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
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

function NavigationBar({ onPageChange, activePage, pages = [] }) {
 return (
  <div className="fixed bottom-0 w-full block">
   {pages.map((e) => {
    return (
     <span
      className={
       "w-1/3 inline-block p-5 text-center text-white cursor-pointer hover:bg-blue-500/20 hover:text-blue-400 transition-colors duration-200 " +
       (activePage == e ? "text-blue-500 bg-blue-500/10" : "")
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
  <div className="h-full block w-full">
   <h1 className="p-2 text-3xl font-bold">
    {title}{" "}
    <Button
     className="bg-transparent hover:bg-white/10 text-white hover:text-white fixed top-1 right-1"
     onClick={() => {
      fetch("/logout", {
		  method: "POST"
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

function Firewall() {
 return <Page title="Firewall"></Page>;
}

function Log() {
 const [sudoPassword, setSudoPassword] = useState("")
 const [since, setSince] = useState(Date.now() * 1000)
 const [until, setUntil] = useState((Date.now() + 1000 * 60) * 1000)
 const [log, setLog] = useState("")
 const [errorMessage, setErrorMessage] = useState("")
 var sudoPasswordInputReference = useRef();

 useEffect(() => {
	if (sudoPassword !== "") {
		fetch("/api/logs/messages/" + since + "/" + until, {
			method: "POST",
			body: JSON.stringify({
				sudoPassword: sudoPassword
			}),
			headers: {
				"Content-Type": "application/json"
			}
		}).then((res) => {
			if (res.ok) {
				res.json().then((j) => {
					setLog(j)
				});
			} else {
				setErrorMessage("Failed to fetch log entries")
			}
		})
	}
 }, [sudoPassword])

 return <Page title="Log">
	<div hidden={sudoPassword !== ""} className="p-2">
		<h2 className="font-semibold text-xl block">Sudo Password</h2>
		<small className="block text-white/70">Zentrox requires a sudo password to view log files.</small>
		<Input type="password" placeholder="Sudo Password" className="block mb-2" ref={sudoPasswordInputReference} />
		<Button onClick={
			() => {
				setSudoPassword(sudoPasswordInputReference.current.value)
			}
		}>Confirm</Button>
	</div>
	<div hidden={sudoPassword === ""} className="h-full p-2">
		<span className="text-red-500 block">{errorMessage}</span>
		<Button className="bg-transparent border-0 text-white active:bg-white/10 hover:bg-transparent left-2 fixed">-1h</Button>
		<Button className="bg-transparent border-0 text-white active:bg-white/10 hover:bg-transparent right-2 fixed">+1h</Button>
		{

		}
	</div>
 </Page>;
}

function Home() {
 const [hostname, setHostname] = useState("");
 const [uptime, setUptime] = useState("");
 const [temperature, setTemperature] = useState("");
 const [powersupply, setPowersupply] = useState("");
 const [statusWarning, setStatusWarning] = useState("");
 const [cpuPercentage, setCpuPercentage] = useState("");
 const [memoryPercentage, setMemoryPercentage] = useState("");
 const [storagePercentage, setStoragePercentage] = useState("");

 const [warnings, setWarnings] = useState([]);

 const Td = function ({ children }) {
  return <td className="inline-table mr-2">{children}</td>;
 };

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
    <CircleAlertIcon className="w-4 h-4 mr-2 inline-block" /> Warning<br />
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
      <Td>{Math.round(cpuPercentage)}%</Td>
     </tr>
     <tr>
      <Td>
       <MemoryStickIcon className="w-4 h-4 inline-block mr-1" />
       Memory Ussage
      </Td>
      <Td>{Math.round(memoryPercentage)}%</Td>
     </tr>
     <tr>
      <Td>
       <FileIcon className="w-4 h-4 inline-block mr-1" />
       Storage Ussage
      </Td>
      <Td>{Math.round(storagePercentage)}%</Td>
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
   {activePage === "firewall" && <Firewall />}
   <NavigationBar
    onPageChange={setActivePage}
    activePage={activePage}
    pages={["home", "log", "firewall"]}
   />
  </div>
 );
}
