/** @file A page listing users in the same organization. */
import * as React from 'react'

import DefaultUserIcon from 'enso-assets/default_user.svg'
import PlusIcon from 'enso-assets/plus.svg'

import * as app from '../../components/app'
import * as backendModule from '../backend'
import * as backendProvider from '../../providers/backend'
import * as hooks from '../../hooks'
import * as modalProvider from '../../providers/modal'

import Rows from './rows'
import TopBar from './topBar'
import UserInviteForm from './userInviteForm'

// =================
// === Constants ===
// =================

/** This should never happen as the organization always contains the current user. */
const PLACEHOLDER = <>There are no users in this organization.</>

// =============
// === Types ===
// =============

/** Possible columns for a {@link backendModule.SimpleUser}. */
enum UserColumn {
    name = 'name',
    email = 'email',
}

// =================
// === ListUsers ===
// =================

/** Returns a user's id. This is useful for use in React hooks, to avoid dependencies. */
function getUserId(user: backendModule.SimpleUser) {
    return user.id
}

/** A page listing users in the same organization. */
function ListUsers() {
    const navigate = hooks.useNavigate()
    const { backend } = backendProvider.useBackend()
    const { modal } = modalProvider.useModal()
    const { setModal } = modalProvider.useSetModal()
    const [refresh, doRefresh] = hooks.useRefresh()
    const [isLoading, setIsLoading] = React.useState(true)
    const [users, setUsers] = React.useState<backendModule.SimpleUser[]>([])

    React.useEffect(() => {
        void (async () => {
            if (backend.type !== backendModule.BackendType.local) {
                setUsers(await backend.listUsers())
            }
            setIsLoading(false)
        })()
    }, [backend, refresh])

    React.useEffect(() => {
        if (backend.type !== backendModule.BackendType.remote) {
            navigate(app.DASHBOARD_PATH)
        }
    }, [backend.type, /* should never change */ navigate])

    return (
        <div className="flex flex-col gap-2 relative select-none text-primary text-xs h-screen py-2">
            <TopBar />
            <table className="items-center self-start border-collapse mt-2 whitespace-nowrap">
                <tbody>
                    <tr className="h-0">
                        <td className="w-48" />
                        <td className="w-60" />
                    </tr>
                    <Rows
                        items={users}
                        getKey={getUserId}
                        placeholder={PLACEHOLDER}
                        columns={[
                            {
                                id: UserColumn.name,
                                heading: (
                                    <div className="flex items-center gap-1">
                                        User
                                        <button
                                            onClick={event => {
                                                const buttonPosition =
                                                    event.currentTarget.getBoundingClientRect()
                                                setModal(() => (
                                                    <UserInviteForm
                                                        left={buttonPosition.left + window.scrollX}
                                                        top={buttonPosition.top + window.scrollY}
                                                        onSuccess={doRefresh}
                                                    />
                                                ))
                                            }}
                                        >
                                            <img src={PlusIcon} />
                                        </button>
                                    </div>
                                ),
                                render: user => (
                                    <div className="flex items-center gap-2">
                                        <img src={DefaultUserIcon} /> {user.name}
                                    </div>
                                ),
                            },
                            {
                                id: UserColumn.email,
                                heading: <>Email</>,
                                render: user => <>{user.email}</>,
                            },
                        ]}
                        isLoading={isLoading}
                        onClick={() => {
                            // Nothing for now.
                        }}
                        onContextMenu={() => {
                            // Nothing for now.
                        }}
                    />
                </tbody>
            </table>
            {modal && <>{modal}</>}
        </div>
    )
}

export default ListUsers
