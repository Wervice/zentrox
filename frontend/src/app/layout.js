import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

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
			<body className={inter.className}>{children}</body>
		</html>
	);
}
