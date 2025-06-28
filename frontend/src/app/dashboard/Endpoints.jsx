import Page from "@/components/ui/PageWrapper";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import useNotification from "@/lib/notificationState";
import {
  AntennaIcon,
  ArrowDownIcon,
  ArrowUpIcon,
  CableIcon,
  EthernetPortIcon,
  MicrochipIcon,
  PackageIcon,
  RouteIcon,
  TrashIcon,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import { ActionTh, ActionTd, Table, Td, Th, Tr } from "@/components/ui/table";
import {
  AlertDialogHeader,
  AlertDialog,
  AlertDialogContent,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Details } from "@/components/ui/Details";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
  DialogHeader,
} from "@/components/ui/dialog";

class Route {
  /**
   * @constructs Route
   * @param {string} destination
   * @param {string} table
   * @param {string} device
   * @param {string} preferredSource
   * @param {string} protocol
   * @param {string} scope
   * @param {string} gateway */
  constructor(
    destination,
    gateway,
    device,
    protocol,
    preferredSource,
    table,
    scope,
  ) {
    this.destination = destination;
    this.gateway = gateway;
    this.device = device;
    this.protocol = protocol;
    this.preferredSource = preferredSource;
    this.table = table;
    this.scope = scope;
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

function InterfaceCard({
  title,
  children,
  skeleton,
  focused,
  defaultActive,
  setActivity,
}) {
  const [active, setActive] = useState(defaultActive);
  var sudoPasswordInput = useRef();

  return (
    <span
      className={
        "p-4 mr-2 mt-2 w-64 relative rounded-xl overflow-hidden overflow-ellipsis whitespace-pre border-zinc-900 border inline-block transition-all duration-500" +
        (skeleton ? " opacity-30 scale-90" : "") +
        (focused
          ? " border-neutral-300 bg-white/5"
          : " border-zinc-900 hover:border-zinc-800")
      }
    >
      <Dialog>
        <DialogTrigger asChild>
          <Checkbox
            title="Set interface activity"
            className="absolute right-2 top-2"
            checked={active}
          />
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {active ? "Disable" : "Enable"} interface?
            </DialogTitle>
            <DialogDescription>
              Please enter your sudo password to {active ? "disable" : "enable"}{" "}
              this interfce.
            </DialogDescription>
          </DialogHeader>
          <p>
            <Input type="password" className="w-full" ref={sudoPasswordInput} />
          </p>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Cancel</Button>
            </DialogClose>
            <DialogClose asChild>
              <Button
                onClick={() => {
                  setActivity(!active, sudoPasswordInput.current.value);
                  setActive(!active);
                }}
              >
                Confirm
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <span className="flex items-center mb-1 font-bold text-lg w-full">
        {title}
      </span>
      {skeleton ? (
        <></>
      ) : (
        <span className="animate-fadein duration-300">{children}</span>
      )}
    </span>
  );
}

function MacAddress({ children }) {
  const [visible, setVisible] = useState(false);
  return (
    <span
      title="Click to show sensitive data"
      className={
        "font-[monospace] cursor-pointer" +
        (visible ? " text-red-500" : " opacity-75")
      }
      onClick={() => setVisible(!visible)}
    >
      {visible ? children : "##:##:##:##:##:##"}
    </span>
  );
}

export default function Endpoints() {
  const [routes, setRoutes] = useState([]);
  const [interfaces, setInterfaces] = useState([]);

  const [selectedInterface, setSelectedInterface] = useState("");
  const [tableFilter, setTableFilter] = useState(null);

  const [showRouteDeletionModal, setShowDeletionModal] = useState(false);
  const [routeDeletionModalRoute, setRouteDeletionModalRoute] = useState(null);

  const [showRouteEditModal, setShowEditModal] = useState(false);
  const [routeEditModalRoute, setRouteEditModalRoute] = useState(null);

  const { deleteNotification, notifications, notify } = useNotification();
  var sudoPasswordInput = useRef();

  function fetchRoutes() {
    fetch(fetchURLPrefix + "/api/networkRoutes").then((res) => {
      if (res.ok) {
        res.json().then((j) => {
          let r = j.routes;
          let parsed_routes = [];
          for (let route of r) {
            var constructed = new Route();
            constructed.destination = route.destination;
            constructed.device = route.device;
            constructed.scope = route.scope;
            constructed.table = route.table;
            constructed.gateway = route.gateway;
            constructed.protocol = route.protocol;
            constructed.preferredSource = route.preferred_source;
            constructed.link_type = route.link_type;
            parsed_routes.push(constructed);
          }
          setRoutes(parsed_routes);
        });
      } else {
        notify("Failed to get networking routes");
      }
    });
  }

  function fetchInterfaces() {
    fetch(fetchURLPrefix + "/api/networkInterfaces").then((res) => {
      if (res.ok) {
        res.json().then((j) => {
          setInterfaces(j.interfaces);
        });
      } else {
        notify("Failed to get network interfaces");
      }
    });
  }

  function refreshInformation() {
    const iv = setInterval(() => {
      fetchInterfaces();
      fetchRoutes();
    }, 500);
    return () => clearInterval(iv);
  }

  function deleteRouteRequest(e) {
    const dev = e.device;
    const dest = e.destination;
    const gateway = e.gateway;

    // The fields of the entire route have to be split up, so the Rust backend can handle the appropriately

    const data = {
      device: dev,
      destination: [
        dest === "Default",
        dest !== "Default" ? dest.Prefix.address : null,
        dest !== "Default" ? dest.Prefix.subnet : null,
      ],
      gateway: [
        gateway === null,
        gateway !== null ? gateway.address : null,
        gateway !== null ? gateway.subnet : null,
      ],
      sudo_password: sudoPasswordInput.current.value,
    };

    fetch(`${fetchURLPrefix}/api/deleteNetworkRoute`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    }).then((res) => {
      if (res.ok) {
        notify("Deleted network route");
      } else {
        notify("Failed to delete network route");
      }
    });
  }

  useEffect(fetchRoutes, []);
  useEffect(fetchInterfaces, []);
  useEffect(refreshInformation, []);

  return (
    <>
      <Page name="Endpoints">
        <Details
          rememberState
          name="endpointsInterfaces"
          title={
            <>
              <EthernetPortIcon className="h-5 inline-block mr-1" /> Interfaces
            </>
          }
          open
        >
          <div>
            {interfaces.map((e, k) => {
              return (
                <InterfaceCard
                  focused={e.ifname == selectedInterface}
                  key={k}
                  id={"interface_" + e.ifname}
                  title={e.ifname}
                  defaultActive={e.operstate == "Up"}
                  setActivity={(ev, sp) => {
                    fetch(`${fetchURLPrefix}/api/networkingInterfaceActive`, {
                      method: "POST",
                      body: JSON.stringify({
                        interface: e.ifname,
                        activity: ev,
                        sudoPassword: sp,
                      }),
                      headers: {
                        "Content-Type": "application/json",
                      },
                    }).then((r) => {
                      if (r.ok) {
                        notify(`Set ${e.ifname} to ${ev ? "Up" : "Down"}`);
                      } else {
                        notify(
                          `Failed to set ${e.ifname} to ${ev ? "Up" : "Down"}`,
                        );
                      }
                      fetchInterfaces();
                    });
                  }}
                >
                  <span className="flex items-center pl-1" title="MAC address">
                    <span
                      className={
                        "rounded-full h-[10px] w-[10px] " +
                        (e.operstate == "Up"
                          ? "bg-green-500"
                          : e.operstate == "Down"
                            ? "bg-red-500"
                            : e.operstate == "Unknown"
                              ? "bg-neutral-400"
                              : e.operstate == "NotPresent"
                                ? "bg-blue-500"
                                : "bg-yellow-500")
                      }
                    ></span>{" "}
                    {e.operstate}
                  </span>
                  <span className="flex items-center" title="MAC address">
                    <MicrochipIcon className="inline-block h-4" />{" "}
                    <MacAddress>{e.address || "N/A"}</MacAddress>
                    <br />
                  </span>
                  <span className="flex items-center" title="Broadcast address">
                    <AntennaIcon className="inline-block h-4" />{" "}
                    <MacAddress>{e.broadcast}</MacAddress>
                  </span>
                  <span
                    className="flex items-center"
                    title="Maximum transmission unit"
                  >
                    <PackageIcon className="inline-block h-4" /> {e.mtu}
                  </span>
                  <span className="flex items-center" title="Link type">
                    <CableIcon className="inline-block h-4" />{" "}
                    {e.link_type || "N/A"}
                    <br />
                  </span>
                  <span className="flex items-center" title="Bytes up">
                    <ArrowUpIcon className="inline-block h-4" />{" "}
                    {prettyBytes(Math.floor(e.up)) || "N/A"}
                    <br />
                  </span>
                  <span className="flex items-center" title="Bytes down">
                    <ArrowDownIcon className="inline-block h-4" />{" "}
                    {prettyBytes(Math.floor(e.down)) || "N/A"}
                  </span>
                </InterfaceCard>
              );
            })}
          </div>
        </Details>

        <Details
          rememberState
          name="endpointsRoutes"
          title={
            <>
              <RouteIcon className="h-5 inline-block mr-1" /> Routes
            </>
          }
          open
        >
          <Select value={tableFilter} onValueChange={setTableFilter}>
            <SelectTrigger className="w-[300px]" title="Table select">
              Select table ({tableFilter || "N/A"})
            </SelectTrigger>
            <SelectContent>
              {(function () {
                var s = new Set();
                for (var v of routes) {
                  s.add(v.table);
                }
                let a = Array.from(s);
                return a.map((e, i) => {
                  return (
                    <SelectItem
                      key={i}
                      value={e}
                      onClick={() => {
                        setTableFilter(e);
                      }}
                    >
                      {e || "N/A"}
                    </SelectItem>
                  );
                });
              })()}
            </SelectContent>
          </Select>
          <Table>
            <Tr>
              <ActionTh />
              <Th>Destination</Th>
              <Th>Gateway</Th>
              <Th>Interface</Th>
              <Th>Preferred source</Th>
              <Th>Protocol</Th>
            </Tr>
            {routes
              .filter((x) => {
                return x.table === tableFilter;
              })
              .sort((a, b) => a.destination > b.destination)
              .map((r) => {
                let dest;
                let gw;
                if (typeof r.destination == "string") {
                  dest = <span className="text-white/40">{r.destination}</span>;
                } else {
                  if (r.destination.Prefix.subnet == null) {
                    dest = r.destination.Prefix.address;
                  } else {
                    dest = `${r.destination.Prefix.address}/${r.destination.Prefix.subnet}`;
                  }
                }
                if (r.gateway == null) {
                  gw = <span className="text-white/40">None</span>;
                } else {
                  if (r.gateway.subnet == null) {
                    gw = r.gateway.address;
                  } else {
                    gw = `${r.gateway.address}/${r.gateway.subnet}`;
                  }
                }

                return (
                  <Tr>
                    <ActionTd>
                      <AlertDialog>
                        <AlertDialogTrigger>
                          <TrashIcon className="w-4 h-4 opacity-75 text-red-500 hover:opacity-100 transition-all duration-200 inline-block" />
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>Delete route</AlertDialogTitle>
                            <AlertDialogDescription>
                              Are you sure you want to delete this route? This
                              may break your connection to Zentrox.
                              <Table>
                                <Tr>
                                  <Th>Attribute</Th>
                                  <Th>Value</Th>
                                </Tr>
                                <Tr>
                                  <Td>Interface</Td>
                                  <Td>{r.device}</Td>
                                </Tr>
                                <Tr>
                                  <Td>Gateway</Td>
                                  <Td>{gw}</Td>
                                </Tr>
                                <Tr>
                                  <Td>Destination</Td>
                                  <Td>{dest}</Td>
                                </Tr>
                              </Table>
                            </AlertDialogDescription>
                            <Input
                              type="password"
                              className="w-[400px]"
                              ref={sudoPasswordInput}
                              placeholder="Sudo password"
                            />
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel>Cancel</AlertDialogCancel>
                            <AlertDialogAction asChild>
                              <Button
                                variant="destructive"
                                onClick={() => {
                                  deleteRouteRequest(r);
                                }}
                              >
                                Delete
                              </Button>
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    </ActionTd>
                    <Td>{dest}</Td>
                    <Td>{gw}</Td>
                    <Td>
                      {r.device ? (
                        <span onClick={() => setSelectedInterface(r.device)}>
                          {r.device}
                        </span>
                      ) : (
                        <span className="text-white/40">N/A</span>
                      )}
                    </Td>
                    <Td>
                      {r.preferredSource || (
                        <span className="text-white/40">N/A</span>
                      )}
                    </Td>
                    <Td>{r.protocol}</Td>
                  </Tr>
                );
              })}
          </Table>
        </Details>
      </Page>
    </>
  );
}
