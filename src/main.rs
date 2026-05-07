//Use Alacritty for the best experience.
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

const BASE_SCALE: f64 = 4.0;
const CHAR_ASPECT: f64 = 0.5;

// Planeten
struct PlaneetDef {
    naam:           &'static str,
    grote_as_au:    f64,
    excentriciteit: f64,
    omloop_dagen:   f64,
    straal_km:      f64,
    glyph:          char,
    kleur:          Color,
    begin_hoek:     f64,
}

static PLANETEN: &[PlaneetDef] = &[
    PlaneetDef { naam:"Mercurius", grote_as_au:0.387,  excentriciteit:0.206, omloop_dagen:87.97,      straal_km:2_439.0,  glyph:'*', kleur:Color::Rgb{r:160,g:140,b:130}, begin_hoek:0.8 },
    PlaneetDef { naam:"Venus",     grote_as_au:0.723,  excentriciteit:0.007, omloop_dagen:224.70,     straal_km:6_052.0,  glyph:'*', kleur:Color::Rgb{r:230,g:210,b:130}, begin_hoek:2.1 },
    PlaneetDef { naam:"Aarde",     grote_as_au:1.000,  excentriciteit:0.017, omloop_dagen:365.25,     straal_km:6_371.0,  glyph:'*', kleur:Color::Rgb{r:80,g:160,b:240},  begin_hoek:4.0 },
    PlaneetDef { naam:"Mars",      grote_as_au:1.524,  excentriciteit:0.093, omloop_dagen:686.97,     straal_km:3_390.0,  glyph:'*', kleur:Color::Rgb{r:220,g:100,b:60},  begin_hoek:1.0 },
    PlaneetDef { naam:"Jupiter",   grote_as_au:5.203,  excentriciteit:0.049, omloop_dagen:4_332.59,   straal_km:71_492.0, glyph:'O', kleur:Color::Rgb{r:210,g:175,b:130}, begin_hoek:3.5 },
    PlaneetDef { naam:"Saturnus",  grote_as_au:9.537,  excentriciteit:0.057, omloop_dagen:10_759.22,  straal_km:60_268.0, glyph:'O', kleur:Color::Rgb{r:220,g:205,b:155}, begin_hoek:5.2 },
    PlaneetDef { naam:"Uranus",    grote_as_au:19.191, excentriciteit:0.046, omloop_dagen:30_688.50,  straal_km:25_559.0, glyph:'o', kleur:Color::Rgb{r:150,g:230,b:235}, begin_hoek:0.3 },
    PlaneetDef { naam:"Neptunus",  grote_as_au:30.069, excentriciteit:0.010, omloop_dagen:60_182.00,  straal_km:24_764.0, glyph:'o', kleur:Color::Rgb{r:80,g:120,b:230},  begin_hoek:2.8 },
];

// ── Dwergplaneten ─────────────────────────────────────────────────────────────
struct DwergPlaneetDef {
    naam:           &'static str,
    grote_as_au:    f64,
    excentriciteit: f64,
    omloop_dagen:   f64,
    kleur:          Color,
    begin_hoek:     f64,
}

static DWERGPLANETEN: &[DwergPlaneetDef] = &[
    DwergPlaneetDef { naam:"Ceres",    grote_as_au:2.767,  excentriciteit:0.076, omloop_dagen:1_681.63,  kleur:Color::Rgb{r:160,g:150,b:140}, begin_hoek:1.3 },
    DwergPlaneetDef { naam:"Pluto",    grote_as_au:39.482, excentriciteit:0.249, omloop_dagen:90_560.0,  kleur:Color::Rgb{r:210,g:180,b:140}, begin_hoek:3.7 },
    DwergPlaneetDef { naam:"Eris",     grote_as_au:67.864, excentriciteit:0.436, omloop_dagen:204_199.0, kleur:Color::Rgb{r:200,g:200,b:200}, begin_hoek:0.6 },
    DwergPlaneetDef { naam:"Makemake", grote_as_au:45.430, excentriciteit:0.159, omloop_dagen:111_690.0, kleur:Color::Rgb{r:220,g:160,b:110}, begin_hoek:5.1 },
    DwergPlaneetDef { naam:"Haumea",   grote_as_au:43.335, excentriciteit:0.191, omloop_dagen:103_774.0, kleur:Color::Rgb{r:200,g:210,b:220}, begin_hoek:2.2 },
];

// Manen
// baan om de planeet in AU, omloop in dagen
struct MaanDef {
    naam:            &'static str,
    planeet_idx:     usize,   // index in PLANETEN
    baan_au:         f64,     // straal van de baan om de planeet
    omloop_dagen:    f64,
    kleur:           Color,
    begin_hoek:      f64,
}

static MANEN: &[MaanDef] = &[
    // Aarde (idx 2)
    MaanDef { naam:"Maan",     planeet_idx:2, baan_au:0.00257, omloop_dagen:27.32,  kleur:Color::Rgb{r:180,g:180,b:180}, begin_hoek:0.0 },
    // Mars (idx 3)
    MaanDef { naam:"Phobos",   planeet_idx:3, baan_au:0.0000627, omloop_dagen:0.319, kleur:Color::Rgb{r:150,g:120,b:100}, begin_hoek:1.0 },
    MaanDef { naam:"Deimos",   planeet_idx:3, baan_au:0.000157,  omloop_dagen:1.263, kleur:Color::Rgb{r:140,g:115,b:95},  begin_hoek:3.0 },
    // Jupiter (idx 4)
    MaanDef { naam:"Io",       planeet_idx:4, baan_au:0.002819, omloop_dagen:1.769,  kleur:Color::Rgb{r:230,g:200,b:80},  begin_hoek:0.5 },
    MaanDef { naam:"Europa",   planeet_idx:4, baan_au:0.004486, omloop_dagen:3.551,  kleur:Color::Rgb{r:200,g:170,b:130}, begin_hoek:2.0 },
    MaanDef { naam:"Ganymede", planeet_idx:4, baan_au:0.007155, omloop_dagen:7.155,  kleur:Color::Rgb{r:160,g:150,b:130}, begin_hoek:4.0 },
    MaanDef { naam:"Callisto", planeet_idx:4, baan_au:0.012585, omloop_dagen:16.69,  kleur:Color::Rgb{r:120,g:110,b:100}, begin_hoek:1.5 },
    // Saturnus (idx 5)
    MaanDef { naam:"Titan",    planeet_idx:5, baan_au:0.008168, omloop_dagen:15.945, kleur:Color::Rgb{r:220,g:180,b:100}, begin_hoek:0.8 },
    MaanDef { naam:"Enceladus",planeet_idx:5, baan_au:0.001590, omloop_dagen:1.370,  kleur:Color::Rgb{r:230,g:240,b:255}, begin_hoek:2.5 },
    MaanDef { naam:"Rhea",     planeet_idx:5, baan_au:0.003524, omloop_dagen:4.518,  kleur:Color::Rgb{r:190,g:180,b:170}, begin_hoek:4.2 },
    // Uranus (idx 6)
    MaanDef { naam:"Titania",  planeet_idx:6, baan_au:0.002917, omloop_dagen:8.706,  kleur:Color::Rgb{r:140,g:170,b:180}, begin_hoek:1.1 },
    MaanDef { naam:"Oberon",   planeet_idx:6, baan_au:0.003903, omloop_dagen:13.46,  kleur:Color::Rgb{r:130,g:155,b:165}, begin_hoek:3.3 },
    // Neptunus (idx 7)
    MaanDef { naam:"Triton",   planeet_idx:7, baan_au:0.002371, omloop_dagen:5.877,  kleur:Color::Rgb{r:100,g:140,b:200}, begin_hoek:2.8 },
];

const ZON_STRAAL_KM: f64 = 696_000.0;

// dubbele buffer
#[derive(Clone, PartialEq)]
struct Cel { teken: char, fg: Color, vet: bool }
impl Default for Cel {
    fn default() -> Self { Cel { teken: ' ', fg: Color::Reset, vet: false } }
}

struct Scherm {
    breedte: usize,
    hoogte:  usize,
    voor:    Vec<Cel>,
    achter:  Vec<Cel>,
}

impl Scherm {
    fn nieuw(w: u16, h: u16) -> Self {
        let n = w as usize * h as usize;
        Scherm {
            breedte: w as usize, hoogte: h as usize,
            voor:   vec![Cel { teken: '~', ..Default::default() }; n],
            achter: vec![Default::default(); n],
        }
    }
    fn resize(&mut self, w: u16, h: u16) {
        self.breedte = w as usize; self.hoogte = h as usize;
        let n = self.breedte * self.hoogte;
        self.voor   = vec![Cel { teken: '~', ..Default::default() }; n];
        self.achter = vec![Default::default(); n];
    }
    fn wis(&mut self) { for c in &mut self.achter { *c = Default::default(); } }

    #[inline]
    fn zet(&mut self, x: i64, y: i64, teken: char, fg: Color, vet: bool) {
        if x < 0 || y < 0 { return; }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.breedte || uy >= self.hoogte { return; }
        self.achter[uy * self.breedte + ux] = Cel { teken, fg, vet };
    }

    fn flush<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let mut huidig_vet = false;
        for idx in 0..self.voor.len() {
            let a = &self.achter[idx];
            if a == &self.voor[idx] { continue; }
            let x = (idx % self.breedte) as u16;
            let y = (idx / self.breedte) as u16;
            queue!(out, cursor::MoveTo(x, y))?;
            if a.vet && !huidig_vet {
                queue!(out, SetAttribute(Attribute::Bold))?;
                huidig_vet = true;
            } else if !a.vet && huidig_vet {
                queue!(out, SetAttribute(Attribute::Reset))?;
                huidig_vet = false;
            }
            queue!(out, SetForegroundColor(a.fg), Print(a.teken))?;
            self.voor[idx] = a.clone();
        }
        if huidig_vet { queue!(out, SetAttribute(Attribute::Reset))?; }
        queue!(out, ResetColor)?;
        out.flush()
    }
}

// wiskunde
fn excentrieke_anomalie(ma: f64, e: f64) -> f64 {
    let mut ea = ma;
    for _ in 0..60 {
        let d = (ma - (ea - e * ea.sin())) / (1.0 - e * ea.cos());
        ea += d;
        if d.abs() < 1e-11 { break; }
    }
    ea
}

// positie van een lichaam op een Kepleriaanse baan, geeft (x,y) in AU
fn baan_positie(grote_as: f64, exc: f64, periode: f64, begin_hoek: f64, sim_dagen: f64) -> (f64, f64) {
    use std::f64::consts::PI;
    let mm = 2.0 * PI / periode;
    let ma = (mm * sim_dagen + begin_hoek).rem_euclid(2.0 * PI);
    let ea = excentrieke_anomalie(ma, exc);
    let ta = 2.0 * f64::atan2(
        ((1.0 + exc).sqrt()) * (ea / 2.0).sin(),
        ((1.0 - exc).sqrt()) * (ea / 2.0).cos(),
    );
    let r = grote_as * (1.0 - exc.powi(2)) / (1.0 + exc * ta.cos());
    (r * ta.cos(), r * ta.sin())
}

fn planeet_3d(def: &PlaneetDef, sim_dagen: f64) -> (f64, f64, f64) {
    let (x, y) = baan_positie(def.grote_as_au, def.excentriciteit, def.omloop_dagen, def.begin_hoek, sim_dagen);
    (x, y, 0.0)
}

fn dwerg_3d(def: &DwergPlaneetDef, sim_dagen: f64) -> (f64, f64, f64) {
    let (x, y) = baan_positie(def.grote_as_au, def.excentriciteit, def.omloop_dagen, def.begin_hoek, sim_dagen);
    (x, y, 0.0)
}

// maan-positie = planeet-positie + baan om de planeet
fn maan_3d(maan: &MaanDef, sim_dagen: f64) -> (f64, f64, f64) {
    let planeet = &PLANETEN[maan.planeet_idx];
    let (px, py) = baan_positie(planeet.grote_as_au, planeet.excentriciteit,
                                 planeet.omloop_dagen, planeet.begin_hoek, sim_dagen);
    // maan heeft een cirkelvormige baan (excentriciteit ≈ 0 voor de grote manen)
    let (mx, my) = baan_positie(maan.baan_au, 0.0, maan.omloop_dagen, maan.begin_hoek, sim_dagen);
    (px + mx, py + my, 0.0)
}

fn projecteer(
    wx: f64, wy: f64, wz: f64,
    yaw: f64, pitch: f64,
    schaal: f64,
    scroll_x: f64, scroll_y: f64,
    cx: f64, cy: f64,
) -> (f64, f64, f64) {
    let (sy, cy2) = yaw.sin_cos();
    let rx1 =  wx * cy2 + wy * sy;
    let ry1 = -wx * sy  + wy * cy2;
    let rz1 =  wz;
    let (sp, cp) = pitch.sin_cos();
    let rx2 = rx1;
    let ry2 = ry1 * cp - rz1 * sp;
    let rz2 = ry1 * sp + rz1 * cp;
    (
        rx2 * schaal + cx - scroll_x,
        ry2 * schaal * CHAR_ASPECT + cy - scroll_y,
        rz2,
    )
}

fn straal_cellen(straal_km: f64, schaal: f64) -> i64 {
    let basis = ((straal_km.ln() - 7000_f64.ln())
                 / (ZON_STRAAL_KM.ln() - 7000_f64.ln()) * 4.0).max(0.0);
    ((basis * (schaal / BASE_SCALE).sqrt()).round() as i64).max(0)
}

// tekenhulpies
fn teken_cirkel(
    scherm: &mut Scherm,
    cx: f64, cy: f64, radius: i64,
    glyph: char, kleur: Color, vet: bool, usable_h: i64,
) {
    let r = radius.max(0);
    if r == 0 { scherm.zet(cx as i64, cy as i64, glyph, kleur, vet); return; }
    for dy in -r..=r {
        for dx in -(r * 2)..=(r * 2) {
            if (dx as f64 / 2.0).powi(2) + (dy as f64).powi(2) <= (r as f64 + 0.5).powi(2) {
                let sy = cy as i64 + dy;
                if sy < usable_h { scherm.zet(cx as i64 + dx, sy, glyph, kleur, vet); }
            }
        }
    }
}

fn teken_baan(
    scherm: &mut Scherm,
    grote_as: f64, exc: f64,
    yaw: f64, pitch: f64, schaal: f64,
    scroll_x: f64, scroll_y: f64,
    cx: f64, cy: f64, usable_h: i64,
    zon_sx: f64, zon_sy: f64, zon_r: i64,
    kleur: Color,
) {
    use std::f64::consts::PI;
    let a  = grote_as;
    let b  = a * (1.0 - exc.powi(2)).sqrt();
    let fc = a * exc;
    let stappen = ((a * schaal * 2.0 * PI) as usize).clamp(120, 2000);
    let skip = (stappen / 600 + 1).max(1);
    let skip_r = (zon_r + 1) as f64;
    for i in (0..stappen).step_by(skip) {
        let theta = 2.0 * PI * i as f64 / stappen as f64;
        let (sx, sy, _) = projecteer(
            a * theta.cos() - fc, b * theta.sin(), 0.0,
            yaw, pitch, schaal, scroll_x, scroll_y, cx, cy,
        );
        if sy >= 0.0 && sy < usable_h as f64 {
            let dx = (sx - zon_sx) * 0.5;
            let dy = sy - zon_sy;
            if dx * dx + dy * dy < skip_r * skip_r { continue; }
            scherm.zet(sx as i64, sy as i64, '.', kleur, false);
        }
    }
}

// asteroidengordel — stochastische stipjes tussen 2.2 en 3.2 AU
fn teken_asteroiden(
    scherm: &mut Scherm,
    yaw: f64, pitch: f64, schaal: f64,
    scroll_x: f64, scroll_y: f64,
    cx: f64, cy: f64, usable_h: i64,
    zon_sx: f64, zon_sy: f64, zon_r: i64,
) {
    use std::f64::consts::PI;
    let skip_r = (zon_r + 1) as f64;
    // 300 willekeurige asteroiden, vaste seed
    for i in 0usize..300 {
        let h1 = i.wrapping_mul(2246822519).wrapping_add(1013904223) as u64;
        let h2 = i.wrapping_mul(2654435761).wrapping_add(22695477) as u64;
        let au = 2.2 + ((h1 & 0xFFFF) as f64 / 65535.0) * 1.0; // 2.2–3.2 AU
        let theta = ((h2 & 0xFFFF) as f64 / 65535.0) * 2.0 * PI;
        let wx = au * theta.cos();
        let wy = au * theta.sin();
        let (sx, sy, _) = projecteer(wx, wy, 0.0, yaw, pitch, schaal,
                                      scroll_x, scroll_y, cx, cy);
        if sy >= 0.0 && sy < usable_h as f64 {
            let dx = (sx - zon_sx) * 0.5;
            let dy = sy - zon_sy;
            if dx * dx + dy * dy < skip_r * skip_r { continue; }
            scherm.zet(sx as i64, sy as i64, ',', Color::Rgb { r: 90, g: 80, b: 70 }, false);
        }
    }
}

// kuipergordel 2 AU breed van 30 tot 50 AU
fn teken_kuipergordel(
    scherm: &mut Scherm,
    yaw: f64, pitch: f64, schaal: f64,
    scroll_x: f64, scroll_y: f64,
    cx: f64, cy: f64, usable_h: i64,
) {
    use std::f64::consts::PI;
    for i in 0usize..400 {
        let h1 = i.wrapping_mul(1664525).wrapping_add(1013904223) as u64;
        let h2 = i.wrapping_mul(69069).wrapping_add(12345) as u64;
        let au = 30.0 + ((h1 & 0xFFFF) as f64 / 65535.0) * 20.0;
        let theta = ((h2 & 0xFFFF) as f64 / 65535.0) * 2.0 * PI;
        let wx = au * theta.cos();
        let wy = au * theta.sin();
        let (sx, sy, _) = projecteer(wx, wy, 0.0, yaw, pitch, schaal,
                                      scroll_x, scroll_y, cx, cy);
        if sy >= 0.0 && sy < usable_h as f64 {
            scherm.zet(sx as i64, sy as i64, '.', Color::Rgb { r: 50, g: 55, b: 70 }, false);
        }
    }
}

// sterren
struct Ster { x: f64, y: f64, helderheid: u8 }

fn genereer_sterren(n: usize) -> Vec<Ster> {
    (0..n).map(|i| {
        let hx = i.wrapping_mul(2654435761).wrapping_add(1013904223) as u64;
        let hy = i.wrapping_mul(1664525).wrapping_add(22695477) as u64;
        let hb = i.wrapping_mul(134775813).wrapping_add(1) as u64;
        Ster {
            x: ((hx & 0xFFFF) as f64 / 65535.0) * 4000.0 - 2000.0,
            y: ((hy & 0xFFFF) as f64 / 65535.0) * 4000.0 - 2000.0,
            helderheid: match hb % 4 { 0 => 65, 1 => 50, 2 => 40, _ => 30 },
        }
    }).collect()
}

fn teken_sterren(
    scherm: &mut Scherm, sterren: &[Ster],
    scroll_x: f64, scroll_y: f64,
    yaw: f64, pitch: f64,
    cx: f64, cy: f64, usable_h: i64,
) {
    let tw = scherm.breedte as f64;
    let th = usable_h as f64;
    for s in sterren.iter().step_by(3) {
        let (px, py, _) = projecteer(
            s.x, s.y, 0.0,
            yaw * 0.06, pitch * 0.06, 1.0,
            scroll_x * 0.03, scroll_y * 0.03,
            cx, cy,
        );
        let x = px.rem_euclid(tw * 1.4) - tw * 0.2;
        let y = py.rem_euclid(th * 1.4) - th * 0.2;
        if x < 0.0 || y < 0.0 || x >= tw || y >= th { continue; }
        let b = s.helderheid;
        scherm.zet(x as i64, y as i64, '.', Color::Rgb { r: b, g: b, b }, false);
    }
}

// application
struct App {
    scherm:       Scherm,
    scroll_x:     f64,
    scroll_y:     f64,
    zoom:         f64,
    yaw:          f64,
    pitch:        f64,
    snelheid:     f64,
    last_tick:    Instant,
    sim_dagen:    f64,
    toon_help:    bool,
    toon_legenda: bool,
    toon_manen:   bool,
    gepauzeerd:   bool,
    sterren:      Vec<Ster>,
}

impl App {
    fn nieuw() -> io::Result<Self> {
        let (w, h) = terminal::size()?;
        Ok(App {
            scherm:       Scherm::nieuw(w, h),
            scroll_x:     0.0, scroll_y: 0.0,
            zoom:         1.0, yaw: 0.0, pitch: 0.0,
            snelheid:     1.0,
            last_tick:    Instant::now(),
            sim_dagen:    0.0,
            toon_help:    true,
            toon_legenda: true,
            toon_manen:   true,
            gepauzeerd:   false,
            sterren:      genereer_sterren(900),
        })
    }

    fn update(&mut self) {
        let dt = self.last_tick.elapsed().as_secs_f64().min(0.1);
        self.last_tick = Instant::now();
        if !self.gepauzeerd { self.sim_dagen += dt * self.snelheid; }
    }

    fn toets(&mut self, key: KeyEvent) -> bool {
        let schaal      = BASE_SCALE * self.zoom;
        let pan_stap    = schaal * 0.4;
        let rot_stap    = 0.05_f64;
        let zoom_factor = 1.15_f64;

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return false,

            KeyCode::Char('a') | KeyCode::Char('A') => self.scroll_x -= pan_stap,
            KeyCode::Char('d') | KeyCode::Char('D') => self.scroll_x += pan_stap,
            KeyCode::Char('w') | KeyCode::Char('W') => self.scroll_y -= pan_stap * CHAR_ASPECT,
            KeyCode::Char('s') | KeyCode::Char('S') => self.scroll_y += pan_stap * CHAR_ASPECT,

            KeyCode::Left  => self.yaw   = (self.yaw   - rot_stap).rem_euclid(std::f64::consts::TAU),
            KeyCode::Right => self.yaw   = (self.yaw   + rot_stap).rem_euclid(std::f64::consts::TAU),
            KeyCode::Up    => self.pitch = (self.pitch - rot_stap).rem_euclid(std::f64::consts::TAU),
            KeyCode::Down  => self.pitch = (self.pitch + rot_stap).rem_euclid(std::f64::consts::TAU),

            KeyCode::Char('+') | KeyCode::Char('=') => self.zoom = (self.zoom * zoom_factor).min(50.0),
            KeyCode::Char('-') | KeyCode::Char('_') => self.zoom = (self.zoom / zoom_factor).max(0.05),

            KeyCode::Char('1') => self.snelheid = 1.0,
            KeyCode::Char('2') => self.snelheid = 365.25,
            KeyCode::Char('3') => self.snelheid = 3652.5,
            KeyCode::Char('4') => self.snelheid = 36525.0,

            KeyCode::Char(' ') => self.gepauzeerd = !self.gepauzeerd,
            KeyCode::Char('h') | KeyCode::Char('H') => self.toon_help    = !self.toon_help,
            KeyCode::Char('l') | KeyCode::Char('L') => self.toon_legenda = !self.toon_legenda,
            KeyCode::Char('m') | KeyCode::Char('M') => self.toon_manen   = !self.toon_manen,
            KeyCode::Char('0') => {
                self.scroll_x = 0.0; self.scroll_y = 0.0;
                self.yaw = 0.0;      self.pitch = 0.0;
                self.zoom = 1.0;
            }
            _ => {}
        }
        true
    }

    fn render<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let ui_rijen = 4usize;
        let w  = self.scherm.breedte;
        let h  = self.scherm.hoogte;
        let uh = (h.saturating_sub(ui_rijen)) as i64;
        let sc = BASE_SCALE * self.zoom;
        let cx = w as f64 / 2.0;
        let cy = uh as f64 / 2.0;

        self.scherm.wis();

        teken_sterren(&mut self.scherm, &self.sterren,
                      self.scroll_x, self.scroll_y,
                      self.yaw, self.pitch, cx, cy, uh);

        // zon positie vooraf berekenen zodat banen hem kunnen vermijden
        let zon_r = straal_cellen(ZON_STRAAL_KM, sc).max(2);
        let (zx, zy, _) = projecteer(0.0, 0.0, 0.0,
                                      self.yaw, self.pitch, sc,
                                      self.scroll_x, self.scroll_y, cx, cy);

        // kuipergordel eerst (achterste laag)
        teken_kuipergordel(&mut self.scherm, self.yaw, self.pitch, sc,
                            self.scroll_x, self.scroll_y, cx, cy, uh);

        // asteroidengordel
        teken_asteroiden(&mut self.scherm, self.yaw, self.pitch, sc,
                         self.scroll_x, self.scroll_y, cx, cy, uh, zx, zy, zon_r);

        // planeetbanen
        for def in PLANETEN {
            teken_baan(&mut self.scherm,
                       def.grote_as_au, def.excentriciteit,
                       self.yaw, self.pitch, sc,
                       self.scroll_x, self.scroll_y, cx, cy, uh,
                       zx, zy, zon_r,
                       Color::Rgb { r: 38, g: 38, b: 50 });
        }

        // dwergplaneetbanen (gestippeld kleur)
        for def in DWERGPLANETEN {
            teken_baan(&mut self.scherm,
                       def.grote_as_au, def.excentriciteit,
                       self.yaw, self.pitch, sc,
                       self.scroll_x, self.scroll_y, cx, cy, uh,
                       zx, zy, zon_r,
                       Color::Rgb { r: 55, g: 45, b: 60 });
        }

        // the sun is a deadly laser
        for ar in (zon_r + 1)..=(zon_r + 2) {
            teken_cirkel(&mut self.scherm, zx, zy, ar, '.',
                         Color::Rgb { r: 120, g: 80, b: 0 }, false, uh);
        }
        teken_cirkel(&mut self.scherm, zx, zy, zon_r, 'O',
                     Color::Rgb { r: 255, g: 220, b: 60 }, true, uh);

        // planeten
        for def in PLANETEN {
            let (bx, by, bz) = planeet_3d(def, self.sim_dagen);
            let (sx, sy, _)  = projecteer(bx, by, bz,
                                           self.yaw, self.pitch, sc,
                                           self.scroll_x, self.scroll_y, cx, cy);
            let r = straal_cellen(def.straal_km, sc);
            teken_cirkel(&mut self.scherm, sx, sy, r, def.glyph, def.kleur, false, uh);
        }

        // dwergplaneten altijd als punt
        for def in DWERGPLANETEN {
            let (bx, by, bz) = dwerg_3d(def, self.sim_dagen);
            let (sx, sy, _)  = projecteer(bx, by, bz,
                                           self.yaw, self.pitch, sc,
                                           self.scroll_x, self.scroll_y, cx, cy);
            if sy >= 0.0 && sy < uh as f64 {
                self.scherm.zet(sx as i64, sy as i64, '*', def.kleur, false);
            }
        }

        // manen
        if self.toon_manen {
            for maan in MANEN {
                let (mx, my, mz) = maan_3d(maan, self.sim_dagen);
                let (sx, sy, _)  = projecteer(mx, my, mz,
                                               self.yaw, self.pitch, sc,
                                               self.scroll_x, self.scroll_y, cx, cy);
                if sy >= 0.0 && sy < uh as f64 {
                    self.scherm.zet(sx as i64, sy as i64, '.', maan.kleur, false);
                }
            }
        }

        // legenda
        if self.toon_legenda {
            let lx = 2i64;
            let mut ly = 2i64;

            let titel = "Legenda";
            for (i, ch) in titel.chars().enumerate() {
                self.scherm.zet(lx + i as i64, ly, ch, Color::Rgb{r:180,g:180,b:200}, true);
            }
            ly += 1;
            for i in 0..7 {
                self.scherm.zet(lx + i, ly, '-', Color::Rgb{r:60,g:60,b:75}, false);
            }
            ly += 1;

            // zon
            self.scherm.zet(lx, ly, 'O', Color::Rgb{r:255,g:220,b:60}, true);
            for (i, ch) in " Zon".chars().enumerate() {
                self.scherm.zet(lx + 1 + i as i64, ly, ch, Color::Rgb{r:200,g:180,b:80}, false);
            }
            ly += 1;

            // planeten
            for def in PLANETEN {
                self.scherm.zet(lx, ly, def.glyph, def.kleur, false);
                for (i, ch) in format!(" {}", def.naam).chars().enumerate() {
                    self.scherm.zet(lx + 1 + i as i64, ly, ch, def.kleur, false);
                }
                ly += 1;
                if ly >= uh { break; }
            }

            // dwergplaneten
            if ly + 2 < uh {
                for (i, ch) in "Dwerg:".chars().enumerate() {
                    self.scherm.zet(lx + i as i64, ly, ch, Color::Rgb{r:130,g:110,b:140}, false);
                }
                ly += 1;
                for def in DWERGPLANETEN {
                    self.scherm.zet(lx, ly, '*', def.kleur, false);
                    for (i, ch) in format!(" {}", def.naam).chars().enumerate() {
                        self.scherm.zet(lx + 1 + i as i64, ly, ch, def.kleur, false);
                    }
                    ly += 1;
                    if ly >= uh { break; }
                }
            }

            // manen toggle hint
            if ly < uh {
                let maan_hint = if self.toon_manen { "M: manen aan" } else { "M: manen uit" };
                for (i, ch) in maan_hint.chars().enumerate() {
                    self.scherm.zet(lx + i as i64, ly, ch, Color::Rgb{r:80,g:100,b:120}, false);
                }
            }
        }

        // ui
        let bar_y = uh as i64;

        for x in 0..w {
            self.scherm.zet(x as i64, bar_y, '-', Color::Rgb { r: 140, g: 140, b: 140 }, false);
        }

        // speed
        let snelheden: &[(f64, &str)] = &[
            (1.0,     " 1x echt "),
            (365.25,  " 1jr/s "),
            (3652.5,  " 10jr/s "),
            (36525.0, " 100jr/s "),
        ];
        let mut bx = 1i64;
        for (mult, label) in snelheden {
            let actief = (self.snelheid - mult).abs() < 0.01;
            let kleur = if actief {
                Color::Rgb { r: 255, g: 215, b: 60 }
            } else {
                Color::Rgb { r: 100, g: 100, b: 120 }
            };
            if actief {
                self.scherm.zet(bx - 1, bar_y + 1, '[', Color::Rgb { r: 255, g: 215, b: 60 }, true);
            }
            for (i, ch) in label.chars().enumerate() {
                self.scherm.zet(bx + i as i64, bar_y + 1, ch, kleur, actief);
            }
            if actief {
                self.scherm.zet(bx + label.len() as i64, bar_y + 1, ']',
                                Color::Rgb { r: 255, g: 215, b: 60 }, true);
            }
            bx += label.len() as i64 + 1;
        }

        // pauze
        let (pl, pc) = if self.gepauzeerd {
            ("  GEPAUZEERD", Color::Rgb { r: 220, g: 180, b: 0 })
        } else {
            ("  actief    ", Color::Rgb { r: 80, g: 180, b: 80 })
        };
        for (i, ch) in pl.chars().enumerate() {
            self.scherm.zet(bx + i as i64, bar_y + 1, ch, pc, false);
        }

        // klok
        let jaren = self.sim_dagen / 365.25;
        let klok = if jaren < 2.0 {
            format!("dag {:.0}", self.sim_dagen)
        } else {
            format!("jaar {:.1}", jaren)
        };
        let kx = (w as i64 - klok.len() as i64 - 2).max(0);
        for (i, ch) in klok.chars().enumerate() {
            self.scherm.zet(kx + i as i64, bar_y + 1, ch,
                            Color::Rgb { r: 100, g: 110, b: 130 }, false);
        }

        // infootjes
        let info = format!(" zoom:{:.1}x  draai:{:.0}  helling:{:.0}",
                           self.zoom,
                           self.yaw.to_degrees().rem_euclid(360.0),
                           { let p = self.pitch.to_degrees().rem_euclid(360.0); if p > 180.0 { p - 360.0 } else { p } });
        for (i, ch) in info.chars().enumerate() {
            self.scherm.zet(i as i64, bar_y + 2, ch, Color::Rgb { r: 65, g: 80, b: 105 }, false);
        }

        // HEEEELLLPPPP!!!!! HEEEEEEEEEELLLLLLPPPPPPPP!!!!!!!!!!!
        let help = if self.toon_help {
            " WASD:pannen  pijltjes:draaien/kantelen  +/-:zoom  1-4:snelheid  spatie:pauze  0:reset  M:manen  L:legenda  H:help  Q:sluiten"
        } else {
            " H:hulp  Q:sluiten"
        };
        for (i, ch) in help.chars().take(w - 1).enumerate() {
            self.scherm.zet(i as i64, bar_y + 3, ch, Color::Rgb { r: 60, g: 65, b: 85 }, false);
        }

        self.scherm.flush(out)
    }

    fn resize(&mut self, w: u16, h: u16) { self.scherm.resize(w, h); }
}

fn main() -> io::Result<()> {
    let stdout_raw = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 18, stdout_raw.lock());

    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide, terminal::DisableLineWrap)?;

    let mut app = App::nieuw()?;
    let tick = Duration::from_millis(33);

    'hoofd: loop {
        app.update();
        app.render(&mut out)?;

        let deadline = Instant::now() + tick;
        loop {
            let now = Instant::now();
            if now >= deadline { break; }
            if event::poll(deadline - now)? {
                match event::read()? {
                    Event::Key(k) => {
                        use crossterm::event::KeyEventKind;
                        if k.kind == KeyEventKind::Press || k.kind == KeyEventKind::Repeat {
                            if !app.toets(k) { break 'hoofd; }
                        }
                    }
                    Event::Resize(w, h) => {
                        app.resize(w, h);
                        queue!(out, terminal::Clear(ClearType::All))?;
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(out, terminal::LeaveAlternateScreen, cursor::Show, terminal::EnableLineWrap)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
