"use client";

import { Button } from "@/components/ui/button.jsx";
import { Input } from "@/components/ui/input.jsx";
import {
	InputOTP,
	InputOTPGroup,
	InputOTPSlot,
} from "@/components/ui/input-otp";
import { Toaster } from "@/components/ui/toaster";
import { toast, useToast } from "@/components/ui/use-toast";
import { ToastAction } from "@/components/ui/toast.jsx";

import Label from "@/components/ui/ShortLabel.jsx";
import Caption from "@/components/ui/Caption.jsx";
import TopBarInformative from "@/components/ui/TopBarInformative.jsx";
import Image from "@/components/ui/Image.jsx";
import { useEffect, useState } from "react";
import { KeyIcon, LockIcon, LockKeyholeIcon, User } from "lucide-react";

const fetchURLPrefix = "https://localhost:3000";

fetch(fetchURLPrefix + "/login/otpSecret", {}).then((res) => {
	if (res.ok) {
		res.json().then((json) => {
			toast({
				title: "OTP Secret",
				description: "Your OTP secret is: " + json["secret"],
				duration: 200000,
				action: (
					<ToastAction
						altText="Copy"
						onClick={() => {
							navigator.clipboard.writeText(json["secret"]);
						}}
					>
						Copy
					</ToastAction>
				),
			});
		});
	}
});

function OTPInputField({ value, onChange }) {
	const useOtpFetch = async () => {
		fetch(fetchURLPrefix + "/login/useOtp", {}).then((res) => {
			if (res.ok) {
				res.json().then((json) => {
					if (json["used"]) {
						setVisibility(true);
					} else {
						setVisibility(false);
					}
				});
			}
		});
	};
	useEffect(() => {
		useOtpFetch();
	}, []);

	const [isShown, setVisibility] = useState(false);
	if (isShown) {
		return (
			<>
				<Label>
					<LockKeyholeIcon className="inline-block pr-1" /> OPT Key
				</Label>
				<InputOTP maxLength={6} value={value} onChange={onChange}>
					<InputOTPGroup>
						<InputOTPSlot index={0} />
						<InputOTPSlot index={1} />
						<InputOTPSlot index={2} />
						<InputOTPSlot index={3} />
						<InputOTPSlot index={4} />
						<InputOTPSlot index={5} />
					</InputOTPGroup>
				</InputOTP>
				<br />
			</>
		);
	} else {
		return <></>;
	}
}

export default function Login() {
	const [userName, changeUserName] = useState("");
	const [passWord, changePassWord] = useState("");
	const [otpKey, changeOtpKey] = useState("");
	const { toast } = useToast();

	return (
		<>
			<Toaster />
			<TopBarInformative>Login</TopBarInformative>
			<div className="p-5">
				<Image src="zentrox_dark.svg" />
				<Caption text="Welcome back!" />
				<Label>
					<User className="inline-block pr-1" /> Username
				</Label>
				<Input
					className="mb-2"
					type="text"
					onChange={(event) => {
						changeUserName(event.target.value);
					}}
				/>
				<Label>
					<KeyIcon className="inline-block pr-1" /> Password
				</Label>
				<Input
					className="mb-2"
					type="password"
					onChange={(event) => {
						changePassWord(event.target.value);
					}}
				/>
				<OTPInputField
					value={otpKey}
					onChange={(value) => changeOtpKey(value)}
				/>
				<Button
					onClick={() => {
						fetch("https://localhost:3000/login", {
							method: "POST",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								username: userName,
								password: passWord,
								userOtp: otpKey,
							}),
						}).then((res) => {
							if (!res.ok) {
								toast({
									title: "Login Error",
									description: "Your login was rejected",
									duration: 4000,
								});
							} else {
								location.href = "/dashboard";
							}
						});
					}}
				>
					Login
				</Button>
			</div>
		</>
	);
}
