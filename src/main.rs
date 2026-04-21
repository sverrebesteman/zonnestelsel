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

struct PlaneetDef {
    naam: &'static str,
    grote_as_au: f64,
    excentriciteit: f64,
    omloop_dagen: f64,
    straal_km: f64,
    glyph: char,
    kleur: Color,
    begin_hoek: f64,
}

//Define planeten, met alle info.
static PLANETEN: &[PlaneetDef] = &[
    PlaneetDef {
        naam: "Mercurius",
        grote_as_au: 0.387,
        excentriciteit: 0.206,
        omloop_dagen: 87.97,
        straal_km: 2_439.0,
        glyph: '●',
        kleur: Color::Rgb {
            r: 160,
            g: 140,
            b: 130,
        },
        begin_hoek: 0.8,
    },
    PlaneetDef {
        naam: "Venus",
        grote_as_au: 0.723,
        excentriciteit: 0.007,
        omloop_dagen: 224.70,
        straal_km: 6_052.0,
        glyph: '●',
        kleur: Color::Rgb {
            r: 230,
            g: 210,
            b: 130,
        },
        begin_hoek: 2.1,
    },
    PlaneetDef {
        naam: "Aarde",
        grote_as_au: 1.000,
        excentriciteit: 0.017,
        omloop_dagen: 365.25,
        straal_km: 6_371.0,
        glyph: '●',
        kleur: Color::Rgb {
            r: 80,
            g: 160,
            b: 240,
        },
        begin_hoek: 4.0,
    },
    PlaneetDef {
        naam: "Mars",
        grote_as_au: 1.524,
        excentriciteit: 0.093,
        omloop_dagen: 686.97,
        straal_km: 3_390.0,
        glyph: '●',
        kleur: Color::Rgb {
            r: 220,
            g: 100,
            b: 60,
        },
        begin_hoek: 1.0,
    },
    PlaneetDef {
        naam: "Jupiter",
        grote_as_au: 5.203,
        excentriciteit: 0.049,
        omloop_dagen: 4_332.59,
        straal_km: 71_492.0,
        glyph: '◉',
        kleur: Color::Rgb {
            r: 210,
            g: 175,
            b: 130,
        },
        begin_hoek: 3.5,
    },
    PlaneetDef {
        naam: "Saturnus",
        grote_as_au: 9.537,
        excentriciteit: 0.057,
        omloop_dagen: 10_759.22,
        straal_km: 60_268.0,
        glyph: '◎',
        kleur: Color::Rgb {
            r: 220,
            g: 205,
            b: 155,
        },
        begin_hoek: 5.2,
    },
    PlaneetDef {
        naam: "Uranus",
        grote_as_au: 19.191,
        excentriciteit: 0.046,
        omloop_dagen: 30_688.50,
        straal_km: 25_559.0,
        glyph: '○',
        kleur: Color::Rgb {
            r: 150,
            g: 230,
            b: 235,
        },
        begin_hoek: 0.3,
    },
    PlaneetDef {
        naam: "Neptunus",
        grote_as_au: 30.069,
        excentriciteit: 0.010,
        omloop_dagen: 60_182.00,
        straal_km: 24_764.0,
        glyph: '○',
        kleur: Color::Rgb {
            r: 80,
            g: 120,
            b: 230,
        },
        begin_hoek: 2.8,
    },
];

const ZON_STRAAL_KM: f64 = 696_000.0;

// Dubbele buffer
#[derive(Clone, PartialEq)]
struct Cel {
    teken: char,
    fg: Color,
    vet: bool,
}
impl Default for Cel {
    fn default() -> Self {
        Cel {
            teken: ' ',
            fg: Color::Reset,
            vet: false,
        }
    }
}

struct Scherm {
    breedte: usize,
    hoogte: usize,
    voor: Vec<Cel>,
    achter: Vec<Cel>,
}

impl Scherm {
    fn nieuw(w: u16, h: u16) -> Self {
        let n = w as usize * h as usize;
        Scherm {
            breedte: w as usize,
            hoogte: h as usize,
            voor: vec![
                Cel {
                    teken: '~',
                    ..Default::default()
                };
                n
            ],
            achter: vec![Default::default(); n],
        }
    }
    fn resize(&mut self, w: u16, h: u16) {
        self.breedte = w as usize;
        self.hoogte = h as usize;
        let n = self.breedte * self.hoogte;
        self.voor = vec![
            Cel {
                teken: '~',
                ..Default::default()
            };
            n
        ];
        self.achter = vec![Default::default(); n];
    }
    fn wis(&mut self) {
        for c in &mut self.achter {
            *c = Default::default();
        }
    }

    #[inline]
    fn zet(&mut self, x: i64, y: i64, teken: char, fg: Color, vet: bool) {
        if x < 0 || y < 0 {
            return;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.breedte || uy >= self.hoogte {
            return;
        }
        self.achter[uy * self.breedte + ux] = Cel { teken, fg, vet };
    }

    fn flush<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let mut huidig_vet = false;
        for idx in 0..self.voor.len() {
            let a = &self.achter[idx];
            if a == &self.voor[idx] {
                continue;
            }
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
        if huidig_vet {
            queue!(out, SetAttribute(Attribute::Reset))?;
        }
        queue!(out, ResetColor)?;
        out.flush()
    }
}

// Wiskunde (gemaakt door mijn vriend Chet Djipiti uit Silicon Valley)
fn excentrieke_anomalie(ma: f64, e: f64) -> f64 {
    let mut ea = ma;
    for _ in 0..60 {
        let d = (ma - (ea - e * ea.sin())) / (1.0 - e * ea.cos());
        ea += d;
        if d.abs() < 1e-11 {
            break;
        }
    }
    ea
}

fn planeet_3d(def: &PlaneetDef, sim_dagen: f64) -> (f64, f64, f64) {
    use std::f64::consts::PI;
    let mm = 2.0 * PI / def.omloop_dagen;
    let ma = (mm * sim_dagen + def.begin_hoek).rem_euclid(2.0 * PI);
    let ea = excentrieke_anomalie(ma, def.excentriciteit);
    let ta = 2.0
        * (((1.0 + def.excentriciteit) / (1.0 - def.excentriciteit)).sqrt() * (ea / 2.0).tan())
            .atan();
    let r = def.grote_as_au * (1.0 - def.excentriciteit.powi(2))
        / (1.0 + def.excentriciteit * ta.cos());
    (r * ta.cos(), r * ta.sin(), 0.0)
}

fn projecteer(
    wx: f64,
    wy: f64,
    wz: f64,
    yaw: f64,
    pitch: f64,
    schaal: f64,
    scroll_x: f64,
    scroll_y: f64,
    cx: f64,
    cy: f64,
) -> (f64, f64, f64) {
    let (sy, cy2) = yaw.sin_cos();
    let rx1 = wx * cy2 + wy * sy;
    let ry1 = -wx * sy + wy * cy2;
    let rz1 = wz;
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
    let basis =
        ((straal_km.ln() - 7000_f64.ln()) / (ZON_STRAAL_KM.ln() - 7000_f64.ln()) * 4.0).max(0.0);
    ((basis * (schaal / BASE_SCALE).sqrt()).round() as i64).max(0)
}

// Teken helpers
fn teken_cirkel(
    scherm: &mut Scherm,
    cx: f64,
    cy: f64,
    radius: i64,
    glyph: char,
    kleur: Color,
    vet: bool,
    usable_h: i64,
) {
    let r = radius.max(0);
    if r == 0 {
        scherm.zet(cx as i64, cy as i64, glyph, kleur, vet);
        return;
    }
    for dy in -r..=r {
        for dx in -(r * 2)..=(r * 2) {
            if (dx as f64 / 2.0).powi(2) + (dy as f64).powi(2) <= (r as f64 + 0.5).powi(2) {
                let sy = cy as i64 + dy;
                if sy < usable_h {
                    scherm.zet(cx as i64 + dx, sy, glyph, kleur, vet);
                }
            }
        }
    }
}

fn teken_baan(
    scherm: &mut Scherm,
    def: &PlaneetDef,
    yaw: f64,
    pitch: f64,
    schaal: f64,
    scroll_x: f64,
    scroll_y: f64,
    cx: f64,
    cy: f64,
    usable_h: i64,
) {
    use std::f64::consts::PI;
    let a = def.grote_as_au;
    let b = a * (1.0 - def.excentriciteit.powi(2)).sqrt();
    let fc = a * def.excentriciteit;
    let stappen = ((a * schaal * 2.0 * PI) as usize).clamp(120, 2000);
    let skip = (stappen / 600 + 1).max(1);
    for i in (0..stappen).step_by(skip) {
        let theta = 2.0 * PI * i as f64 / stappen as f64;
        let (sx, sy, _) = projecteer(
            a * theta.cos() - fc,
            b * theta.sin(),
            0.0,
            yaw,
            pitch,
            schaal,
            scroll_x,
            scroll_y,
            cx,
            cy,
        );
        if sy >= 0.0 && sy < usable_h as f64 {
            scherm.zet(
                sx as i64,
                sy as i64,
                '·',
                Color::Rgb {
                    r: 40,
                    g: 42,
                    b: 58,
                },
                false,
            );
        }
    }
}

// Sterren
struct Ster {
    x: f64,
    y: f64,
    soort: u8,
}

fn genereer_sterren(n: usize) -> Vec<Ster> {
    (0..n)
        .map(|i| {
            let hx = (i.wrapping_mul(2654435761).wrapping_add(1013904223)) as u64;
            let hy = (i.wrapping_mul(1664525).wrapping_add(22695477)) as u64;
            Ster {
                x: ((hx & 0xFFFF) as f64 / 65535.0) * 4000.0 - 2000.0,
                y: ((hy & 0xFFFF) as f64 / 65535.0) * 4000.0 - 2000.0,
                soort: (i % 7) as u8,
            }
        })
        .collect()
}

fn teken_sterren(
    scherm: &mut Scherm,
    sterren: &[Ster],
    scroll_x: f64,
    scroll_y: f64,
    yaw: f64,
    pitch: f64,
    cx: f64,
    cy: f64,
    usable_h: i64,
    tijd: f64,
) {
    let tw = scherm.breedte as f64;
    let th = usable_h as f64;
    for s in sterren {
        let (px, py, _) = projecteer(
            s.x,
            s.y,
            0.0,
            yaw * 0.08,
            pitch * 0.08,
            1.0,
            scroll_x * 0.04,
            scroll_y * 0.04,
            cx,
            cy,
        );
        let x = px.rem_euclid(tw * 1.5) - tw * 0.25;
        let y = py.rem_euclid(th * 1.5) - th * 0.25;
        if x < 0.0 || y < 0.0 || x >= tw || y >= th {
            continue;
        }
        let tw2 = ((tijd * 3.0 + s.x * 0.1 + s.y * 0.07).sin() * 0.5 + 0.5) as f32;
        let (ch, kleur) = match s.soort {
            0 => {
                let b = (180.0 + tw2 * 75.0) as u8;
                ('✦', Color::Rgb { r: b, g: b, b: 255 })
            }
            1 => {
                let b = (200.0 + tw2 * 55.0) as u8;
                (
                    '+',
                    Color::Rgb {
                        r: 255,
                        g: b,
                        b: (b as u16 * 3 / 4) as u8,
                    },
                )
            }
            2 => {
                let b = (160.0 + tw2 * 60.0) as u8;
                ('·', Color::Rgb { r: b, g: 255, b: b })
            }
            3 => {
                let b = (120.0 + tw2 * 80.0) as u8;
                ('·', Color::Rgb { r: b, g: b, b: b })
            }
            4 => (
                '★',
                Color::Rgb {
                    r: 255,
                    g: 240,
                    b: 200,
                },
            ),
            5 => {
                let b = (100.0 + tw2 * 80.0) as u8;
                (
                    '░',
                    Color::Rgb {
                        r: b / 2,
                        g: b / 3,
                        b: b,
                    },
                )
            }
            _ => {
                let b = (90.0 + tw2 * 50.0) as u8;
                ('·', Color::Rgb { r: b, g: b, b: b })
            }
        };
        scherm.zet(x as i64, y as i64, ch, kleur, s.soort == 4);
    }
}

// Applicatie
struct App {
    scherm: Scherm,
    scroll_x: f64,
    scroll_y: f64,
    zoom: f64,
    yaw: f64,
    pitch: f64,
    snelheid: f64,
    last_tick: Instant,
    sim_dagen: f64,
    toon_help: bool,
    gepauzeerd: bool,
    sterren: Vec<Ster>,
    tijd: f64,
}

impl App {
    fn nieuw() -> io::Result<Self> {
        let (w, h) = terminal::size()?;
        Ok(App {
            scherm: Scherm::nieuw(w, h),
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
            yaw: 0.0,
            pitch: 0.0,
            snelheid: 1.0,
            last_tick: Instant::now(),
            sim_dagen: 0.0,
            toon_help: true,
            gepauzeerd: false,
            sterren: genereer_sterren(800),
            tijd: 0.0,
        })
    }

    fn update(&mut self) {
        let dt = self.last_tick.elapsed().as_secs_f64();
        self.last_tick = Instant::now();
        self.tijd += dt;
        if !self.gepauzeerd {
            self.sim_dagen += dt * self.snelheid;
        }
        self.pitch = self.pitch.clamp(
            -std::f64::consts::FRAC_PI_2 + 0.05,
            std::f64::consts::FRAC_PI_2 - 0.05,
        );
    }

    // Beweging per keypress ipv held.
    fn toets(&mut self, key: KeyEvent) -> bool {
        // Stap-grootten afhankelijk van huidige zoom
        let schaal = BASE_SCALE * self.zoom;
        let pan_stap = schaal * 0.4;
        let rot_stap = 0.08_f64; // radialen per druk
        let zoom_factor = 1.15_f64;

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return false,

            // Pan — WASD
            KeyCode::Char('a') | KeyCode::Char('A') => self.scroll_x -= pan_stap,
            KeyCode::Char('d') | KeyCode::Char('D') => self.scroll_x += pan_stap,
            KeyCode::Char('w') | KeyCode::Char('W') => self.scroll_y -= pan_stap * CHAR_ASPECT,
            KeyCode::Char('s') | KeyCode::Char('S') => self.scroll_y += pan_stap * CHAR_ASPECT,

            // Draaien / kantelen — pijltjes
            KeyCode::Left => self.yaw -= rot_stap,
            KeyCode::Right => self.yaw += rot_stap,
            KeyCode::Up => self.pitch -= rot_stap,
            KeyCode::Down => self.pitch += rot_stap,

            // Zoom — + / -
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.zoom = (self.zoom * zoom_factor).min(50.0)
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.zoom = (self.zoom / zoom_factor).max(0.05)
            }

            // Snelheid
            KeyCode::Char('1') => self.snelheid = 1.0,
            KeyCode::Char('2') => self.snelheid = 365.25,
            KeyCode::Char('3') => self.snelheid = 3652.5,
            KeyCode::Char('4') => self.snelheid = 36525.0,

            // Overig
            KeyCode::Char(' ') => self.gepauzeerd = !self.gepauzeerd,
            KeyCode::Char('h') | KeyCode::Char('H') => self.toon_help = !self.toon_help,
            KeyCode::Char('0') => {
                self.scroll_x = 0.0;
                self.scroll_y = 0.0;
                self.yaw = 0.0;
                self.pitch = 0.0;
                self.zoom = 1.0;
            }
            _ => {}
        }
        true
    }

    fn render<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let ui_rijen = 4usize;
        let w = self.scherm.breedte;
        let h = self.scherm.hoogte;
        let uh = (h.saturating_sub(ui_rijen)) as i64;
        let sc = BASE_SCALE * self.zoom;
        let cx = w as f64 / 2.0;
        let cy = uh as f64 / 2.0;

        self.scherm.wis();

        teken_sterren(
            &mut self.scherm,
            &self.sterren,
            self.scroll_x,
            self.scroll_y,
            self.yaw,
            self.pitch,
            cx,
            cy,
            uh,
            self.tijd,
        );

        for def in PLANETEN {
            teken_baan(
                &mut self.scherm,
                def,
                self.yaw,
                self.pitch,
                sc,
                self.scroll_x,
                self.scroll_y,
                cx,
                cy,
                uh,
            );
        }

        // Zon
        let zon_r = straal_cellen(ZON_STRAAL_KM, sc).max(2);
        let (zx, zy, _) = projecteer(
            0.0,
            0.0,
            0.0,
            self.yaw,
            self.pitch,
            sc,
            self.scroll_x,
            self.scroll_y,
            cx,
            cy,
        );
        for ar in (zon_r + 1)..=(zon_r + 3) {
            let a = (255u16.saturating_sub((ar - zon_r) as u16 * 60)).max(60) as u8;
            teken_cirkel(
                &mut self.scherm,
                zx,
                zy,
                ar,
                '·',
                Color::Rgb { r: 255, g: a, b: 0 },
                false,
                uh,
            );
        }
        teken_cirkel(
            &mut self.scherm,
            zx,
            zy,
            zon_r,
            '☀',
            Color::Rgb {
                r: 255,
                g: 230,
                b: 60,
            },
            true,
            uh,
        );
        for (i, ch) in "Zon".chars().enumerate() {
            self.scherm.zet(
                zx as i64 - 1 + i as i64,
                zy as i64 + zon_r + 1,
                ch,
                Color::Rgb {
                    r: 255,
                    g: 230,
                    b: 100,
                },
                true,
            );
        }

        // Planeten
        for def in PLANETEN {
            let (bx, by, bz) = planeet_3d(def, self.sim_dagen);
            let (sx, sy, _) = projecteer(
                bx,
                by,
                bz,
                self.yaw,
                self.pitch,
                sc,
                self.scroll_x,
                self.scroll_y,
                cx,
                cy,
            );
            let r = straal_cellen(def.straal_km, sc);
            if r >= 1 {
                let gc = match def.kleur {
                    Color::Rgb { r, g, b } => Color::Rgb {
                        r: r / 2,
                        g: g / 2,
                        b: b / 2,
                    },
                    o => o,
                };
                teken_cirkel(&mut self.scherm, sx, sy, r + 1, '·', gc, false, uh);
            }
            teken_cirkel(&mut self.scherm, sx, sy, r, def.glyph, def.kleur, true, uh);
            let lx = sx as i64 + r + 1;
            let ly = sy as i64;
            if ly >= 0 && ly < uh {
                for (i, ch) in def.naam.chars().enumerate() {
                    self.scherm.zet(lx + i as i64, ly, ch, def.kleur, false);
                }
            }
        }

        // UI spul
        let bar_y = uh as i64;

        // Scheiding
        for x in 0..w {
            let f = x as f64 / w as f64;
            let r = (30.0 + f * 40.0) as u8;
            let b = (50.0 + f * 30.0) as u8;
            self.scherm
                .zet(x as i64, bar_y, '▄', Color::Rgb { r, g: 30, b }, false);
        }

        // Snelheidsknopjes
        let snelheden: &[(f64, &str)] = &[
            (1.0, " 1× echt "),
            (365.25, " 1jr/s "),
            (3652.5, " 10jr/s "),
            (36525.0, " 100jr/s "),
        ];
        let mut bx = 1i64;
        for (mult, label) in snelheden {
            let actief = (self.snelheid - mult).abs() < 0.01;
            let kleur = if actief {
                Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 60,
                }
            } else {
                Color::Rgb {
                    r: 110,
                    g: 110,
                    b: 140,
                }
            };
            for (i, ch) in label.chars().enumerate() {
                self.scherm.zet(bx + i as i64, bar_y + 1, ch, kleur, actief);
            }
            if actief {
                self.scherm.zet(
                    bx - 1,
                    bar_y + 1,
                    '[',
                    Color::Rgb {
                        r: 255,
                        g: 215,
                        b: 60,
                    },
                    true,
                );
                self.scherm.zet(
                    bx + label.len() as i64,
                    bar_y + 1,
                    ']',
                    Color::Rgb {
                        r: 255,
                        g: 215,
                        b: 60,
                    },
                    true,
                );
            }
            bx += label.len() as i64 + 1;
        }

        // Pauze
        let (pl, pc) = if self.gepauzeerd {
            (
                "GEPAUZEERD",
                Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                },
            )
        } else {
            (
                "ACTIEF",
                Color::Rgb {
                    r: 80,
                    g: 220,
                    b: 120,
                },
            )
        };
        for (i, ch) in pl.chars().enumerate() {
            self.scherm.zet(bx + i as i64, bar_y + 1, ch, pc, false);
        }

        // Klok
        let jaren = self.sim_dagen / 365.25;
        let klok = if jaren < 2.0 {
            format!("Dag {:.1}", self.sim_dagen)
        } else {
            format!("Jaar {:.2}", jaren)
        };
        let kx = (w as i64 - klok.len() as i64 - 2).max(0);
        for (i, ch) in klok.chars().enumerate() {
            self.scherm.zet(
                kx + i as i64,
                bar_y + 1,
                ch,
                Color::Rgb {
                    r: 110,
                    g: 120,
                    b: 150,
                },
                false,
            );
        }

        // Info-rij
        let info = format!(
            " Zoom:{:.1}×  Draai:{:.0}°  Helling:{:.0}°",
            self.zoom,
            self.yaw.to_degrees(),
            self.pitch.to_degrees()
        );
        for (i, ch) in info.chars().enumerate() {
            self.scherm.zet(
                i as i64,
                bar_y + 2,
                ch,
                Color::Rgb {
                    r: 70,
                    g: 90,
                    b: 120,
                },
                false,
            );
        }

        // Help
        let help = if self.toon_help {
            " WASD:pannen  ←→:draaien  ↑↓:kantelen  +:zoom-in  -:zoom-uit  1-4:snelheid  Spatie:pauze  0:reset  H:help  Q:sluiten"
        } else {
            " H:hulp  Q:sluiten"
        };
        for (i, ch) in help.chars().enumerate() {
            if i + 1 >= w {
                break;
            }
            self.scherm.zet(
                i as i64,
                bar_y + 3,
                ch,
                Color::Rgb {
                    r: 60,
                    g: 65,
                    b: 90,
                },
                false,
            );
        }

        self.scherm.flush(out)
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.scherm.resize(w, h);
    }
}

fn main() -> io::Result<()> {
    let stdout_raw = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 18, stdout_raw.lock());

    terminal::enable_raw_mode()?;
    execute!(
        out,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::DisableLineWrap
    )?;

    let mut app = App::nieuw()?;
    let tick = Duration::from_millis(33);

    'hoofd: loop {
        app.update();
        app.render(&mut out)?;

        let deadline = Instant::now() + tick;
        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            if event::poll(deadline - now)? {
                match event::read()? {
                    Event::Key(k) => {
                        // Alleen reageren op Press (en Repeat voor key-herhaling)
                        use crossterm::event::KeyEventKind;
                        if k.kind == KeyEventKind::Press || k.kind == KeyEventKind::Repeat {
                            if !app.toets(k) {
                                break 'hoofd;
                            }
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

    execute!(
        out,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        terminal::EnableLineWrap
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}
