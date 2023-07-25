/** @file The top-bar of dashboard. */
import * as React from 'react'

import FindIcon from 'enso-assets/find.svg'

import * as backendModule from '../backend'

import PageSwitcher, * as pageSwitcher from './pageSwitcher'
import BackendSwitcher from './backendSwitcher'
import UserBar from './userBar'

// ==============
// === TopBar ===
// ==============

/** Props for a {@link TopBar}. */
export interface TopBarProps {
    /** Whether the application may have the local backend running. */
    supportsLocalBackend: boolean
    projectName: string | null
    page: pageSwitcher.Page
    setPage: (page: pageSwitcher.Page) => void
    isEditorDisabled: boolean
    setBackendType: (backendType: backendModule.BackendType) => void
    isHelpChatOpen: boolean
    setIsHelpChatOpen: (isHelpChatOpen: boolean) => void
    query: string
    setQuery: (value: string) => void
}

/** The {@link TopBarProps.setQuery} parameter is used to communicate with the parent component,
 * because `searchVal` may change parent component's project list. */
function TopBar(props: TopBarProps) {
    const {
        supportsLocalBackend,
        page,
        setPage,
        isEditorDisabled,
        setBackendType,
        isHelpChatOpen,
        setIsHelpChatOpen,
        query,
        setQuery,
    } = props

    return (
        <div className="relative flex mx-4.75 h-8 gap-6">
            <PageSwitcher page={page} setPage={setPage} isEditorDisabled={isEditorDisabled} />
            {supportsLocalBackend && <BackendSwitcher setBackendType={setBackendType} />}
            <div className="grow" />
            <div className="search-bar absolute flex items-center text-normal bg-frame-bg rounded-full -translate-x-1/2 gap-2.5 left-1/2 h-8 w-98.25 px-2">
                <label htmlFor="search">
                    <img src={FindIcon} className="opacity-80" />
                </label>
                <input
                    type="text"
                    size={1}
                    id="search"
                    placeholder="Type to search for projects, data connectors, users, and more."
                    value={query}
                    onChange={event => {
                        setQuery(event.target.value)
                    }}
                    className="grow bg-transparent h-5.5 py-px"
                />
            </div>
            <div className="grow" />
            <UserBar isHelpChatOpen={isHelpChatOpen} setIsHelpChatOpen={setIsHelpChatOpen} />
        </div>
    )
}

export default TopBar
