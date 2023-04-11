/** @file Entry point for the bundler. */
import * as esbuild from 'esbuild'

import * as bundler from './esbuild-config'

try {
    // FIXME[sb]: Do I need to bundle tailwind too?
    void esbuild.build(bundler.bundleOptions({ platform: 'desktop' }))
} catch (error) {
    console.error(error)
    // This is a top-level statement, so a `return` cannot be placed in the other branch.
    // eslint-disable-next-line no-restricted-syntax
    throw error
}
