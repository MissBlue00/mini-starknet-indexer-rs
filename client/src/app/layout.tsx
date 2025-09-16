import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { ApolloWrapper } from './apollo-wrapper';
import { StarknetProvider } from './starknet-provider';

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Starknet Indexer - Contract Deployments",
  description: "Monitor and analyze Starknet smart contract deployments and events with real-time indexing and GraphQL API",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <StarknetProvider>
          <ApolloWrapper>
            {children}
          </ApolloWrapper>
        </StarknetProvider>
      </body>
    </html>
  );
}
