import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import GObject from 'gi://GObject';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const ICON_ON = 'audio-input-microphone-symbolic';
const ICON_OFF = 'microphone-sensitivity-muted-symbolic';

const MicToggle = GObject.registerClass(
class MicToggle extends QuickSettings.QuickToggle {
    _init() {
        super._init({
            title: 'Microphones',
            iconName: ICON_ON,
        });
    }
});

export default class StfuExtension extends Extension {
    enable() {
        const stfu = GLib.find_program_in_path('stfu')
            ?? GLib.build_filenamev([GLib.get_home_dir(), '.local', 'bin', 'stfu']);
        this._stfu = GLib.file_test(stfu, GLib.FileTest.IS_EXECUTABLE) ? stfu : null;

        this._toggle = new MicToggle();
        this._toggle.connect('clicked', () => this._onClicked());

        this._indicator = new QuickSettings.SystemIndicator();
        this._indicator.quickSettingsItems.push(this._toggle);
        this._panelIcon = this._indicator._addIndicator();
        this._panelIcon.icon_name = ICON_OFF;

        Main.panel.statusArea.quickSettings.addExternalIndicator(this._indicator);

        const menu = Main.panel.statusArea.quickSettings.menu;
        this._menuOpenId = menu.connect('open-state-changed', (_m, open) => {
            if (open)
                this._refresh();
        });

        // react to state changes made outside the shell (terminal, scripts)
        const confDir = GLib.build_filenamev(
            [GLib.get_user_config_dir(), 'wireplumber', 'wireplumber.conf.d']);
        try {
            this._dirMonitor = Gio.File.new_for_path(confDir)
                .monitor_directory(Gio.FileMonitorFlags.NONE, null);
            this._dirMonitor.connect('changed', () => this._refresh());
        } catch (e) {
            this._dirMonitor = null;
            logError(e, 'stfu');
        }

        if (this._stfu) {
            this._refresh();
        } else {
            this._toggle.sensitive = false;
            this._toggle.subtitle = 'stfu CLI not found';
        }
    }

    disable() {
        if (this._menuOpenId) {
            Main.panel.statusArea.quickSettings.menu.disconnect(this._menuOpenId);
            this._menuOpenId = null;
        }
        this._dirMonitor?.cancel();
        this._dirMonitor = null;
        this._toggle?.destroy();
        this._toggle = null;
        this._indicator?.destroy();
        this._indicator = null;
        this._panelIcon = null;
        this._stfu = null;
    }

    _onClicked() {
        if (!this._stfu)
            return;
        // optimistic flip; _apply() corrects when stfu answers
        this._toggle.checked = !this._toggle.checked;
        this._toggle.subtitle = '…';
        this._run(['toggle'], out => this._apply(out));
    }

    _refresh() {
        this._run(['status'], out => this._apply(out));
    }

    _run(args, cb) {
        if (!this._stfu)
            return;
        try {
            const proc = Gio.Subprocess.new(
                [this._stfu, ...args],
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE);
            proc.communicate_utf8_async(null, null, (p, res) => {
                try {
                    const [, stdout, stderr] = p.communicate_utf8_finish(res);
                    const out = stdout?.trim();
                    if (out === 'on' || out === 'off') {
                        cb(out);
                    } else {
                        this._toggle.subtitle = 'stfu error';
                        logError(new Error(`stfu ${args[0]} failed: ${stderr?.trim()}`), 'stfu');
                    }
                } catch (e) {
                    logError(e, 'stfu');
                }
            });
        } catch (e) {
            logError(e, 'stfu');
        }
    }

    _apply(state) {
        const on = state === 'on';
        this._toggle.checked = on;
        this._toggle.iconName = on ? ICON_ON : ICON_OFF;
        this._toggle.subtitle = on ? 'On' : 'Muted';
        this._panelIcon.visible = !on;
    }
}
