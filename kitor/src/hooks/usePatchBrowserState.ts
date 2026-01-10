import { useMemo, useReducer } from "react";
import type { PatchWithStatus } from "@/store/store";

type SelectedGuide = {
  scummvmId: string;
  title: string;
};

type State = {
  patches: PatchWithStatus[];
  loading: boolean;
  showAddDialog: boolean;
  selectedGuide: SelectedGuide | null;
};

type Action =
  | { type: "SET_PATCHES"; patches: PatchWithStatus[] }
  | { type: "SET_LOADING"; loading: boolean }
  | { type: "SHOW_ADD_DIALOG" }
  | { type: "HIDE_ADD_DIALOG" }
  | { type: "SET_SELECTED_GUIDE"; guide: SelectedGuide }
  | { type: "CLEAR_SELECTED_GUIDE" };

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case "SET_PATCHES":
      return { ...state, patches: action.patches };
    case "SET_LOADING":
      return { ...state, loading: action.loading };
    case "SHOW_ADD_DIALOG":
      return { ...state, showAddDialog: true };
    case "HIDE_ADD_DIALOG":
      return { ...state, showAddDialog: false };
    case "SET_SELECTED_GUIDE":
      return { ...state, selectedGuide: action.guide };
    case "CLEAR_SELECTED_GUIDE":
      return { ...state, selectedGuide: null };
    default:
      return state;
  }
};

const initialState: State = {
  patches: [],
  loading: false,
  showAddDialog: false,
  selectedGuide: null,
};

export const usePatchBrowserState = () => {
  const [state, dispatch] = useReducer(reducer, initialState);

  const actions = useMemo(
    () => ({
      setPatches: (patches: PatchWithStatus[]) => dispatch({ type: "SET_PATCHES", patches }),
      setLoading: (loading: boolean) => dispatch({ type: "SET_LOADING", loading }),
      showAddDialog: () => dispatch({ type: "SHOW_ADD_DIALOG" }),
      hideAddDialog: () => dispatch({ type: "HIDE_ADD_DIALOG" }),
      setSelectedGuide: (guide: SelectedGuide) => dispatch({ type: "SET_SELECTED_GUIDE", guide }),
      clearSelectedGuide: () => dispatch({ type: "CLEAR_SELECTED_GUIDE" }),
    }),
    []
  );

  return { state, actions };
};
