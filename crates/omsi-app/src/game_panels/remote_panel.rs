use super::*;
use omsi_plugin::ui::Anchor;
use omsi_ui::tr;
use std::path::PathBuf;

const ACCENT: Rgba = Rgba([245, 166, 35, 255]);
const MODULE_PX: u32 = 8;

#[derive(Default)]
pub(crate) struct RemotePanel {
    said: Option<String>,
}

fn qr_png(url: &str) -> Option<PathBuf> {
    let dir = crate::remote::dir()?;
    let name = format!("qr-{:08x}.png", url.bytes().fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619)));
    let path = dir.join(name);
    if path.is_file() {
        return Some(path);
    }
    for old in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        if old.file_name().to_string_lossy().starts_with("qr-") {
            let _ = std::fs::remove_file(old.path());
        }
    }
    let (n, cells) = omsi_remote::qr(url)?;
    let side = (n as u32 + 8) * MODULE_PX;
    let img = image::GrayImage::from_fn(side, side, |x, y| {
        let (cx, cy) = ((x / MODULE_PX) as i64 - 4, (y / MODULE_PX) as i64 - 4);
        let dark = cx >= 0 && cy >= 0 && (cx as usize) < n && (cy as usize) < n && cells[cy as usize * n + cx as usize];
        image::Luma([if dark { 0 } else { 255 }])
    });
    std::fs::create_dir_all(&dir).ok()?;
    img.save(&path).ok()?;
    Some(path)
}

fn step(n: &str, t: &str) -> Element {
    row(vec![Element { color: Some(ACCENT), ..text(n, 15.0, Weight::Bold) }, Element { grow: true, ..text(t, 13.0, Weight::Regular) }], RowAlign::Start)
}

impl RemotePanel {
    pub(crate) fn panel(&self, app: &crate::App) -> Panel {
        let mut c = vec![
            row(vec![el(None, Kind::Icon { name: "smartphone".into(), size: 22.0 }), Element { grow: true, ..title(&tr("Remote cockpit")) }], RowAlign::Start),
            text(&tr("The bus's switches, its destination sign and a dashboard on a phone or tablet of the same network"), 13.0, Weight::Regular),
            divider(),
        ];
        let remote = &app.integrations.remote;
        match remote.server.as_ref() {
            Some(server) => {
                let ip = omsi_net::addrs::joinable_addresses().first().map(|a| a.ip.to_string()).unwrap_or_else(|| "127.0.0.1".into());
                let address = format!("http://{ip}:{}", server.port());
                let url = omsi_remote::pairing_url(&ip, server.port(), &server.pin());
                let steps = vec![
                    step("1", &tr("Connect the phone or tablet to the same network (Wi-Fi) as this computer.")),
                    step("2", &tr("Scan this picture with its camera: the cockpit opens and pairs by itself.")),
                    step("3", &tr("Or open this address in its browser and enter the code:")),
                    Element { color: Some(ACCENT), ..text(&address, 18.0, Weight::Bold) },
                    text(&server.pin().chars().map(|ch| ch.to_string()).collect::<Vec<_>>().join(" "), 30.0, Weight::Bold),
                ];
                if let Some(path) = qr_png(&url) {
                    c.push(row(vec![el(None, Kind::Image { path: Some(path), width: 200.0, height: 200.0 })], RowAlign::Center));
                }
                c.extend(steps);
                let n = server.clients();
                c.push(divider());
                c.push(row(
                    vec![
                        el(None, Kind::Icon { name: if n > 0 { "check_circle".into() } else { "wifi_tethering".into() }, size: 18.0 }),
                        text(&if n > 0 { format!("{} {n}", tr("Devices connected:")) } else { tr("No device connected yet").into_owned() }, 13.0, Weight::Medium),
                    ],
                    RowAlign::Start,
                ));
                c.push(note(&tr("A device that does not connect: allow openOMSI in this computer's firewall, and check that both are on the same network.")));
                c.push(grow_row(vec![button("forget", &tr("Forget paired devices"), Some("delete")), button("off", &tr("Switch off"), Some("power_settings_new")), button("close", &tr("Close"), Some("close"))]));
            }
            None => {
                if remote.failed() {
                    c.push(Element { color: Some(Rgba([255, 107, 107, 255])), ..text(&app.remote_description(), 13.0, Weight::Medium) });
                }
                c.push(note(&tr("Nothing is reachable from the network while it is off.")));
                c.push(grow_row(vec![button("on", &tr("Switch on"), Some("power_settings_new")), button("close", &tr("Close"), Some("close"))]));
            }
        }
        if let Some(said) = &self.said {
            c.push(note(said));
        }
        card(Anchor::Center, 0.0, 0.0, 520.0, 240, c)
    }
}

pub(crate) fn open(app: &mut crate::App) {
    app.close_game_menu();
    app.menus.game_panels.remote = Some(RemotePanel::default());
}

fn switch(app: &mut crate::App, on: bool) {
    app.settings.remote_cockpit = on;
    app.integrations.remote.forget_failure();
    crate::game_lists::remember_setting("remote_cockpit", if on { "1" } else { "0" });
}

pub(crate) fn click(app: &mut crate::App, id: &str) {
    match id {
        "on" => switch(app, true),
        "off" => switch(app, false),
        "forget" => {
            if let Some(s) = app.integrations.remote.server.as_ref() {
                s.forget_devices();
            }
            if let Some(p) = app.menus.game_panels.remote.as_mut() {
                p.said = Some(tr("Every device has to enter the code again").into_owned());
            }
        }
        "close" => app.menus.game_panels.remote = None,
        _ => {}
    }
}
