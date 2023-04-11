/** @file Container that launches the IDE. */
import * as react from 'react'

import * as service from '../service'

// =================
// === Component ===
// =================

interface Props {
    project: service.Project | null
    backendService: service.Backend
}

/** Container that launches the IDE. */
function Ide({ project, backendService }: Props) {
    const [initialized, setInitialized] = react.useState(false)
    // FIXME[sb]: remove
    const [[loaded, resolveLoaded]] = react.useState((): [Promise<void>, () => void] => {
        let resolve!: () => void
        const promise = new Promise<void>(innerResolve => {
            resolve = innerResolve
        })
        return [promise, resolve]
    })

    react.useEffect(() => {
        void (async () => {
            if (!project) {
                return
            }
            if (!initialized) {
                setInitialized(true)
            }
        })()
    }, [project])

    return <div id="root" />
}

export default Ide
