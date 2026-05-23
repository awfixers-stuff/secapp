import Link from "next/link";
import { Shield } from "lucide-react";

export function SiteFooter() {
  return (
    <footer className="border-t bg-muted/40">
      <div className="mx-auto max-w-6xl px-4 py-8">
        <div className="grid gap-8 md:grid-cols-3">
          <div>
            <div className="flex items-center gap-2 font-semibold">
              <Shield className="h-5 w-5" />
              <span>secapp</span>
            </div>
            <p className="mt-2 text-sm text-muted-foreground">
              Linux system keystore protecting sensitive directories against
              ransomware.
            </p>
          </div>

          <div>
            <h3 className="text-sm font-semibold">Resources</h3>
            <ul className="mt-2 space-y-1 text-sm text-muted-foreground">
              <li>
                <Link href="/docs" className="hover:text-foreground transition-colors">
                  Documentation
                </Link>
              </li>
              <li>
                <Link href="/info/roadmap" className="hover:text-foreground transition-colors">
                  Roadmap
                </Link>
              </li>
              <li>
                <Link href="/info/ideas" className="hover:text-foreground transition-colors">
                  Ideas
                </Link>
              </li>
            </ul>
          </div>

          <div>
            <h3 className="text-sm font-semibold">Legal</h3>
            <ul className="mt-2 space-y-1 text-sm text-muted-foreground">
              <li>
                <Link href="/legal/license" className="hover:text-foreground transition-colors">
                  License
                </Link>
              </li>
              <li>
                <Link href="/legal/privacy" className="hover:text-foreground transition-colors">
                  Privacy Policy
                </Link>
              </li>
              <li>
                <Link href="/legal/terms" className="hover:text-foreground transition-colors">
                  Terms of Service
                </Link>
              </li>
              <li>
                <Link href="/legal/agreements" className="hover:text-foreground transition-colors">
                  Agreements
                </Link>
              </li>
              <li>
                <Link href="/legal/subprocessors" className="hover:text-foreground transition-colors">
                  Subprocessors
                </Link>
              </li>
            </ul>
          </div>
        </div>

        <div className="mt-8 border-t pt-4 text-center text-xs text-muted-foreground">
          &copy; {new Date().getFullYear()} AWFixer. Source Available under the{" "}
          <Link href="/legal/license" className="underline hover:text-foreground transition-colors">
            AWFixer Source Available License v0.4
          </Link>
          .
        </div>
      </div>
    </footer>
  );
}