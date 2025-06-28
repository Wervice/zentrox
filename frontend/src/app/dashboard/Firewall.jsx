import { Switch } from "@/components/ui/switch.jsx";
import { Button } from "@/components/ui/button";
import {
  Plus,
  Ban,
  CircleCheck,
  BrickWall,
  ArrowUpFromDot,
  ArrowDownToDot,
  Shield,
  TrashIcon,
  LockIcon,
} from "lucide-react";
import { useState, useRef } from "react";
import "./scroll.css";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { useToast } from "@/components/ui/use-toast";
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
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import "./scroll.css";
import Page from "@/components/ui/PageWrapper";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import useNotification from "@/lib/notificationState";
import { Td, Tr, Th, Table, ActionTh, ActionTd } from "@/components/ui/table";
import Label from "@/components/ui/ShortLabel";
import {
  Placeholder,
  PlaceholderIcon,
  PlaceholderSubtitle,
} from "@/components/ui/placeholder";
import SudoDialog from "@/components/ui/SudoDialog";

function Firewall() {
  const [rules, setRules] = useState([]); // Firewall rules that are displayed on the frontend and fetched from UFW
  const [fireWallEnabled, setFireWallEnabled] = useState(null); // Is the firewall enabled?

  // New rule creation
  const [newRuleAction, setNewRuleAction] = useState("allow");
  const [sudoPassword, setSudoPassword] = useState("");
  const [sudoDialogOpen, setSudoDialogOpen] = useState(true);
  const [portOrRange, setPortOrRange] = useState("port");
  const [anyIp, setAnyIp] = useState(false);
  const [networkProtocol, setNetworkProtocol] = useState("tcp");
  const [hasUfw, setHasUfw] = useState(true);
  var newRuleToSinglePort = useRef();
  var newRuleToRangeLeft = useRef();
  var newRuleToRangeRight = useRef();
  var newRuleFrom = useRef();
  var sudoPasswordInput = useRef();
  const { deleteNotification, notify, notifications } = useNotification();

  const { toast } = useToast();

  function verifyUfwPersence() {
    fetch(fetchURLPrefix + "/api/hasUfw").then((res) => {
      res.text().then((text) => {
        setHasUfw(text == "true");
      });
    });
  }

  function fetchFireWallInformation(password = sudoPassword) {
    fetch(fetchURLPrefix + "/api/fireWallInformation", {
      headers: {
        "Content-Type": "application/json",
      },
      method: "POST",
      body: JSON.stringify({
        sudoPassword: password,
      }),
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setRules(json["rules"]);
          setFireWallEnabled(json["enabled"]);
        });
      } else {
        notify("Wrong sudo password used to view firewall configuration");
        toast({
          title: "Wrong sudo password",
          description: "Zentrox failed to validate your sudo password",
        });
      }
    });
  }

  function RuleView() {
    if (fireWallEnabled) {
      return (
        <div className="max-h-full min-h-fit overflow-y-scroll overflow-x-hidden w-fit no-scroll">
          <Table>
            <Tr>
              <Th>
                <ArrowUpFromDot className="w-4 h-4 pb-0.5 inline" /> To
              </Th>
              <Th>
                <ArrowDownToDot className="w-4 h-4 pb-0.5 inline" /> From
              </Th>
              <Th>
                <Shield className="w-4 h-4 pb-0.5 inline" /> Action
              </Th>
              <ActionTh />
            </Tr>
            {rules.map((rule, i) => {
              return (
                <Tr key={i} className="w-fit">
                  <Td>{rule.to.replaceAll("(v6)", "IPv6")}</Td>
                  <Td>{rule.from.replaceAll("(v6)", "IPv6")}</Td>
                  <Td className="align-middle">
                    {rule.action === "DENY" ? (
                      <>
                        <Ban className="h-6 w-6 mt-[-2px] inline-block text-red-500 pr-1" />
                        Deny
                      </>
                    ) : (
                      <>
                        <CircleCheck className="h-6 w-6 mt-[-2px] inline-block text-green-500 pr-1" />
                        Allow
                      </>
                    )}
                  </Td>
                  <ActionTd>
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <TrashIcon className="w-4 h-4 opacity-75 text-red-500 hover:opacity-100 transition-all duration-200 inline-block cursor-pointer" />
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>Delete rule</AlertDialogTitle>
                          <AlertDialogDescription>
                            Do you really want to remove this rule? This action
                            can not be undone.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction
                            onClick={() => {
                              fetch(
                                fetchURLPrefix +
                                  "/api/deleteFireWallRule/" +
                                  rule.index,
                                {
                                  method: "POST",
                                  headers: {
                                    "Content-Type": "application/json",
                                  },
                                  body: JSON.stringify({
                                    sudoPassword: sudoPassword,
                                  }),
                                },
                              ).then((res) => {
                                if (!res.ok) {
                                  notify(
                                    "Failed to delete firewall rule " +
                                      rule.index,
                                  );
                                  toast({
                                    title: "Failed to delete firewall rule",
                                    description: `Zentrox failed to delete rule ${rule.index}.`,
                                  });
                                } else {
                                  fetchFireWallInformation();
                                }
                              });
                            }}
                          >
                            Proceed
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </ActionTd>
                </Tr>
              );
            })}
          </Table>
        </div>
      );
    } else {
      return (
        <Placeholder>
          <PlaceholderIcon icon={BrickWall} />
          <PlaceholderSubtitle>Firewall is disabled</PlaceholderSubtitle>
        </Placeholder>
      );
    }
  }

  return (
    <>
      <SudoDialog
        onFinish={(password) => {
          setSudoPassword(password);
          fetchFireWallInformation(password);
          verifyUfwPersence();
        }}
        modalOpen={sudoDialogOpen}
        onOpenChange={setSudoDialogOpen}
      />

      <Toaster />
      <Page name="Firewall" titleAbsolute={sudoPassword === ""}>
        {" "}
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
                  setSudoDialogOpen(true);
                }}
              >
                Enter password
              </Button>
            </span>
          </span>
        )}
        <div className="w-full">
          <span className={sudoPassword.length === 0 ? "hidden" : ""}>
            {!hasUfw && (
              <>
                <span className="p-2 rounded-lg bg-red-500/10 border border-red-500/30 block lg:w-[500px] md:w-full my-2 text-red-500">
                  This section does not work without UFW. <br />
                  Please ensure, that you have UFW installed on your machine.
                </span>
              </>
            )}

            <Dialog>
              <DialogTrigger disabled={sudoPassword.length === 0} asChild>
                <Button className="mb-2" disabled={sudoPassword.length === 0}>
                  <Plus className="h-4 w-4 inline" />
                  New Rule
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>New firewall rule</DialogTitle>
                  <DialogDescription>
                    You can create a new rule that applies to your firewall.
                  </DialogDescription>
                  <p>
                    <Label htmlFor="ruleTo" className="block">
                      <ArrowUpFromDot className="w-4 h-4 inline" /> To
                    </Label>
                    <small className="text-neutral-600 mb-2">
                      The port or port range on which protocol the request was
                      sent to
                    </small>
                    <Select onValueChange={setPortOrRange}>
                      <SelectTrigger className="w-[210px] mb-1">
                        <SelectValue placeholder="Port or port range" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="port">Single port</SelectItem>
                        <SelectItem value="range">Port range</SelectItem>
                      </SelectContent>
                    </Select>
                    {portOrRange == "port" ? (
                      <Input
                        id="ruleTo"
                        placeholder="port"
                        ref={newRuleToSinglePort}
                        className="inline-flex mb-1 w-[210px]"
                      />
                    ) : (
                      <>
                        <Input
                          id="ruleTo"
                          placeholder="port"
                          ref={newRuleToRangeLeft}
                          className="inline-flex w-[100px] mb-1 mr-2"
                        />
                        <Input
                          id="ruleTo"
                          placeholder="port"
                          ref={newRuleToRangeRight}
                          className="inline-flex w-[100px] mb-1"
                        />
                        <br />
                      </>
                    )}

                    <Select
                      value={networkProtocol}
                      onValueChange={setNetworkProtocol}
                    >
                      <SelectTrigger className="w-[210px] mb-1">
                        <SelectValue placeholder="Network protocol" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="tcp">TCP</SelectItem>
                        <SelectItem value="upd">UDP</SelectItem>
                      </SelectContent>
                    </Select>

                    <Label htmlFor="ruleFrom" className="block m-1">
                      <ArrowDownToDot className="w-4 h-4 inline" /> From
                    </Label>
                    <small className="text-neutral-600 mb-2">
                      The ip address or hostname the request was sent from
                    </small>
                    <Button
                      variant={anyIp ? "" : "outline"}
                      onClick={() => setAnyIp(!anyIp)}
                      className="block mb-1"
                    >
                      From any IP
                    </Button>
                    <Input
                      id="ruleFrom"
                      disabled={anyIp}
                      placeholder="ip address"
                      ref={newRuleFrom}
                      className="inline-flex mr-1 w-[210px]"
                    />
                    <label htmlFor="ruleAction" className="block m-1">
                      <Shield className="w-4 h-4 inline" /> Action
                    </label>
                    <Select
                      value={newRuleAction}
                      onValueChange={(e) => {
                        setNewRuleAction(e);
                      }}
                    >
                      <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder="Select an action" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectGroup>
                          <SelectItem value="allow">
                            <CircleCheck className="w-4 h-4 inline mr-1 text-green-500" />{" "}
                            Allow
                          </SelectItem>
                          <SelectItem value="deny">
                            <Ban className="w-4 h-4 inline mr-1 text-red-500" />{" "}
                            Deny
                          </SelectItem>
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                  </p>
                  <DialogFooter>
                    <DialogClose asChild>
                      <Button variant="outline">Cancel</Button>
                    </DialogClose>
                    <DialogClose asChild>
                      <Button
                        onClick={() => {
                          let isPortOrRangeFormatted =
                            portOrRange == "port" ? "p" : "r";
                          let portOrRangeFormatted;
                          if (portOrRange === "port") {
                            var iv = newRuleToSinglePort.current.value;
                            if (!iv || iv.length === 0) {
                              toast({
                                title: "Missing port",
                                description:
                                  "Please specify a destination port",
                              });
                              return;
                            }
                            portOrRangeFormatted = encodeURIComponent(iv);
                          } else {
                            let ivLeft = newRuleToRangeLeft.current.value;
                            let ivRight = newRuleToRangeRight.current.value;
                            if (
                              !ivLeft ||
                              ivLeft.length === 0 ||
                              !ivRight ||
                              ivRight.length === 0
                            ) {
                              toast({
                                title: "Missing port range",
                                description:
                                  "Please specify a destination port range",
                              });
                              return;
                            }
                            portOrRangeFormatted = `${encodeURIComponent(ivLeft)}:${encodeURIComponent(ivRight)}`;
                          }
                          let specificOrAny = anyIp ? "any" : "specific";
                          let ipFormatted = encodeURIComponent(
                            anyIp ? " " : newRuleFrom.current.value,
                          );
                          fetch(
                            fetchURLPrefix +
                              `/api/newFireWallRule/${isPortOrRangeFormatted}/${portOrRangeFormatted}/${networkProtocol}/${specificOrAny}/${ipFormatted}/${newRuleAction}`,
                            {
                              method: "POST",
                              headers: {
                                "Content-Type": "application/json",
                              },
                              body: JSON.stringify({
                                sudoPassword: sudoPassword,
                              }),
                            },
                          ).then((res) => {
                            if (res.ok) {
                              notify(
                                "Zentrox created a new firewall rule using UFW",
                              );
                              fetchFireWallInformation();
                            } else {
                              notify(
                                "Zentrox failed to create a new firewall rule using UFW",
                              );
                              toast({
                                title: "Failed to create firewall rule",
                                description:
                                  "Zentrox failed to create a new firewall rule using UFW",
                              });
                            }
                          });
                        }}
                      >
                        Create
                      </Button>
                    </DialogClose>
                  </DialogFooter>
                </DialogHeader>
              </DialogContent>
            </Dialog>

            <span className="flex items-center mb-1 space-x-2">
              <Switch
                disabled={sudoPassword.length === 0}
                onClick={(e) => {
                  if (sudoPassword.length === 0) {
                    return;
                  }

                  e.target.disabled = true;
                  fetch(fetchURLPrefix + "/api/switchUfw/" + !fireWallEnabled, {
                    method: "POST",
                    headers: {
                      "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                      sudoPassword: sudoPassword,
                    }),
                  }).then((res) => {
                    if (res.ok) {
                      setFireWallEnabled(!fireWallEnabled);
                      notify(
                        "Zentrox " +
                          (!fireWallEnabled
                            ? "enabled the firewall using UFW"
                            : "disabled the firewall using UFW"),
                      );
                    } else {
                      notify(
                        "Zentrox failed to " +
                          (!fireWallEnabled
                            ? "enable firewall"
                            : "disable firewall") +
                          " using UFW",
                      );
                      toast({
                        title: "Failed to apply firewall configuration",
                        description:
                          "Zentrox failed to change the state of the firewall.",
                      });
                    }
                    e.target.disabled = false;
                    fetchFireWallInformation();
                  });
                }}
                value={fireWallEnabled ? "on" : "off"}
                checked={fireWallEnabled}
                hidden={fireWallEnabled === null}
                title="Enable firewall"
              />
              <Label>Enable firewall</Label>
            </span>
            <RuleView />
          </span>
        </div>
      </Page>
    </>
  );
}

export default Firewall;
