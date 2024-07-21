export default function StatCard({ name, value, Icon = (<></>) }) {
	return (
		<div className="inline-block border border-neutral-600 rounded-xl w-56 p-4 mb-2 mt-2 mr-5">
			<span className="text-neutral-400 block">{Icon} {name}</span>
			<span className="text-2xl text-white">{value}</span>
		</div>
	)
}
