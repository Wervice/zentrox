import { Button } from "@/components/ui/button.jsx"

import TopBarInformative from "@/components/ui/TopBarInformative"

export default function Dashboard() {
	return (
		<main>
			<TopBarInformative><Button className="float-right text-white p-0 h-fit" variant="link">Logout</Button> Dashboard</TopBarInformative>
		</main>
	)
}
