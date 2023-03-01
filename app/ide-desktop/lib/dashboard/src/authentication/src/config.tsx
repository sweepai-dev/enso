/** @file Configuration definition for the Dashboard. */



// ===========
// === Api ===
// ===========

/** Base URL for requests to our Cloud API backend. */
type ApiUrl = string;

const PROD_API_URL: ApiUrl = "https://cloud.enso.org";
const PBUCHU_API_URL: ApiUrl =
  "https://xw0g8j3tsb.execute-api.eu-west-1.amazonaws.com";

export const API_URL = PROD_API_URL;
