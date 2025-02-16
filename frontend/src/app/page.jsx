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
import { ClipboardIcon, KeyIcon, LockKeyholeIcon, User } from "lucide-react";

const fetchURLPrefix = require("@/lib/fetchPrefix");

if (fetchURLPrefix.length > 0) {
  console.error("Fetch URL Prefix is enabled");
}

function OTPInputField({ value, onChange, hidden }) {
  if (!hidden) {
    return (
      <>
        <Label>
          <LockKeyholeIcon className="inline-block pr-1" /> OPT Key
        </Label>
        <InputOTP
          maxLength={6}
          value={value}
          onChange={onChange}
          onKeyPress={(event) => {
            if (event.key == "Enter") {
              fetch(fetchURLPrefix + "/login", {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  username: username,
                  password: password,
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
            }
          }}
        >
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
  const [username, changeUsername] = useState("");
  const [password, changePassword] = useState("");
  const [otpKey, changeOtpKey] = useState("");
  const { toast } = useToast();
  const [otpVisible, setOtpVisible] = useState(null);
const useOtpFetch = async () => {
    fetch(fetchURLPrefix + "/login/useOtp", { method: "POST" }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          if (json["used"]) {
            setOtpVisible(true);
          } else {
            setOtpVisible(false);
          }
        });
      }
    });
  };
  useEffect(() => {
    useOtpFetch();
  }, []);

  useEffect(() => {
    fetch(fetchURLPrefix + "/login/otpSecret", {
      method: "POST",
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          toast({
            title: "OTP Secret",
            description: <>Your OTP secret is: <code>{json["secret"]}</code>
<ToastAction
                altText="Copy"
                onClick={() => {
                  navigator.clipboard.writeText(json["secret"]);
                }}
              >
                <ClipboardIcon className="w-4 h-4 inline-block mr-1 mt-[2px]" /> Copy
              </ToastAction>

			  </>,
            duration: 200000,
          });
        });
      }
    });
  }, []);

  return (
    <>
      <Toaster />
      <div className={"w-screen h-screen relative justify-center items-center" + (otpVisible === null ? " hidden" : " flex") }>
	  <div className="p-5 rounded-lg bg-neutral-900/10 border border-neutral-700/30 animate-fadeup duration-500 absolute">
        <Image src="zentrox_dark.svg" />
        <Caption text="Welcome" />
        <Label>
          <User className="inline-block pr-1" /> Username
        </Label>
        <Input
          className="mb-2"
          type="text"
          onChange={(event) => {
            changeUsername(event.target.value);
          }}
        />
        <Label>
          <KeyIcon className="inline-block pr-1" /> Password
        </Label>
        <Input
          className="mb-2"
          type="password"
          onChange={(event) => {
            changePassword(event.target.value);
          }}
          onKeyPress={(event) => {
            if (event.key == "Enter") {
              fetch(fetchURLPrefix + "/login", {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  username: username,
                  password: password,
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
                  const currentUrl = new URL(location.href);
                  let appParam = currentUrl.searchParams.get("app");
                  if (appParam === "true") {
                    localStorage.setItem("setup", "true");
                    location.href = "/alerts";
                  } else {
                    location.href = "/dashboard";
                  }
                }
              });
            }
          }}
        />
        <OTPInputField
          value={otpKey}
          onChange={(value) => changeOtpKey(value)}
	      hidden={!otpVisible}
        />
        <Button
			className={!otpVisible ? "mt-2" : ""}
          onClick={() => {
            fetch(fetchURLPrefix + "/login", {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                username: username,
                password: password,
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
                if (new URL(location.href).searchParams.get("app") !== "true") {
                  location.href = "/dashboard";
                } else {
                  location.href = "/alerts";
                }
              }
            });
          }}
        >
          Login
        </Button></div>
      </div>
    </>
  );
}
