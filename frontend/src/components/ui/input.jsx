import * as React from "react";
import { useState } from "react";

import { cn } from "@/lib/utils";
import { EyeIcon, EyeOffIcon, SearchIcon } from "lucide-react";

const Input = React.forwardRef(({ className, type, ...props }, ref) => {
  const [forceText, setForceText] = useState(false);
  if (type == "password") {
    const I = !forceText ? EyeIcon : EyeOffIcon;

    return (
      <span
        className={cn(
          "flex h-10 w-56 rounded-md bg-transparent border-neutral-700 transition-colors duration-200 hover:border-neutral-600 border px-3 py-2 mt-0.5 text-sm file:bg-transparent file:text-sm file:font-medium placeholder:text-white/80 focus-visible:outline-none focus-visible:border-transparent disabled:cursor-not-allowed disabled:opacity-50 focus-visible:ring-2 items-center",
          className,
        )}
      >
        <input
          className="border-0 outline-none p-0 m-0 w-full h-full bg-transparent text-white text-sm"
          type={forceText ? "text" : "password"}
          ref={ref}
          {...props}
        />
        <I
          onClick={() => setForceText(!forceText)}
          className="inline-block h-4 ml-1"
        />
      </span>
    );
  } else {
    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-56 rounded-md bg-transparent border-neutral-700 transition-colors duration-200 hover:border-neutral-600 border px-3 py-2 mt-0.5 text-sm file:bg-transparent file:text-sm file:font-medium placeholder:text-white/80 focus-visible:outline-none focus-visible:border-transparent disabled:cursor-not-allowed disabled:opacity-50 focus-visible:ring-2",
          className,
        )}
        ref={ref}
        {...props}
      />
    );
  }
});
Input.displayName = "Input";

export { Input };
