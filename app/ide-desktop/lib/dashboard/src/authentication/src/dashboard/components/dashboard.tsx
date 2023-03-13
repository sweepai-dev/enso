/** @file Main dashboard container responsible for listing user's projects as well as other
 * interactive components. */

import * as auth from "../../authentication/providers/auth";
import withRouter from "../../navigation";

// ==========================
// === dashboardContainer ===
// ==========================

const dashboardContainer = () => {
  const { signOut } = auth.useAuth();
  const { accessToken } = auth.useFullUserSession();
  return (
    <>
      <h1>Hello dummy cloud dashboard</h1>
      <p>Access token: {accessToken}</p>
      <button onClick={signOut}>Log out</button>
    </>
  );
};

export default withRouter(dashboardContainer);
