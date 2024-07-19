"use client";

import { Button } from "@/components/ui/button.jsx";
import TopBarInformative from "@/components/ui/TopBarInformative";
import { LogOut } from "lucide-react";
import { useState } from "react";
import { Label } from "@/components/ui/label";
import { SideWayBarChart } from "@/components/ui/Charts.jsx";

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

function Page({ name, children }) {
	return (
		<div className="w-full h-full flex-grow overflow-y-auto text-white">
			<div className="p-4">
				<h2 className="text-3xl font-bold">{name}</h2>
				{children}
			</div>
		</div>
	);
}

function Overview() {
	const [cpuUssagePercent, setCpuUssagePercent] = useState(20);
	return (
		<Page name="Overview">
			<Label>Resources</Label>
			<br />
			<Label className="text-muted-foreground">Processor</Label><br />	
			<SideWayBarChart percentage={cpuUssagePercent}/>
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
