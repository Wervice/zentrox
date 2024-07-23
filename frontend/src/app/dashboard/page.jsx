"use client";


import { Checkbox } from "@/components/ui/checkbox.jsx";
import { Separator } from "@/components/ui/separator.jsx"
import { Button } from "@/components/ui/button.jsx"
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
	CircleCheck,
	SearchIcon,
	TrashIcon,
    Paintbrush2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { SideWayBarChart } from "@/components/ui/Charts.jsx";
import { useInterval } from "usehooks-ts";
import { table, tr, td, tbody } from "react-table";
import "./table.css";
import Spinner from "@/components/ui/Spinner.jsx";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { useToast } from "@/components/ui/use-toast";
import InfoButton from "@/components/ui/InfoButton.jsx";

// const fetchURLPrefix = "";
const fetchURLPrefix = "https://localhost:3000";

if (fetchURLPrefix.length > 0) {
	console.error("Fetch URL Prefix is enabled");
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
					<Label className="text-muted-foreground">
						<CpuIcon className="inline-block h-6 pb-1 pr-1" />
						Processor
					</Label>
					<br />
					<SideWayBarChart percentage={cpuUssagePercent} />
					<Label className="text-muted-foreground">
						<MemoryStickIcon className="inline-block h-6 pb-1 pr-1" />
						Memory
					</Label>
					<br />
					<SideWayBarChart percentage={ramUssagePercent} />
					<Label className="text-muted-foreground">
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
					/>{" "}
					<label htmlFor="ftpEnabled">
						<Share2 className="inline-block h-4 w-4" />{" "}
						<Spinner visible={ftpEnableSpinner} /> FTP Server
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

	const [installedPackages, setInstalledPackages] = useState([]);
	const [installedApps, setInstalledApps] = useState([]);
	const [otherPackages, setOtherPackages] = useState([]);
	const [autoRemovePackages, setAutoRemovePackages] = useState([]);
	const [visible, setVisibility] = useState(false);
	const [packageSearchValue, setPackageSearchValue] = useState("");
	const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] = useState("default")
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
				/> <Button variant="secondary" className="inline" onClick={
					() => {
						setClearAutoRemoveButtonState("working")
						fetch("/api/clearAutoRemove").then((res) => {
							if (res.ok) {
								res.json().then((json) => {
									setAutoRemovePackages(json["packages"])
								})
							}
						})
					}
				}><Paintbrush2 className="h-4 w-4 inline-block" /> Autoremove</Button>
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

function Security() {
	return (<Page name="Security"></Page>)
}

export default function Dashboard() {
	const [activeTab, setActiveTab] = useState("Overview");

	function PageToShow() {
		if (activeTab == "Overview") {
			return Overview();
		} else if (activeTab == "Packages") {
			return Packages();
		} else if (activeTab == "Security") {
			return Security();
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
					<img src="zentrox_dark.svg" className="inline-block pb-0.5 w-5 h-5" />{" "}
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
