import { useReducer } from "react";

type SettingsState = {
  isClearingCovers: boolean;
  isUninstalling: boolean;
  updateAvailable: boolean;
};

type SettingsAction =
  | { type: "SET_IS_CLEARING_COVERS"; payload: boolean }
  | { type: "SET_IS_UNINSTALLING"; payload: boolean }
  | { type: "SET_UPDATE_AVAILABLE"; payload: boolean };

const initialState: SettingsState = {
  isClearingCovers: false,
  isUninstalling: false,
  updateAvailable: false,
};

function settingsReducer(state: SettingsState, action: SettingsAction): SettingsState {
  switch (action.type) {
    case "SET_IS_CLEARING_COVERS":
      return { ...state, isClearingCovers: action.payload };
    case "SET_IS_UNINSTALLING":
      return { ...state, isUninstalling: action.payload };
    case "SET_UPDATE_AVAILABLE":
      return { ...state, updateAvailable: action.payload };
    default:
      return state;
  }
}

export function useSettingsState() {
  const [state, dispatch] = useReducer(settingsReducer, initialState);

  return {
    ...state,
    setIsClearingCovers: (value: boolean) =>
      dispatch({ type: "SET_IS_CLEARING_COVERS", payload: value }),
    setIsUninstalling: (value: boolean) =>
      dispatch({ type: "SET_IS_UNINSTALLING", payload: value }),
    setUpdateAvailable: (value: boolean) =>
      dispatch({ type: "SET_UPDATE_AVAILABLE", payload: value }),
  };
}
