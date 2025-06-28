import { useEffect, useState, useRef } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import CalendarButton from "@/components/ui/calendar";
import Page from "@/components/ui/PageWrapper";
QRCodeSVG;
import { QRCodeSVG } from "qrcode.react";
import secondsToFormat from "@/lib/dates";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { Td, Tr, Th, Table } from "@/components/ui/table";
import { LockIcon } from "lucide-react";
import useNotification from "@/lib/notificationState";
import SudoDialog from "@/components/ui/SudoDialog";

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

  const LogTable = ({ log }) => {
    const [logEntries, setLogEntries] = useState([["", "", "", ""]]);
    const { deleteNotification, notify, notifications } = useNotification();

    const fetchLog = (log, sudo, since, until) => {
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
          res.json().then((j) => {
            setLogEntries(j.logs);
          });
        } else {
          notify(
            "Failed to fetch log entries. Please validate your sudo password.",
          );
        }
      });
    };

    useEffect(() => {
      if (sudoPassword !== "") {
        fetchLog(log, sudoPassword, since, until);
      }
    }, []);

    return (
      <>
        <div className="overflow-scroll no-scroll">
          {console.log(logEntries.length)}
          {logEntries.length > 1 && (
            <Table className="overflow-y-scroll max-h-[calc(100vh - 300px)] block">
              <Tr>
                <Th>Time</Th>
                <Th>Application</Th>
                <Th>Message</Th>
              </Tr>
              {logEntries
                .filter((e) => {
                  if (e[0] === "") return false;
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
                    } else if (
                      tableFilter.startsWith("excluded_application: ")
                    ) {
                      return (
                        e[2].toLowerCase() ==
                        tableFilter
                          .split("excluded_application: ")[1]
                          .toLowerCase()
                      );
                    } else {
                      return (
                        e[2]
                          .toLowerCase()
                          .includes(tableFilter.toLowerCase()) ||
                        e[3]
                          .toLowerCase()
                          .includes(tableFilter.toLowerCase()) ||
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
          )}
        </div>
      </>
    );
  };

  return (
    <Page name="Logs" titleAbsolute={sudoPassword === ""}>
      <SudoDialog
        onFinish={(password) => {
          setSudoPassword(password);
          setSelectedLog("messages");
        }}
        modalOpen={sudoPasswordModal}
        onOpenChange={setSudoPasswordModal}
      />
      {sudoPassword === "" && (
        <span className="flex items-center justify-center h-full overflow-hidden">
          <span className="h-fit">
            <div className="text-center text-2xl opacity-50">
              <LockIcon className="m-auto h-52 w-52" />
              Sudo password is required
            </div>
            <Button
              className="m-auto block mt-4"
              onClick={() => {
                setSudoPasswordModal(true);
              }}
            >
              Enter password
            </Button>
          </span>
        </span>
      )}
      <span
        className={
          "items-center space-x-1" + (sudoPassword === "" ? " hidden" : " flex")
        }
      >
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
          className="mt-0 mb-0"
          ref={searchInput}
          onKeyPress={(event) => {
            if (event.key == "Enter") {
              setTableFilter(searchInput.current.value);
            }
          }}
        />

        <CalendarButton
          placeholder="Since"
          onValueChange={setSince}
          variant={"secondary"}
          confirmMode={true}
        />
        <CalendarButton
          placeholder="Until"
          onValueChange={setUntil}
          variant={"secondary"}
          confirmMode={true}
        />
      </span>
      <span hidden={sudoPassword === ""}>
        <LogTable
          log={selectedLog}
          refreshKey={logRefreshKeyValue}
          tableFilter={tableFilter}
        />
      </span>
    </Page>
  );
}

export default Logs;
