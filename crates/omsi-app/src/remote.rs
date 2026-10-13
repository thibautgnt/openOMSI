use super::*;
use omsi_remote::{Command, Remote, Vehicle};
use omsi_sim::htmltex::HtmlRequest;

const PUBLISH_EVERY: f32 = 1.0 / 30.0;

type Skins = std::sync::mpsc::Receiver<(String, Vec<(String, Vec<u8>)>)>;
type Doors = std::sync::mpsc::Receiver<(String, Vec<(Vec<String>, Vec<usize>)>)>;

#[derive(Default)]
pub(crate) struct RemoteCockpit {
    pub(crate) server: Option<Remote>,
    failed_port: Option<u16>,
    publish_t: f32,
    vehicle: String,
    lang: String,
    skins: Option<Skins>,
    tried: Option<Doors>,
    sign: Option<omsi_remote::sign::Sign>,
    sign_t: f32,
    nav_t: f32,
    door_triggers: Vec<Vec<String>>,
    door_ways: Vec<Vec<usize>>,
    all_fired: Vec<String>,
}

fn aliases(p: &crate::player::Player, buttons: &[Vec<String>]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let keys = crate::player::door_keys(&p.vehicle.ty);
    let doors = if buttons.is_empty() { &keys[..] } else { buttons };
    for (n, group) in doors.iter().enumerate() {
        for t in group.iter().flat_map(|g| g.split('|')) {
            out.push((format!("door_{}", n + 1), t.to_string()));
            out.push(("doors_all".into(), t.to_string()));
        }
    }
    for (action, names) in crate::player::ACTION_ALIASES {
        for n in *names {
            out.push((action.to_string(), n.to_string()));
        }
    }
    out
}

fn ways(p: &crate::player::Player) -> Vec<Vec<usize>> {
    let mut ways = omsi_remote::doorways(&p.vehicle.ty);
    for t in &p.vehicle.trailers {
        ways.extend(omsi_remote::doorways(&t.ty));
    }
    ways
}

fn doors(p: &crate::player::Player) -> (Vec<Vec<usize>>, Vec<Vec<String>>) {
    let ty = &p.vehicle.ty;
    let ways = ways(p);
    let keys = crate::player::door_keys(ty).len();
    let fronts: Vec<String> = (0..ways.len()).map(|k| format!("bus_doorfront{k}")).collect();
    let direct = keys < ways.len() && fronts.iter().all(|f| ty.program.trigger(f).is_some());
    match (direct, keys == 0 || ways.len() == keys) {
        (true, _) => (ways, fronts.into_iter().map(|f| vec![f]).collect()),
        (false, true) => (ways, Vec::new()),
        _ => (Vec::new(), Vec::new()),
    }
}

fn try_doors(p: &crate::player::Player, key: &str) -> Doors {
    let (tx, rx) = std::sync::mpsc::channel();
    let (ty, ways, key) = (p.vehicle.ty.clone(), omsi_remote::doorways(&p.vehicle.ty), key.to_string());
    let _ = std::thread::Builder::new().name("remote cockpit doors".into()).spawn(move || {
        let tried = omsi_remote::tried_doors(ty, &ways);
        log::info!("remote cockpit: door buttons {:?}", tried.iter().map(|(t, l)| format!("{} = {l:?}", t.join(" + "))).collect::<Vec<_>>());
        let _ = tx.send((key, tried));
    });
    rx
}

fn direct_doors(name: &str, down: bool, r: &mut RemoteCockpit, v: &omsi_sim::VehicleInstance) -> Option<Vec<String>> {
    if r.door_triggers.is_empty() {
        return None;
    }
    let name = name.to_ascii_lowercase();
    if name == "doors_all" {
        if !down {
            return Some(std::mem::take(&mut r.all_fired));
        }
        if !r.all_fired.is_empty() {
            return Some(Vec::new());
        }
        let open: Vec<bool> = r.door_ways.iter().map(|w| w.iter().any(|l| v.var(&format!("door_{l}")).is_some_and(|x| x > 0.05))).collect();
        let any = open.contains(&true);
        r.all_fired = r.door_triggers.iter().zip(&open).filter(|(_, o)| !any || **o).flat_map(|(t, _)| t.clone()).filter(|t| t != "bus_dooraft").collect();
        return Some(r.all_fired.clone());
    }
    let n: usize = name.strip_prefix("door_")?.parse().ok()?;
    r.door_triggers.get(n.checked_sub(1)?).cloned()
}

fn start_skins(p: &crate::player::Player, root: &std::path::Path, key: &str) -> Skins {
    let (tx, rx) = std::sync::mpsc::channel();
    let (ty, root, key) = (p.vehicle.ty.clone(), root.to_path_buf(), key.to_string());
    let scheme = p.vehicle.host.paint_scheme.flatten();
    let _ = std::thread::Builder::new().name("remote cockpit buttons".into()).spawn(move || {
        let skins = omsi_remote::extract_skins(&ty, &root, scheme);
        log::info!("remote cockpit: {} pictures of the bus's own buttons", skins.len());
        let _ = tx.send((key, skins));
    });
    rx
}

fn vehicle_key(p: &crate::player::Player) -> String {
    let path = &p.vehicle.ty.def.path;
    let folder = path.parent().and_then(|d| d.file_name()).map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
    let file = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let hof = p.vehicle.host.hof.as_ref().map(|h| h.name.trim().to_string()).unwrap_or_default();
    format!("{folder}/{file}#{hof}")
}

fn vehicle(p: &crate::player::Player, doors: Vec<Vec<usize>>, buttons: &[Vec<String>]) -> Vehicle {
    let def = &p.vehicle.ty.def;
    let short = omsi_launcher_lib::vehicle_type_label(&def.type_name, &def.path);
    let mut actions = p.vehicle.ty.program.trigger_names();
    actions.extend(p.bound_actions());
    actions.sort_by_key(|a| a.to_ascii_lowercase());
    actions.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    let key = vehicle_key(p);
    Vehicle {
        key: key.split('#').next().unwrap_or("").to_string(),
        name: omsi_launcher_lib::display_bus_name(&format!("{} {short}", def.manufacturer)),
        folder: def.path.parent().map(std::path::Path::to_path_buf).unwrap_or_default(),
        actions,
        aliases: aliases(p, buttons),
        depot: p.vehicle.host.hof.as_deref().map(omsi_sim::vehicle_api::depot),
        doors,
    }
}

impl RemoteCockpit {
    pub(crate) fn failed(&self) -> bool {
        self.failed_port.is_some()
    }

    pub(crate) fn forget_failure(&mut self) {
        self.failed_port = None;
    }
}

pub(crate) fn lan_ip() -> String {
    use omsi_net::addrs::AddrKind;
    let addrs = omsi_net::addrs::joinable_addresses();
    let best = addrs.iter().find(|a| a.kind == AddrKind::Lan).or_else(|| addrs.iter().find(|a| !a.kind.is_vpn())).or(addrs.first());
    best.map(|a| a.ip.to_string()).unwrap_or_else(|| "127.0.0.1".into())
}

pub(crate) fn dir() -> Option<std::path::PathBuf> {
    crate::settings::Settings::path().and_then(|p| p.parent().map(|d| d.join("remote")))
}

impl App {
    pub(crate) fn remote_description(&self) -> String {
        match self.integrations.remote.server.as_ref() {
            Some(r) => {
                let url = format!("http://{}:{}", lan_ip(), r.port());
                format!("Open {url} on a phone or tablet of the same network")
            }
            None if self.integrations.remote.failed_port.is_some() => format!("Port {} is taken by another program", self.settings.remote_cockpit_port),
            None => "The bus's switches, its destination sign and a dashboard on a phone or tablet of the same network".into(),
        }
    }

    fn remote_stop(&mut self) {
        if let Some(server) = self.integrations.remote.server.take() {
            for c in server.take_commands() {
                self.remote_command(c);
            }
            for name in server.release_all() {
                self.remote_command(Command::Action { name, down: false });
            }
        }
        if let Some(n) = self.menus.navigator.as_mut() {
            n.remote = false;
            n.forget_remote_area();
        }
    }

    fn remote_switch(&mut self) {
        let wanted = self.settings.remote_cockpit && self.args.server.is_none();
        let port = self.settings.remote_cockpit_port;
        if !wanted || self.integrations.remote.server.as_ref().is_some_and(|s| s.port() != port) {
            self.remote_stop();
        }
        let r = &mut self.integrations.remote;
        if !wanted {
            r.failed_port = None;
            return;
        }
        if r.server.is_some() || r.failed_port == Some(port) {
            return;
        }
        let Some(dir) = dir() else { return };
        match Remote::start(port, dir) {
            Ok(s) => {
                r.server = Some(s);
                r.failed_port = None;
                r.vehicle.clear();
                r.lang.clear();
            }
            Err(e) => {
                log::warn!("remote cockpit: port {port}: {e}");
                r.failed_port = Some(port);
            }
        }
    }

    pub(crate) fn remote_input(&mut self) {
        let Some(server) = self.integrations.remote.server.as_ref() else { return };
        for c in server.take_commands() {
            self.remote_command(c);
        }
        let Some(server) = self.integrations.remote.server.as_ref() else { return };
        if let (Some(d), Some(p), false) = (server.drive(), self.player.as_mut(), self.paused) {
            let a = &mut p.analog;
            a.steering = Some(d.steering);
            a.stick = false;
            a.throttle = Some(a.throttle.unwrap_or(0.0).max(d.throttle));
            a.brake = Some(a.brake.unwrap_or(0.0).max(d.brake));
        }
    }

    fn remote_command(&mut self, c: Command) {
        if let Command::Game(name) = &c {
            if omsi_input::actions::is_game_action(name) {
                self.game_action(name);
            }
            return;
        }
        if let Command::Indicator(v @ 1..=3) = c {
            self.blinker(v);
            return;
        }
        if let (Command::Action { name, down }, Some(p)) = (&c, self.player.as_mut()) {
            let r = &mut self.integrations.remote;
            if let Some(fire) = direct_doors(name, *down, r, &p.vehicle) {
                for t in fire {
                    p.action(&t, *down);
                }
                return;
            }
        }
        let duty = self.session.duty.as_ref().map(|d| d.trip_for_ibis());
        let Some(p) = self.player.as_mut() else { return };
        let request = match c {
            Command::Action { name, down } => {
                match omsi_sim::engine_action(&name) {
                    Some(a) => p.axes.set(a, down),
                    None => {
                        p.action(&name, down);
                    }
                }
                return;
            }
            Command::StartUp => {
                self.service_msg = Some((p.start_up(), 6.0));
                if let Some((trip, stop)) = duty {
                    p.set_duty_destination(trip, stop);
                }
                return;
            }
            Command::SaloonLights => {
                self.service_msg = Some((p.toggle_saloon_lights(), 3.0));
                return;
            }
            Command::Route(i) => HtmlRequest::SetRoute(i),
            Command::Line(l) => HtmlRequest::SetLine(l),
            Command::Destination(i) => HtmlRequest::SetDestination(i),
            Command::ClearLine => HtmlRequest::ClearLine,
            Command::NextStop(i) => HtmlRequest::SetNextStop(i),
            Command::Indicator(_) => {
                p.blinker_key_state = 0;
                p.action("blinker_off", true);
                p.action("blinker_off", false);
                return;
            }
            Command::Game(_) => return,
        };
        if p.vehicle.host.html_requests.len() < 32 {
            p.vehicle.host.html_requests.push(request);
        }
    }

    pub(crate) fn frame_remote(&mut self, dt: f32) {
        self.remote_switch();
        let paused = self.paused;
        let r = &mut self.integrations.remote;
        let Some(server) = r.server.as_mut() else { return };
        server.set_pairing(self.menus.game_panels.remote.is_some());
        if server.take_paired() && self.menus.game_panels.remote.take().is_some() {
            self.service_msg = Some((omsi_ui::tr("Phone or tablet connected").into_owned(), 4.0));
        }
        let lang = omsi_sim::vehicle_api::locale();
        if r.lang != lang {
            let texts: Vec<(&str, String)> = omsi_remote::TEXTS.iter().map(|k| (*k, omsi_ui::tr(k).into_owned())).collect();
            server.set_texts(&lang, &texts);
            r.lang = lang;
        }
        let watched = server.clients() > 0;
        if let Some(n) = self.menus.navigator.as_mut() {
            n.remote = watched;
        }
        if !watched {
            return;
        }
        r.nav_t -= dt;
        if let (Some(n), Some(p), true) = (self.menus.navigator.as_mut(), self.player.as_ref(), r.nav_t <= 0.0) {
            r.nav_t = 0.4;
            if let Some((anchor, roads)) = n.remote_roads(p.vehicle.position) {
                server.publish("roads", omsi_remote::nav::roads_message(anchor, &roads));
            }
            if let Some(info) = n.remote_info() {
                server.publish("nav", omsi_remote::nav::nav_message(&info));
            }
        }
        let key = self.player.as_ref().map(vehicle_key).unwrap_or_default();
        if r.vehicle != key {
            let _ = server.release_all();
            r.all_fired.clear();
            (r.door_ways, r.door_triggers) = self.player.as_ref().map(doors).unwrap_or_default();
            server.set_vehicle(self.player.as_ref().map(|p| vehicle(p, r.door_ways.clone(), &r.door_triggers)));
            r.skins = self.player.as_ref().map(|p| start_skins(p, &self.args.root, &key));
            r.tried = self.player.as_ref().map(|p| try_doors(p, &key));
            r.sign = self.player.as_ref().and_then(|p| omsi_remote::sign::Sign::find(&p.vehicle.ty, &self.args.root));
            r.vehicle = key;
        }
        let ready = !server.holds_a_door() && self.player.as_ref().is_some_and(|p| p.door_buttons.is_empty());
        if let (true, Some(Ok((for_key, tried))), Some(p)) = (ready, r.tried.as_ref().map(|rx| rx.try_recv()), self.player.as_ref()) {
            r.tried = None;
            if for_key == r.vehicle && !tried.is_empty() {
                (r.door_triggers, r.door_ways) = tried.into_iter().unzip();
                server.set_doors(r.door_ways.clone(), aliases(p, &r.door_triggers));
            }
        }
        if let Some(Ok((for_key, skins))) = r.skins.as_ref().map(|rx| rx.try_recv()) {
            if for_key == r.vehicle {
                server.set_skins(skins);
            }
            r.skins = None;
        }
        r.sign_t -= dt;
        if let (Some(sign), Some(p), true) = (r.sign.as_mut(), self.player.as_ref(), r.sign_t <= 0.0) {
            r.sign_t = 0.25;
            if let Some(png) = sign.render(&p.vehicle) {
                server.set_sign(png);
            }
        }
        r.publish_t -= dt;
        if r.publish_t > 0.0 {
            return;
        }
        r.publish_t = PUBLISH_EVERY;
        match self.player.as_ref() {
            Some(p) => server.set_state(Some(&p.vehicle.html_api_snapshot()), &p.vehicle.html_env_snapshot(), paused),
            None => server.set_state(None, &omsi_sim::vehicle_api::environment(&self.clock, &omsi_sim::vehicle_api::locale()), paused),
        }
    }
}


