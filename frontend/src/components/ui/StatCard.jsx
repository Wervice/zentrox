import InfoButton from "./InfoButton.jsx";

export default function StatCard({ name, value, Info = null, Icon = <></> }) {
 if (Info !== null) {
  var I = <InfoButton title={name} info={Info} />;
 } else {
  var I = <></>;
 }
 return (
  <div className="inline-block border border-neutral-600 rounded-xl w-64 p-4 mb-2 mt-2 mr-5">
   <span className="text-neutral-400 block">
    {Icon} {name} {I}
   </span>
   <span className="text-2xl text-white font-bold">{value}</span>
  </div>
 );
}
