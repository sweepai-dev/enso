/** @file Module containing utility functions used throughout our Dashboard code, but that don't fit
 * anywhere else. */

export const handleEvent =
    <T>(callback: () => Promise<T>) =>
    async (event: React.FormEvent) => {
        event.preventDefault()
        await callback()
    }
