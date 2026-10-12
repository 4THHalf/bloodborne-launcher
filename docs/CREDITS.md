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
