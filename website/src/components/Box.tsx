import { GameBox, type GameBoxProps } from "../generated/GameBox";
/** Keep the desktop's box renderer, rotation, and accessible dialog shared. */
export default function Box(props: GameBoxProps) {
  return (
    <GameBox
      {...props}
      defaultLanguage="he"
      locale="he"
      className={`site-box ${props.className ?? ""}`}
    />
  );
}
