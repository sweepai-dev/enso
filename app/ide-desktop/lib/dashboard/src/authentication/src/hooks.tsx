import * as react from "react";
import * as loggerProvider from "./providers/logger";



// ================
// === useInput ===
// ================

/** A custom hook to handle input fields.
 *
 * In React, managing state (e.g., user input values) must be done via the `useState` hook, which
 * returns a prop (e.g., `value`) containing the current value of the state, and a function (e.g.,
 * `setValue`) to update the state. Because of this, to bind a `value` to an input field, we must
 * use the `value` prop and the `onChange` event handler. However, this can be tedious to do for
 * every input field, so we can use a custom hook to handle this for us. */
export const useInput = (initialValue: string) => {
    const [value, setValue] = react.useState(initialValue);
    const onChange = (event: React.ChangeEvent<HTMLInputElement>) => setValue(event.target.value);
    const bind = {
        value,
        onChange,
    };

    return {
        value,
        bind,
    };
}



// ======================
// === useAsyncEffect ===
// ======================

/** A React hook for re-rendering a component once an asynchronous call is over.
 * 
 * This hook will take care of setting an initial value for the component state (so that it can
 * render immediately), updating the state once the asynchronous call is over (to re-render the
 * component), and cancelling any in-progress asynchronous calls when the component is unmounted (to
 * avoid race conditions where "update 1" starts, "update 2" starts and finishes, then "update 1"
 * finishes and sets the state).
 * 
 * For further details, see: https://devtrium.com/posts/async-functions-useeffect
 * Also see: https://stackoverflow.com/questions/61751728/asynchronous-calls-with-react-usememo
 *
 * @param initialValue - The initial value of the state controlled by this hook. 
 * @param fetch - The asynchronous function used to load the state controlled by this hook.
 * @param deps - The list of dependencies that, when updated, trigger the asynchronous fetch.
 * @returns value - The current value of the state controlled by this hook. */
export function useAsyncEffect<T>(
    initialValue: T,
    fetch: () => Promise<T>,
    deps?: react.DependencyList,
): [T] {
    const logger = loggerProvider.useLogger();
    const [value, setValue] = react.useState<T>(initialValue);

    react.useEffect(() => {
        let active = true;

        // Declare the async data fetching function.
        const load = async () => {
            const result = await fetch();

            // Set state with the result if `active` is true. This is what prevents race conditions.
            if (!active) return;
            setValue(result);
        }

        load().catch(error => logger.error("Error while fetching data", error));

        // Cancel any future `setValue` calls.
        return () => { active = false }
    }, deps)

    return [value]
}
