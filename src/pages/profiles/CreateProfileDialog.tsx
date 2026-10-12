import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Input, Label, Dialog, DialogTrigger, DialogContent, DialogTitle, DialogDescription, DialogClose, DialogHeader, DialogFooter } from "@/components/ui";

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
    <Dialog open={open} onOpenChange={(next, details) => {
      if (!next && busy) { details.cancel(); return; }
      setOpen(next);
      if (!next) { setName(""); setError(undefined); }
    }}>
      <DialogTrigger render={<Button disabled={disabled} />}>{t("saves.newProfile")}</DialogTrigger>
      <DialogContent initialFocus={input} showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>{t("saves.newProfile")}</DialogTitle>
          <DialogDescription>{t("saves.fresh")}</DialogDescription>
        </DialogHeader>
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
          <DialogFooter>
            <DialogClose render={<Button variant="outline" disabled={busy} />}>{t("saves.cancel")}</DialogClose>
            <Button type="submit" disabled={disabled || !name.trim()}>{t("saves.create")}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
