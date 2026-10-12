"use strict";

const Drive = {
  steer: 0,
  throttle: 0,
  brake: 0,
  wheels: new Set(),
  fingers: new Map(),
  lock: 270,
  timer: null,
  quiet: 0,
  last: 0,
  sent: null,

  holding() {
    for (const f of this.fingers.values()) if (f.kind === "wheel") return true;
    return false;
  },

  wake() {
    if (this.timer) return;
    this.last = performance.now();
    this.quiet = 0;
    this.timer = setInterval(() => this.tick(), 16);
  },

  tick() {
    const now = performance.now();
    const dt = Math.min(0.1, (now - this.last) / 1000);
    this.last = now;
    if (!this.holding() && this.steer !== 0) {
      const kmh = Math.abs(Number(R.get(App.state, "vehicle.motion.speedKmh")) || 0);
      const back = Math.min(1, kmh / 25) * 1.6 * dt;
      this.steer = Math.abs(this.steer) <= back ? 0 : this.steer - Math.sign(this.steer) * back;
    }
    const msg = { t: "drive", s: Math.round(this.steer * 1000) / 1000, g: Math.round(this.throttle * 1000) / 1000, b: Math.round(this.brake * 1000) / 1000 };
    const moved = !this.sent || msg.s !== this.sent.s;
    const busy = this.holding() || this.throttle > 0 || this.brake > 0 || moved;
    this.quiet = busy ? 0 : this.quiet + 1;
    if (App.ws && App.ws.readyState === 1 && this.quiet < 6) {
      App.send(msg);
      this.sent = msg;
    }
    for (const w of this.wheels) {
      if (w.isConnected) this.paint(w);
      else this.wheels.delete(w);
    }
    if (this.quiet >= 6) {
      clearInterval(this.timer);
      this.timer = null;
    }
  },

  paint(node) {
    const deg = this.steer * (Number(node.dataset.lock) || this.lock);
    node.querySelector(".rim").style.transform = `rotate(${deg}deg)`;
    node.querySelector(".deg").textContent = `${Math.round(deg)}°`;
  },

  release(id) {
    const f = this.fingers.get(id);
    if (!f) return;
    this.fingers.delete(id);
    if (f.kind !== "wheel") {
      this[f.kind] = 0;
      if (f.node) f.node.style.setProperty("--p", 0);
    }
    this.wake();
  },

  letGo() {
    this.fingers.clear();
    this.throttle = 0;
    this.brake = 0;
    for (const p of document.querySelectorAll(".pedal")) p.style.setProperty("--p", 0);
    this.sent = null;
    this.wake();
  },

  grab(e, node, kind) {
    if (document.body.classList.contains("editing")) return false;
    for (const f of this.fingers.values()) if (f.node === node) return false;
    e.preventDefault();
    this.fingers.set(e.pointerId, { kind, node, angle: 0 });
    try { node.setPointerCapture(e.pointerId); } catch (_) {}
    return true;
  },

  wheel(w) {
    const d = R.el("div", "wheel");
    d.dataset.lock = String(w.max || this.lock);
    d.innerHTML = '<svg class="rim" viewBox="0 0 200 200"><circle cx="100" cy="100" r="88" fill="none" stroke-width="16"/><circle class="top" cx="100" cy="12" r="5"/><path class="spoke" d="M24 112 L76 104 L100 128 L124 104 L176 112" fill="none" stroke-width="14" stroke-linejoin="round"/><circle class="hub" cx="100" cy="104" r="24"/></svg><span class="deg">0°</span>';
    const at = (e) => {
      const r = d.getBoundingClientRect();
      return Math.atan2(e.clientY - (r.top + r.height / 2), e.clientX - (r.left + r.width / 2));
    };
    d.addEventListener("pointerdown", (e) => {
      if (!this.grab(e, d, "wheel")) return;
      this.fingers.get(e.pointerId).angle = at(e);
      this.wake();
    });
    d.addEventListener("pointermove", (e) => {
      const f = this.fingers.get(e.pointerId);
      if (!f || f.node !== d) return;
      const a = at(e);
      let da = a - f.angle;
      if (da > Math.PI) da -= 2 * Math.PI;
      if (da < -Math.PI) da += 2 * Math.PI;
      f.angle = a;
      const lock = ((Number(d.dataset.lock) || this.lock) * Math.PI) / 180;
      this.steer = Math.max(-1, Math.min(1, this.steer + da / lock));
      this.wake();
    });
    d.addEventListener("contextmenu", (e) => e.preventDefault());
    this.wheels.add(d);
    this.paint(d);
    return d;
  },

  pedals() {
    const d = R.el("div", "pedals");
    for (const [k, label] of [["brake", "Brake"], ["throttle", "Throttle"]]) {
      const p = R.el("div", "pedal " + k);
      p.append(R.el("div", "fill"), R.el("span", "txt", R.T(label)));
      const set = (e) => {
        const r = p.getBoundingClientRect();
        const v = Math.max(0, Math.min(1, ((e.clientY - r.top) / Math.max(1, r.height)) * 1.15 - 0.05));
        this[k] = v;
        p.style.setProperty("--p", v);
        this.wake();
      };
      p.addEventListener("pointerdown", (e) => {
        if (!this.grab(e, p, k)) return;
        R.buzz();
        set(e);
      });
      p.addEventListener("pointermove", (e) => {
        const f = this.fingers.get(e.pointerId);
        if (f && f.node === p) set(e);
      });
      p.addEventListener("contextmenu", (e) => e.preventDefault());
      d.append(p);
    }
    return d;
  },
};

for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) document.addEventListener(type, (e) => Drive.release(e.pointerId), true);
document.addEventListener("visibilitychange", () => { if (document.hidden) Drive.letGo(); });
