"use client";

import { Button } from "@/components/ui/button.jsx";
import TopBarInformative from "@/components/ui/TopBarInformative";
import {
	ComputerIcon,
	CpuIcon,
	Disc2,
	List,
	LogOut,
	MemoryStickIcon,
	MonitorIcon,
	MonitorUpIcon,
	MonitorX,
	Network,
	PieChart,
	Plug,
	TableIcon,
	Thermometer,
	WatchIcon,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { SideWayBarChart } from "@/components/ui/Charts.jsx";
import { useInterval } from "usehooks-ts";
import { table, tr, td, tbody } from "react-table";
import "./table.css";

const fetchURLPrefix = "https://localhost:3000"; // I need this while developing

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

	useInterval(() => overviewFetch(), 5000);
	useEffect(() => overviewFetch(), []);
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
									<Network className="inline-block h-6 pb-1 pr-1" /> Hostname
								</td>
								<td>{deviceInformation["hostname"]}</td>
							</tr>
							<tr>
								<td>
									<Thermometer className="inline-block h-6 pb-1 pr-1" />{" "}
									Temperature
								</td>
								<td>
									{deviceInformation["temperature"] === null
										? "No data"
										: deviceInformation["temperature"]}
								</td>
							</tr>
							<tr>
								<td>
									<WatchIcon className="inline-block h-6 pb-1 pr-1" /> Uptime
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
									Processes
								</td>
								<td>{deviceInformation["process_number"]}</td>
							</tr>
							<tr>
								<td>
									<TableIcon className="inline-block h-6 pb-1 pr-1" /> Zentrox
									PID
								</td>
								<td>{deviceInformation["zentrox_pid"]}</td>
							</tr>
						</tbody>
					</table>
				</div>
			</div>
		</Page>
	);
}

export default function Dashboard() {
	const [activeTab, setActiveTab] = useState("Overview");

	function PageToShow() {
		if (activeTab == "Overview") {
			return Overview();
		} else if (activeTab == "Packages") {
			return <Page name="Packages">DEF</Page>;
		}
	}

	return (
		<main className="h-screen w-screen overflow-hidden p-0 m-0 flex flex-col">
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
