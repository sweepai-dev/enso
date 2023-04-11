/** @file The cloud implementation of the Project Manager. */
import * as backendModule from './service'
import * as projectManager from 'enso-content/src/project_manager'

// =================
// === Constants ===
// =================

const IDE_CDN_URL = 'https://ensocdn.s3.us-west-1.amazonaws.com/ide'

// =============================
// === Cloud project manager ===
// =============================

// TODO[sb]: Consider extracting `ProjectManager` types to a common location.
export class ProjectManager {
    backend: backendModule.Backend
    projectManager: projectManager.ProjectManager
    // FIXME[sb]: Backend MUST change when e.g. a different user logs in.
    constructor(public backend: backendModule.Backend) {}

    async openProject(project: backendModule.Project) {
        if (false) {
            const ideVersion = (
                await this.backend.listVersions({
                    versionType: backendModule.VersionType.ide,
                    default: true,
                })
            )[0]
            const projectIdeVersion = project.ideVersion?.value ?? ideVersion.number.value
            const backendVersion = (
                await this.backend.listVersions({
                    versionType: backendModule.VersionType.backend,
                    default: true,
                })
            )[0]
            const projectEngineVersion = project.engineVersion?.value ?? backendVersion.number.value
            // FIXME[sb]: Add style.css to dashboard build (if it isn't already present)
            await window.enso.main({
                loader: {
                    assetsUrl: `${IDE_CDN_URL}/${projectIdeVersion}/dynamic-assets`,
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
            })
        }
    }

    async closeProject(params: projectManager.CloseProjectParams): Promise<void> {
        window.enso = null
    }
}
