import {
 ArrowDownIcon,
 ArrowUpIcon,
 EthernetPortIcon,
 HourglassIcon,
 KeyIcon
} from "lucide-react";

import { useState, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import fetchURLPrefix from "@/lib/fetchPrefix";
import InfoButton from "@/components/ui/InfoButton";
import localFont from 'next/font/local'
const segment7 = localFont({ src: '../../../public/7segment.ttf' })

function Card({ title, children, skeleton, variant = "square" }) {
 return (
  <span
   className={
    "p-4 m-2 rounded-xl overflow-hidden overflow-ellipsis whitespace-pre bg-zinc-950 border-zinc-900 border inline-block hover:border-zinc-800 duration-500" +
    (skeleton ? " animate-pulse duration-[5000ms]" : "") +
    (variant == "square" ? " h-44 aspect-square" : " h-44 aspect-video")
   }
  >
   <span className="block mb-2 font-bold text-lg">{title}</span>
   {skeleton ? (
    <></>
   ) : (
    <span className="animate-fadein duration-300">{children}</span>
   )}
  </span>
 );
}

function FancyCounter ({ children }) {
	return (
		// Google Fonts' "Workbench" for smaller texts
		// https://torinak.com/font/7-segment for larger numbers
		<span className="bg-black/5 p-2 rounded-lg w-full block text-white transition-all duration-200 hover:bg-black/25 text-center cursor-default select-none">
			{
				children
			}
		</span>
	)
}
function FancyCounterDigit ({ children }) {
	return <span className="pl-1 pr-1 h-auto inline-block relative">
		<span className={segment7.className + " font-semibold text-4xl absolute bottom-[-2px] z-10"}>
		{children}
</span>
<span className={segment7.className + " font-semibold text-4xl inline-block bottom-[-2px] relative z-0 opacity-15"}>8</span>
	</span>
}
function FancyCounterCaption ({children}) {
	return <span className={segment7.className + " mr-1 text-xl text-white/80 relative bottom-[-2px]"}>
		{children}
	</span>
}

export default function Overview() {
 async function overviewFetch() {
  var t_a = Date.now()
  setReadyForFetch(false);
  // Fetch new data for CPU usage
   // Fetch new data for Device Information
   var devInfoFetch = await fetch(fetchURLPrefix + "/api/deviceInformation", {
    method: "GET",
    headers: {
     "Content-Type": "application/json",
    },
   });



   devInfoFetch.json().then((json) => {
    setDeviceInformation(json);
   });

	 setFetchDuration(Date.now() - t_a)
  setTimeout(() => {
  setReadyForFetch(true);}, 500)
 }

 const [deviceInformation, setDeviceInformation] = useState({
  zentrox_pid: "",
  process_number: "",
  hostname: "",
  uptime: 0,
  temperature: "",
  net_up: 0,
  net_down: 0,
  net_connected_interfaces: 0,
  net_interface: "",
  memory_total: 0,
  memory_free: 0,
  cpu_usage: 0,
  package_manager: "",
  ssh_active: false,
  unloaded: true
 });
 const [packageStatistics, setPackageStatistics] = useState({
	installed: [""],
	available: [""],
	packageManager: "",
	 canProvideUpdates: false,
	 updates: [""],
	 unloaded: true
	
 })
 const [readyForFetch, setReadyForFetch] = useState(true);
 const [fetchDuration, setFetchDuration] = useState(0)
 const tryOverviewFetch = () => {
	 if (readyForFetch) {
   overviewFetch();
  }
 };

 useEffect(() => {
  const interval = setInterval(() => {
   tryOverviewFetch();
  }, 5000);

  return () => clearInterval(interval);
 }, [readyForFetch]);

 useEffect(() => {
  tryOverviewFetch();
	 fetch(fetchURLPrefix + "/api/packageDatabase", {
   headers: {
    "Content-Type": "application/json",
   },
}).then((res) => {
	 res.json().then((json) => {
		 setPackageStatistics({
			 installed: json.packages,
			 available: json.others,
			 packageManager: json.packageManager,
			 canProvideUpdates: json.canProvideUpdates,
			 updates: json.updates,
			 unloaded: false
		 })
	 })})
 }, []);


function millisecondsToArray(milliseconds) {
    const MS_IN_SECOND = 1000;
    const MS_IN_MINUTE = MS_IN_SECOND * 60;
    const MS_IN_HOUR = MS_IN_MINUTE * 60;
    const MS_IN_DAY = MS_IN_HOUR * 24;

    // Helper function to round down and reduce remaining milliseconds
    const getUnitValue = (ms, unitMs) => {
        const value = Math.floor(ms / unitMs);
        return { value, remainder: ms % unitMs };
    };

    if (milliseconds >= MS_IN_DAY * 100) {
        const days = Math.floor(milliseconds / MS_IN_DAY);
        return [[days.toString(), "d"]];
    } else if (milliseconds >= MS_IN_DAY) {
        const { value: days, remainder } = getUnitValue(milliseconds, MS_IN_DAY);
        const { value: hours } = getUnitValue(remainder, MS_IN_HOUR);
        return [[days.toString(), "d"], [hours.toString(), "h"]];
    } else if (milliseconds >= MS_IN_HOUR) {
        const { value: hours, remainder } = getUnitValue(milliseconds, MS_IN_HOUR);
        const { value: minutes } = getUnitValue(remainder, MS_IN_MINUTE);
        return [[hours.toString(), "h"], [minutes.toString(), "m"]];
    } else if (milliseconds >= MS_IN_MINUTE) {
        const { value: minutes, remainder } = getUnitValue(milliseconds, MS_IN_MINUTE);
        const { value: seconds } = getUnitValue(remainder, MS_IN_SECOND);
        return [[minutes.toString(), "m"], [seconds.toString(), "s"]];
    } else {
        const seconds = Math.floor(milliseconds / MS_IN_SECOND);
        return [[seconds.toString(), "s"]];
    }
}
 function prettyBytes(bytesPerSecond) {
  if (bytesPerSecond === 0) return "0 B/s";

  const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
  const k = 1000;
  const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
  const value = Math.round(bytesPerSecond / Math.pow(k, i));

  return `${value.toFixed(2)} ${units[i]}`;
 }

 return (
  <Page name="Overview" className="align-top">
   <Card title={"Memory usage"} skeleton={deviceInformation.unloaded === true}>
    <span
     className={
      "text-5xl mb-2 font-semibold inline-block" +
      ((deviceInformation.memory_total - deviceInformation.memory_free) / deviceInformation.memory_total * 100 < 30
       ? " text-green-500"
       : (deviceInformation.memory_total - deviceInformation.memory_free) / deviceInformation.memory_total * 100 < 60
         ? " text-orange-500"
         : (deviceInformation.memory_total - deviceInformation.memory_free) / deviceInformation.memory_total * 100 < 90
           ? " text-red-500"
           : " text-purple-500")
     }
    >
     {Math.round((deviceInformation.memory_total - deviceInformation.memory_free) / deviceInformation.memory_total * 100)}%
    </span>
    <br />
    {Math.round((deviceInformation.memory_total - deviceInformation.memory_free) / Math.pow(1000, 3))}GB /{" "}
    {Math.round(deviceInformation.memory_total / Math.pow(1000, 3))}GB
   </Card>

   <Card title={"CPU usage"} skeleton={deviceInformation.unloaded === true}>
    <span
     className={
      "text-5xl mb-2 font-semibold inline-block" +
      (deviceInformation.cpu_usage < 30
       ? " text-green-500"
       : deviceInformation.cpu_usage < 60
         ? " text-orange-500"
         : deviceInformation.cpu_usage < 90
           ? " text-red-500"
           : " text-purple-500")
     }
    >
     {Math.round(deviceInformation.cpu_usage)}%
    </span>
    <br />
    {deviceInformation.temperature == -300
     ? "No temperature"
     : Math.round(deviceInformation.temperature) + "°C"}
   </Card>

   <Card
    title={"Networking"}
    variant="wide"
skeleton={deviceInformation.unloaded === true}
   >
    <span className="inline-block mr-2 mb-2">
     <strong className="block">Hostname</strong>
     {deviceInformation.hostname}
    </span>
    <span className="inline-block mr-2 mb-2">
     <strong className="block">Private IP</strong>
     <span
      className="cursor-pointer acitve:text-green-500"
      onClick={() => {
       try {
        window.navigator.clipboard.writeText(deviceInformation.ip);
       } catch {}
      }}
     >
      {deviceInformation.ip}
     </span>
    </span>
    <br />
    <span className="inline-block mr-2 mb-2">
     <strong>Activity on {deviceInformation.net_interface}</strong>{" "}
     <InfoButton
      title={"Network statistics"}
      info={
       <>
        Up: Bytes transmitted
        <br />
        Down: Bytes received
        <br />
        <p>
         Network statistics rely on the IP command on your system to measure
         transmitted and received bytes. Zentrox measures the change in bytes in
         an interval of 5 seconds and calculates the average resulting in B/s.
        </p>
       </>
      }
     />
     <br />
     <ArrowUpIcon className="h-4 w-4 mr-1 inline-block" />
     {prettyBytes(deviceInformation.net_up)}{" "}
     <ArrowDownIcon className="h-4 w-4 ml-1 mr-1 inline-block" />
     {prettyBytes(deviceInformation.net_down)}
    </span>
   </Card><br />
	 <Card title={"Uptime"} variant="square" 
skeleton={deviceInformation.unloaded === true}
>
		<FancyCounter>
	 {
		 millisecondsToArray(deviceInformation.uptime).map((e, k) => {
			 return <span key={k}>{e[0].split("").map((d, dk) => {
			  return <FancyCounterDigit key={dk}>{d}</FancyCounterDigit>
			 })}

			 <FancyCounterCaption>{e[1]}</FancyCounterCaption>

			</span>
			 
		 })
	 }
		</FancyCounter><strong>Active since:</strong><br />
	{ new Date((Date.now() - deviceInformation.uptime)).toLocaleDateString("en-US",{
		 day: "2-digit",
		 weekday: "narrow",
		 year: "numeric",
		 month: "2-digit",
	 }) }
	 </Card>
<Card title={"Packages" + {
	"": "",
	"pacman": " using PacMan",
	"apt": " using APT",
	"dnf": " using DNF",
}[packageStatistics.packageManager]} variant="wide" 
skeleton={packageStatistics.unloaded}
>
	<span className="inline-block mr-2 mb-2">
     <strong className="block">Available packages</strong>
     {packageStatistics.available.length}
    </span>

	{
		!packageStatistics.canProvideUpdates ?
	 <span className="block mr-2 mb-2">
     <strong className="block">Installed packages</strong>
     {packageStatistics.installed.length}
    </span> :  <span className="block mr-2 mb-2">
     <strong className="block">Available updates</strong>
     {packageStatistics.updates.length}
    </span>
	}

	</Card>
	 
<Card title={"Connectivity"} variant="square" 
skeleton={deviceInformation.unloaded}
>
	<span title={deviceInformation.net_connected_interfaces + " connected interface" + (deviceInformation.net_connected_interfaces > 1 ? "s" : "")}>
	<EthernetPortIcon className="h-4 w-4 mr-1 inline-block" />
	{
		deviceInformation.net_connected_interfaces
	} interface {deviceInformation.net_connected_interfaces > 1 ? "s" : ""}</span><br />
	<span title={deviceInformation.net_connected_interfaces + " connected interfaces"}>
	<KeyIcon className="h-4 w-4 mr-1 inline-block" />
	SSHd {deviceInformation.ssh_active ? "active" : "inactive"} <InfoButton title={"SSH activitiy detection"} info={
		"Zentrox checks if the sshd command is running. It can not detect any other SSH servers."
	} />
	</span>
	<br />
<span title={"latency between Zentrox server and backend"}>
	<HourglassIcon className="h-4 w-4 mr-1 inline-block" />
	{Math.round(fetchDuration / 1000) < 1 ? "< 1" : Math.round(fetchDuration / 1000)}s latency <InfoButton title={"Latency measurement"} info={
		"Zentrox measures the time it takes to complete a request for the current server status. Such a request is only sent every five seconds."
	} />
	</span>
	</Card>

  </Page>
 );
}
