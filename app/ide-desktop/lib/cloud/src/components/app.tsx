import * as React from "react";

import {Auth} from "@aws-amplify/auth";
import {CognitoUserSession} from "amazon-cognito-identity-js";

import DashboardContainer from "./dashboard";
import {useEffect, useState} from "react";
import {unstable_batchedUpdates as batchedUpdate} from "react-dom";

// import * as dotenv from "dotenv";


export interface AppProps {
    runningOnDesktop: boolean;
    entrypoint: any;
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
        return <div>LOADING</div>
    }

}

export default App;