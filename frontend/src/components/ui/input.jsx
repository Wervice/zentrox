import * as React from "react";

import { cn } from "@/lib/utils";
import { useState } from "react";

const Input = React.forwardRef(({ className, type, ...props }, ref) => {
 return (
  <input
   type={type}
   className={cn(
    "flex h-10 w-56 rounded-md bg-transparent border-neutral-600 border px-3 py-2 ml-0.5 mt-0.5 text-sm file:bg-transparent file:text-sm file:font-medium placeholder:text-white/80 focus-visible:outline-none focus-visible:border-transparent disabled:cursor-not-allowed disabled:opacity-50 focus-visible:ring-2",
    className,
   )}
   ref={ref}
   {...props}
  />
 );
});
Input.displayName = "Input";

export { Input };
