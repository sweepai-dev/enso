/** @file Container that launches the IDE. */
import * as react from "react";

import * as service from "../service";

// ======================================
// === Global namespace augmentations ===
// ======================================

declare global {
    /** Defined by index.js.gz. The entry point of the IDE. */
    const enso: { main: (config: object) => void };
}

// =================
// === Constants ===
// =================

const IDE_CDN_URL = "https://ensocdn.s3.us-west-1.amazonaws.com/ide";

// =================
// === Component ===
// =================

interface Props {
    project: service.Project;
    backendService: service.Backend;
}

/** Container that launches the IDE. */
const Ide = ({ project, backendService }: Props) => {
    react.useEffect(() => {
        void (async () => {
            const ideVersions = await backendService.listVersions({
                versionType: service.VersionType.ide,
                default: true,
            });
            const projectIdeVersion =
                project.ideVersion?.value ?? ideVersions[0]!.number.value;
            const stylesheetLink = document.createElement("link");
            stylesheetLink.rel = "stylesheet";
            stylesheetLink.href = `${IDE_CDN_URL}/${projectIdeVersion}/style.css`;
            const indexScript = document.createElement("script");
            indexScript.src = `${IDE_CDN_URL}/${projectIdeVersion}/index.js.gz`;
            document.head.append(stylesheetLink);
            document.body.append(indexScript);
        })();
    }, []);

    react.useEffect(() => {
        void (async () => {
            const ideVersions = await backendService.listVersions({
                versionType: service.VersionType.ide,
                default: true,
            });
            const backendVersions = await backendService.listVersions({
                versionType: service.VersionType.backend,
                default: true,
            });
            const projectIdeVersion =
                project.ideVersion?.value ?? ideVersions[0]!.number.value;
            const projectEngineVersion =
                project.engineVersion?.value ??
                backendVersions[0]!.number.value;
            enso.main({
                loader: {
                    shadersUrl: `${IDE_CDN_URL}/${projectIdeVersion}/shaders`,
                    wasmUrl: `${IDE_CDN_URL}/${projectIdeVersion}/pkg-opt.wasm`,
                    jsUrl: `${IDE_CDN_URL}/${projectIdeVersion}/pkg.js.gz`,
                },
                engine: {
                    rpcUrl: `${project.address!}json`,
                    dataUrl: `${project.address!}binary`,
                    preferredVersion: projectEngineVersion,
                },
                startup: {
                    project: project.packageName,
                },
            });
        })();
    }, [project]);

    return <div id="root" />;
};

export default Ide;
