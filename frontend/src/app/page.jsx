"use client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input.jsx";
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp";
import { Toaster } from "@/components/ui/toaster";
import { useToast } from "@/components/ui/use-toast";

import Label from "@/components/ui/ShortLabel.jsx";
import Caption from "@/components/ui/Caption.jsx";
import Image from "@/components/ui/Image.jsx";
import { useEffect, useRef, useState } from "react";
import { KeyIcon, LockKeyholeIcon, User } from "lucide-react";
import { fetchURLPrefix } from "@/lib/fetchPrefix";

if (fetchURLPrefix.length > 0) {
  console.error("Fetch URL Prefix is enabled");
}

function OTPInputField({ value, onChange, hidden }) {
  if (!hidden) {
    return (
      <>
        <Label>
          <LockKeyholeIcon className="inline-block pr-1" /> OTP code
        </Label>
        <InputOTP maxLength={6} value={value} autoSubmit onChange={onChange}>
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
  const [otpKey, changeOtpKey] = useState("");
  const { toast } = useToast();
  const [otpVisible, setOtpVisible] = useState(null);
  const [formMoveOut, setFormMoveOut] = useState(false);
  const [formHidden, setFormHidden] = useState(false);
  let usernameInput = useRef();
  let passwordInput = useRef();
  function redirectDashboard() {
    setFormMoveOut(true);
    setTimeout(() => {
      setFormHidden(true);
      location.href = "/dashboard";
    }, 400);
  }

  function verifyLogin() {
    fetch(`${fetchURLPrefix}/login/verify`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username: usernameInput.current.value,
        password: passwordInput.current.value,
        userOtp: otpKey,
      }),
    })
      .catch(() => {
        toast({
          title: "Request failed",
          description:
            "Your login request failed due to an unknown error. Please try again in two minutes.",
        });
      })
      .then((res) => {
        if (res.ok) {
          redirectDashboard();
        } else {
          if (res.status === 403) {
            toast({
              title: "Wrong credentials",
              description: "Your login request has been rejected.",
            });
          } else if (res.status === 429) {
            toast({
              title: "You're rate-limited",
              description:
                "You have been rate-limited by sending more than 2 login requests per minute.",
            });
          } else {
            toast({
              title: "Unknown login error",
              description: `An unknown error occured during login (${res.status})`,
            });
          }
        }
      });
  }

  const useOtpFetch = async () => {
    fetch(fetchURLPrefix + "/api/useOtp", { method: "POST" })
      .catch(() => {
        toast({
          title: "Request failed",
          description:
            "Your login request failed due to an unknown error. Please try again in two minutes.",
        });
      })
      .then((res) => {
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
    usernameInput.current.click();
  }, []);

  return (
    <>
      <Toaster />
      <div
        className={
          "w-screen h-screen relative justify-center items-center" +
          (otpVisible === null || formHidden ? " hidden" : " flex")
        }
      >
        <div
          className={
            "p-5 rounded-lg bg-neutral-900/10 border border-neutral-700/30 duration-500 absolute transition-all mb-0 " +
            (formMoveOut ? " animate-fadeout mb-20" : " animate-fadeup")
          }
        >
          <Image src="zentrox_dark_emblem.svg" />
          <Caption text="Login" />
          <Label>
            <User className="inline-block" /> Username
          </Label>
          <Input className="mb-2 w-full" type="text" ref={usernameInput} />
          <Label>
            <KeyIcon className="inline-block" /> Password
          </Label>
          <Input
            className="mb-2 w-full"
            type="password"
            ref={passwordInput}
            onKeyPress={(event) => {
              if (event.key == "Enter") {
                verifyLogin();
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
              verifyLogin();
            }}
          >
            Login
          </Button>
        </div>
      </div>
    </>
  );
}
