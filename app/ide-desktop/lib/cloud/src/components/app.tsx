import * as React from "react";

import {Auth} from "@aws-amplify/auth";
import {CognitoUserSession} from "amazon-cognito-identity-js";

import DashboardContainer from "./dashboard";
import {useEffect, useState} from "react";
import {unstable_batchedUpdates as batchedUpdate} from "react-dom";

const browserAmplifyConfigPbuchu = {
    region: "eu-west-1",
    // FIXME [NP]
    //identityPoolId: "",
    userPoolId: "eu-west-1_jSF1RbgPK",
    userPoolWebClientId: "1bnib0jfon3aqc5g3lkia2infr",
    oauth: {
        options: {}, // FIXME [NP]
        //domain: "https://npekin-enso-domain.auth.eu-west-1.amazoncognito.com",
        domain: "pb-enso-domain.auth.eu-west-1.amazoncognito.com",
        scope: ['email', 'openid'], // FIXME [NP]
        redirectSignIn: "http://localhost:8081",
        redirectSignOut: "http://localhost:8081",
        responseType: "code",
    },
}
const amplifyConfig = browserAmplifyConfigPbuchu;
Auth.configure(amplifyConfig);

export interface AppProps {
    runningOnDesktop: boolean;
    entrypoint: any;
}

export class UnauthorizedError extends Error {
}

const App = (_: AppProps) => {
    const [accessToken, setAccessToken] = useState<string>();
    const [userEmail, setUserEmail] = useState<string>();

    useEffect(() => {
        void (async (): Promise<void> => {
            const session: CognitoUserSession = await Auth.currentSession();
            const accessToken = session.getAccessToken().getJwtToken();
            const userEmail: string = session.getIdToken().payload.email;

            batchedUpdate(() => {
                if (accessToken) setAccessToken(accessToken);
                if (userEmail) setUserEmail(userEmail);
            });
        })();
    }, []);

    if (accessToken && userEmail) {
        return <DashboardContainer accessToken={accessToken} email={userEmail}/>
    } else {
        throw new Error("Unauthorized")
    }

}

export default App;