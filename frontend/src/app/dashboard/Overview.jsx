import {
 ArrowDownIcon,
 ArrowUpIcon,
 WifiIcon,
 EthernetPortIcon,
} from "lucide-react";

import { useState, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import fetchURLPrefix from "@/lib/fetchPrefix";
import InfoButton from "@/components/ui/InfoButton";

function Card({ title, children, skeleton, variant = "square" }) {
 console.log(skeleton);
 return (
  <span
   className={
    "p-4 m-2 rounded-xl overflow-hidden overflow-ellipsis whitespace-pre bg-zinc-950 border-zinc-900 border inline-block hover:bg-zinc-900 hover:duration-300" +
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

function Overview() {
 async function overviewFetch() {
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
	 
  setTimeout(() => {
  setReadyForFetch(true);}, 500)
 }

 const [deviceInformation, setDeviceInformation] = useState({
  os_name: "",
  zentrox_pid: "",
  process_number: "",
  hostname: "",
  uptime: "",
  temperature: "",
  net_up: 0,
  net_down: 0,
  net_interface: "",
  memory_total: 0,
  memory_free: 0,
  cpu_usage: 0,
  amount_installed_packages: 0
 });
 const [readyForFetch, setReadyForFetch] = useState(true);

 const tryOverviewFetch = () => {
  console.log(readyForFetch)
	 if (readyForFetch) {
   overviewFetch();
  }
 };

 useEffect(() => {
  const interval = setInterval(() => {
   tryOverviewFetch();
  }, 1000);

  return () => clearInterval(interval);
 }, [readyForFetch]);

 useEffect(() => {
  tryOverviewFetch();
 }, []);

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
   <Card title={"Memory usage"} skeleton={deviceInformation.memory_total == 0}>
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

   <Card title={"CPU usage"} skeleton={deviceInformation.cpu_usage == 0}>
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
     : Math.round(deviceInformation.cpu_usage) + "°C"}
   </Card>

   <Card
    title={"Networking"}
    variant="wide"
    skeleton={deviceInformation.hostname == ""}
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
   </Card>
	 <br />
	 <Card variant="square" title={"Packages"} skeleton={deviceInformation.amount_installed_packages === 0}>    <span className="inline-block mr-2 mb-2">
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
     /></span></Card>
  </Page>
 );
}

export default Overview;
