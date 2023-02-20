/** @file Index file declaring main DOM structure for the app. */

// eslint-disable-next-line
// @ts-ignore
import * as ReactDOM from "react-dom/client";
import * as React from "react";

import * as authentication from 'enso-studio-authentication'

import {AppProps} from "./components/app";
import App from "./components/app";
import {Auth} from "@aws-amplify/auth";
import {getUsersMe} from "./cloud-utils/src";

const amplifyConfig = {
    region: process.env.REACT_APP_AUTH_USER_POOL_ID,
    userPoolId: process.env.REACT_APP_AUTH_USER_POOL_ID,
    userPoolWebClientId: process.env.REACT_APP_AUTH_USER_POOL_WEB_CLIENT_ID,
    oauth: {
        domain: process.env.REACT_APP_AUTH_DOMAIN,
        scope: ['email', 'openid'],
        redirectSignIn: process.env.REACT_APP_AUTH_REDIRECT_SIGN_IN,
        redirectSignOut: process.env.REACT_APP_AUTH_REDIRECT_SIGN_OUT,
        responseType: "code",
    },
}

Auth.configure(amplifyConfig);


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
    ReactDOM.createRoot(root).render(<App {...props} />);
}

const init_auth = () => {
    authentication.run({
        logger: console,
        runningOnDesktop: false,
        onAuthenticated: () => {
            window.location.reload();
        }
    })
}

Auth.currentSession()
    .then((data) => {
        getUsersMe(data.getAccessToken().getJwtToken()).then((user) => {
            if (user) {
                run(console, {
                    runningOnDesktop: false, entrypoint: () => {}
                })
            } else {
                init_auth()
            }
        })
    })
    .catch((_) => {
        init_auth()
    })


export default {run}