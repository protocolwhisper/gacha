import type { Metadata } from "next";
import { headers } from "next/headers";
import "./globals.css";

const title = "Gacha Lab — Interactive Probability Explorer";
const description = "Explore a live Rust-powered gacha model with soft pity, hard pity, and multiple jackpots.";

export async function generateMetadata(): Promise<Metadata> {
  const requestHeaders = await headers();
  const host = requestHeaders.get("host") ?? "localhost:3000";
  const protocol = requestHeaders.get("x-forwarded-proto") ?? (host.startsWith("localhost") ? "http" : "https");
  const image = `${protocol}://${host}/og.png`;

  return {
    title,
    description,
    openGraph: { title, description, images: [{ url: image, width: 1742, height: 907 }] },
    twitter: { card: "summary_large_image", title, description, images: [image] },
  };
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
