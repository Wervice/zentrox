import { Details } from "@/components/ui/Details";
import Page from "@/components/ui/PageWrapper";
import StatCard from "@/components/ui/StatCard";
import fetchURLPrefix from "@/lib/fetchPrefix";
import useNotification from "@/lib/notificationState";
import {
  AmpersandIcon,
  BellIcon,
  BrainIcon,
  ClockIcon,
  CpuIcon,
  FileIcon,
  GitBranchIcon,
  ListIcon,
  Loader2,
  MemoryStickIcon,
  SkullIcon,
  TablePropertiesIcon,
  TagIcon,
  TerminalIcon,
  UserIcon,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  LineChart,
  Line,
  YAxis,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { ActionTd, ActionTh, Table, Td, Th, Tr } from "@/components/ui/table";
import { toast } from "@/components/ui/use-toast";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogAction,
  AlertDialogFooter,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogHeader,
  DialogContent,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";

function InformationHighlight({ title, children, Icon, collapsible = false }) {
  const [isOpen, setIsOpen] = useState(!collapsible);

  return (
    <div className="p-2 rounded border border-neutral-900 mb-2">
      <span className="w-full flex items-center opacity-75">
        <Icon className="h-4 w-4 mr-1" />
        {title}
      </span>
      {isOpen && <span className="text-xl">{children}</span>}
      {collapsible && (
        <Button
          className="block mt-2"
          variant="secondary"
          onClick={() => setIsOpen(!isOpen)}
        >
          {isOpen ? "Collaps" : "Expand"}
        </Button>
      )}
    </div>
  );
}

function Processes() {
  const [deviceInformation, setDeviceInformation] = useState(null);
  const [processList, setProcessList] = useState(null);
  const [cpuHistory, setCpuHistory] = useState([]);
  const [killAlertModalOpen, setKillAlertModalOpen] = useState(false);
  const [killAlertModalProcess, setKillAlertModalProcess] = useState(null);
  const [processListFilter, setProcessListFilter] = useState("");
  const [processDetails, setProcessDetails] = useState(null);
  const [processDetailsModalOpen, setProcessDetailsModalOpen] = useState(false);
  const { deleteNotification, notify, notifications } = useNotification();
  var searchInput = useRef();

  function fetchDeviceInformation() {
    fetch(fetchURLPrefix + "/api/deviceInformation", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setDeviceInformation(json);
          let newHistory = cpuHistory;
          newHistory.push({
            time: Date.now(),
            cpu: json.cpu_usage,
            memory: Math.round(
              ((json.memory_total - json.memory_free) / json.memory_total) *
                100,
            ),
          });
          if (newHistory.length > 15) {
            newHistory.shift();
          }
          setCpuHistory(newHistory);
        });
      } else {
        notify("Failed to fetch device information for process overview");
      }
    });
  }
  function prettyBytes(bytes) {
    if (bytes === 0) return "0 B";

    const units = ["b", "KB", "MB", "GB", "TB"];
    const k = 1000;
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const value = bytes / Math.pow(k, i);

    return `${value.toFixed(2)} ${units[i]}`;
  }
  function fetchProcessList() {
    fetch(fetchURLPrefix + "/api/listProcesses", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setProcessList(json.processes);
        });
      } else {
        notify("Failed to fetch list of active processes");
      }
    });
  }

  useEffect(fetchDeviceInformation, []);
  useEffect(fetchProcessList, []);

  useEffect(() => {
    let i = setInterval(() => {
      fetchDeviceInformation();
    }, 500);
    return () => clearInterval(i);
  }, []);

  useEffect(() => {
    let i = setInterval(() => {
      fetchProcessList();
    }, 2500);
    return () => clearInterval(i);
  }, []);

  function killProcess(pid) {
    fetch(fetchURLPrefix + "/api/killProcess/" + pid).then((res) => {
      if (res.ok) {
      } else {
        res.json().then((json) => {
          if (json.error == "SignalError") {
            notify(`Zentrox could not send the kill signal to process ${pid}.`);
            toast({
              title: "Failed to kill process",
              description:
                "The kill signal could not be sent to the process " + pid,
            });
          } else if (json.error == "WrongPID") {
            notify(
              `The process with the PID ${pid} could not be found and was not killed.`,
            );
            toast({
              title: "Failed to kill process",
              description: `The PID ${pid} could not be found`,
            });
          }
        });
      }
    });
  }

  function filterProcesses() {
    let query = searchInput.current.value;
    setProcessListFilter(query);
  }

  return (
    <Page
      name="Processes"
      titleAbsolute={deviceInformation === null || processList === null}
    >
      <AlertDialog
        open={killAlertModalOpen}
        onOpenChange={setKillAlertModalOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              Kill {killAlertModalProcess && killAlertModalProcess.name} with
              PID {killAlertModalProcess && killAlertModalProcess.pid}?
            </AlertDialogTitle>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction asChild>
              <Button onClick={() => killProcess(killAlertModalProcess.pid)}>
                Kill
              </Button>
            </AlertDialogAction>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Dialog
        open={processDetailsModalOpen}
        onOpenChange={setProcessDetailsModalOpen}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Details of process {processDetails && processDetails.name}
            </DialogTitle>
          </DialogHeader>
          <p className="block max-h-[500px] overflow-y-scroll no-scroll">
            <InformationHighlight title={"Name"} Icon={TagIcon}>
              {processDetails && processDetails.name}
            </InformationHighlight>

            <InformationHighlight title={"PID"} Icon={AmpersandIcon}>
              {processDetails && processDetails.pid}
            </InformationHighlight>

            <InformationHighlight title={"User"} Icon={UserIcon}>
              {processDetails && processDetails.username} (
              {processDetails && processDetails.uid})
            </InformationHighlight>

            <InformationHighlight title={"Memory usage"} Icon={MemoryStickIcon}>
              {processDetails && prettyBytes(processDetails.memory_usage_bytes)}
            </InformationHighlight>

            <InformationHighlight title={"Processor usage"} Icon={CpuIcon}>
              {processDetails &&
                Math.round(processDetails.cpu_usage * 100) / 100}
              %
            </InformationHighlight>

            <InformationHighlight
              collapsible
              title={"Command line"}
              Icon={TerminalIcon}
            >
              <code className="text-sm">
                {processDetails &&
                  processDetails.command_line.map((x) => x + " ")}
              </code>
            </InformationHighlight>

            <InformationHighlight title={"Executable path"} Icon={FileIcon}>
              <code className="text-sm">
                {processDetails && processDetails.executable_path}
              </code>
            </InformationHighlight>

            <InformationHighlight title={"Run time"} Icon={ClockIcon}>
              {processDetails && processDetails.run_time}s
            </InformationHighlight>

            <InformationHighlight title={"Priority"} Icon={BellIcon}>
              {processDetails && processDetails.priority}
            </InformationHighlight>

            <InformationHighlight title={"Thread count"} Icon={GitBranchIcon}>
              {processDetails && processDetails.threads}
            </InformationHighlight>
          </p>
          <DialogFooter>
            <DialogClose>
              <Button>Close</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {(function () {
        if (deviceInformation === null || processList === null) {
          return (
            <div className="h-full w-full flex items-center justify-center">
              <Loader2 className="animate-spin m-auto w-32 h-32 opacity-75" />
            </div>
          );
        } else {
          return (
            <>
              <Details
                title={"System resources"}
                rememberState
                name={"processesResources"}
                open
              >
                <StatCard
                  name="CPU temperature"
                  Icon={<BrainIcon className="inline-block h-5 w-5" />}
                >
                  {deviceInformation && deviceInformation.temperature === -300
                    ? "No reading"
                    : deviceInformation.temperature + "°C"}
                </StatCard>

                <StatCard
                  name="Total memory"
                  Icon={<BrainIcon className="inline-block h-5 w-5" />}
                >
                  {deviceInformation &&
                    Math.round(
                      deviceInformation.memory_total / (1000 * 1000 * 1000),
                    )}
                  GB
                </StatCard>
                <StatCard
                  name="Active processes"
                  Icon={<ListIcon className="inline-block h-5 w-5" />}
                >
                  {processList.length}
                </StatCard>
                <br />
                <br />
                <ResponsiveContainer
                  width="50%"
                  height="25%"
                  className="inline-block"
                  key={"rc_cpu_" + Math.round(Date.now() / 500)}
                >
                  <LineChart data={cpuHistory}>
                    <YAxis unit={"%"} width={50} domain={[0, 100]} />
                    <Legend />
                    <CartesianGrid className="opacity-50" />
                    <Line
                      type="linear"
                      dataKey="cpu"
                      stroke="#8884D8"
                      strokeWidth={3}
                      dot={false}
                      animationDuration={0}
                      name="Processor usage"
                    />
                  </LineChart>
                </ResponsiveContainer>
                <ResponsiveContainer
                  width="50%"
                  height="25%"
                  className="inline-block"
                  key={"rc_memory_" + Math.round(Date.now() / 500)}
                >
                  <LineChart data={cpuHistory}>
                    <YAxis unit={"%"} width={50} domain={[0, 100]} />
                    <CartesianGrid className="opacity-50" />
                    <Legend />
                    <Line
                      type="linear"
                      dataKey="memory"
                      stroke="#FF0051"
                      strokeWidth={3}
                      dot={false}
                      animationDuration={0}
                      name="Memory"
                    />
                  </LineChart>
                </ResponsiveContainer>
                <small className="pl-2 mb-2 opacity-75">
                  Update interval: 500ms | Accumulates up to 15 datapoints at a
                  time
                </small>
              </Details>
              <Details
                rememberState
                name={"processesActiveTable"}
                open={true}
                title="Active processes"
              >
                <span className="flex space-x-1 items-center">
                  <Input
                    placeholder="Search by PID, name and user"
                    className="mt-0"
                    ref={searchInput}
                    onKeyDown={(e) => {
                      if (e.key == "Enter") {
                        filterProcesses();
                      }
                    }}
                  />{" "}
                  <Button variant="secondary" onClick={filterProcesses}>
                    Search
                  </Button>
                </span>
                <Table>
                  <Tr>
                    <ActionTh />
                    <Th>Name</Th>
                    <Th className="min-w-[100px]">PID</Th>
                    <Th className="min-w-[100px]">CPU</Th>
                    <Th className="min-w-[100px]">Memory</Th>
                    <Th>User</Th>
                  </Tr>
                  {processList
                    .filter((x) => {
                      return (
                        x.name.includes(processListFilter) ||
                        x.username.includes(processListFilter) ||
                        x.pid + "" == processListFilter
                      );
                    })
                    .toSorted((x, y) => {
                      return x.memory_usage_bytes < y.memory_usage_bytes;
                    })
                    .map((process) => {
                      return (
                        <Tr>
                          <ActionTd>
                            <span className="flex items-center space-x-1">
                              <SkullIcon
                                className="w-4 h-4 opacity-75 text-red-500 hover:opacity-100 transition-all duration-200 inline-block cursor-pointer"
                                onClick={() => {
                                  setKillAlertModalOpen(true);
                                  setKillAlertModalProcess(process);
                                }}
                              />
                              <TablePropertiesIcon
                                onClick={() =>
                                  fetch(
                                    fetchURLPrefix +
                                      "/api/detailsProcess/" +
                                      process.pid,
                                  ).then((res) => {
                                    if (res.ok) {
                                      res.json().then((json) => {
                                        setProcessDetails(json);
                                        setProcessDetailsModalOpen(true);
                                      });
                                    } else {
                                      notify(
                                        "Zentrox faield to fetch details about process " +
                                          process.pid +
                                          ".",
                                      );
                                      toast({
                                        title: "Failed to fetch details",
                                        description:
                                          "Zentrox failed to fetch details about process " +
                                          process.pid +
                                          ".",
                                      });
                                    }
                                  })
                                }
                                className="w-4 h-4 opacity-75 hover:opacity-100 transition-all duration-200 inline-block cursor-pointer"
                              />
                            </span>{" "}
                          </ActionTd>
                          <Td className="max-w-[250px] w-[200px] whitespace-nowrap">
                            {process.name.length > 30
                              ? process.name.slice(0, 30) + "..."
                              : process.name}
                          </Td>
                          <Td className="min-w-[100px]">
                            {process.pid || "N/A"}
                          </Td>
                          <Td className="min-w-[100px]">
                            {Math.round(process.cpu_usage) + "%" || "N/A"}
                          </Td>
                          <Td className="min-w-[100px]">
                            {prettyBytes(process.memory_usage_bytes) || "N/A"}
                          </Td>
                          <Td className="max-w-[150px] w-[100px] whitespace-nowrap">
                            {process.username || "N/A"}
                          </Td>
                        </Tr>
                      );
                    })}
                </Table>
              </Details>
            </>
          );
        }
      })()}
    </Page>
  );
}

export default Processes;
