import { useRef, useState } from "react";
import { Dialog } from "@base-ui/react/dialog";
import { useTranslation } from "react-i18next";
import { Button, Input, Label } from "@/components/ui";

export function CreateProfileDialog({ disabled, busy, onCreate }: {
  disabled: boolean;
  busy: boolean;
  onCreate: (name: string) => Promise<void>;
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [error, setError] = useState<string>();
  const input = useRef<HTMLInputElement>(null);

  return (
    <Dialog.Root open={open} onOpenChange={(next, details) => {
      if (!next && busy) { details.cancel(); return; }
      setOpen(next);
      if (!next) { setName(""); setError(undefined); }
    }}>
      <Dialog.Trigger render={<Button disabled={disabled} />}>{t("saves.newProfile")}</Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-50 bg-black/70" />
        <Dialog.Popup initialFocus={input} className="fixed left-1/2 top-1/2 z-50 w-[calc(100%_-_2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl border border-border bg-card p-6 shadow-xl">
          <Dialog.Title className="font-display text-xl">{t("saves.newProfile")}</Dialog.Title>
          <Dialog.Description className="mb-5 mt-2 text-sm text-muted-foreground">{t("saves.fresh")}</Dialog.Description>
          <form onSubmit={async (event) => {
            event.preventDefault();
            if (disabled || !name.trim()) return;
            setError(undefined);
            try {
              await onCreate(name.trim());
              setOpen(false);
              setName("");
            } catch (failure) { setError(String(failure)); }
          }}>
            <Label htmlFor="profile-name">{t("saves.name")}</Label>
            <Input ref={input} id="profile-name" className="mb-4 mt-2" maxLength={80} required value={name} disabled={disabled} onChange={(event) => setName(event.target.value)} />
            {error && <p role="alert" className="mb-4 break-words text-sm text-destructive">{error}</p>}
            <div className="flex justify-end gap-2">
              <Dialog.Close render={<Button variant="outline" disabled={busy} />}>{t("saves.cancel")}</Dialog.Close>
              <Button type="submit" disabled={disabled || !name.trim()}>{t("saves.create")}</Button>
            </div>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
