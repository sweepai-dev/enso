/** @file Configuration for Tailwind. */
import * as path from 'node:path'
import * as url from 'node:url'

// =================
// === Constants ===
// =================

const THIS_PATH = path.resolve(path.dirname(url.fileURLToPath(import.meta.url)))

// =====================
// === Configuration ===
// =====================

// The names come from a third-party API and cannot be changed.
/* eslint-disable no-restricted-syntax, @typescript-eslint/naming-convention */
export const content = [THIS_PATH + '/src/**/*.tsx']
export const theme = {
    extend: {
        colors: {
            // Should be `#3e515fe5`, but `bg-opacity` does not work with RGBA.
            /** The default color of all text. */
            primary: '#52636f',
            label: '#3e515f14',
            help: '#3f68ce',
            warning: '#eab120',
            'severe-warning': '#e06740',
            'perm-owner': '#51626e',
            'perm-admin': '#e06a50',
            'perm-write': '#efa043',
            'perm-read': '#b6cb34',
            'perm-exec': '#ad69e3',
            'perm-docs-write': '#2db1c3',
            // Should be `#3e515f14`, but `bg-opacity` does not work with RGBA.
            'perm-none': '#f0f1f3',
            'gray-250': '#dbdee3',
        },
        flexGrow: {
            2: '2',
        },
        fontSize: {
            xs: '0.71875rem',
            vs: '0.8125rem',
        },
        spacing: {
            '0.75': '0.1875rem',
            '1.25': '0.3125rem',
            '1.75': '0.4375rem',
            '2.25': '0.5625rem',
            '2.75': '0.6875rem',
            '3.25': '0.8125rem',
            '4.75': '1.1875rem',
            '5.5': '1.375rem',
            '8.75': '2.1875rem',
            '15.25': '3.8125rem',
            '35': '8.75rem',
            '39': '9.75rem',
            '39.5': '9.875rem',
            '57.5': '14.375rem',
            '68.75': '17.1875rem',
            '84.25': '21.0625rem',
            '118.25': '29.5625rem',
            '140': '35rem',
            '10lh': '10lh',
        },
        scale: {
            '103-1/3': '1.03333',
        },
        lineHeight: {
            normal: '1.445',
        },
        opacity: {
            '56': '0.56',
        },
        borderRadius: {
            '2.25xl': '1.125rem',
        },
        minWidth: {
            '30': '7.5rem',
        },
        boxShadow: {
            soft: `0 0.5px 2.2px 0px #00000008, 0 1.2px 5.3px 0px #0000000b, \
0 2.3px 10px 0 #0000000e, 0 4px 18px 0 #00000011, 0 7.5px 33.4px 0 #00000014, \
0 18px 80px 0 #0000001c`,
            'soft-dark': `0 0.5px 2.2px 0px #00000010, 0 1.2px 5.3px 0px #00000014, \
0 2.3px 10px 0 #0000001c, 0 4px 18px 0 #00000022, 0 7.5px 33.4px 0 #00000028, \
0 18px 80px 0 #00000038`,
            'inset-t-lg': `inset 0 1px 1.4px -1.4px #00000002, \
inset 0 2.4px 3.4px -3.4px #00000003, inset 0 4.5px 6.4px -6.4px #00000004, \
inset 0 8px 11.4px -11.4px #00000005, inset 0 15px 21.3px -21.3px #00000006, \
inset 0 36px 51px -51px #00000014`,
            'inset-b-lg': `inset 0 -1px 1.4px -1.4px #00000002, \
inset 0 -2.4px 3.4px -3.4px #00000003, inset 0 -4.5px 6.4px -6.4px #00000004, \
inset 0 -8px 11.4px -11.4px #00000005, inset 0 -15px 21.3px -21.3px #00000006, \
inset 0 -36px 51px -51px #00000014`,
            'inset-v-lg': `inset 0 1px 1.4px -1.4px #00000002, \
inset 0 2.4px 3.4px -3.4px #00000003, inset 0 4.5px 6.4px -6.4px #00000004, \
inset 0 8px 11.4px -11.4px #00000005, inset 0 15px 21.3px -21.3px #00000006, \
inset 0 36px 51px -51px #00000014, inset 0 -1px 1.4px -1.4px #00000002, \
inset 0 -2.4px 3.4px -3.4px #00000003, inset 0 -4.5px 6.4px -6.4px #00000004, \
inset 0 -8px 11.4px -11.4px #00000005, inset 0 -15px 21.3px -21.3px #00000006, \
inset 0 -36px 51px -51px #00000014`,
        },
        animation: {
            'spin-ease': 'spin cubic-bezier(0.67, 0.33, 0.33, 0.67) 1.5s infinite',
        },
        transitionProperty: {
            width: 'width',
            'stroke-dasharray': 'stroke-dasharray',
        },
        transitionDuration: {
            '5000': '5000ms',
            '90000': '90000ms',
        },
        gridTemplateColumns: {
            'fill-60': 'repeat(auto-fill, minmax(15rem, 1fr))',
        },
    },
}
