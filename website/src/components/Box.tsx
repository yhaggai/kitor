import { GameBox, type GameBoxProps } from "../generated/GameBox";
/** Keep the desktop's box renderer, rotation, and accessible dialog shared. */
export default function Box(props: Omit<GameBoxProps, "showLanguageControls">) {
  return (
    <GameBox
      defaultLanguage={props.locale ?? "he"}
      locale="he"
      {...props}
      showLanguageControls={false}
      className={`site-box ${props.className ?? ""}`}
    />
  );
}
