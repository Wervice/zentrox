import { GeistSans } from "geist/font/sans";
import "./globals.css";

export const metadata = {
  title: "Zentrox",
  description: "Server admininstration with batteries included",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" type="image/x-icon" href="zentrox_dark_256.png" />
      </head>
      <body className={GeistSans.className}>{children}</body>
    </html>
  );
}
