/** @file The UserMenu component provides a dropdown menu of user actions and settings. */
import { PropsWithChildren, useState } from 'react'
import * as auth from '../../../authentication/providers/auth'
import ChangePasswordModal from './changePasswordModal'

// ================
// === UserMenu ===
// ================

/** This is the UI component for a `UserMenu` list item.
 * The main interaction logic is in the `onClick` injected by `UserMenu`. */
interface UserMenuItemProps {
    needHoverClass?: boolean
    onClick?: React.MouseEventHandler<HTMLDivElement>
}

function UserMenuItem(props: PropsWithChildren<UserMenuItemProps>) {
    const { children, onClick, needHoverClass = false } = props
    /** User menu cell normal className. */
    const cellClassName = 'whitespace-nowrap px-4 py-2'
    /** User menu cell hover className. */
    const cellHoverClassName = 'hover:bg-blue-500 hover:text-white cursor-pointer'

    let className = cellClassName
    if (needHoverClass) {
        className += ` ${cellHoverClassName}`
    }
    return (
        <div className={className} onClick={onClick}>
            {children}
        </div>
    )
}

/** Handling the UserMenuItem click event logic and displaying its content. */
function UserMenu() {
    const { signOut } = auth.useAuth()
    const { organization } = auth.useFullUserSession()

    const [visibleChangePassword, setVisibleChangePassword] = useState(false)
    const handleResetPassword = () => {
        setVisibleChangePassword(true)
    }
    const handleCancelChangePassword = () => {
        setVisibleChangePassword(false)
    }

    return (
        <>
            <div className="absolute right-0 top-9 flex flex-col rounded-md bg-white py-1 border">
                {organization ? (
                    <>
                        <UserMenuItem>
                            Signed in as <span className="font-bold">{organization.name}</span>
                        </UserMenuItem>
                        <UserMenuItem needHoverClass>Your profile</UserMenuItem>
                        <UserMenuItem needHoverClass onClick={handleResetPassword}>
                            Change your password
                        </UserMenuItem>
                        <UserMenuItem needHoverClass onClick={signOut}>
                            Sign out
                        </UserMenuItem>
                    </>
                ) : (
                    <>
                        <UserMenuItem>Not logged in currently.</UserMenuItem>
                    </>
                )}
            </div>
            <ChangePasswordModal
                visible={visibleChangePassword}
                handleCancel={handleCancelChangePassword}
            />
        </>
    )
}

export default UserMenu
