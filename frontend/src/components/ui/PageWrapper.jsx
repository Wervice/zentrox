import { cn } from "@/lib/utils";

export default function Page({
  name,
  children,
  className = "",
  titleAbsolute = false,
  ...props
}) {
  return (
    <div
      className={cn(
        "w-full h-screen max-h-screen flex-grow overflow-y-hidden text-white animate-fadein duration-300 overflow-hidden p-4",
        className,
      )}
      {...props}
    >
      <h2 className={"text-3xl font-bold " + (titleAbsolute && "absolute")}>
        {name}
      </h2>
      {children}
    </div>
  );
}
