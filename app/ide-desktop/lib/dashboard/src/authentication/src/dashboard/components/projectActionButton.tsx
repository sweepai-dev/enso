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
        <path
            d="m10 12L8.4 12a1 1 0 0 1 -0.719 -1.695L11.281 6.719a1 1 0 0 1 1.409 0L16.305 10.305a1 1 0 0 1 -0.719 1.695L14 12 14 15.5a1 1 0 0 1 -1 1L11 16.5a1 1 0 0 1 -1 -1"
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
            const response = await backendService.getProjectDetails(project.projectId);

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
                <>
                    <button onClick={handleCloseProject}>
                        {stopIcon(spinnerState)}
                    </button>
                </>
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
