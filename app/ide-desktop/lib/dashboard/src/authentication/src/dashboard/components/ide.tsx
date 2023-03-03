/** @file  */
import * as react from "react";

import * as service from "../service";

const IDE_CDN_URL = "https://ensocdn.s3.us-west-1.amazonaws.com/ide";

interface Props {
  project: service.Project;
  backendService: service.Backend;
}

declare global {
  const enso: { main: (config: object) => void; };
}

// eslint-disable-next-line @typescript-eslint/naming-convention
const Ide = ({ project, backendService }: Props) => {
  // FIXME: The IDE pollutes the `window` object.
  // The easiest way to work around this when unloading is to reload the window,
  // but we want to avoid that to be able to switch back to the dashboard smoothly

  react.useEffect(() => {
    document.title = `${project.name} - Enso Cloud IDE`;
    const projectIdeVersion = project.ideVersion!.version_number;
    const stylesheetLink = document.createElement("link");
    stylesheetLink.rel = "stylesheet";
    stylesheetLink.href = `${IDE_CDN_URL}/${projectIdeVersion}/style.css`;
    const indexScript = document.createElement("script");
    indexScript.src = `${IDE_CDN_URL}/${projectIdeVersion}/index.js.gz`;
    document.head.append(stylesheetLink);
    document.body.append(indexScript);
  }, []);

  react.useEffect(() => {
    void (async () => {
      const user = await backendService.getUser();
      const projectIdeVersion = project.ideVersion!.version_number;
      const projectEngineVersion = project.engineVersion!.version_number;
      enso.main({
        pkgWasmUrl: `${IDE_CDN_URL}/${projectIdeVersion}/pkg-opt.wasm`,
        pkgJsUrl: `${IDE_CDN_URL}/${projectIdeVersion}/pkg.js.gz`,
        shadersUrl: `${IDE_CDN_URL}/${projectIdeVersion}/shaders`,
        languageServerRpc: `${project.address!}json`,
        languageServerData: `${project.address!}binary`,
        isInCloud: true,
        useLoader: true,
        authenticationEnabled: false,
        project: project.packageName,
        preferredEngineVersion: projectEngineVersion,
        dataGathering: true,
        email: user!.userEmail,
      });
    })
  }, [project]);

  return <div id="ide"><div id="root" /></div>;
};

export default Ide;
