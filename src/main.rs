use clap_conf::prelude::*;
use mksvg::{PathD, SvgArg, SvgWrite, Tag};
use serde_derive::*;

fn mcos8(n: u8) -> f64 {
    match n % 8 {
        0 | 1 | 7 => 1.,
        2 | 6 => 0.,
        _ => -1.,
    }
}

fn msin8(n: u8) -> f64 {
    match n % 8 {
        0 | 4 => 0.,
        1 | 2 | 3 => 1.,
        _ => -1.,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AnimalCard<'a> {
    a: Animal,
    rel: &'a str,
    promoted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Animal {
    img: String,
    points: Vec<u8>,
}

impl<'a> mksvg::Card<f64> for AnimalCard<'a> {
    fn front<S: SvgWrite>(&self, s: &mut S, w: f64, h: f64) {
        Tag::new("path")
            .arg(
                "d",
                PathD::abs()
                    .m(0., h)
                    .l(w * 0.2, 0.)
                    .l(w * 0.8, 0.)
                    .l(w, h)
                    .l(0., h),
            )
            .fill("#bbbbff")
            .write(s);

        let ground = match self.promoted {
            true => "#ffff99",
            false => "#bbffbb",
        };
        Tag::new("path")
            .arg(
                "d",
                PathD::abs()
                    .m(0., h)
                    .l(w * 0.1, h * 0.5)
                    .l(w * 0.9, h * 0.5)
                    .l(w, h)
                    .l(0., h),
            )
            .fill(ground)
            .write(s);
        let path = format!("{}{}", self.rel, self.a.img);
        Tag::img(&path, w * 0.15, h * 0.15, w * 0.7, h * 0.7).write(s);

        for p in &self.a.points {
            let x = w * (0.5 + msin8(*p) * 0.28);
            let y = h * (0.5 - mcos8(*p) * 0.4);
            Tag::ellipse(x, y, w * 0.02, h * 0.02)
                .fill("black")
                .write(s);
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ALoader {
    img: String,
    points: Vec<u8>,
    back: Option<Animal>,
}

impl ALoader {
    fn split<'a>(self, rel: &'a str) -> (AnimalCard<'a>, AnimalCard<'a>) {
        let fcard = AnimalCard {
            a: Animal {
                img: self.img,
                points: self.points,
            },
            rel,
            promoted: false,
        };
        (
            fcard.clone(),
            match self.back {
                Some(b) => AnimalCard {
                    a: Animal {
                        img: b.img,
                        points: b.points,
                    },
                    promoted: true,
                    rel,
                },
                None => fcard,
            },
        )
    }
}

fn main() {
    let clap = clap_app!(
    shogi_cards =>
    (about:"Make cards for shogi")
    (author:"Matthew Stoodley")
    (version:crate_version!())
    (@arg file:+required "The toml file containing the cards")
    (@arg rel:--rel +takes_value "Relative path of cards")
    (@arg out:+required "root of the output")
    )
    .get_matches();

    let cfg = with_toml_env(
        &clap,
        &["conf.toml", "{HOME}/.config/shogi_cards/conf.toml"],
    );
    let l = cfg.grab().arg("file").done().unwrap();

    let s = std::fs::read_to_string(l).expect("Could not find file");

    let cards: std::collections::BTreeMap<String, ALoader> =
        toml::from_str(&s).expect("Could not parse file");

    let mut fronts = Vec::new();
    let mut backs = Vec::new();
    let rel = cfg
        .grab()
        .arg("rel")
        .ask_def("What folder should the cards link to for images?", "");
    for (_n, c) in cards {
        let (f, b) = c.split(&rel);
        for _ in 0..5 {
            fronts.push(f.clone());
            backs.push(b.clone());
        }
    }

    let out_base = clap.value_of("out").unwrap_or("out/s");
    let f_base = format!("{}_f", out_base);
    let b_base = format!("{}_b", out_base);

    let f_locs = mksvg::page::pages_a4(f_base, 5, 7, &fronts).expect("Could not write files");
    let b_locs = mksvg::page::pages_a4(b_base, 5, 7, &backs).expect("Could not write backs");
    let all_pages = mksvg::page::interlace(f_locs, b_locs);
    mksvg::page::unite_as_pdf(all_pages, format!("{}_res.pdf", out_base));
}
