const { ArrowUp, HouseIcon, PenIcon, FolderTreeIcon, CopyIcon } = require("lucide-react");
import { useRef } from "react";
import { Dialog, DialogClose, DialogFooter, DialogTitle, DialogOverlay, DialogPortal, DialogHeader, DialogTrigger, DialogContent, DialogDescription } from "./dialog";
import { Input } from "./input";
import { Button } from "./button";

function PathViewer({ className, onValueChange, value, home, hidden }) {
	var specificLocationInput = useRef();

	function goUp() {
		let path = value;
		let segments = path.split("/");
		segments.pop();
		segments.pop()
		let newPath = segments.join("/");
		onValueChange(newPath + "/")
	}

	function goHome() {
		onValueChange(home)
	}

	return (
		<span className={"flex items-center p-1 w-full whitespace-nowrap overflow-hidden max-w-full " + className + (hidden ? " hidden" : "")}>
				<HouseIcon className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-2" onClick={goHome} />
				<CopyIcon className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-2" onClick={() => {
					navigator.clipboard.writeText(value)
				}} />
				<Dialog>	
				<DialogTrigger asChild>
					<PenIcon className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1" />
				</DialogTrigger>
				<DialogContent>
					<DialogHeader>
					<DialogTitle>Specific location</DialogTitle>
					<DialogDescription>Move to a specific location</DialogDescription>
					</DialogHeader>
		<p>
			<Input type="text" ref={specificLocationInput} defaultValue={value} className="w-full block" />
		</p>
		<DialogFooter>
			<DialogClose asChild><Button variant="outline">Close</Button></DialogClose>
			<DialogClose asChild><Button onClick={
				() => {
					let newPath = specificLocationInput.current.value;
					if (newPath.endsWith("/")) {
						onValueChange(newPath)
					} else {
						onValueChange(newPath + "/")
					}
				}
			}>Confirm</Button></DialogClose>
		</DialogFooter>
				</DialogContent>
				</Dialog>
				<ArrowUp className="w-5 h-5 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1" onClick={goUp} /> 
				{value} </span>
	)
}

export default PathViewer
