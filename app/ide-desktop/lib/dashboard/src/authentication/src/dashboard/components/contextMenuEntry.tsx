/** @file An entry in a context menu. */

import * as React from 'react'

export interface ContextMenuEntryProps {
    disabled?: boolean
    onClick: (event: React.MouseEvent<HTMLButtonElement>) => void
}

// This component MUST NOT use `useState` because it is not rendered directly.
function ContextMenuEntry(props: React.PropsWithChildren<ContextMenuEntryProps>) {
    const { children, disabled, onClick } = props
    return (
        <button
            disabled={disabled}
            className={`${
                disabled ? 'opacity-50' : ''
            } p-1 hover:bg-gray-200 first:rounded-t-lg last:rounded-b-lg text-left`}
            onClick={event => {
                event.stopPropagation()
                onClick(event)
            }}
        >
            {children}
        </button>
    )
}

export default ContextMenuEntry
