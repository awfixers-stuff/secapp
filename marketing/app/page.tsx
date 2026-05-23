import Link from "next/link";
import { Shield, Lock, Eye, FileKey, Terminal, Cpu, ArrowRight } from "lucide-react";

export default function Page() {
  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="relative overflow-hidden border-b">
        <div className="absolute inset-0 bg-gradient-to-b from-muted/50 to-background" />
        <div className="relative mx-auto max-w-6xl px-4 py-20 md:py-28">
          <div className="mx-auto max-w-2xl text-center">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border bg-muted/50 px-4 py-1.5 text-sm">
              <Shield className="h-4 w-4" />
              Linux System Keystore
            </div>
            <h1 className="text-4xl font-bold tracking-tight md:text-5xl lg:text-6xl">
              Protect your sensitive files{" "}
              <span className="text-muted-foreground">from ransomware</span>
            </h1>
            <p className="mt-4 text-lg text-muted-foreground md:text-xl">
              secapp sits between your processes and the files in{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-sm font-mono">
                ~/.ssh/
              </code>
              ,{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-sm font-mono">
                ~/.config/
              </code>
              ,{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-sm font-mono">
                ~/.gnupg/
              </code>
              , encrypting them at rest and serving plaintext only to authorized
              processes.
            </p>
            <div className="mt-8 flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
              <Link
                href="/docs/getting-started"
                className="inline-flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
              >
                Get Started
                <ArrowRight className="h-4 w-4" />
              </Link>
              <Link
                href="/docs"
                className="inline-flex items-center gap-2 rounded-lg border px-6 py-3 text-sm font-medium transition-colors hover:bg-muted"
              >
                Documentation
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* How it works */}
      <section className="border-b">
        <div className="mx-auto max-w-6xl px-4 py-16">
          <h2 className="text-center text-2xl font-bold tracking-tight md:text-3xl">
            How it works
          </h2>
          <div className="mt-10 grid gap-6 md:grid-cols-3">
            {[
              {
                icon: Eye,
                title: "Monitor",
                description:
                  "The daemon watches configured directories for file creation and modification using inotify.",
              },
              {
                icon: Lock,
                title: "Encrypt",
                description:
                  "When a file is written, secapp encrypts it with a per-file AES-256-GCM key, replacing plaintext on disk.",
              },
              {
                icon: FileKey,
                title: "Control",
                description:
                  "Policy rules (allow/deny/log) match on path globs and process names. Future versions add per-process allowlisting via FUSE.",
              },
            ].map(({ icon: Icon, title, description }) => (
              <div
                key={title}
                className="rounded-lg border bg-card p-6 text-card-foreground"
              >
                <Icon className="h-8 w-8 mb-3 text-foreground" />
                <h3 className="text-lg font-semibold">{title}</h3>
                <p className="mt-2 text-sm text-muted-foreground">{description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="border-b">
        <div className="mx-auto max-w-6xl px-4 py-16">
          <h2 className="text-center text-2xl font-bold tracking-tight md:text-3xl">
            Built for security, from the ground up
          </h2>
          <div className="mt-10 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {[
              {
                title: "AES-256-GCM encryption",
                description:
                  "Per-file random keys, wrapped under a master key derived with Argon2id (OWASP-recommended parameters). No key reuse.",
              },
              {
                title: "Kernel keyring integration",
                description:
                  "Master key cached in the Linux kernel session keyring via keyutils. Survives re-authentication without re-entering your password.",
              },
              {
                title: "Audit logging",
                description:
                  "Every encryption, decryption, and policy decision is recorded in the local database. Review via TUI or CLI.",
              },
              {
                title: "Pure Rust",
                description:
                  "All components — daemon, TUI, CLI, and the planned GUI and tray — are written in Rust. No Go, no Python, no shell scripts for core logic.",
              },
              {
                title: "Local-first, no telemetry",
                description:
                  "All data stays on your machine. No analytics, no crash reports, no server calls. Cloud sync (when available) is end-to-end encrypted and opt-in.",
              },
              {
                title: "NixOS ready",
                description:
                  "Designed for NixOS deployment via a Nix flake that replaces gnome-keyring and activates with proper permissions.",
              },
            ].map(({ title, description }) => (
              <div
                key={title}
                className="rounded-lg border bg-card p-6 text-card-foreground"
              >
                <h3 className="text-lg font-semibold">{title}</h3>
                <p className="mt-2 text-sm text-muted-foreground">{description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Component architecture */}
      <section className="border-b">
        <div className="mx-auto max-w-6xl px-4 py-16">
          <h2 className="text-center text-2xl font-bold tracking-tight md:text-3xl">
            Component architecture
          </h2>
          <div className="mt-10 overflow-x-auto">
            <table className="mx-auto w-full max-w-2xl border-collapse text-sm">
              <thead>
                <tr className="border-b bg-muted">
                  <th className="px-4 py-3 text-left font-semibold">Component</th>
                  <th className="px-4 py-3 text-left font-semibold">Binary</th>
                  <th className="px-4 py-3 text-left font-semibold">Status</th>
                  <th className="px-4 py-3 text-left font-semibold">Description</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ["System Daemon", "secapp daemon", "v1", "Privileged filesystem monitor and encryption engine"],
                  ["TUI", "secapp start", "v1", "ratatui terminal interface for status, logs, policies"],
                  ["CLI", "secapp <subcommand>", "v1", "clap-based CLI for scripted and headless operation"],
                  ["GUI", "secapp-gui", "Planned", "gtk-rs native app with dual authentication"],
                  ["System Tray", "secapp-tray", "Planned", "GNOME/KDE tray icon with lock/unlock and status"],
                ].map(([comp, binary, status, desc]) => (
                  <tr key={binary} className="border-b">
                    <td className="px-4 py-3 font-medium">{comp}</td>
                    <td className="px-4 py-3 font-mono text-xs">{binary}</td>
                    <td className="px-4 py-3">
                      <span
                        className={
                          status === "v1"
                            ? "rounded bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 px-2 py-0.5 text-xs font-medium"
                            : "rounded bg-muted text-muted-foreground px-2 py-0.5 text-xs font-medium"
                        }
                      >
                        {status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-muted-foreground">{desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Quick start */}
      <section className="border-b">
        <div className="mx-auto max-w-6xl px-4 py-16">
          <h2 className="text-center text-2xl font-bold tracking-tight md:text-3xl">
            Get started in seconds
          </h2>
          <div className="mt-8 rounded-lg border bg-muted/50 p-6">
            <pre className="overflow-x-auto text-sm leading-6">
              <code>{`# Build from source
cargo build --release

# Initialize configuration
secapp init

# Start the daemon with TUI
secapp start

# Or start headless
secapp daemon

# Add a custom protected directory
secapp add-path ~/projects/secrets

# Lock the keystore before walking away
secapp lock`}</code>
            </pre>
          </div>
          <div className="mt-6 text-center">
            <Link
              href="/docs/getting-started"
              className="inline-flex items-center gap-2 text-sm font-medium hover:text-foreground/80 transition-colors"
            >
              Read the full getting started guide
              <ArrowRight className="h-4 w-4" />
            </Link>
          </div>
        </div>
      </section>

      {/* Security model */}
      <section>
        <div className="mx-auto max-w-6xl px-4 py-16">
          <h2 className="text-center text-2xl font-bold tracking-tight md:text-3xl">
            Security model
          </h2>
          <div className="mx-auto mt-8 max-w-2xl">
            <div className="rounded-lg border p-6">
              <h3 className="text-lg font-semibold">What secapp protects against</h3>
              <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
                <li className="flex gap-2">
                  <span className="text-green-600 dark:text-green-400">✓</span>
                  Offline access to sensitive files (stolen disk, ransomware exfiltration)
                </li>
                <li className="flex gap-2">
                  <span className="text-green-600 dark:text-green-400">✓</span>
                  Unauthorized process access (ransomware encrypting protected directories)
                </li>
                <li className="flex gap-2">
                  <span className="text-green-600 dark:text-green-400">✓</span>
                  Plaintext file leakage (encrypted blobs replace originals)
                </li>
              </ul>

              <h3 className="mt-6 text-lg font-semibold">What secapp does not protect against</h3>
              <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
                <li className="flex gap-2">
                  <span className="text-red-600 dark:text-red-400">✗</span>
                  Compromised kernel / root-level attackers
                </li>
                <li className="flex gap-2">
                  <span className="text-red-600 dark:text-red-400">✗</span>
                  Physical access while keystore is unlocked
                </li>
                <li className="flex gap-2">
                  <span className="text-red-600 dark:text-red-400">✗</span>
                  Lost master password (v1 has no recovery mechanism)
                </li>
              </ul>
            </div>

            <div className="mt-6 rounded-lg border border-amber-500/30 bg-amber-500/5 p-4 text-sm">
              <p className="font-medium text-amber-700 dark:text-amber-400">
                Important: No recovery in v1
              </p>
              <p className="mt-1 text-muted-foreground">
                If you lose your master password, your encrypted files cannot be recovered.
                Recovery keys are planned for v3. Always keep a secure backup of your password.
              </p>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}