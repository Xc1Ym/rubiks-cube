use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Face {
    U = 0,
    D = 1,
    F = 2,
    B = 3,
    R = 4,
    L = 5,
}

impl Face {
    pub const ALL: [Face; 6] = [Face::U, Face::D, Face::F, Face::B, Face::R, Face::L];

    pub fn name(self) -> &'static str {
        match self {
            Face::U => "U",
            Face::D => "D",
            Face::F => "F",
            Face::B => "B",
            Face::R => "R",
            Face::L => "L",
        }
    }
}

impl std::str::FromStr for Face {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Face::U),
            "D" => Ok(Face::D),
            "F" => Ok(Face::F),
            "B" => Ok(Face::B),
            "R" => Ok(Face::R),
            "L" => Ok(Face::L),
            _ => Err(format!("Unknown face: {}", s)),
        }
    }
}

/// 单次旋转操作
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub face: Face,
    pub clockwise: bool, // true = 顺时针 (从该面正前方看)
    pub double: bool,    // true = 180°
}

impl Move {
    pub fn new(face: Face, clockwise: bool, double: bool) -> Self {
        Self {
            face,
            clockwise,
            double,
        }
    }

    pub fn notation(&self) -> String {
        let mut s = self.face.name().to_string();
        if self.double {
            s.push('2');
        } else if !self.clockwise {
            s.push('\'');
        }
        s
    }
}

/// 解析标准魔方记法字符串，如 "R U R' U'"
pub fn parse_moves(input: &str) -> Result<Vec<Move>, String> {
    let mut moves = Vec::new();
    let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut i = 0;
    while i < chars.len() {
        let face_char = chars[i].to_string();
        let face: Face = face_char.parse()?;
        i += 1;
        let mut clockwise = true;
        let mut double = false;
        if i < chars.len() {
            match chars[i] {
                '\'' => {
                    clockwise = false;
                    i += 1;
                }
                '2' => {
                    double = true;
                    i += 1;
                }
                _ => {}
            }
        }
        moves.push(Move::new(face, clockwise, double));
    }
    Ok(moves)
}

/// 生成随机打乱序列
pub fn scramble_moves(count: usize) -> Vec<Move> {
    let mut rng = thread_rng();
    let faces = Face::ALL;
    let mut moves = Vec::with_capacity(count);
    let mut last_face: Option<Face> = None;

    for _ in 0..count {
        let mut face = *faces.choose(&mut rng).unwrap();
        // 避免连续两次同一面（无意义）
        while Some(face) == last_face {
            face = *faces.choose(&mut rng).unwrap();
        }
        last_face = Some(face);

        let rand_val: f64 = rand::random();
        let (clockwise, double) = if rand_val < 0.33 {
            (true, false)
        } else if rand_val < 0.66 {
            (false, false)
        } else {
            (true, true)
        };
        moves.push(Move::new(face, clockwise, double));
    }
    moves
}

/// 逆向操作序列
#[allow(dead_code)]
pub fn inverse_moves(moves: &[Move]) -> Vec<Move> {
    moves
        .iter()
        .rev()
        .map(|m| {
            if m.double {
                *m
            } else {
                Move::new(m.face, !m.clockwise, false)
            }
        })
        .collect()
}
