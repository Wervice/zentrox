export default function Page({
  name,
  children,
  className,
  titleAbsolute = false,
  ...props
}) {
  return (
    <div
      className={
        "w-full h-screen flex-grow overflow-y-auto text-white animate-fadein duration-300 overflow-hidden" +
        className
      }
      {...props}
    >
      <div className="p-4 h-full">
        <h2 className={"text-3xl font-bold " + (titleAbsolute && "absolute")}>
          {name}
        </h2>
        {children}
      </div>
    </div>
  );
}
