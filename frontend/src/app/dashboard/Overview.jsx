import {
  ArrowDownIcon,
  ArrowUpIcon,
  EthernetPortIcon,
  HourglassIcon,
} from "lucide-react";

import { useState, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import fetchURLPrefix from "@/lib/fetchPrefix";
import InfoButton from "@/components/ui/InfoButton";
import localFont from "next/font/local";
import { toast } from "@/components/ui/use-toast";
import { VT323 } from "next/font/google";
import useNotification from "@/lib/notificationState";
import DateWrapper from "@/components/ui/Date";
const vt323 = VT323({
  weight: "400",
  subsets: ["latin"],
});
const segment7 = localFont({ src: "../../../public/7segment.ttf" });

function Card({ title, children, skeleton, variant = "square" }) {
  return (
    <span
      className={
        "p-4 m-2 rounded-xl overflow-hidden overflow-ellipsis whitespace-pre bg-zinc-950 border-zinc-900 border inline-block hover:border-zinc-800 transition-all duration-500" +
        (skeleton ? " opacity-30 scale-90" : "") +
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

function FancyCounter({ children }) {
  return (
    <span className="bg-black/5 p-2 rounded-lg w-full block text-white transition-all duration-200 hover:bg-black/25 text-center cursor-default select-none">
      {children}
    </span>
  );
}
function FancyCounterDigit({ children }) {
  return (
    <span className="pl-1 pr-1 h-auto inline-block relative">
      <span
        className={
          segment7.className +
          " font-semibold text-4xl absolute bottom-[-2px] z-10"
        }
      >
        {children}
      </span>
      <span
        className={
          segment7.className +
          " font-semibold text-4xl inline-block bottom-[-2px] relative z-0 opacity-15"
        }
      >
        8
      </span>
    </span>
  );
}
function FancyCounterCaption({ children }) {
  return (
    <span
      className={
        vt323.className + " mr-1 text-xl text-white/80 relative bottom-[-2px]"
      }
    >
      {children}
    </span>
  );
}

export default function Overview() {
  const { deleteNotification, notifications, notify } = useNotification();
  async function overviewFetch() {
    var t_a = Date.now();
    setReadyForFetch(false);

    var devInfoFetch = await fetch(fetchURLPrefix + "/api/deviceInformation", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    });

    devInfoFetch.json().then((json) => {
      let jk = json;
      jk.unloaded = false;

      if (!packageStatisticsFetchedOnce) {
        setPackageStatisticsFetchedOnce(true);
        fetch(fetchURLPrefix + "/api/packageDatabase/true", {
          headers: {
            "Content-Type": "application/json",
          },
        })
          .then((res) => {
            res.json().then((json) => {
              setPackageStatistics({
                installed: json.packages,
                available: json.others,
                packageManager: json.packageManager,
                canProvideUpdates: json.canProvideUpdates,
                updates: json.updates,
                unloaded: false,
              });
            });
          })
          .catch((_e) => {
            notify("Reading package database failed");
          });
      }

      setDeviceInformation(jk);
    });

    setFetchDuration(Date.now() - t_a);
    setTimeout(() => {
      setReadyForFetch(true);
    }, 500);
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
    unloaded: true,
  });
  const [packageStatistics, setPackageStatistics] = useState({
    installed: 0,
    available: 0,
    packageManager: "",
    canProvideUpdates: false,
    updates: 0,
    unloaded: true,
  });
  const [readyForFetch, setReadyForFetch] = useState(true);
  const [fetchDuration, setFetchDuration] = useState(0);
  const [packageStatisticsFetchedOnce, setPackageStatisticsFetchedOnce] =
    useState(false);
  const tryOverviewFetch = () => {
    if (readyForFetch) {
      overviewFetch();
    }
  };

  useEffect(() => {
    const interval = setInterval(() => {
      tryOverviewFetch();
    }, 500);

    return () => clearInterval(interval);
  }, [readyForFetch]);

  useEffect(() => {
    tryOverviewFetch();
  }, []);

  function millisecondsToArray(ms) {
    const units = [
      { label: "y", value: 1000 * 60 * 60 * 24 * 365 }, // years
      { label: "mo", value: 1000 * 60 * 60 * 24 * 30 }, // months
      { label: "w", value: 1000 * 60 * 60 * 24 * 7 }, // weeks
      { label: "d", value: 1000 * 60 * 60 * 24 }, // days
      { label: "h", value: 1000 * 60 * 60 }, // hours
      { label: "m", value: 1000 * 60 }, // minutes
      { label: "s", value: 1000 }, // seconds
    ];

    let result = [];
    let remaining = ms;

    for (let unit of units) {
      if (remaining >= unit.value) {
        let amount = Math.floor(remaining / unit.value);
        remaining %= unit.value;
        result.push([amount, unit.label]);

        let digitCount = result.reduce(
          (acc, [num]) => acc + num.toString().length,
          0,
        );
        if (digitCount >= 2) break;
      }
    }

    return result;
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
      <Card title={"Memory usage"} skeleton={deviceInformation.unloaded}>
        <span
          className={
            "text-5xl mb-2 font-semibold inline-block" +
            (((deviceInformation.memory_total - deviceInformation.memory_free) /
              deviceInformation.memory_total) *
              100 <
            30
              ? " text-green-500"
              : ((deviceInformation.memory_total -
                    deviceInformation.memory_free) /
                    deviceInformation.memory_total) *
                    100 <
                  60
                ? " text-orange-500"
                : ((deviceInformation.memory_total -
                      deviceInformation.memory_free) /
                      deviceInformation.memory_total) *
                      100 <
                    90
                  ? " text-red-500"
                  : " text-purple-500")
          }
        >
          {Math.round(
            ((deviceInformation.memory_total - deviceInformation.memory_free) /
              deviceInformation.memory_total) *
              100,
          )}
          %
        </span>
        <br />
        {Math.round(
          (deviceInformation.memory_total - deviceInformation.memory_free) /
            Math.pow(1000, 3),
        ) === 0
          ? "< 1"
          : Math.round(
              (deviceInformation.memory_total - deviceInformation.memory_free) /
                Math.pow(1000, 3),
            )}
        GB / {Math.round(deviceInformation.memory_total / Math.pow(1000, 3))}GB
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
          {Math.round(deviceInformation.cpu_usage) == 0
            ? "< 1"
            : Math.round(deviceInformation.cpu_usage)}
          %
        </span>
        <br />
        {deviceInformation.temperature == -300
          ? "No temperature"
          : Math.round(deviceInformation.temperature) + "°C"}
      </Card>

      <Card
        title={"Networking"}
        variant="wide"
        skeleton={
          deviceInformation.unloaded ||
          deviceInformation.net_interface == "MISSING_INTERFACE"
        }
      >
        <span className="inline-block mr-2 mb-2">
          <strong className="block">Hostname</strong>

          <button
            className="cursor-pointer active:scale-110 active:text-green-500 transition-all duration-75 bg-transparent border-transparent inline-block"
            onClick={() => {
              try {
                window.navigator.clipboard.writeText(deviceInformation.ip);
                toast({
                  title: "Copied hostname to clipboard",
                });
              } catch {}
            }}
          >
            {deviceInformation.hostname}
          </button>
        </span>
        <span className="inline-block mr-2 mb-2">
          <strong className="block">Private IP</strong>
          <button
            className="cursor-pointer active:scale-110 active:text-green-500 transition-all duration-75 bg-transparent border-transparent inline-block"
            onClick={() => {
              try {
                window.navigator.clipboard.writeText(deviceInformation.ip);
                toast({
                  title: "Copied private IP to clipboard",
                });
              } catch {}
            }}
          >
            {deviceInformation.ip}
          </button>
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
                  Network statistics rely on the IP command on your system to
                  measure transmitted and received bytes. Zentrox measures the
                  change in bytes in an interval of 1000ms and calculates the
                  average resulting in B/s.
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
      <Card
        title={"Uptime"}
        variant="square"
        skeleton={deviceInformation.unloaded}
      >
        <FancyCounter>
          {millisecondsToArray(deviceInformation.uptime).map((e, k) => {
            return (
              <span key={k}>
                {e[0]
                  .toString()
                  .split("")
                  .map((d, dk) => {
                    return <FancyCounterDigit key={dk}>{d}</FancyCounterDigit>;
                  })}

                <FancyCounterCaption>{e[1]}</FancyCounterCaption>
              </span>
            );
          })}
        </FancyCounter>
        <strong className="block text-sm">Active since:</strong>
        <DateWrapper
          updating={false}
          seconds={(Date.now() - deviceInformation.uptime) / 1000}
          className="text-sm"
        />
      </Card>
      <Card
        title={
          "Packages" +
          {
            "": "",
            pacman: " using PacMan",
            apt: " using APT",
            dnf: " using DNF",
          }[packageStatistics.packageManager]
        }
        variant="wide"
        skeleton={packageStatistics.unloaded}
      >
        <span className="inline-block mr-2 mb-2">
          <strong className="block">Available packages</strong>
          {packageStatistics.available}
        </span>

        {!packageStatistics.canProvideUpdates ? (
          <span className="block mr-2 mb-2">
            <strong className="block">Installed packages</strong>
            {packageStatistics.installed}
          </span>
        ) : (
          <span className="block mr-2 mb-2">
            <strong className="block">Available updates</strong>
            {packageStatistics.updates}
          </span>
        )}
      </Card>

      <Card
        title={"Connectivity"}
        variant="square"
        skeleton={
          deviceInformation.unloaded ||
          deviceInformation.net_interface == "MISSING_INTERFACE"
        }
      >
        <span
          title={
            deviceInformation.net_connected_interfaces +
            " connected interface" +
            (deviceInformation.net_connected_interfaces !== 1 ? "s" : "")
          }
        >
          <EthernetPortIcon className="h-4 w-4 mr-1 inline-block" />
          {deviceInformation.net_connected_interfaces} interface
          {deviceInformation.net_connected_interfaces !== 1 ? "s" : ""}
        </span>
        <br />
        <span title={"latency between Zentrox frontend and backend"}>
          <HourglassIcon className="h-4 w-4 mr-1 inline-block" />
          {Math.round(fetchDuration / 1000) < 1
            ? "< 1"
            : Math.round(fetchDuration / 1000)}
          s latency{" "}
          <InfoButton
            title={"Latency measurement"}
            info={
              "Zentrox measures the time it takes to complete a request for the current server status. Such a request is only sent every 500ms."
            }
          />
        </span>
      </Card>
    </Page>
  );
}
