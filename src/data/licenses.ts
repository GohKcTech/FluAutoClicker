export interface LicenseItem {
    name?: string;
    license?: string;
    author?: string;
    link?: string;
    licenseLink?: string;
    licenseLinkAlt?: string;
    licenseLinkThird?: string;
    linkAlt?: string;
    linkArchived?: string;
    packageLink?: string;
    siteLink?: string;
    description?: string;
    flagSvg?: string;
}

export interface LicenseCategory {
    titleKey?: string;
    titleText?: string;
    items: LicenseItem[];
}

export const LICENSE_CATEGORIES: LicenseCategory[] = [
    {
        titleKey: "licenses.application",
        items: [
            {
                name: "FluAutoClicker",
                license: "MIT",
                author: "Agzes",
                description:
                    'Copyright \u00a9 2026 Agzes. All rights reserved to the extent of the MIT License terms.\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:\n\nThe above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\n\nTHE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.',
                link: "https://github.com/Agzes/FluAutoClicker",
                licenseLink:
                    "https://github.com/Agzes/FluAutoClicker/blob/next/LICENSE",
            },
        ],
    },
    {
        titleKey: "licenses.rust",
        items: [
            {
                name: "Tauri v2",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description:
                    "Build smaller, faster, and more secure desktop and mobile applications with a web frontend.",
                packageLink: "https://crates.io/crates/tauri",
                siteLink: "https://v2.tauri.app",
                link: "https://github.com/tauri-apps/tauri",
                licenseLink:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE-2.0",
            },
            {
                name: "tauri-plugin-opener",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Open URLs and files from the Tauri app.",
                packageLink: "https://crates.io/crates/tauri-plugin-opener",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/opener",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "tauri-plugin-global-shortcut",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Global hotkey registration for Tauri.",
                packageLink:
                    "https://crates.io/crates/tauri-plugin-global-shortcut",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/global-shortcut",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "tauri-plugin-single-instance",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Single instance enforcement for Tauri apps.",
                packageLink:
                    "https://crates.io/crates/tauri-plugin-single-instance",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/single-instance",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "tauri-plugin-shell",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Shell command execution from Tauri.",
                packageLink: "https://crates.io/crates/tauri-plugin-shell",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/shell",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "tauri-plugin-cli",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "CLI argument parsing for Tauri apps.",
                packageLink: "https://crates.io/crates/tauri-plugin-cli",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/cli",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "tauri-plugin-dialog",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Native dialog support for Tauri.",
                packageLink: "https://crates.io/crates/tauri-plugin-dialog",
                link: "https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/dialog",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "enigo",
                license: "MIT",
                author: "Dustin Bensing (pythoneer), pentamassiv",
                description:
                    "Cross-platform library to simulate keyboard and mouse events.",
                packageLink: "https://crates.io/crates/enigo",
                link: "https://github.com/enigo-rs/enigo",
                licenseLink:
                    "https://github.com/enigo-rs/enigo/blob/main/LICENSE",
            },
            {
                name: "rdev",
                license: "MIT",
                author: "Nicolas Patry (Narsil)",
                description:
                    "Listen to and send global keyboard and mouse events.",
                packageLink: "https://crates.io/crates/rdev",
                link: "https://github.com/Narsil/rdev",
                licenseLink: "https://github.com/Narsil/rdev/blob/main/LICENSE",
            },
            {
                name: "tokio",
                license: "MIT",
                author: "Carl Lerche (carllerche), Alice Ryhl (Darksonn) and Tokio Contributors",
                description:
                    "A runtime for writing reliable, asynchronous, and slim applications with the Rust programming language.",
                packageLink: "https://crates.io/crates/tokio",
                link: "https://github.com/tokio-rs/tokio",
                licenseLink:
                    "https://github.com/tokio-rs/tokio/blob/master/LICENSE",
            },
            {
                name: "once_cell",
                license: "MIT / Apache-2.0",
                author: "Alex Kladov (matklad)",
                description:
                    "Rust library for single assignment cells and lazy statics without macros.",
                packageLink: "https://crates.io/crates/once_cell",
                link: "https://github.com/matklad/once_cell",
                licenseLink:
                    "https://github.com/matklad/once_cell/blob/master/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/matklad/once_cell/blob/master/LICENSE-APACHE",
            },
            {
                name: "serde",
                license: "MIT / Apache-2.0",
                author: "Erick Tryzelaar (erickt), David Tolnay (dtolnay) and Serde Contributors",
                description:
                    "A generic serialization/deserialization framework.",
                packageLink: "https://crates.io/crates/serde",
                link: "https://github.com/serde-rs/serde",
                licenseLink:
                    "https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE",
                licenseLinkAlt:
                    "https://github.com/serde-rs/serde/blob/master/LICENSE-MIT",
            },
            {
                name: "serde_json",
                license: "MIT / Apache-2.0",
                author: "Erick Tryzelaar (erickt), David Tolnay (dtolnay) and Serde Contributors",
                description: "JSON serialization for serde.",
                packageLink: "https://crates.io/crates/serde_json",
                link: "https://github.com/serde-rs/json",
                licenseLink:
                    "https://github.com/serde-rs/json/blob/master/LICENSE-APACHE",
                licenseLinkAlt:
                    "https://github.com/serde-rs/json/blob/master/LICENSE-MIT",
            },
            {
                name: "base64",
                license: "MIT / Apache-2.0",
                author: "Alice Maz (alicemaz), Marshall Pierce (marshallpierce)",
                description: "Encodes and decodes base64 as bytes or utf8.",
                packageLink: "https://crates.io/crates/base64",
                link: "https://github.com/marshallpierce/rust-base64",
                licenseLink:
                    "https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-APACHE",
            },
            {
                name: "rand",
                license: "MIT / Apache-2.0",
                author: "Diggory Hardy (dhardy), Paul Dicker (pitdicker), vks and Rand Contributors",
                description:
                    "Random number generators and other randomness functionality.",
                packageLink: "https://crates.io/crates/rand",
                link: "https://github.com/rust-random/rand",
                licenseLink:
                    "https://github.com/rust-random/rand/blob/master/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/rust-random/rand/blob/master/LICENSE-APACHE",
            },
            {
                name: "dirs",
                license: "MIT / Apache-2.0",
                author: "Simon Ochsenreither (soc)",
                description: "Platform-specific standard directory paths.",
                packageLink: "https://crates.io/crates/dirs",
                link: "https://codeberg.org/dirs/dirs-rs",
                linkArchived: "https://github.com/dirs-dev/dirs-rs",
                licenseLink:
                    "https://codeberg.org/dirs/dirs-rs/src/branch/main/LICENSE-MIT",
                licenseLinkAlt:
                    "https://codeberg.org/dirs/dirs-rs/src/branch/main/LICENSE-APACHE",
            },
            {
                name: "raw-window-handle",
                license: "MIT / Apache-2.0 / zlib",
                author: "Osspial, Lokathor, Kirill Chibisov (kchibisov) and raw-window-handle Contributors",
                description: "Interoperable window handle types.",
                packageLink: "https://crates.io/crates/raw-window-handle",
                link: "https://github.com/rust-windowing/raw-window-handle",
                licenseLink:
                    "https://github.com/rust-windowing/raw-window-handle/blob/master/LICENSE-MIT.md",
                licenseLinkAlt:
                    "https://github.com/rust-windowing/raw-window-handle/blob/master/LICENSE-APACHE.md",
                licenseLinkThird:
                    "https://github.com/rust-windowing/raw-window-handle/blob/master/LICENSE-ZLIB.md",
            },
            {
                name: "tauri-build",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Build script helpers for Tauri applications.",
                packageLink: "https://crates.io/crates/tauri-build",
                link: "https://github.com/tauri-apps/tauri",
                licenseLink:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE-2.0",
            },
            {
                name: "window-vibrancy",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description:
                    "Windows-only: Acrylic and blur effects for windows.",
                packageLink: "https://crates.io/crates/window-vibrancy",
                link: "https://github.com/tauri-apps/window-vibrancy/",
                licenseLink:
                    "https://github.com/tauri-apps/window-vibrancy/blob/dev/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/window-vibrancy/blob/dev/LICENSE-APACHE",
            },
            {
                name: "winreg",
                license: "MIT",
                author: "Igor Shaula (gentoo90)",
                description: "Windows-only: Windows Registry API bindings.",
                packageLink: "https://crates.io/crates/winreg",
                link: "https://github.com/gentoo90/winreg-rs",
                licenseLink:
                    "https://github.com/gentoo90/winreg-rs/blob/master/LICENSE",
            },
            {
                name: "input-linux",
                license: "MIT",
                author: "arcnmx",
                description: "Linux-only: uinput device interface bindings.",
                packageLink: "https://crates.io/crates/input-linux",
                link: "https://github.com/arcnmx/input-linux-rs",
                licenseLink:
                    "https://github.com/arcnmx/input-linux-rs/blob/main/COPYING",
            },
            {
                name: "input-linux-sys",
                license: "MIT",
                author: "arcnmx, meh",
                description: "Linux-only: Raw FFI bindings for uinput.",
                packageLink: "https://crates.io/crates/input-linux-sys",
                link: "https://github.com/arcnmx/input-linux-sys-rs",
                licenseLink:
                    "https://github.com/arcnmx/input-linux-sys-rs/blob/main/COPYING",
            },
            {
                name: "evdev",
                license: "MIT / Apache-2.0",
                author: "emberian, Noa (coolreader18)",
                description: "Linux-only: Evdev input subsystem bindings.",
                packageLink: "https://crates.io/crates/evdev",
                link: "https://github.com/emberian/evdev",
                licenseLink:
                    "https://github.com/emberian/evdev/blob/main/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/emberian/evdev/blob/main/LICENSE-APACHE",
            },
        ],
    },
    {
        titleKey: "licenses.npm",
        items: [
            {
                name: "@tauri-apps/api",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description:
                    "Tauri JavaScript API bindings for frontend-backend IPC.",
                packageLink: "https://www.npmjs.com/package/@tauri-apps/api",
                link: "https://github.com/tauri-apps/tauri",
                licenseLink:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE-2.0",
            },
            {
                name: "@tauri-apps/plugin-dialog",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Native dialog API for Tauri on the frontend.",
                packageLink:
                    "https://www.npmjs.com/package/@tauri-apps/plugin-dialog",
                link: "https://github.com/tauri-apps/plugins-workspace",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "@tauri-apps/plugin-opener",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description: "Open URLs and files from the frontend.",
                packageLink:
                    "https://www.npmjs.com/package/@tauri-apps/plugin-opener",
                link: "https://github.com/tauri-apps/plugins-workspace",
                licenseLink:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/plugins-workspace/blob/v2/LICENSE_APACHE-2.0",
            },
            {
                name: "@tauri-apps/cli",
                license: "MIT / Apache-2.0",
                author: "Tauri Programme within The Commons Conservancy",
                description:
                    "Tauri CLI for building and bundling applications.",
                packageLink: "https://www.npmjs.com/package/@tauri-apps/cli",
                link: "https://github.com/tauri-apps/tauri",
                licenseLink:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT",
                licenseLinkAlt:
                    "https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE-2.0",
            },
            {
                name: "TypeScript",
                license: "Apache-2.0",
                author: "Microsoft",
                description:
                    "TypeScript is a language for application-scale JavaScript development.",
                packageLink: "https://www.npmjs.com/package/typescript",
                link: "https://github.com/microsoft/TypeScript",
                licenseLink:
                    "https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt",
            },
            {
                name: "Vite",
                license: "MIT",
                author: "Evan You, Vite Contributors",
                description:
                    "Next generation frontend tooling. Lightning-fast HMR and optimized builds.",
                packageLink: "https://www.npmjs.com/package/vite",
                link: "https://github.com/vitejs/vite",
                licenseLink: "https://github.com/vitejs/vite/blob/main/LICENSE",
            },
        ],
    },
    {
        titleKey: "licenses.builtin_fonts",
        items: [
            {
                name: "MuseoModerno",
                license: "SIL OFL-1.1",
                author: "Omnibus-Type",
                description:
                    "Copyright 2020 The MuseoModerno Project Authors. Geometric display font.",
                link: "https://fonts.google.com/specimen/MuseoModerno",
                licenseLink:
                    "https://fonts.google.com/specimen/MuseoModerno/license",
            },
            {
                name: "Geologica",
                license: "SIL OFL-1.1",
                author: "Monokrom, Sindre Bremnes, Frode Helland",
                description:
                    "Copyright 2020 The Geologisk Project Authors. Sans-serif font.",
                link: "https://fonts.google.com/specimen/Geologica",
                licenseLink:
                    "https://fonts.google.com/specimen/Geologica/license",
            },
            {
                name: "Montserrat",
                license: "SIL OFL-1.1",
                author: "Julieta Ulanovsky",
                description:
                    "Copyright 2024 The Montserrat.Git Project Authors. Sans-serif font.",
                link: "https://fonts.google.com/specimen/Montserrat",
                licenseLink:
                    "https://fonts.google.com/specimen/Montserrat/license",
            },
        ],
    },
    {
        titleKey: "licenses.google_fonts",
        items: [
            {
                name: "Google Fonts",
                license: "SIL OFL-1.1",
                description:
                    "All additional fonts (Inter, Open Sans, Noto Sans, Lato, etc.) are hosted by Google Fonts under the SIL Open Font License 1.1.",
                link: "https://fonts.google.com",
                licenseLink: "https://openfontlicense.org",
            },
        ],
    },
    {
        titleKey: "licenses.icons",
        items: [
            {
                name: "Lucide Icons",
                license: "ISC License",
                author: "Lucide Contributors",
                description: "Open-source icon set used throughout the UI.",
                link: "https://github.com/lucide-icons/lucide",
                licenseLink:
                    "https://github.com/lucide-icons/lucide/blob/main/LICENSE",
            },
            {
                name: "lipis/flag-icons",
                license: "MIT",
                author: "lipis and lipis/flah-icons Contributors",
                description:
                    "Country flag SVGs used in language picker and translations section.",
                link: "https://github.com/lipis/flag-icons",
                licenseLink:
                    "https://github.com/lipis/flag-icons/blob/main/LICENSE",
            },
        ],
    },
];
