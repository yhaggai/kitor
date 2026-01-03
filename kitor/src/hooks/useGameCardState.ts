import { useReducer } from "react";

type State = {
  isInstalling: boolean;
  showDeleteDialog: boolean;
  isDeleting: boolean;
  showRestoreDialog: boolean;
  isRestoring: boolean;
};

type Action =
  | { type: "START_INSTALL" }
  | { type: "FINISH_INSTALL" }
  | { type: "SHOW_DELETE_DIALOG" }
  | { type: "HIDE_DELETE_DIALOG" }
  | { type: "START_DELETE" }
  | { type: "FINISH_DELETE" }
  | { type: "SHOW_RESTORE_DIALOG" }
  | { type: "HIDE_RESTORE_DIALOG" }
  | { type: "START_RESTORE" }
  | { type: "FINISH_RESTORE" };

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case "START_INSTALL":
      return { ...state, isInstalling: true };
    case "FINISH_INSTALL":
      return { ...state, isInstalling: false };
    case "SHOW_DELETE_DIALOG":
      return { ...state, showDeleteDialog: true };
    case "HIDE_DELETE_DIALOG":
      return { ...state, showDeleteDialog: false };
    case "START_DELETE":
      return { ...state, isDeleting: true };
    case "FINISH_DELETE":
      return { ...state, isDeleting: false };
    case "SHOW_RESTORE_DIALOG":
      return { ...state, showRestoreDialog: true };
    case "HIDE_RESTORE_DIALOG":
      return { ...state, showRestoreDialog: false };
    case "START_RESTORE":
      return { ...state, isRestoring: true };
    case "FINISH_RESTORE":
      return { ...state, isRestoring: false };
    default:
      return state;
  }
};

const initialState: State = {
  isInstalling: false,
  showDeleteDialog: false,
  isDeleting: false,
  showRestoreDialog: false,
  isRestoring: false,
};

export const useGameCardState = () => {
  const [state, dispatch] = useReducer(reducer, initialState);

  return {
    state,
    actions: {
      startInstall: () => dispatch({ type: "START_INSTALL" }),
      finishInstall: () => dispatch({ type: "FINISH_INSTALL" }),
      showDeleteDialog: () => dispatch({ type: "SHOW_DELETE_DIALOG" }),
      hideDeleteDialog: () => dispatch({ type: "HIDE_DELETE_DIALOG" }),
      startDelete: () => dispatch({ type: "START_DELETE" }),
      finishDelete: () => dispatch({ type: "FINISH_DELETE" }),
      showRestoreDialog: () => dispatch({ type: "SHOW_RESTORE_DIALOG" }),
      hideRestoreDialog: () => dispatch({ type: "HIDE_RESTORE_DIALOG" }),
      startRestore: () => dispatch({ type: "START_RESTORE" }),
      finishRestore: () => dispatch({ type: "FINISH_RESTORE" }),
    },
  };
};

