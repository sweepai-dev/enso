/** @file Form to invite a user. */
import * as React from 'react'
import toast from 'react-hot-toast'

import * as backendModule from '../backend'
import * as newtype from '../../newtype'

import * as authProvider from '../../authentication/providers/auth'
import * as backendProvider from '../../providers/backend'
import * as modalProvider from '../../providers/modal'

import CreateForm, * as createForm from './createForm'

// ========================
// === SecretCreateForm ===
// ========================

/** Props for a {@link UserInviteForm}. */
export interface SecretCreateFormProps extends createForm.CreateFormPassthroughProps {
    onSuccess: () => void
}

/** A form to create a secret. */
function UserInviteForm(props: SecretCreateFormProps) {
    const { onSuccess, ...passThrough } = props
    const { organization } = authProvider.useNonPartialUserSession()
    const { backend } = backendProvider.useBackend()
    const { unsetModal } = modalProvider.useSetModal()

    const [email, setEmail] = React.useState<string | null>(null)

    if (backend.type === backendModule.BackendType.local || organization == null) {
        return <></>
    } else {
        const onSubmit = async (event: React.FormEvent) => {
            event.preventDefault()
            if (email == null || email === '') {
                toast.error('Please provide a user email.')
            } else {
                unsetModal()
                await backend.inviteUser({
                    organizationId: organization.id,
                    userEmail: newtype.asNewtype<backendModule.EmailAddress>(email),
                })
                onSuccess()
            }
        }

        return (
            <CreateForm
                title="Invite user"
                submitButtonText="Invite"
                onSubmit={onSubmit}
                {...passThrough}
            >
                <div className="flex flex-row flex-nowrap m-1">
                    <label className="inline-block flex-1 grow m-1" htmlFor="project_name">
                        Email
                    </label>
                    <input
                        id="project_name"
                        type="email"
                        size={1}
                        className="bg-gray-200 rounded-full flex-1 grow-2 px-2 m-1"
                        onChange={event => {
                            setEmail(event.target.value)
                        }}
                    />
                </div>
            </CreateForm>
        )
    }
}

export default UserInviteForm
