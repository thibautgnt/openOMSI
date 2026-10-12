use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const LOCK_AFTER: usize = 5;
const LOCK_WINDOW: Duration = Duration::from_secs(120);
const MAX_DEVICES: usize = 32;

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    SystemRandom::new().fill(&mut b).expect("the system's random numbers");
    b
}

pub(crate) fn new_pin() -> String {
    let n = u32::from_le_bytes(random_bytes::<4>()) % 1_000_000;
    format!("{n:06}")
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

pub(crate) fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut d = (a.len() != b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        d |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    d == 0
}

pub(crate) struct Devices {
    path: PathBuf,
    known: Vec<(String, String)>,
    failures: Vec<Instant>,
}

pub(crate) enum Pairing {
    Paired(String),
    Wrong,
    Locked,
}

impl Devices {
    pub(crate) fn load(path: PathBuf) -> Devices {
        let known = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| {
                let (h, name) = l.split_once('\t').unwrap_or((l, ""));
                (h.len() == 64 && h.bytes().all(|c| c.is_ascii_hexdigit())).then(|| (h.to_string(), name.to_string()))
            })
            .collect();
        Devices { path, known, failures: Vec::new() }
    }

    fn save(&self) {
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let text: String = self.known.iter().map(|(h, n)| format!("{h}\t{n}\n")).collect();
        if let Err(e) = std::fs::write(&self.path, text) {
            log::warn!("remote cockpit: cannot write {}: {e}", self.path.display());
        }
    }

    pub(crate) fn knows(&self, token: &str) -> bool {
        let h = hash(token);
        self.known.iter().fold(false, |found, (k, _)| found | same(k, &h))
    }

    fn locked(&mut self) -> bool {
        self.failures.retain(|t| t.elapsed() < LOCK_WINDOW);
        self.failures.len() >= LOCK_AFTER
    }

    pub(crate) fn pair(&mut self, pin: &str, wanted: &str, name: &str) -> Pairing {
        if self.locked() {
            return Pairing::Locked;
        }
        if !same(pin, wanted.trim()) {
            self.failures.push(Instant::now());
            return Pairing::Wrong;
        }
        let token = hex(&random_bytes::<32>());
        let name: String = name.chars().filter(|c| !c.is_control()).take(48).collect();
        self.known.push((hash(&token), name));
        if self.known.len() > MAX_DEVICES {
            self.known.remove(0);
        }
        self.save();
        Pairing::Paired(token)
    }

    pub(crate) fn forget_all(&mut self) {
        self.known.clear();
        self.save();
    }
}
