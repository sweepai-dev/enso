/** @file Module containing the API client for the Cloud backend API.
 *
 * Each exported function in the {@link Backend} in this module corresponds to an API endpoint. The
 * functions are asynchronous and return a `Promise` that resolves to the response from the API. */
import * as http from "../http";
import * as config from "../config";
import * as loggerProvider from "../providers/logger";



// =================
// === Constants ===
// =================

/** Relative HTTP path to the "get user" endpoint of the Cloud backend API. */
const GET_USER_PATH = "users/me";



// =============
// === Types ===
// =============

/** A user/organization in the application. These are the primary owners of a project. */
export interface Organization {
    id: string;
    userEmail: string;
    name: string;
}



// ===============
// === Backend ===
// ===============

/** Class for sending requests to the Cloud backend API endpoints. */
export class Backend {
    private client: http.Client;
    private logger: loggerProvider.Logger;

    /** Creates a new instance of the {@link Backend} API client.
     *
     * @throws An error if the `Authorization` header is not set on the given `client`. */
    constructor(client: http.Client, logger: loggerProvider.Logger) {
        this.client = client;
        this.logger = logger;

        /** All of our API endpoints are authenticated, so we expect the `Authorization` header to
         * be set. */
        if (!this.client.defaultHeaders?.has("Authorization")) {
            throw new Error("Authorization header not set.");
        }
    }

    /** Returns a {@link RequestBuilder} for an HTTP GET request to the given path. */
    get = (path: string) => this.client.get(`${config.ACTIVE_CONFIG.apiUrl}/${path}`);

    /** Returns a {@link RequestBuilder} for an HTTP POST request to the given path. */
    post = (path: string) => this.client.post(`${config.ACTIVE_CONFIG.apiUrl}/${path}`);

    /** Logs the error that occurred and throws a new one with a more user-friendly message. */
    errorHandler = (message: string) => (error: Error) => {
        this.logger.error(error.message);
        throw new Error(message);
    };

    /** Returns organization info for the current user, from the Cloud backend API.
     *
     * @returns `null` if status code 401 or 404 was received. */
    getUser = (): Promise<Organization | null> =>
        this.get(GET_USER_PATH)
            .send()
            .then((response) => {
                if (response.status() == 401 || response.status() == 404) {
                    return null;
                }

                return response.model<Organization>();
            });
}



// =====================
// === createBackend ===
// =====================

/** Shorthand method for creating a new instance of the backend API, along with the necessary
 * headers. */
/* TODO [NP]: https://github.com/enso-org/cloud-v2/issues/343
 * This is a hack to quickly create the backend in the format we want, until we get the provider
 * working. This should be removed entirely in favour of creating the backend once and using it from
 * the context. */
export const createBackend = (
    accessToken: string,
    logger: loggerProvider.Logger
): Backend => {
    const headers = new Headers();
    headers.append("Authorization", `Bearer ${accessToken}`);
    const client = http.Client.builder().defaultHeaders(headers).build();
    return new Backend(client, logger);
};
