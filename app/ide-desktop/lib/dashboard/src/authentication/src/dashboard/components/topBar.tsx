/** @file A basic top bar, for all pages  other than the dashboard. */
import * as React from 'react'

import ArrowLeftIcon from 'enso-assets/arrow_left.svg'
import DefaultUserIcon from 'enso-assets/default_user.svg'
import SpeechBubbleIcon from 'enso-assets/speech_bubble.svg'

import * as app from '../../components/app'
import * as hooks from '../../hooks'
import * as modalProvider from '../../providers/modal'

import UserMenu from './userMenu'

// ==============
// === TopBar ===
// ==============

/** A basic top bar, for all pages  other than the dashboard. */
function TopBar() {
    const navigate = hooks.useNavigate()
    const { modal } = modalProvider.useModal()
    const { setModal, unsetModal } = modalProvider.useSetModal()
    const [isUserMenuVisible, setIsUserMenuVisible] = React.useState(false)

    React.useEffect(() => {
        if (!modal) {
            setIsUserMenuVisible(false)
        }
    }, [modal])

    React.useEffect(() => {
        if (isUserMenuVisible) {
            setModal(() => <UserMenu />)
        } else {
            unsetModal()
        }
    }, [isUserMenuVisible, setModal, unsetModal])

    const goToDashboard = React.useCallback(() => {
        navigate(app.DASHBOARD_PATH)
    }, [/* should never change */ navigate])

    return (
        <div className="flex mx-2 h-8">
            <button onClick={goToDashboard}>
                <img src={ArrowLeftIcon} width={24} height={24} />
            </button>
            {/* Padding. */}
            <div className="grow" />
            <a
                href="https://discord.gg/enso"
                target="_blank"
                rel="noreferrer"
                className="flex items-center bg-help rounded-full px-2.5 text-white mx-2"
            >
                <span className="whitespace-nowrap">help chat</span>
                <div className="ml-2">
                    <img src={SpeechBubbleIcon} />
                </div>
            </a>
            <div className="transform w-8">
                <div
                    onClick={event => {
                        event.stopPropagation()
                        setIsUserMenuVisible(!isUserMenuVisible)
                    }}
                    className="rounded-full w-8 h-8 bg-cover cursor-pointer"
                >
                    <img src={DefaultUserIcon} />
                </div>
            </div>
        </div>
    )
}

export default TopBar
