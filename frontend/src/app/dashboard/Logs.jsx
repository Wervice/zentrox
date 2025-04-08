import { useEffect, useState, useRef } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button.jsx";
import CalendarButton from "@/components/ui/calendar";
import Page from "@/components/ui/PageWrapper";
QRCodeSVG;
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import { QRCodeSVG } from "qrcode.react";
import secondsToFormat from "@/lib/dates";
const fetchURLPrefix = require("@/lib/fetchPrefix");
import { Td, Tr, Th, Table } from "@/components/ui/table";

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

    return (
      <>
        <div className="overflow-scroll no-scroll">
          <Table>
            <thead>
              <Tr>
                <Th expand={true}>Time</Th>
                <Th expand={true}>Application</Th>
                <Th expand={true}>Message</Th>
              </Tr>
            </thead>
            {logEntries
              .filter((e) => {
                if (e[0] === "") return;
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
                  <Tr>
                    <Td expand={true}>
                      {secondsToFormat(
                        logEntry[0] / 1000000,
                        localStorage.getItem("dateFormat") || "8601",
                      )}
                    </Td>
                    <Td expand={true}>{logEntry[2]}</Td>
                    <Td
                      expand={true}
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
                  </Tr>
                );
              })}
          </Table>
        </div>
      </>
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
      {sudoPassword === "" ? (
        <Button
          onClick={() => {
            setSudoPasswordModal(true);
          }}
          className="mr-1"
        >
          Enter sudo password
        </Button>
      ) : (
        <></>
      )}
      <div hidden={sudoPassword === ""}>
        <Button
          onClick={() => {
            setLogRefreshKeyValue(new Date().getTime());
          }}
          variant="secondary"
        >
          Refresh
        </Button>
        <Input
          type="text"
          placeholder="Search"
          ref={searchInput}
          className="inline-flex"
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
          variant={"secondary"}
          confirmMode={true}
        />
        <CalendarButton
          placeholder="Until"
          className="mr-1"
          onValueChange={setUntil}
          variant={"secondary"}
          confirmMode={true}
        />
        <LogTable
          log={selectedLog}
          refreshKey={logRefreshKeyValue}
          tableFilter={tableFilter}
        />
      </div>
    </Page>
  );
}

export default Logs;
