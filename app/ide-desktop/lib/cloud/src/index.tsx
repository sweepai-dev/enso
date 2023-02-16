/** @file Index file declaring main DOM structure for the app. */

// eslint-disable-next-line
// @ts-ignore
import * as ReactDOM from "react-dom/client";
import * as React from "react";

import * as authentication from 'enso-studio-authentication'

import {AppProps, UnauthorizedError} from "./components/app";
import App from "./components/app";


// ===========
// === run ===
// ===========

// Interface used to log logs, errors, etc.
//
// In the browser, this is the `Console` interface. In Electron, this is the `Logger` interface
// provided by the EnsoGL packager.
interface Logger {
    /// Logs a message to the console.
    log: (message?: any, ...optionalParams: any[]) => void,
}

/**
 * Entrypoint for the authentication/dashboard app.
 *
 * Running this function finds a `div` element with the ID `authentication`, and renders the
 * authentication/dashboard UI using React. It also handles routing and other interactions (e.g.,
 * for redirecting the user to/from the login page).
 */
export const run = (logger: Logger, props: AppProps) => {
    logger.log("Starting dashboard UI.")

    // The `id` attribute of the root element that the app will be rendered into.
    const rootElementId = 'dashboard'
    // The root element that the authentication/dashboard app will be rendered into.
    //
    // Return interface for `getElementById` is `HTMLElement` or `null`. Since we are fetching the
    // `authentication` element, and that element is expected to always be present in the `index.html`,
    // we can disable the `no-non-null-assertion` on this line.
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const root: HTMLElement = document.getElementById(rootElementId)!
    try {
        ReactDOM.createRoot(root).render(<App {...props} />);

    } catch {
        console.log("adsad")
        authentication.run(console, {
            runningOnDesktop: false, onAuthenticated: () => {
                window.location.reload()
            }
        })
    }
}

run(console, {
    runningOnDesktop: false, entrypoint: () => {
    }
})


export default {run}