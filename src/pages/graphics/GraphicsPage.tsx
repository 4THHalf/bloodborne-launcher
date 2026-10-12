import { useTranslation } from "react-i18next";
import { NumberRow, PageHeader, SelectRow, SettingsSection, SliderRow, SwitchRow } from "@/components/settings";
import { useIni, useSetting, useSettingsData } from "@/hooks";
import { EFFECTS, FPS_MODES, MODEL_LOD, OUTPUT_RES, PRESENT_MODES, PRESETS, UPSCALERS, renderSize } from "@/lib/data";
import type { Option } from "@/lib";
import { DisplayPicker } from "@/pages/graphics/DisplayPicker";

export function GraphicsPage() {
  const { t } = useTranslation();
  const { settings, ini } = useSettingsData();
  const setIni = useIni();
  const setSetting = useSetting();
  const flag = (key: string, fallback: boolean) => (key in ini ? ini[key] === "1" : fallback);
  const upscaler = ini.upscaler ?? "fsr3";
  const output = ini.output_res ?? "1920x1080";
  const preset = Number(ini.preset ?? 2);
  const fsrOff = upscaler === "off";
  const presets: Option<number>[] = PRESETS.map((p) => ({
    value: p.value,
    label: t("graphics.presetRender", { name: t(p.label), size: renderSize(output, p.value, upscaler) }),
    literal: true,
  }));
  return (
    <>
      <PageHeader title={t("graphics.title")} subtitle={t("graphics.subtitle")} />
      <SettingsSection title={t("graphics.display")}>
        <SelectRow
          title={t("graphics.outputRes")}
          description={`${t("graphics.outputResHint")} ${t("common.restartNote")}`}
          value={output}
          options={OUTPUT_RES}
          onChange={(v) => setIni("output_res", v)}
        />
        <DisplayPicker />
        <SwitchRow title={t("graphics.fullscreen")} checked={settings.fullscreen} onChange={(v) => setSetting("fullscreen", v)} />
        <SelectRow
          title={t("graphics.presentMode")}
          value={settings.present_mode}
          options={PRESENT_MODES}
          onChange={(v) => setSetting("present_mode", v)}
        />
      </SettingsSection>
      <SettingsSection title={t("graphics.frameRate")}>
        <SelectRow title={t("graphics.fpsMode")} value={settings.fps_mode} options={FPS_MODES} onChange={(v) => setSetting("fps_mode", v)} />
        <NumberRow title={t("graphics.fpsLimit")} description={t("graphics.fpsLimitHint")}
          value={settings.fps_limit ?? 0} min={0} max={480} onChange={(v) => setSetting("fps_limit", v)} />
        <SwitchRow title={t("graphics.showFps")} checked={ini.show_fps !== "0"} onChange={(v) => setIni("show_fps", v ? "1" : "0")} />
      </SettingsSection>
      <SettingsSection title={t("graphics.upscaling")} description={t("graphics.upscalingHint")}>
        <SelectRow title={t("graphics.upscaler")} value={upscaler} options={UPSCALERS} onChange={(v) => setIni("upscaler", v)} />
        <SelectRow
          title={t("graphics.preset")}
          description={t("common.restartNote")}
          value={preset}
          options={presets}
          onChange={(v) => setIni("preset", String(v))}
          disabled={fsrOff}
        />
        <SwitchRow
          title={t("graphics.sharpen")}
          checked={ini.sharpen !== "0"}
          onChange={(v) => setIni("sharpen", v ? "1" : "0")}
          disabled={fsrOff}
        />
        <SliderRow
          title={t("graphics.sharpness")}
          value={Number(ini.sharpness ?? 0.5)}
          min={0}
          max={2}
          step={0.05}
          format={(v) => v.toFixed(2)}
          onChange={(v) => setIni("sharpness", v.toFixed(2))}
          disabled={fsrOff || ini.sharpen === "0"}
        />
        <SwitchRow
          title={t("graphics.objectMotion")}
          description={t("graphics.objectMotionHint")}
          checked={ini.object_motion === "1"}
          onChange={(v) => setIni("object_motion", v ? "1" : "0")}
          disabled={fsrOff}
        />
      </SettingsSection>
      <SettingsSection title={t("graphics.detail")}>
        <SelectRow
          title={t("graphics.modelLod")}
          description={t("common.restartNote")}
          value={ini.model_lod ?? "0"}
          options={MODEL_LOD}
          onChange={(v) => setIni("model_lod", v)}
        />
      </SettingsSection>
      <SettingsSection title={t("graphics.effects")} description={`${t("graphics.effectsHint")} ${t("common.restartNote")}`}>
        {EFFECTS.map((effect) => (
          <SwitchRow key={effect.key} title={t(effect.label)} description={effect.hint && t(effect.hint)}
            checked={flag(effect.key, effect.defaultOn)} onChange={(v) => setIni(effect.key, v ? "1" : "0")} />
        ))}
      </SettingsSection>
    </>
  );
}
