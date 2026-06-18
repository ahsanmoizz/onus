import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Onus Console — AI Agent Firewall",
  description: "Onus product frontend: approval management, audit trail, and system status",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-zinc-950 text-zinc-100">
        <header className="border-b border-zinc-800 px-6 py-3 flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="text-xl font-bold text-orange-400">Onus</span>
            <span className="text-sm text-zinc-500">Console</span>
          </div>
          <nav className="ml-auto flex gap-4 text-sm">
            <a href="/" className="text-zinc-400 hover:text-zinc-100 transition-colors">Dashboard</a>
            <a href="/approvals" className="text-zinc-400 hover:text-zinc-100 transition-colors">Approvals</a>
            <a href="/audit" className="text-zinc-400 hover:text-zinc-100 transition-colors">Audit</a>
            <a href="/status" className="text-zinc-400 hover:text-zinc-100 transition-colors">Status</a>
          </nav>
        </header>
        <main className="flex-1">{children}</main>
      </body>
    </html>
  );
}
