"use client";

import { Checkbox } from "@/components/ui/checkbox.jsx";
import { Switch } from "@/components/ui/switch.jsx";
import { Button } from "@/components/ui/button.jsx";
import {
	ComputerIcon,
	CpuIcon,
	Disc2,
	List,
	LogOut,
	MemoryStickIcon,
	MonitorIcon,
	Network,
	PieChart,
	Plug,
	TableIcon,
	Thermometer,
	WatchIcon,
	Share2,
	HardDriveIcon,
	AppWindow,
	Package2,
	Loader2,
	CircleX,
	SearchIcon,
	TrashIcon,
	Paintbrush2,
	Plus,
	Ban,
	CircleCheck,
	BrickWall,
	ArrowUpFromDot,
	ArrowDownToDot,
	Shield,
	RepeatIcon,
	LayoutPanelTopIcon,
	TagIcon,
	MapPinIcon,
	WeightIcon,
	UserIcon,
	MountainIcon,
	PieChartIcon,
	KeyIcon,
	LockIcon,
	LockOpenIcon,
	FolderIcon,
	UploadIcon,
	FileIcon,
	ArrowUp,
	LoaderIcon,
	DeleteIcon,
	PenLineIcon,
	DownloadIcon,
	PowerOffIcon,
} from "lucide-react";
import { useEffect, useState, useRef } from "react";
import { Label } from "@/components/ui/label";
import { SideWayBarChart } from "@/components/ui/Charts.jsx";
import { useInterval } from "usehooks-ts";
import { table, tr, td, tbody } from "react-table";
import "./table.css";
import "./scroll.css";
import Spinner from "@/components/ui/Spinner.jsx";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { toast, useToast } from "@/components/ui/use-toast";
import InfoButton from "@/components/ui/InfoButton.jsx";
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
	SelectLabel,
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
import FileView from "@/components/ui/fileview";
import "./scroll.css";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";
// const fetchURLPrefix = "";
const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
	console.error("Fetch URL Prefix is enabled");
}

/**
 * @param {string} value to check
 * @description Returns the string or "N/A" when the string is not defined*/
function na(value) {
	if (typeof value === "undefined" || value === null) return "N/A";
	return value;
}

function TopBar({ children }) {
	return (
		<nav className="bg-transparent text-neutral-100 p-3 border-neutral-900 border-b font-semibold text-xl">
			{children}
		</nav>
	);
}

function TabButton({ onClick, isDefault, isActive, children }) {
	const [isOnloadDefault, setOnloadDefault] = useState(isDefault);
	if (isOnloadDefault || isActive) {
		var style =
			"mr-2 ml-2 text-lg hover:bg-neutral-900 text-white bg-neutral-900 hover:bg-neutral-800 hover:text-neutral-100";
	} else {
		var style =
			"bg-transparent mr-2 ml-2 text-lg hover:bg-neutral-800 hover:text-neutral-200 text-neutral-400";
	}
	if (isOnloadDefault) {
		onClick();
		setOnloadDefault(false);
	}
	return (
		<Button
			className={style}
			onClick={() => {
				onClick();
			}}
		>
			{children}
		</Button>
	);
}

function Page({ name, children, className, ...props }) {
	return (
		<div
			className={
				"w-full h-full flex-grow overflow-y-auto text-white " + className
			}
			{...props}
		>
			<div className="p-4 h-full">
				<h2 className="text-3xl font-bold">{name}</h2>
				{children}
			</div>
		</div>
	);
}

function Overview() {
	function overviewFetch() {
		fetch(fetchURLPrefix + "/api/cpuPercent", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setCpuUssagePercent(json["p"]);
				});
			}
		});
		fetch(fetchURLPrefix + "/api/ramPercent", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setRamUssagePercent(json["p"]);
				});
			}
		});
		fetch(fetchURLPrefix + "/api/diskPercent", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setDiskUssagePercent(json["p"]);
				});
			}
		});
		fetch(fetchURLPrefix + "/api/deviceInformation", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setDeviceInformation(json);
				});
			}
		});
	}

	function ftpStatusFetch() {
		fetch(fetchURLPrefix + "/api/fetchFTPconfig", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setFtpEnabled(json["enabled"]);
				});
			}
		});
	}

	const [cpuUssagePercent, setCpuUssagePercent] = useState(0);
	const [ramUssagePercent, setRamUssagePercent] = useState(0);
	const [diskUssagePercent, setDiskUssagePercent] = useState(0);
	const [deviceInformation, setDeviceInformation] = useState({
		os_name: "",
		power_supply: "",
		zentrox_pid: "",
		process_number: "",
		hostname: "",
		uptime: "",
		temperature: "",
	});
	const [ftpEnabled, setFtpEnabled] = useState(false);
	const [updatingFTP, setUpdatingFTP] = useState(false);
	const [ftpEnableSpinner, setFtpEnableSpinner] = useState(false);
	useInterval(() => overviewFetch(), 2500);
	useInterval(() => {
		if (!updatingFTP) {
			ftpStatusFetch();
		}
	}, 5000);
	useEffect(() => {
		overviewFetch();
		ftpStatusFetch();
	}, []);
	return (
		<Page name="Overview" className="align-top">
			<div className="h-72">
				<div className="inline-block w-72 h-full align-top p-4 rounded-2xl border border-neutral-700 m-2">
					<Label className="text-lg">
						<PieChart className="inline-block h-6 pr-1" /> Resources
					</Label>
					<br />
					<Label className="text-neutral-700">
						<CpuIcon className="inline-block h-6 pb-1 pr-1" />
						Processor
					</Label>
					<br />
					<SideWayBarChart percentage={cpuUssagePercent} />
					<Label className="text-neutral-700">
						<MemoryStickIcon className="inline-block h-6 pb-1 pr-1" />
						Memory
					</Label>
					<br />
					<SideWayBarChart percentage={ramUssagePercent} />
					<Label className="text-neutral-700">
						<Disc2 className="inline-block h-6 pb-1 pr-1" />
						Disk
					</Label>
					<br />
					<SideWayBarChart percentage={diskUssagePercent} />
				</div>
				<div className="inline-block align-top w-fit h-full p-4 rounded-2xl border border-neutral-700 m-2">
					<Label className="text-lg">
						<MonitorIcon className="inline-block h-7 pr-1" /> System information
					</Label>
					<br />
					<table>
						<tbody>
							<tr>
								<td>
									<ComputerIcon className="inline-block h-6 pb-1 pr-1" />{" "}
									Operating System
								</td>
								<td>
									{deviceInformation["os_name"].replaceAll(/\(.*\)/g, "")}
								</td>
							</tr>

							<tr>
								<td>
									<Network className="inline-block h-6 pb-1 pr-1" /> Hostname{" "}
									<InfoButton
										title="Hostname"
										info="The name of your computer in your local network"
									/>
								</td>
								<td>{deviceInformation["hostname"]}</td>
							</tr>
							<tr>
								<td>
									<Thermometer className="inline-block h-6 pb-1 pr-1" />{" "}
									Temperature{" "}
									<InfoButton
										title="Temperature"
										info="The temperature of your computer's CPU"
									/>
								</td>
								<td>
									{deviceInformation["temperature"] === null
										? "No data"
										: deviceInformation["temperature"]}
								</td>
							</tr>
							<tr>
								<td>
									<WatchIcon className="inline-block h-6 pb-1 pr-1" /> Uptime{" "}
									<InfoButton
										title="Uptime"
										info="The time your computer is running since the last boot"
									/>
								</td>
								<td>{deviceInformation["uptime"]}</td>
							</tr>
							<tr>
								<td>
									<Plug className="inline-block h-6 pb-1 pr-1" /> Power Supply
								</td>
								<td>{deviceInformation["power_supply"]}</td>
							</tr>
							<tr>
								<td>
									<List className="inline-block h-6 pb-1 pr-1" /> Active
									Processes{" "}
									<InfoButton
										title="Active Processes"
										info="The number of currently running processes on your system"
									/>
								</td>
								<td>{deviceInformation["process_number"]}</td>
							</tr>
							<tr>
								<td>
									<TableIcon className="inline-block h-6 pb-1 pr-1" /> Zentrox
									PID{" "}
									<InfoButton
										title="Zentrox PID"
										info="The process ID of the currently running Zentrox instance"
									/>
								</td>
								<td>{deviceInformation["zentrox_pid"]}</td>
							</tr>
						</tbody>
					</table>
				</div>
				<div className="inline-block align-top w-48 h-full p-4 rounded-2xl border border-neutral-700 m-2">
					<Label className="text-lg">Servers</Label>
					<br />
					<Checkbox
						checked={ftpEnabled}
						id="ftpEnabled"
						onClick={(e) => {
							setFtpEnabled(!ftpEnabled);
							setFtpEnableSpinner(true);
							setUpdatingFTP(true);
							e.target.disabled = true;
							fetch(fetchURLPrefix + "/api/updateFTPConfig", {
								method: "POST",
								headers: {
									"Content-Type": "application/json",
								},
								body: JSON.stringify({
									enableFTP: !ftpEnabled,
									enableDisable: true,
								}),
							}).then((res) => {
								setTimeout(() => {
									ftpStatusFetch();
									setUpdatingFTP(false);
									setFtpEnableSpinner(false);
									e.target.disabled = false;
								}, 2000);
							});
						}}
					/>
					<label htmlFor="ftpEnabled">
						<Share2 className="inline-block h-4 w-4 ml-1" /> FTP Server
					</label>
				</div>
			</div>
		</Page>
	);
}

function Packages() {
	const { toast } = useToast();
	function fetchPackageList() {
		if (
			installedPackages.length + installedApps.length + otherPackages.length !==
			0
		)
			return;
		fetch(fetchURLPrefix + "/api/packageDatabase", {
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					var content = JSON.parse(json["content"]);
					setInstalledPackages(Array.from(content["packages"]));
					setInstalledApps(Array.from(content["apps"]));
					setOtherPackages(Array.from(content["others"]));
					setVisibility(true);
				});
			} else {
				toast({
					title: "Package Database Error",
					message: "Zentrox failed to retrieve a list of packages",
				});
				setVisibility(false);
			}
		});

		fetch(fetchURLPrefix + "/api/packageDatabaseAutoremove", {
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setAutoRemovePackages(json["packages"]);
				});
			} else {
				toast({
					title: "Package Database Error",
					message: "Zentrox failed to retrieve a list of packages",
				});
				setVisibility(false);
			}
		});
	}

	function installPackage(packageName, stateFn) {
		stateFn("working");
		fetch(
			fetchURLPrefix + "/api/installPackage/" + encodeURIComponent(packageName),
		).then((res) => {
			if (!res.ok) {
				stateFn("failed");
			} else {
				stateFn("done");
				setOtherPackages(
					otherPackages.filter((entry) => {
						if (entry.split(".")[0] === packageName) return false;
						return true;
					}),
				);
				setInstalledPackages([packageName, ...installedPackages]);
			}
		});
	}

	function removePackage(packageName, stateFn) {
		stateFn("working");
		fetch(
			fetchURLPrefix + "/api/removePackage/" + encodeURIComponent(packageName),
		).then((res) => {
			if (!res.ok) {
				stateFn("failed");
			} else {
				stateFn("done");
				setInstalledPackages(
					installedPackages.filter((entry) => {
						if (entry.split(".")[0] === packageName) return false;
						return true;
					}),
				);
				setOtherPackages([packageName, ...otherPackages]);
			}
		});
	}

	function PackageBox({ packageName, task, key }) {
		const [buttonState, setButtonState] = useState("default");
		return (
			<div
				className="inline-block p-4 m-2 w-72 h-24 border border-neutral-600 md:w-52 text-white rounded-sm align-top relative"
				title={packageName}
			>
				<span className="block mb-1">
					{packageName.length > 20
						? packageName.substring(0, 17) + "..."
						: packageName}
				</span>
				<Button
					className="block right-2 bottom-2 absolute"
					variant={task == "remove" ? "destructive" : "default"}
					onClick={(function () {
						if (task === "remove") {
							return () => {
								removePackage(packageName, setButtonState);
							};
						} else if (task === "install") {
							return () => {
								installPackage(packageName, setButtonState);
							};
						}
					})()}
				>
					{(function () {
						if (task === "remove" && buttonState === "default") {
							return "Remove";
						} else if (task === "install" && buttonState === "default") {
							return "Install";
						} else if (buttonState === "working") {
							return (
								<>
									<Loader2 className="h-4 w-4 inline-block animate-spin" />{" "}
									Working
								</>
							);
						} else if (buttonState === "failed") {
							return (
								<>
									<CircleX className="h-4 w-4 inline-block text-red-900" />{" "}
									Failed
								</>
							);
						} else if (buttonState === "done") {
							return (
								<>
									<CircleCheck className="w-4 h-4 inline-block text-green-800" />{" "}
									Done
								</>
							);
						}
					})()}
				</Button>
			</div>
		);
	}

	function AutoRemoveButon() {
		if (clearAutoRemoveButtonState === "default") {
			return (
				<Button
					className="inline"
					onClick={() => {
						setClearAutoRemoveButtonState("working");
						fetch("/api/clearAutoRemove").then((res) => {
							if (res.ok) {
								res.json().then((json) => {
									setAutoRemovePackages(json["packages"]);
								});
							}
							setClearAutoRemoveButtonState("default");
						});
					}}
				>
					<Paintbrush2 className="h-4 w-4 inline-block" /> Autoremove
				</Button>
			);
		} else {
			return (
				<Button
					className="inline"
					onClick={() => {
						setClearAutoRemoveButtonState("working");
						fetch("/api/clearAutoRemove").then((res) => {
							if (res.ok) {
								res.json().then((json) => {
									setAutoRemovePackages(json["packages"]);
								});
								setClearAutoRemoveButtonState("default");
							}
						});
					}}
				>
					<Loader2 className="h-4 w-4 inline-block animate-spin" /> Working
				</Button>
			);
		}
	}

	const [installedPackages, setInstalledPackages] = useState([]);
	const [installedApps, setInstalledApps] = useState([]);
	const [otherPackages, setOtherPackages] = useState([]);
	const [autoRemovePackages, setAutoRemovePackages] = useState([]);
	const [visible, setVisibility] = useState(false);
	const [packageSearchValue, setPackageSearchValue] = useState("");
	const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] =
		useState("default");
	useEffect(() => fetchPackageList(), []);

	if (visible) {
		if (packageSearchValue.length > 2) {
			var PackageView = (
				<>
					{installedPackages
						.filter((pkg) => {
							if (pkg == "Available") return false;
							if (pkg.length === 0) return false;
							if (pkg.includes(packageSearchValue)) {
								return true;
							}
							return false;
						})
						.sort((pkg) => {
							if (pkg == packageSearchValue) return -1;
							return +1;
						})
						.map((pkg, i) => {
							return (
								<PackageBox
									packageName={pkg.split(".")[0]}
									task="remove"
									key={i}
								></PackageBox>
							);
						})}
					{otherPackages
						.filter((pkg) => {
							if (pkg == "Available") return false;
							if (pkg.length === 0) return false;
							if (pkg.includes(packageSearchValue)) {
								return true;
							}
							return false;
						})
						.sort((pkg) => {
							if (pkg == packageSearchValue) return -1;
							return +1;
						})
						.map((pkg, i) => {
							return (
								<PackageBox
									packageName={pkg.split(".")[0]}
									task="install"
									key={i}
								></PackageBox>
							);
						})}
				</>
			);
		} else {
			var PackageView = (
				<>
					<div className="p-auto">
						<SearchIcon className="w-16 h-16 m-auto mt-8 text-neutral-600" />
						<br />
						<h3 className="text-xl text-neutral-600 m-auto text-center">
							Search for package to install or uninstall
						</h3>
					</div>
				</>
			);
		}
		return (
			<Page name="Packages">
				<StatCard
					name="Installed Packages"
					value={installedPackages.length}
					Icon={<HardDriveIcon className="h-5 w-5 inline-block" />}
					Info="Packages that are installed on your system. This includes apps."
				/>
				<StatCard
					name="Installed Apps"
					value={installedApps.length}
					Icon={<AppWindow className="h-5 w-5 inline-block" />}
					Info="Packages that have a graphical interface and are installed on your system."
				/>
				<StatCard
					name="Other Packages"
					value={otherPackages.length}
					Icon={<Package2 className="h-5 w-5 inline-block" />}
					Info="Packages including apps, that are not installed on your system but listed in your package manager."
				/>
				<StatCard
					name="Autoremove Packages"
					value={autoRemovePackages.length}
					Icon={<TrashIcon className="h-5 w-5 inline-block" />}
					Info="Packages that are not required by the system anymore"
				/>

				<br />
				<div className="h-fit">
					<Input
						placeholder="Search for package"
						onChange={(e) => {
							setPackageSearchValue(e.target.value);
						}}
						className="mt-2 inline-block"
					/>{" "}
					<AutoRemoveButon />
				</div>
				<br />
				{PackageView}
			</Page>
		);
	} else {
		return (
			<div className="p-auto pt-5">
				<Loader2 className="animate-spin m-auto w-20 h-20" />
			</div>
		);
	}
}

function Firewall() {
	const [rules, setRules] = useState([]);
	const [fireWallEnabled, setFireWallEnabled] = useState(false);
	const [newRuleAction, setNewRuleAction] = useState("allow");
	const [preventRefetch, setPreventRefetch] = useState(false);
	var newRuleTo = useRef("");
	var newRuleFrom = useRef("");
	const { toast } = useToast();

	function fetchFireWallInformation() {
		fetch(fetchURLPrefix + "/api/fireWallInformation", {
			headers: {
				"Content-Type": "application/json",
			},
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setRules(json["rules"]);
					setFireWallEnabled(json["enabled"]);
				});
			}
		});
	}

	useEffect(() => {
		if (!preventRefetch) fetchFireWallInformation();
	}, []);

	function RuleView() {
		if (fireWallEnabled) {
			return (
				<div className="max-h-64 overflow-y-scroll overflow-x-hidden w-fit no-scroll">
					<table className="pt-2 fireWall block">
						<tbody>
							<tr
								className="w-fit animate-fadein"
								style={{
									animationDuration: `100ms`,
								}}
							>
								<td>
									<ArrowUpFromDot className="w-4 h-4 pb-0.5 inline" /> To
								</td>
								<td>
									<ArrowDownToDot className="w-4 h-4 pb-0.5 inline" /> From
								</td>
								<td>
									<Shield className="w-4 h-4 pb-0.5 inline" /> Action
								</td>
								<td></td>
							</tr>
							{rules.map((rule, i) => {
								return (
									<tr
										key={i}
										className="w-fit animate-fadein"
										style={{
											animationDuration: `${i > 6 ? 600 : i * 100}ms`,
										}}
									>
										<td>{rule.to.replaceAll("(v6)", "IPv6")}</td>
										<td>{rule.from.replaceAll("(v6)", "IPv6")}</td>
										<td>
											{rule.action === "DENY" ? (
												<>
													<Ban className="h-4 w-4 inline-block text-red-500 pr-1" />
													Deny
												</>
											) : (
												<>
													<CircleCheck className="h-4 w-4 inline-block text-green-500 pr-1" />
													Allow
												</>
											)}
										</td>
										<td>
											<AlertDialog>
												<AlertDialogTrigger asChild>
													<Button className="bg-transparent text-white p-0 m-0 hover:bg-red-500/20 active:bg-red-500/30 w-12">
														<TrashIcon />
													</Button>
												</AlertDialogTrigger>
												<AlertDialogContent>
													<AlertDialogHeader>
														<AlertDialogTitle>Delete Rule</AlertDialogTitle>
														<AlertDialogDescription>
															Do you really want to remove this rule? This
															action can not be undone.
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
																).then((res) => {
																	if (!res.ok) {
																		toast({
																			title: "Failed to delete rule",
																			description:
																				"Zentrox failed to delete this rule.",
																		});
																	} else {
																		fetchFireWallInformation();
																	}
																});
															}}
														>
															Continue
														</AlertDialogAction>
													</AlertDialogFooter>
												</AlertDialogContent>
											</AlertDialog>
										</td>
									</tr>
								);
							})}
						</tbody>
					</table>
				</div>
			);
		} else {
			return (
				<span className="align-middle p-2 block">
					<BrickWall className="w-8 h-8 inline text-neutral-600" /> Firewall is
					disabled
				</span>
			);
		}
	}

	return (
		<>
			<Toaster />
			<Page name="Firewall">
				<div className="w-64">
					<div>
						<Dialog>
							<DialogTrigger>
								<Button className="mr-1">
									<Plus className="h-6 w-6 inline" />
									New Rule
								</Button>
							</DialogTrigger>
							<DialogContent>
								<DialogHeader>
									<DialogTitle>New Firewall Rule</DialogTitle>
									<DialogDescription>
										You can create a new rule that applies to your firewall.
									</DialogDescription>
									<label htmlFor="ruleTo">
										<ArrowUpFromDot className="w-4 h-4 inline" /> To
									</label>
									<Input id="ruleTo" placeholder="Port" ref={newRuleTo} />
									<label htmlFor="ruleFrom">
										<ArrowDownToDot className="w-4 h-4 inline" /> From
									</label>
									<Input
										id="ruleFrom"
										placeholder="IP Adress"
										ref={newRuleFrom}
									/>
									<label htmlFor="ruleAction">
										<Shield className="w-4 h-4 inline" /> Action
									</label>
									<Select
										value={newRuleAction}
										onValueChange={(e) => {
											setPreventRefetch(true);
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
									<DialogFooter>
										<DialogClose asChild>
											<Button
												onClick={() => {
													setPreventRefetch(false);
													if (
														newRuleFrom.current.value.length === 0 ||
														newRuleTo.current.value.length === 0 ||
														typeof newRuleTo.current.value === "undefined" ||
														typeof newRuleFrom.current.value === "undefined"
													) {
														toast({
															title: "Invalid rule",
															description:
																"Zentrox can not create a rule with these values",
														});
														return;
													}
													fetch(
														fetchURLPrefix +
															"/api/newFireWallRule/" +
															encodeURIComponent(newRuleFrom.current.value) +
															"/" +
															encodeURIComponent(newRuleTo.current.value) +
															"/" +
															encodeURIComponent(newRuleAction),
													).then((res) => {
														if (res.ok) {
															fetchFireWallInformation();
														} else {
															res.json().then((json) => {
																if (json["msg"] !== undefined) {
																	toast({
																		title: "Failed to create new rule",
																		description:
																			"Zentrox failed to create new rule with the UFW: " +
																			json["msg"],
																	});
																} else {
																	toast({
																		title: "Failed to create new rule",
																		description:
																			"Zentrox failed to create new rule with the UFW",
																	});
																}
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
						<Switch
							onClick={(e) => {
								e.target.disabled = true;
								fetch(
									fetchURLPrefix + "/api/switchUFW/" + !fireWallEnabled,
								).then((res) => {
									setFireWallEnabled(!fireWallEnabled);
									e.target.disabled = false;
									fetchFireWallInformation();
								});
							}}
							value={fireWallEnabled ? "on" : "off"}
							checked={fireWallEnabled}
							title="Enable Firewall"
							className="ml-1"
						/>
					</div>
					<RuleView />
				</div>
			</Page>
		</>
	);
}

function Files() {
	return (
		<>
			<Page name="Files">
				<FileView className=""></FileView>
			</Page>
		</>
	);
}

function Storage() {
	const { toast } = useToast();
	const [drivesList, setDrivesList] = useState([]);
	const [driveInformation, setDriveInformation] = useState({
		drives: {
			model: "N/A",
			path: "N/A",
			owner: "N/A",
			mountpoint: "",
			size: 0,
		},
		ussage: [],
	});
	const [currentDrive, setCurrentDrive] = useState([]);
	const [driveInformationDialogOpen, setDriveInformationDialogOpen] =
		useState(false);

	useEffect(() => {
		fetchDrivesList();
	}, []);

	function fetchDrivesList() {
		fetch(fetchURLPrefix + "/api/driveList").then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setDrivesList(json["drives"]);
				});
			} else {
				toast({
					title: "Failed to fetch drives list",
					description:
						"Zentrox failed to fetch a list of all connected storage mediums.",
				});
			}
		});
	}

	function showDriveDetails(driveName) {
		setDriveInformationDialogOpen(true);
		setCurrentDrive(driveName);
		fetch(
			fetchURLPrefix + "/api/driveInformation/" + encodeURIComponent(driveName),
		).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setDriveInformation(json);
				});
			} else {
				toast({
					title: "Failed to fetch drive informaiton",
					description: "Zentrox failed to fetch drive details",
				});
			}
		});
	}

	function DriveEntry({ entry, inset = 0 }) {
		var children = <></>;
		if (entry.children != null) {
			children = entry.children.map((entry) => {
				return <DriveEntry entry={entry} inset={inset + 1} />;
			});
		}

		return (
			<>
				{" "}
				<span
					className="w-full p-4 bg-transparent border border-neutral-800 border-x-transparent block cursor-default select-none hover:bg-neutral-800 hover:transition-bg hover:duration-400 duration-200 animate-fadein focus:bg-neutral-800 focus:duration-50"
					style={{
						paddingLeft: 16 + inset * 10,
					}}
					onClick={() => {
						showDriveDetails(entry.name);
					}}
				>
					{(function (entry) {
						if (entry.name.startsWith("loop")) {
							return (
								<RepeatIcon className="inline-block h-6 w-6 pr-1 text-neutral-700" />
							);
						} else if (inset != 0) {
							return (
								<LayoutPanelTopIcon className="inline-block h-6 w-6 pr-1" />
							);
						} else {
							return <HardDriveIcon className="inline-block h-6 w-6 pr-1" />;
						}
					})(entry)}{" "}
					{entry.name}
				</span>
				{children}
			</>
		);
	}

	/**
	 * @param {number} bytes
	 * @description Converts the unit Bytes into a higher unit and add a unit symbol*
	 * @returns {string} */
	function bytesUnitToOther(bytes) {
		if (bytes >= 1024 * 1024 * 1024) {
			return Math.round(bytes / (1024 * 1024 * 1024)) + " GB";
		} else if (bytes >= 1024 * 1024) {
			return Math.round(bytes / (1024 * 1024)) + " MB";
		} else if (bytes >= 1024) {
			return Math.round(bytes / 1024) + " KB";
		} else {
			return bytes + " B";
		}
	}

	var driveCapacity = "N/A";
	var drive;
	for (drive of driveInformation.ussage) {
		if (drive.mounted === driveInformation.drives.mountpoint) {
			driveCapacity = drive.capacity;
		}
	}

	return (
		<>
			<Toaster />
			<Dialog
				open={driveInformationDialogOpen}
				onOpenChange={setDriveInformationDialogOpen}
			>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>{currentDrive}</DialogTitle>
					</DialogHeader>
					<DialogDescription className="text-white">
						<b className="block mb-1">
							<TagIcon className="w-4 h-4 inline" /> Model
						</b>
						{na(driveInformation.drives.model)}
						<br />
						<b className="block mb-1">
							<MapPinIcon className="w-4 h-4 inline" /> Path
						</b>
						{na(driveInformation.drives.path)} <br />
						<b className="block mb-1">
							<WeightIcon className="w-4 h-4 inline" /> Size
						</b>
						{na(bytesUnitToOther(driveInformation.drives.size))} <br />
						<b className="block mb-1">
							<UserIcon className="w-4 h-4 inline" /> Owner
						</b>
						{na(driveInformation.drives.owner)} <br />
						<b className="block mb-1">
							<MountainIcon className="w-4 h-4 inline" /> Mountpoint
						</b>
						{na(driveInformation.drives.mountpoint)}
						<b className="block mb-1">
							<PieChartIcon className="w-4 h-4 inline" /> Ussage (Capacity)
						</b>
						{na(driveCapacity)}
					</DialogDescription>
					<DialogFooter>
						<DialogClose>
							<Button>Close</Button>
						</DialogClose>
					</DialogFooter>
				</DialogContent>
			</Dialog>
			<Page name="Storage">
				<div
					className="rounded-xl m-2 overflow-hidden overflow-y-scroll border-2 border-neutral-800"
					style={{ maxHeight: "calc(100vh - 180px)" }}
				>
					{drivesList
						.sort((a) => {
							if (a.name.includes("loop")) return 1;
							return -1;
						})
						.map((entry) => {
							return <DriveEntry entry={entry} />;
						})}
				</div>
			</Page>
		</>
	);
}

function Vault() {
	var vaultEncryptionKey = useRef();
	var vaultKeyDecryptModal = useRef();
	var uploadInput = useRef();
	var newDirectoryInput = useRef();
	var renamingModalInput = useRef();

	const { toast } = useToast();
	const [decryptKeyModalVisible, setDecryptKeyModalVisibility] =
		useState(false);
	const [decryptModalCallback, setDecryptModalCallback] = useState(() => {});
	const [currentVaultPath, setCurrentVaultPath] = useState("");
	const [currentVaultContents, setCurrentVaultContents] = useState([]);
	const [vaultViewVisible, setVaultViewVisible] = useState(false);
	const [vaultViewFadeOut, setVaultViewFadeOut] = useState(false);
	const [vaultSessionKey, setVaultSessionKey] = useState("");
	const [uploadButton, setUploadButton] = useState("default");
	const [downloadBackupButton, setDownloadBackupButton] = useState("default");
	const [deletionModalVisible, setDeletionModalVisible] = useState(false);
	const [renamingModalVisible, setRenamingModalVisible] = useState(false);
	const [currentVaultFileRename, setCurrentVaultFileRename] = useState("");
	const [currentVaultFileDelete, setCurrentVaultFileDelete] = useState("");

	function parentDir(path) {
		if (!path.endsWith("/")) path += "/";
		var parsedPath = path.split("/");
		parsedPath.pop();
		parsedPath.pop();
		var parentPath = parsedPath.join("/") + "/";
		if (parentPath === "/") parentPath = "";
		return parentPath;
	}

	function vaultTree(key = "") {
		var vaultKey;
		if (key === "") vaultKey = vaultKeyDecryptModal.current.value;
		else vaultKey = key;
		fetch(fetchURLPrefix + "/api/vaultTree", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				key: vaultKey,
			}),
		}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setCurrentVaultContents(json.fs);
					setVaultViewVisible(true);
					setVaultSessionKey(vaultKey);
				});
			} else {
				if (res.status === 403) {
					toast({
						title: "Auth Failed",
						description: "Zentrox was unable to validate your key",
					});
				}
			}
		});
	}

	function noDecryptKeyModal() {
		setDecryptKeyModalVisibility(true);
		var newCallback = function () {
			fetch(fetchURLPrefix + "/api/vaultConfigure", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					newKey: vaultEncryptionKey.current.value,
					oldKey: vaultKeyDecryptModal.current.value,
				}),
			}).then((res) => {
				if (res.ok) {
					toast({
						title: "Changed Key",
						description: "The vault key was changed successfully",
					});
				} else {
					toast({
						title: "Auth Failed",
						description: "Vault was unable to validate your key",
					});
				}
			});
		};
		setDecryptModalCallback(() => newCallback);
	}

	function requestRename() {
		setRenamingModalVisible(true);
	}

	function requestDeletion() {
		setDeletionModalVisible(true);
	}

	return (
		<Page name="Vault">
			<Toaster />
			<Dialog
				open={decryptKeyModalVisible}
				onOpenChange={setDecryptKeyModalVisibility}
			>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Unlock Vault</DialogTitle>
						<DialogDescription>
							Please enter your current vault key to proceed.
						</DialogDescription>
					</DialogHeader>
					<Input
						type="password"
						placeholder="Current key"
						ref={vaultKeyDecryptModal}
					/>
					<DialogFooter>
						<DialogClose>
							<Button
								onClick={() => {
									decryptModalCallback();
									var newCallback = function () {};
									setDecryptModalCallback(() => newCallback);
								}}
							>
								<KeyIcon className="w-4 h-4 pr-1" /> Proceed
							</Button>
						</DialogClose>
					</DialogFooter>
				</DialogContent>
			</Dialog>
			<div hidden={vaultViewVisible}>
				<label htmlFor="vaultEncryptionKey">
					Vault Encryption Key{" "}
					<InfoButton
						title="Vault Encryption Key"
						info={
							<>
								This key is used to encrypt and decrypt the vault.
								<br />
								If you have not yet used the vault, you can set the key now and
								the vault will be configured.
								<br />
								In case that you already have configured a key, you can change
								it by entering a new key. This requires the old key.
								<br />
								You should choose a strong key.
								<br />
								<strong>
									If the key gets lost, the vault can not be decrypted.
								</strong>
							</>
						}
					/>{" "}
				</label>
				<br />
				<Input
					type="password"
					id="vaultEncryptionKey"
					ref={vaultEncryptionKey}
					placeholder="Key"
					className="inline"
				/>{" "}
				<Button
					variant="destructive"
					className="inline-block mb-1"
					onClick={() => {
						/** @type {string}*/
						var key = vaultEncryptionKey.current.value;
						if (key.length === 0) {
							toast({
								title: "No key",
								description: "You need to enter a new key",
							});
							return;
						}
						fetch(fetchURLPrefix + "/api/vaultConfigure", {
							method: "POST",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								key: key,
							}),
						}).then((res) => {
							if (res.ok) {
								res.json().then((json) => {
									if (json.code === "no_decrypt_key") {
										noDecryptKeyModal();
									} else {
										toast({
											title: "Vault is configured",
											description: "A vault file was created",
										});
									}
								});
							} else {
								if (res.status === 400) {
									toast({
										title: "Bad Request",
										description: "The data you provided was incorrect",
									});
								} else {
									toast({
										title: "Error",
										description: "An error occured",
									});
								}
							}
						});
					}}
				>
					<KeyIcon className="w-4 h-4 inline" /> Change
				</Button>
				<br />
				<Button
					className="inline mr-1"
					onClick={() => {
						setDecryptKeyModalVisibility(true);
						var vaultUnlockForView = function () {
							var vaultKey = vaultKeyDecryptModal.current.value;
							if (vaultKey.length === 0) {
								toast({
									title: "No Key",
									description: "You did not provide a key for decryption",
								});
								return;
							}
							vaultTree();
						};
						setDecryptModalCallback(() => vaultUnlockForView);
					}}
				>
					<LockOpenIcon className="w-4 h-4 inline" /> Open Vault
				</Button>
				<Button
					variant="secondary"
					className="inline mr-1"
					onClick={() => {
						setDownloadBackupButton("loading");
						fetch(fetchURLPrefix + "/api/vaultBackup").then((res) => {
							if (res.ok) {
								res.blob().then((blob) => {
									setDownloadBackupButton("default");
									var url = window.URL.createObjectURL(blob);
									var a = document.createElement("a");
									a.href = url;
									a.download = "vault.tar";
									document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
									a.click();
									a.remove();
								});
							} else {
								toast({
									title: "Failed to fetch",
								});
							}
						});
					}}
				>
					{downloadBackupButton === "default" ? (
						<DownloadIcon className="w-4 h-4 inline mr-1" />
					) : (
						<LoaderIcon className="animate-spin h-4 w-4 inline mr-1" />
					)}{" "}
					Download Vault
				</Button>
			</div>
			<div
				hidden={!vaultViewVisible}
				className={vaultViewFadeOut ? "animate-fadeout duration-300" : ""}
			>
				<Button
					onClick={() => {
						setVaultViewFadeOut(true);
						setTimeout(() => {
							setCurrentVaultContents([]);
							setCurrentVaultPath("");
							setVaultViewVisible(false);
							setVaultSessionKey("");
						}, 290);
					}}
					className="mr-1"
					variant="destructive"
				>
					<LockIcon className="w-4 h-4 inline-block mr-1" /> Exit
				</Button>
				<Dialog>
					<DialogTrigger>
						<Button className="mr-1">
							<FolderIcon className="w-4 h-4 inline-block mr-1" /> New Directory
						</Button>
					</DialogTrigger>
					<DialogContent>
						<DialogHeader>
							<DialogTitle>New Directory</DialogTitle>
							<DialogDescription>Create a new directory.</DialogDescription>
						</DialogHeader>
						<Input type="text" ref={newDirectoryInput} placeholder="Name" />
						<DialogFooter>
							<DialogClose>
								<Button
									onClick={() => {
										if (
											newDirectoryInput.current.value.includes("/") ||
											newDirectoryInput.current.value.includes(" ")
										) {
											toast({
												title: "Illegal name",
											});
											return;
										}
										fetch(fetchURLPrefix + "/api/vaultNewFolder", {
											method: "POST",
											headers: {
												"Content-Type": "application/json",
											},
											body: JSON.stringify({
												key: vaultSessionKey,
												folder_name:
													currentVaultPath +
													"/" +
													newDirectoryInput.current.value,
											}),
										}).then((res) => {
											if (res.ok) {
												vaultTree(vaultSessionKey);
											} else {
												toast({
													title: "Can't create folder",
													description: `Vault could not create a directory ${newDirectoryInput.current.value} in ${currentVaultPath}`,
												});
											}
										});
									}}
								>
									<FolderIcon className="w-4 h-4 inline-block mr-1" /> Create
								</Button>
							</DialogClose>
						</DialogFooter>
					</DialogContent>
				</Dialog>
				<Dialog
					open={renamingModalVisible}
					onOpenChange={setRenamingModalVisible}
				>
					<DialogContent>
						<DialogHeader>
							<DialogTitle>Rename File</DialogTitle>
							<DialogDescription>Rename a file</DialogDescription>
						</DialogHeader>
						<Input
							type="text"
							ref={renamingModalInput}
							placeholder="New Name"
						/>
						<DialogFooter>
							<DialogClose asChild>
								<Button
									onClick={() => {
										fetch(fetchURLPrefix + "/api/renameVaultFile", {
											method: "POST",
											headers: {
												"Content-Type": "application/json",
											},
											body: JSON.stringify({
												key: vaultSessionKey,
												path: currentVaultPath + "/" + currentVaultFileRename,
												newName:
													currentVaultPath +
													"/" +
													renamingModalInput.current.value,
											}),
										}).then((res) => {
											if (res.ok) vaultTree(vaultSessionKey);
											else toast({ title: "Failed to rename file" });
										});
									}}
								>
									<PenLineIcon className="w-4 h-4 inline-block mr-1" /> Rename
								</Button>
							</DialogClose>
						</DialogFooter>
					</DialogContent>
				</Dialog>
				<AlertDialog
					open={deletionModalVisible}
					onOpenChange={setDeletionModalVisible}
				>
					<AlertDialogContent>
						<AlertDialogTitle>Delete File</AlertDialogTitle>
						<AlertDialogDescription>
							Do you really want to delete {currentVaultFileDelete}?<br />
							This action can not be undone.
						</AlertDialogDescription>
						<AlertDialogFooter>
							<AlertDialogAction
								onClick={() => {
									fetch(fetchURLPrefix + "/api/deleteVaultFile", {
										method: "POST",
										headers: {
											"Content-Type": "application/json",
										},
										body: JSON.stringify({
											key: vaultSessionKey,
											deletePath:
												currentVaultPath + "/" + currentVaultFileDelete,
										}),
									}).then((res) => {
										if (res.ok) vaultTree(vaultSessionKey);
										else toast({ title: "Failed to delete file" });
									});
								}}
							>
								Delete
							</AlertDialogAction>
						</AlertDialogFooter>
					</AlertDialogContent>
				</AlertDialog>
				<Button
					className="mr-1"
					onClick={() => {
						uploadInput.current.click();
					}}
				>
					{uploadButton === "default" ? (
						<UploadIcon className="w-4 h-4 inline-block mr-1" />
					) : (
						<LoaderIcon className="animate-spin h-4 w-4 inline mr-1" />
					)}{" "}
					Upload File
				</Button>
				<input
					type="file"
					ref={uploadInput}
					onChange={() => {
						setUploadButton("loading");
						var fileForSubmit = uploadInput.current.files[0];
						if (fileForSubmit.size >= 1024 * 1024 * 1024 * 2) {
							toast({
								title: "File to big",
								description: "The file you provided was larger than 2GB",
							});
						}
						var formData = new FormData();
						formData.append("file", fileForSubmit);
						formData.append("path", currentVaultPath);
						formData.append("key", vaultSessionKey);
						fetch(fetchURLPrefix + "/upload/vault", {
							method: "POST",
							body: formData,
						}).then((res) => {
							if (res.ok) {
								vaultTree(vaultSessionKey);
								setUploadButton("default");
							} else {
								toast({
									title: "File upload error",
								});
							}
						});
					}}
					hidden
				/>
				<Button
					className="mr-1"
					onClick={() => {
						setCurrentVaultPath(parentDir(currentVaultPath));
					}}
				>
					<ArrowUp className="w-4 h-4 inline-block mr-1" /> Up
				</Button>
			</div>
			<div
				className={`no-scroll overflow-scroll ${
					vaultViewFadeOut ? "animate-fadeout duration-300" : ""
				} h-fit rounded-xl mt-2 overflow-hidden overflow-y-scroll border-2 border-neutral-800 no-scroll`}
				style={{
					minHeight: "fit-content",
					maxHeight: "calc(100vh - 220px)",
				}}
				hidden={!vaultViewVisible}
			>
				{
					/*
					 * @param {string} entry*/
					currentVaultContents
						.filter((entry) => {
							if (
								entry.startsWith(currentVaultPath) &&
								entry.replace(currentVaultPath).split("/").length <= 2 &&
								entry != currentVaultPath
							)
								return true;
							return false;
						})
						.map((entry) => {
							var type = "";
							if (entry.endsWith("/")) {
								type = "folder";
							} else {
								type = "file";
							}
							return (
								<ContextMenu>
									<ContextMenuContent>
										<ContextMenuItem
											onClick={() => {
												setCurrentVaultFileDelete(entry);
												requestDeletion(entry);
											}}
										>
											<DeleteIcon className="w-4 h-4 inline-block mr-1" />{" "}
											Delete
										</ContextMenuItem>
										<ContextMenuItem
											onClick={() => {
												setCurrentVaultFileRename(entry);
												requestRename(entry);
											}}
										>
											<PenLineIcon className="w-4 h-4 inline-block mr-1" />{" "}
											Rename
										</ContextMenuItem>
									</ContextMenuContent>
									<ContextMenuTrigger>
										<span
											className="w-full p-4 bg-transparent border border-neutral-800 border-x-transparent block cursor-default select-none hover:bg-neutral-800 hover:transition-bg hover:duration-400 duration-200 animate-fadein focus:bg-neutral-800 focus:duration-50"
											onClick={
												type === "folder"
													? () => {
															setCurrentVaultPath(entry);
														}
													: (e) => {
															e.target.classList.add("animate-pulse");
															e.target.classList.add("duration-300");

															e.target.classList.remove("duration-200");
															fetch(fetchURLPrefix + "/api/vaultFileDownload", {
																method: "POST",
																headers: {
																	"Content-Type": "application/json",
																},
																body: JSON.stringify({
																	key: vaultSessionKey,
																	path: currentVaultPath + entry,
																}),
															}).then((res) => {
																e.target.classList.remove("animate-pulse");
																e.target.classList.remove("duration-300");

																e.target.classList.add("duration-200");
																if (res.ok) {
																	res.blob().then((blob) => {
																		var url = window.URL.createObjectURL(blob);
																		var a = document.createElement("a");
																		a.href = url;
																		a.download = entry;
																		document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
																		a.click();
																		a.remove();
																	});
																} else {
																	toast({
																		title: "File download error",
																	});
																}
															});
														}
											}
										>
											{type === "folder" ? (
												<FolderIcon
													className="w-6 h-6 inline-block mr-1"
													fill="white"
												/>
											) : (
												<FileIcon className="w-6 h-6 inline-block mr-1" />
											)}{" "}
											{type === "folder"
												? entry.split("/").at(-2)
												: entry.split("/").at(-1)}
										</span>
									</ContextMenuTrigger>
								</ContextMenu>
							);
						})
				}
			</div>
		</Page>
	);
}

function Servers() {
	var ftpUserNameInput = useRef();
	var ftpPassWordInput = useRef();
	var ftpRootInput = useRef();
	var ftpUserEnabledCheckBox = useRef();

	const [ftpConfig, setFtpConfig] = useState({
		enabled: false,
		ftpUserName: "",
		ftpLocalRoot: "",
	});
	const [ftpCheckBoxChecked, setFtpCheckBoxChecked] = useState(false);

	const fetchFtpConfig = () => {
		fetch(fetchURLPrefix + "/api/fetchFTPconfig").then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					setFtpConfig(json);
					setFtpCheckBoxChecked(json.enabled);
					ftpUserNameInput.current.value = json.ftpUserUsername;
					ftpRootInput.current.value = json.ftpLocalRoot;
				});
			} else {
				toast({
					title: "Failed to fetch FTP config",
				});
			}
		});
	};

	useEffect(fetchFtpConfig, []);

	return (
		<Page name="Servers">
			<h1 className="text-xl">General</h1>
			Certificates{" "}
			<InfoButton
				title={"Certificates"}
				info={
					<>
						Zentrox automatically generates self signed certificates to provide
						an encrypted connection.
						<br />
						This connection is not protected from Man-In-The-Middle attacks,
						which is why it is recommended to use a SSL certificate by a
						Certificate Authority.
					</>
				}
			/>
			<h1 className="text-xl">
				FTP{" "}
				<InfoButton
					title={"File Transfer Protocol"}
					info={
						<>
							The FTP Protocol is used to transfer files. <br />
							Zentrox automatically encrypts the trafic using the provided
							certificates.
							<br />
						</>
					}
				/>
			</h1>
			<Checkbox
				onClick={() => {
					setFtpCheckBoxChecked(!ftpCheckBoxChecked);
					console.log(ftpCheckBoxChecked);
				}}
				id="ftpUserEnabledCheckBox"
				className="ml-1"
				checked={ftpCheckBoxChecked}
			/>{" "}
			<label htmlFor="ftpUserEnabledCheckBox">Enabled</label>
			<br />
			<Label className="block p-1">Username</Label>
			<Input
				type="text"
				ref={ftpUserNameInput}
				placeholder="Username"
				className="inline-block mb-1"
			/>{" "}
			<InfoButton
				title={"FTP Username"}
				info={
					<>
						The FTP username is the username that is used to connect to your FTP
						server. By Default it is: <code>ftp_zentrox</code>
					</>
				}
			/>
			<br />
			<Label className="block p-1">Password</Label>
			<Input
				type="password"
				ref={ftpPassWordInput}
				placeholder="Password"
				className="inline-block mb-1"
			/>{" "}
			<InfoButton
				title={"FTP Password"}
				info={
					<>
						The FTP password is the password that is used to connect to your FTP
						server. <br />
						You should change it to prevent getting hacked.
						<br />
						By Default it is: <code>change_me</code>
					</>
				}
			/>
			<br />
			<Label className="block p-1">Root Directory</Label>
			<Input
				type="text"
				ref={ftpRootInput}
				placeholder="FTP Root Directory"
				className="inline-block mb-1"
			/>{" "}
			<InfoButton
				title={"FTP Root"}
				info={
					<>
						The FTP root directory is the directory that a connected FTP user
						can access. By Default it is: <code>/</code>
					</>
				}
			/>
			<br />
			<Button
				onClick={() => {
					fetch(fetchURLPrefix + "/api/updateFTPConfig", {
						method: "POST",
						headers: {
							"Content-Type": "application/json",
						},
						body: JSON.stringify({
							enableDisable: false,
							enableFTP: ftpCheckBoxChecked,
							ftpUserUsername: ftpUserNameInput.current.value,
							ftpLocalRoot: ftpRootInput.current.value,
							ftpUserPassword: ftpPassWordInput.current.value,
						}),
					}).then((res) => {
						if (res.ok) {
							toast({
								title: "FTP server updated",
							});
						} else {
							toast({
								title: "FTP server error",
								description: "Failed to update FTP server config",
							});
						}
					});
				}}
			>
				Apply
			</Button>
		</Page>
	);
}

export default function Dashboard() {
	const [activeTab, setActiveTab] = useState("Overview");

	function PageToShow() {
		if (activeTab == "Overview") {
			return Overview();
		} else if (activeTab == "Packages") {
			return Packages();
		} else if (activeTab == "Firewall") {
			return Firewall();
		} else if (activeTab == "Files") {
			return Files();
		} else if (activeTab == "Storage") {
			return Storage();
		} else if (activeTab == "Vault") {
			return Vault();
		} else if (activeTab == "Servers") {
			return Servers();
		}
	}

	return (
		<main className="h-screen w-screen overflow-hidden p-0 m-0 flex flex-col">
			<Toaster />
			<TopBar>
				<span
					className="p-2 pl-4 pr-4 border border-neutral-700 cursor-pointer rounded transition-all content-center inline-block text-lg font-normal"
					onClick={() => {
						window.open("https://github.com/wervice/zentrox");
					}}
				>
					<img
						src="zentrox_dark.svg"
						className="inline-block pb-0.5 w-5 h-5"
						alt="Zentrox Logo"
					/>{" "}
					Zentrox
				</span>{" "}
				<TabButton
					onClick={() => {
						setActiveTab("Overview");
					}}
					isDefault={true}
					isActive={activeTab == "Overview"}
				>
					Overview
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Packages");
					}}
					isDefault={false}
					isActive={activeTab == "Packages"}
				>
					Packages
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Firewall");
					}}
					isDefault={false}
					isActive={activeTab == "Firewall"}
				>
					Firewall
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Files");
					}}
					isDefault={false}
					isActive={activeTab == "Files"}
				>
					Files
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Storage");
					}}
					isDefault={false}
					isActive={activeTab == "Storage"}
				>
					Storage
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Vault");
					}}
					isDefault={false}
					isActive={activeTab == "Vault"}
				>
					Vault
				</TabButton>
				<TabButton
					onClick={() => {
						setActiveTab("Servers");
					}}
					isDefault={false}
					isActive={activeTab == "Servers"}
				>
					Servers
				</TabButton>
				<AlertDialog>
					<AlertDialogTrigger asChild>
						<Button variant="link" className="text-white p-2 m-0 float-right">
							<PowerOffIcon className="h-16 p-1 text-red-500" /> Power Off
						</Button>
					</AlertDialogTrigger>
					<AlertDialogContent>
						<AlertDialogHeader>
							<AlertDialogTitle>Power Off</AlertDialogTitle>
							<AlertDialogDescription>
								Do you really want to power off your machine?
								<br />
								Zentrox can not reboot it automatically.
							</AlertDialogDescription>
						</AlertDialogHeader>
						<AlertDialogFooter>
							<AlertDialogAction
								onClick={() => {
									fetch(fetchURLPrefix + "/api/powerOff").then((res) => {
										if (!res.ok) {
											toast({
												title: "Power Off failed",
											});
										}
									});
								}}
							>
								Power Off
							</AlertDialogAction>
						</AlertDialogFooter>
					</AlertDialogContent>
				</AlertDialog>
				<Button
					variant="link"
					className="text-white p-2 m-0 float-right"
					onClick={() => {
						location.href = "/logout";
					}}
				>
					<LogOut className="h-16 p-1" /> Logout
				</Button>
			</TopBar>
			<PageToShow />
		</main>
	);
}
