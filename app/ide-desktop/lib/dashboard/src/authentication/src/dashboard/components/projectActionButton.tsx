/** @file An interactive button displaying the status of a project. */
import * as react from "react";
import * as reactDom from "react-dom";

import * as auth from "../../authentication/providers/auth";
import * as backend from "../service";
import * as loggerProvider from "../../providers/logger";

// =============
// === Types ===
// =============

/** The state of the spinner. It should go from initial, to loading, to done. */
enum SpinnerState {
    initial = "initial",
    loading = "loading",
    done = "done",
}

// =================
// === Constants ===
// =================

/** The interval between requests checking whether the IDE is ready. */
const STATUS_CHECK_INTERVAL = 10000;

/** Displayed when a project is ready to start. */
const PLAY_ICON = (
    <svg
        width={36}
        height={36}
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
    >
        <path
            d="m10.04 7.34 6 3.85a1 1 0 0 1 0 1.68l-6 3.85a1 1 0 0 1-1.54-.84v-7.7a1 1 0 0 1 1.54-.84Z"
            fill="currentColor"
        />
        <rect
            x={1.5}
            y={1.5}
            width={21}
            height={21}
            rx={10.5}
            stroke="#3E515F"
            strokeOpacity={0.1}
            strokeWidth={3}
        />
    </svg>
);

/** Displayed when a project is ready for starting an IDE. */
const ARROW_UP_ICON = (
    <svg
        width={36}
        height={36}
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
    >
        <rect
            width={21}
            height={21}
            x={1.5}
            y={1.5}
            rx={10.5}
            stroke="currentColor"
            strokeOpacity={0.1}
            strokeWidth={3}
        />
        <path
            d="M12 17a1.5 1.5 0 0 1-1.5-1.5V12h3v3.5A1.5 1.5 0 0 1 12 17Z"
            fill="currentColor"
        />
        <path
            d="M8.943 12a1 1 0 0 1-.814-1.581l3.057-4.28a1 1 0 0 1 1.628 0l3.056 4.28A1 1 0 0 1 15.057 12H8.943Z"
            fill="currentColor"
        />
    </svg>
);

/** Displayed when a project is ready to stop. */
const stopIcon = (spinnerState: SpinnerState) => (
    <svg
        width={36}
        height={36}
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
    >
        <path
            d="m9 8L15 8a1 1 0 0 1 1 1L16 15a1 1 0 0 1 -1 1L9 16a1 1 0 0 1 -1 -1L8 9a1 1 0 0 1 1 -1"
            fill="currentColor"
        />
        <rect
            x={1.5}
            y={1.5}
            width={21}
            height={21}
            rx={10.5}
            stroke="currentColor"
            strokeOpacity={0.1}
            strokeWidth={3}
        />
        <rect
            x={1.5}
            y={1.5}
            width={21}
            height={21}
            rx={10.5}
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth={3}
            className={`spinner spinner-${spinnerState}`}
        />
    </svg>
);

// =================
// === Component ===
// =================

interface Props {
    project: backend.ListedProject;
    openIde: () => void;
    onOpen: () => void;
    onOpenStart: () => void;
    onClose: () => void;
}

/** An interactive button displaying the status of a project. */
const ProjectActionButton = ({
    project,
    openIde,
    onOpen,
    onOpenStart,
    onClose,
}: Props) => {
    const { accessToken } = auth.useFullUserSession();
    const logger = loggerProvider.useLogger();
    const backendService = backend.createBackend(accessToken, logger);
    const [checkStatusInterval, setCheckStatusInterval] = react.useState<
        number | undefined
    >(undefined);
    const [spinnerState, setSpinnerState] = react.useState(
        SpinnerState.initial
    );

    const handleCloseProject = () => {
        void backendService.closeProject(project.projectId);

        reactDom.unstable_batchedUpdates(() => {
            setCheckStatusInterval(undefined);
            clearInterval(checkStatusInterval);
            onClose();
        });
    };

    const handleOpenProject = () => {
        setSpinnerState(SpinnerState.initial);
        setTimeout(() => setSpinnerState(SpinnerState.loading), 0);

        void backendService.openProject(project.projectId);

        const checkProjectStatus = async () => {
            const response = await backendService.getProjectDetails(
                project.projectId
            );

            if (response.state.type === backend.ProjectState.opened) {
                setCheckStatusInterval(undefined);
                clearInterval(checkStatusInterval);
                onOpen();
                setSpinnerState(SpinnerState.done);
            }
        };

        reactDom.unstable_batchedUpdates(() => {
            setCheckStatusInterval(
                window.setInterval(
                    () => void checkProjectStatus(),
                    STATUS_CHECK_INTERVAL
                )
            );
            onOpenStart();
        });
    };

    switch (project.state.type) {
        case backend.ProjectState.created:
        case backend.ProjectState.new:
        case backend.ProjectState.closed:
            return <button onClick={handleOpenProject}>{PLAY_ICON}</button>;
        case backend.ProjectState.openInProgress:
            return (
                <button onClick={handleCloseProject}>
                    {stopIcon(spinnerState)}
                </button>
            );
        case backend.ProjectState.opened:
            return (
                <>
                    <button onClick={handleCloseProject}>
                        {stopIcon(spinnerState)}
                    </button>
                    <button onClick={openIde}>{ARROW_UP_ICON}</button>
                </>
            );
    }
};

export default ProjectActionButton;
