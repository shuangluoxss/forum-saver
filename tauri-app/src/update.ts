
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ask } from '@tauri-apps/plugin-dialog';

export async function checkForUpdates() {
    await check().then(async (update) => {
        if (!update) {
            return;
        }
        const answer = await ask(`发现新版本${update.version} from ${update.date} with notes ${update.body}，是否立即更新？`, {
            title: 'Tauri',
            kind: 'warning',
        });
        if (answer) {
            let downloaded = 0;
            let contentLength = 0;
            // alternatively we could also call update.download() and update.install() separately
            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength || 0;
                        console.log(`started downloading ${event.data.contentLength} bytes`);
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        console.log(`downloaded ${downloaded} from ${contentLength}`);
                        break;
                    case 'Finished':
                        console.log('download finished');
                        break;
                }
            });

            console.log('update installed');
            await relaunch();
        }
        console.log(update)
    }).finally(() => { console.log('check for updates done'); });

}