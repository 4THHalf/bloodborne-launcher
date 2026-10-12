# Credits and license

The launcher was developed in [bloodborne-mac](https://github.com/bloodborne-native/bloodborne-mac)
and extracted into this repository with its launcher history on 2026-10-12. It is licensed under
GPL-2.0-or-later; see [LICENSE](../LICENSE).

The runtime it launches is built on [bbport](https://github.com/deadinside28/bloodborne_pc) by
deadinside28 and [shadPS4](https://github.com/shadps4-emu/shadPS4). Their credits and the additional
runtime contributors remain in [bloodborne-runtime's credits](https://github.com/bloodborne-native/bloodborne-runtime/blob/main/docs/CREDITS.md).

The UI uses Tauri, React, shadcn/ui, Base UI, Tailwind CSS, TanStack Query, Zustand, i18next,
Sonner, lucide, and the Cinzel and Inter fonts. Dependency licenses remain with their packages.
No code from an external save editor has been adopted.

## Leari database architecture

The Drizzle-to-Tauri SQL adapter in `src/db/client.ts` is adapted from
[leari-mail/leari, commit 5876482](https://github.com/leari-mail/leari/tree/5876482)
by Pablo Souza (MIT). Its service/hook organization and shared SQLite access informed this
launcher design. The [original MIT notice](licenses/leari-MIT.txt) is retained here.

## shadcn dialogs

`src/components/ui/dialog.tsx` and `alert-dialog.tsx` were generated with shadcn CLI 4.21.4
using the project's base-nova registry. The Base UI component templates are from
[shadcn-ui/ui, reference 30a58d4](https://github.com/shadcn-ui/ui/tree/30a58d4498f270af1c1e981a7d78a15cf5740841/apps/v4/registry/bases/base/ui).
The [upstream MIT notice](licenses/shadcn-ui-MIT.txt) is retained here.

## Leari menu-bar and window handling

`src-tauri/src/tray.rs` adapts window restoration and Dock/taskbar handling from
[leari-mail/leari 5876482](https://github.com/leari-mail/leari/tree/5876482/src-tauri/src)
(MIT, Pablo Souza). Its tray/menu setup informed the game-driven lifecycle. The original MIT
notice is already retained in `docs/licenses/leari-MIT.txt`. No Leari icon or other artwork was
copied; the hunter's sigil is original launcher artwork.
