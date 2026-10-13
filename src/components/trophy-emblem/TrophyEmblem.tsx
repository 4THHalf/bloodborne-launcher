import { useId } from "react";
import { Trophy } from "lucide-react";
import type { TrophyUnlock } from "@/lib";

const materials = {
  platinum: ["#647b83", "#f1fbff", "#a8c6cf", "#e1edf1", "#70878f"],
  gold: ["#956619", "#fff0a8", "#c89736", "#f7dd82", "#a87522"],
  silver: ["#62686e", "#fafafa", "#a3aaaf", "#e6e8e9", "#747b81"],
  bronze: ["#824a2d", "#f5c39d", "#af7049", "#deb08a", "#955c37"],
  unknown: ["#777777", "#dddddd", "#aaaaaa", "#cccccc", "#888888"],
} satisfies Record<TrophyUnlock["grade"], string[]>;

export function TrophyEmblem({ grade, size = 30 }: { grade: TrophyUnlock["grade"]; size?: number }) {
  const id = `trophy-metal-${useId().replaceAll(":", "")}`;
  return <Trophy size={size} stroke={`url(#${id})`} strokeWidth={1.8} aria-hidden="true">
    <defs><linearGradient id={id} x1="0" y1="0" x2="1" y2="1">
      {materials[grade].map((color, index) => <stop key={color} offset={`${index * 25}%`} stopColor={color} />)}
    </linearGradient></defs>
  </Trophy>;
}
