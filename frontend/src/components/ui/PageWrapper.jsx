export default function Page({ name, children, className, ...props }) {
 return (
  <div
   className={"w-full h-full flex-grow overflow-y-auto text-white " + className}
   {...props}
  >
   <div className="p-4 h-full">
    <h2 className="text-3xl font-bold">{name}</h2>
    {children}
   </div>
  </div>
 );
}
