import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "../styles/globals.css";
import ClientOnly from "@/components/ClientOnly";
import SolanaWalletProvider from "@/components/WalletProvider";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Eggs Example App",
  description: "A simple dApp for interacting with the Eggs Solana program",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <ClientOnly>
          <SolanaWalletProvider>
            {children}
          </SolanaWalletProvider>
        </ClientOnly>
      </body>
    </html>
  );
} 