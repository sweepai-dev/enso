/** @file Entrypoint for cloud watcher. */
import * as watch from './watch'

void watch.watch({ platform: 'cloud' })
