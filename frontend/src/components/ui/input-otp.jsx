"use client";

import * as React from "react";
import { OTPInput, OTPInputContext } from "input-otp";
import { Dot } from "lucide-react";

import { cn } from "@/lib/utils";

const InputOTP = React.forwardRef(
	({ className, containerClassName, ...props }, ref) => (
		<OTPInput
			ref={ref}
			containerClassName={cn(
				"flex items-center gap-2 has-[:disabled]:opacity-50 ml-0.5",
				containerClassName,
			)}
			className={cn("disabled:cursor-not-allowed", className)}
			{...props}
		/>
	),
);
InputOTP.displayName = "InputOTP";

const InputOTPGroup = React.forwardRef(({ className, ...props }, ref) => (
	<div ref={ref} className={cn("flex items-center", className)} {...props} />
));
InputOTPGroup.displayName = "InputOTPGroup";

const InputOTPSlot = React.forwardRef(({ index, className, ...props }, ref) => {
	const inputOTPContext = React.useContext(OTPInputContext);
	const { char, hasFakeCaret, isActive } = inputOTPContext.slots[index];

	return (
		<div
			ref={ref}
			className={cn(
				"relative flex h-10 w-10 items-center justify-center border-y border-r border-neutral-700 text-sm transition-all first:rounded-l-md first:border-l last:rounded-r-md dark:border-zinc-800 mt-0.5",
				isActive && "bg-neutral-900 caret-neutral-200",
				className,
			)}
			{...props}
		>
			{char}
			{hasFakeCaret && (
				<div className="pointer-events-none absolute inset-0 flex items-center justify-center">
					<div className="h-4 w-px animate-caret-blink bg-zinc-200 duration-1000 dark:bg-zinc-200" />
				</div>
			)}
		</div>
	);
});
InputOTPSlot.displayName = "InputOTPSlot";

const InputOTPSeparator = React.forwardRef(({ ...props }, ref) => (
	<div ref={ref} role="separator" {...props}>
		<Dot />
	</div>
));
InputOTPSeparator.displayName = "InputOTPSeparator";

export { InputOTP, InputOTPGroup, InputOTPSlot, InputOTPSeparator };
