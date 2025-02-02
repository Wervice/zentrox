import { Open_Sans } from "next/font/google";
import "./globals.css";

const openSans = Open_Sans({
  weight: ["400", "700", "800"],
  subsets: ["latin"],
});

export const metadata = {
  title: "Zentrox",
  description: "Server admininstration with batteries included",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" type="image/x-icon" href="favicon.ico" />
      </head>
      <body className={openSans.className}>{children}</body>
    </html>
  );
}
