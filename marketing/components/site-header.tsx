"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { Shield, Menu, X } from "lucide-react";
import { cn } from "@/lib/utils";

const navLinks = [
  { href: "/", label: "Home" },
  { href: "/docs", label: "Docs" },
  { href: "/info/roadmap", label: "Roadmap" },
  { href: "/info/ideas", label: "Ideas" },
];

const legalLinks = [
  { href: "/legal/license", label: "License" },
  { href: "/legal/privacy", label: "Privacy" },
  { href: "/legal/terms", label: "Terms" },
  { href: "/legal/agreements", label: "Agreements" },
  { href: "/legal/subprocessors", label: "Subprocessors" },
];

export function SiteHeader() {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-6xl items-center px-4">
        <Link href="/" className="flex items-center gap-2 font-semibold">
          <Shield className="h-5 w-5" />
          <span>secapp</span>
        </Link>

        {/* Desktop nav */}
        <nav className="ml-8 hidden items-center gap-6 text-sm md:flex">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={cn(
                "transition-colors hover:text-foreground/80",
                pathname === link.href
                  ? "text-foreground font-medium"
                  : "text-foreground/60"
              )}
            >
              {link.label}
            </Link>
          ))}

          <div className="group relative">
            <button
              className={cn(
                "transition-colors hover:text-foreground/80",
                pathname.startsWith("/legal")
                  ? "text-foreground font-medium"
                  : "text-foreground/60"
              )}
            >
              Legal ▾
            </button>
            <div className="invisible absolute left-0 top-full pt-2 group-hover:visible">
              <div className="rounded-md border bg-background p-2 shadow-md">
                {legalLinks.map((link) => (
                  <Link
                    key={link.href}
                    href={link.href}
                    className="block rounded-sm px-3 py-1.5 text-sm text-foreground/60 transition-colors hover:bg-muted hover:text-foreground"
                  >
                    {link.label}
                  </Link>
                ))}
              </div>
            </div>
          </div>
        </nav>

        {/* Mobile toggle */}
        <button
          className="ml-auto md:hidden"
          onClick={() => setMobileOpen(!mobileOpen)}
          aria-label="Toggle navigation"
        >
          {mobileOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
        </button>
      </div>

      {/* Mobile nav */}
      {mobileOpen && (
        <nav className="border-t bg-background px-4 py-4 md:hidden">
          <div className="flex flex-col gap-3 text-sm">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className={cn(
                  "transition-colors hover:text-foreground/80",
                  pathname === link.href
                    ? "text-foreground font-medium"
                    : "text-foreground/60"
                )}
              >
                {link.label}
              </Link>
            ))}
            <div className="my-2 border-t" />
            <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Legal
            </p>
            {legalLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className={cn(
                  "transition-colors hover:text-foreground/80",
                  pathname === link.href
                    ? "text-foreground font-medium"
                    : "text-foreground/60"
                )}
              >
                {link.label}
              </Link>
            ))}
          </div>
        </nav>
      )}
    </header>
  );
}