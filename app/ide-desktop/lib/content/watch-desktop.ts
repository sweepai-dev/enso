/** @file Entrypoint for desktop watcher. */
import * as watch from './watch'

void watch.watch({ platform: 'desktop' })
