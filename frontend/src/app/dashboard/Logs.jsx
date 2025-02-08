import { useEffect, useState, useRef } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button.jsx";
import InfoButton from "@/components/ui/InfoButton";
import CalendarButton from "@/components/ui/calendar";
import Page from "@/components/ui/PageWrapper";
QRCodeSVG;
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
import { QRCodeSVG } from "qrcode.react";
const fetchURLPrefix = require("@/lib/fetchPrefix");

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
                    const passedStamp =
                      Number(tableFilter.split("time: ")[1]) * 1000;
                    return Math.abs(passedStamp - stamp) < 10000;
                  } else if (tableFilter.startsWith("priority: ")) {
                    const passedPrio = tableFilter.split("priority: ")[1];
                    return e[4] === passedPrio;
                  } else if (tableFilter.startsWith("excluded_application: ")) {
                    return (
                      e[2].toLowerCase() ==
                      tableFilter
                        .split("excluded_application: ")[1]
                        .toLowerCase()
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
			className="w-full"
          />

          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Cancel</Button>
            </DialogClose>
            <DialogClose asChild>
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

export default Logs;
