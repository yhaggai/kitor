import { GameBox, type GameBoxProps } from "../generated/GameBox";
/** Keep the desktop's box renderer, rotation, and accessible dialog shared. */
export default function Box(props: GameBoxProps) {
  return (
    <GameBox
      defaultLanguage={props.locale ?? "he"}
      locale="he"
      {...props}
      className={`site-box ${props.className ?? ""}`}
    />
  );
}
