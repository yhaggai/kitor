import { useMemo, useReducer } from "react";

type StepStatus = "pending" | "in-progress" | "complete";

type InstallStep = {
  id: string;
  label: string;
  status: StepStatus;
};

type State = {
  isInstalling: boolean;
  showInstallDialog: boolean;
  installSteps: InstallStep[];
  showDeleteDialog: boolean;
  isDeleting: boolean;
  showRestoreDialog: boolean;
  isRestoring: boolean;
};

type Action =
  | { type: "START_INSTALL" }
  | { type: "FINISH_INSTALL" }
  | { type: "SHOW_INSTALL_DIALOG" }
  | { type: "HIDE_INSTALL_DIALOG" }
  | { type: "UPDATE_STEP_STATUS"; stepId: string; status: StepStatus }
  | { type: "UPDATE_PROGRESS"; progress: number; step: string }
  | { type: "SHOW_DELETE_DIALOG" }
  | { type: "HIDE_DELETE_DIALOG" }
  | { type: "START_DELETE" }
  | { type: "FINISH_DELETE" }
  | { type: "SHOW_RESTORE_DIALOG" }
  | { type: "HIDE_RESTORE_DIALOG" }
  | { type: "START_RESTORE" }
  | { type: "FINISH_RESTORE" };

const INITIAL_STEPS: InstallStep[] = [
  { id: "preparing", label: "Preparing installation", status: "pending" },
  { id: "downloading", label: "Downloading patch", status: "pending" },
  { id: "backup", label: "Creating backup", status: "pending" },
  { id: "installing", label: "Installing patch", status: "pending" },
  { id: "scripts", label: "Running post-install scripts", status: "pending" },
  { id: "finalizing", label: "Finalizing", status: "pending" },
];

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case "START_INSTALL":
      return {
        ...state,
        isInstalling: true,
        installSteps: INITIAL_STEPS.map((s) => ({ ...s })),
      };
    case "FINISH_INSTALL":
      return {
        ...state,
        isInstalling: false,
        installSteps: INITIAL_STEPS.map((s) => ({ ...s })),
      };
    case "SHOW_INSTALL_DIALOG":
      return { ...state, showInstallDialog: true };
    case "HIDE_INSTALL_DIALOG":
      return { ...state, showInstallDialog: false };
    case "UPDATE_STEP_STATUS":
      return {
        ...state,
        installSteps: state.installSteps.map((step) =>
          step.id === action.stepId ? { ...step, status: action.status } : step
        ),
      };
    case "UPDATE_PROGRESS": {
      // Map progress percentage to step updates
      const progress = action.progress;
      let updatedSteps = [...state.installSteps];

      if (progress >= 0 && progress < 20) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing" ? { ...s, status: "in-progress" as StepStatus } : s
        );
      } else if (progress >= 20 && progress < 55) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing"
            ? { ...s, status: "complete" as StepStatus }
            : s.id === "downloading"
              ? { ...s, status: "in-progress" as StepStatus }
              : s
        );
      } else if (progress >= 55 && progress < 70) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing"
            ? { ...s, status: "complete" as StepStatus }
            : s.id === "downloading"
              ? { ...s, status: "complete" as StepStatus }
              : s.id === "backup"
                ? { ...s, status: "in-progress" as StepStatus }
                : s
        );
      } else if (progress >= 70 && progress < 85) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing"
            ? { ...s, status: "complete" as StepStatus }
            : s.id === "downloading"
              ? { ...s, status: "complete" as StepStatus }
              : s.id === "backup"
                ? { ...s, status: "complete" as StepStatus }
                : s.id === "installing"
                  ? { ...s, status: "in-progress" as StepStatus }
                  : s
        );
      } else if (progress >= 85 && progress < 90) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing"
            ? { ...s, status: "complete" as StepStatus }
            : s.id === "downloading"
              ? { ...s, status: "complete" as StepStatus }
              : s.id === "backup"
                ? { ...s, status: "complete" as StepStatus }
                : s.id === "installing"
                  ? { ...s, status: "complete" as StepStatus }
                  : s.id === "scripts"
                    ? { ...s, status: "in-progress" as StepStatus }
                    : s
        );
      } else if (progress >= 90) {
        updatedSteps = updatedSteps.map((s) =>
          s.id === "preparing"
            ? { ...s, status: "complete" as StepStatus }
            : s.id === "downloading"
              ? { ...s, status: "complete" as StepStatus }
              : s.id === "backup"
                ? { ...s, status: "complete" as StepStatus }
                : s.id === "installing"
                  ? { ...s, status: "complete" as StepStatus }
                  : s.id === "scripts"
                    ? { ...s, status: "complete" as StepStatus }
                    : s.id === "finalizing"
                      ? {
                          ...s,
                          status:
                            progress === 100
                              ? ("complete" as StepStatus)
                              : ("in-progress" as StepStatus),
                        }
                      : s
        );
      }

      return { ...state, installSteps: updatedSteps };
    }
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
  showInstallDialog: false,
  installSteps: INITIAL_STEPS.map((s) => ({ ...s })),
  showDeleteDialog: false,
  isDeleting: false,
  showRestoreDialog: false,
  isRestoring: false,
};

export const useGameCardState = () => {
  const [state, dispatch] = useReducer(reducer, initialState);

  // Memoize actions to prevent useEffect re-runs on every render
  const actions = useMemo(
    () => ({
      startInstall: () => dispatch({ type: "START_INSTALL" }),
      finishInstall: () => dispatch({ type: "FINISH_INSTALL" }),
      showInstallDialog: () => dispatch({ type: "SHOW_INSTALL_DIALOG" }),
      hideInstallDialog: () => dispatch({ type: "HIDE_INSTALL_DIALOG" }),
      updateStepStatus: (stepId: string, status: StepStatus) =>
        dispatch({ type: "UPDATE_STEP_STATUS", stepId, status }),
      updateProgress: (progress: number, step: string) =>
        dispatch({ type: "UPDATE_PROGRESS", progress, step }),
      showDeleteDialog: () => dispatch({ type: "SHOW_DELETE_DIALOG" }),
      hideDeleteDialog: () => dispatch({ type: "HIDE_DELETE_DIALOG" }),
      startDelete: () => dispatch({ type: "START_DELETE" }),
      finishDelete: () => dispatch({ type: "FINISH_DELETE" }),
      showRestoreDialog: () => dispatch({ type: "SHOW_RESTORE_DIALOG" }),
      hideRestoreDialog: () => dispatch({ type: "HIDE_RESTORE_DIALOG" }),
      startRestore: () => dispatch({ type: "START_RESTORE" }),
      finishRestore: () => dispatch({ type: "FINISH_RESTORE" }),
    }),
    []
  );

  return { state, actions };
};
