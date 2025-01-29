import InfoButton from "./InfoButton.jsx";

export default function StatCard({ name, value, Info = null, Icon = <></> }) {
 if (Info !== null) {
  var I = <InfoButton title={name} info={Info} />;
 } else {
  var I = <></>;
 }
 return (
  <div className="p-4 m-2 ml-0 mb-0 rounded-xl overflow-hidden overflow-ellipsis whitespace-pre bg-zinc-950 border-zinc-900 border inline-block hover:border-zinc-800 duration-500">
   <span className="text-neutral-400 block">
    {Icon} {name} {I}
   </span>
   <span className="text-2xl text-white font-bold">{value}</span>
  </div>
 );
}
