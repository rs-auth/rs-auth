import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { ThemeProvider } from "next-themes";
import "./globals.css";

const geistSans = Geist({
  subsets: ["latin"],
  variable: "--font-geist-sans",
});

const geistMono = Geist_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://rs-auth.com"),
  title: {
    default: "rs-auth",
    template: "%s | rs-auth",
  },
  description: "Composable authentication for Rust with Axum and Postgres.",
  icons: {
    icon: "/favicon.svg",
  },
  openGraph: {
    type: "website",
    url: "https://rs-auth.com",
    siteName: "rs-auth",
    title: "rs-auth",
    description: "Composable authentication for Rust with Axum and Postgres.",
  },
  twitter: {
    card: "summary",
    title: "rs-auth",
    description: "Composable authentication for Rust with Axum and Postgres.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
