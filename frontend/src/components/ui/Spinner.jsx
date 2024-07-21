import { Loader2 } from "lucide-react";

export default function Spinner({ visible = false }) {
	if (!visible) return <></>;	
	return <Loader2 className="inline-block h-4 w-4 animate-spin" />;
}
