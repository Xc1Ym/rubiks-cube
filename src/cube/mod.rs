pub mod moves;

use moves::{Face, Move};

/// 魔方颜色
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Color {
    White,   // U - 上白
    Yellow,  // D - 下黄
    Red,     // F - 前红
    Orange,  // B - 后橙
    Blue,    // R - 右蓝
    Green,   // L - 左绿
}

impl Color {
    pub fn as_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::Yellow => (255, 255, 0),
            Color::Red => (255, 0, 0),
            Color::Orange => (255, 165, 0),
            Color::Blue => (0, 0, 255),
            Color::Green => (0, 255, 0),
        }
    }

    pub fn as_egui(&self) -> egui::Color32 {
        let (r, g, b) = self.as_rgb();
        egui::Color32::from_rgb(r, g, b)
    }

    pub fn from_face(face: Face) -> Self {
        match face {
            Face::U => Color::White,
            Face::D => Color::Yellow,
            Face::F => Color::Red,
            Face::B => Color::Orange,
            Face::R => Color::Blue,
            Face::L => Color::Green,
        }
    }
}

/// 单个面的 3x3 色块数组，索引: [row][col]，row/col 范围 0..3
/// 从该面正前方看，row 0 为上，col 0 为左
pub type FaceColors = [[Color; 3]; 3];

/// 魔方核心结构
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cube {
    pub faces: [FaceColors; 6],
}

impl Default for Cube {
    fn default() -> Self {
        Self::solved()
    }
}

impl Cube {
    pub fn solved() -> Self {
        Cube {
            faces: [
                [[Color::White; 3]; 3],   // U
                [[Color::Yellow; 3]; 3],  // D
                [[Color::Red; 3]; 3],     // F
                [[Color::Orange; 3]; 3],  // B
                [[Color::Blue; 3]; 3],    // R
                [[Color::Green; 3]; 3],   // L
            ],
        }
    }

    pub fn get_face(&self, face: Face) -> &FaceColors {
        &self.faces[face as usize]
    }

    pub fn get_face_mut(&mut self, face: Face) -> &mut FaceColors {
        &mut self.faces[face as usize]
    }

    /// 顺时针旋转指定面（从该面正前方看）
    pub fn rotate_face_clockwise(&mut self, face: Face) {
        let f = face as usize;
        // 保存原面
        let old = self.faces[f];
        // 面内顺时针旋转
        for r in 0..3 {
            for c in 0..3 {
                self.faces[f][c][2 - r] = old[r][c];
            }
        }
    }

    /// 逆时针旋转指定面
    pub fn rotate_face_counter_clockwise(&mut self, face: Face) {
        let f = face as usize;
        let old = self.faces[f];
        for r in 0..3 {
            for c in 0..3 {
                self.faces[f][2 - c][r] = old[r][c];
            }
        }
    }

    /// 执行单次 Move
    pub fn apply_move(&mut self, mv: &Move) {
        let face = mv.face;
        let cw = mv.clockwise;
        let double = mv.double;

        let times = if double { 2 } else { 1 };
        for _ in 0..times {
            if cw {
                self.rotate_face_clockwise(face);
                self.rotate_adjacent_clockwise(face);
            } else {
                self.rotate_face_counter_clockwise(face);
                self.rotate_adjacent_counter_clockwise(face);
            }
        }
    }

    /// 执行一系列 Move
    pub fn apply_moves(&mut self, moves: &[Move]) {
        for mv in moves {
            self.apply_move(mv);
        }
    }

    /// 顺时针旋转时，相邻面的边块循环
    fn rotate_adjacent_clockwise(&mut self, face: Face) {
        use Face::*;
        match face {
            U => {
                // U 面: F[0][*], R[0][*], B[0][*], L[0][*] 循环
                let tmp = self.row(U, F, 0);
                self.set_row(U, F, 0, self.row(U, R, 0));
                self.set_row(U, R, 0, self.row(U, B, 0));
                self.set_row(U, B, 0, self.row(U, L, 0));
                self.set_row(U, L, 0, tmp);
            }
            D => {
                // D 面: F[2][*], L[2][*], B[2][*], R[2][*] 循环
                let tmp = self.row(D, F, 2);
                self.set_row(D, F, 2, self.row(D, L, 2));
                self.set_row(D, L, 2, self.row(D, B, 2));
                self.set_row(D, B, 2, self.row(D, R, 2));
                self.set_row(D, R, 2, tmp);
            }
            F => {
                // F 面: U[2][*], L[*][2], D[0][*], R[*][0] 循环
                let tmp = self.row(F, U, 2);
                self.set_row(F, U, 2, self.col_rev(F, L, 2));
                self.set_col(F, L, 2, self.row(F, D, 0));
                self.set_row(F, D, 0, self.col_rev(F, R, 0));
                self.set_col(F, R, 0, tmp);
            }
            B => {
                // B 面: U[0][*], R[*][2], D[2][*], L[*][0] 循环
                let tmp = self.row(B, U, 0);
                self.set_row(B, U, 0, self.col(B, R, 2));
                self.set_col(B, R, 2, self.row_rev(B, D, 2));
                self.set_row(B, D, 2, self.col_rev(B, L, 0));
                self.set_col(B, L, 0, tmp);
            }
            R => {
                // R 面: U[*][2], F[*][2], D[*][2], B[*][0](逆序) 循环
                let tmp = self.col(R, U, 2);
                self.set_col(R, U, 2, self.col(R, F, 2));
                self.set_col(R, F, 2, self.col(R, D, 2));
                self.set_col(R, D, 2, self.col_rev(R, B, 0));
                self.set_col(R, B, 0, tmp);
            }
            L => {
                // L 面: U[*][0], B[*][2](逆序), D[*][0], F[*][0] 循环
                let tmp = self.col(L, U, 0);
                self.set_col(L, U, 0, self.col(L, B, 2));
                self.set_col(L, B, 2, self.col_rev(L, D, 0));
                self.set_col(L, D, 0, self.col(L, F, 0));
                self.set_col(L, F, 0, tmp);
            }
        }
    }

    fn rotate_adjacent_counter_clockwise(&mut self, face: Face) {
        use Face::*;
        match face {
            U => {
                let tmp = self.row(U, F, 0);
                self.set_row(U, F, 0, self.row(U, L, 0));
                self.set_row(U, L, 0, self.row(U, B, 0));
                self.set_row(U, B, 0, self.row(U, R, 0));
                self.set_row(U, R, 0, tmp);
            }
            D => {
                let tmp = self.row(D, F, 2);
                self.set_row(D, F, 2, self.row(D, R, 2));
                self.set_row(D, R, 2, self.row(D, B, 2));
                self.set_row(D, B, 2, self.row(D, L, 2));
                self.set_row(D, L, 2, tmp);
            }
            F => {
                let tmp = self.row(F, U, 2);
                self.set_row(F, U, 2, self.col(F, R, 0));
                self.set_col(F, R, 0, self.row_rev(F, D, 0));
                self.set_row(F, D, 0, self.col_rev(F, L, 2));
                self.set_col(F, L, 2, tmp);
            }
            B => {
                let tmp = self.row(B, U, 0);
                self.set_row(B, U, 0, self.col_rev(B, L, 0));
                self.set_col(B, L, 0, self.row(B, D, 2));
                self.set_row(B, D, 2, self.col(B, R, 2));
                self.set_col(B, R, 2, tmp);
            }
            R => {
                let tmp = self.col(R, U, 2);
                self.set_col(R, U, 2, self.col_rev(R, B, 0));
                self.set_col(R, B, 0, self.col_rev(R, D, 2));
                self.set_col(R, D, 2, self.col(R, F, 2));
                self.set_col(R, F, 2, tmp);
            }
            L => {
                let tmp = self.col(L, U, 0);
                self.set_col(L, U, 0, self.col(L, F, 0));
                self.set_col(L, F, 0, self.col(L, D, 0));
                self.set_col(L, D, 0, self.col_rev(L, B, 2));
                self.set_col(L, B, 2, tmp);
            }
        }
    }

    // 辅助方法：取某面某行（从左到右）
    fn row(&self, _context: Face, face: Face, r: usize) -> [Color; 3] {
        let f = &self.faces[face as usize];
        [f[r][0], f[r][1], f[r][2]]
    }

    fn set_row(&mut self, _context: Face, face: Face, r: usize, vals: [Color; 3]) {
        let f = &mut self.faces[face as usize];
        f[r][0] = vals[0];
        f[r][1] = vals[1];
        f[r][2] = vals[2];
    }

    fn row_rev(&self, _context: Face, face: Face, r: usize) -> [Color; 3] {
        let f = &self.faces[face as usize];
        [f[r][2], f[r][1], f[r][0]]
    }

    // 取某面某列（从上到下）
    fn col(&self, _context: Face, face: Face, c: usize) -> [Color; 3] {
        let f = &self.faces[face as usize];
        [f[0][c], f[1][c], f[2][c]]
    }

    fn set_col(&mut self, _context: Face, face: Face, c: usize, vals: [Color; 3]) {
        let f = &mut self.faces[face as usize];
        f[0][c] = vals[0];
        f[1][c] = vals[1];
        f[2][c] = vals[2];
    }

    fn col_rev(&self, _context: Face, face: Face, c: usize) -> [Color; 3] {
        let f = &self.faces[face as usize];
        [f[2][c], f[1][c], f[0][c]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solved() {
        let c = Cube::solved();
        assert_eq!(c.faces[0][0][0], Color::White);
        assert_eq!(c.faces[1][0][0], Color::Yellow);
    }

    #[test]
    fn test_rotate_u() {
        let mut c = Cube::solved();
        c.apply_move(&Move::new(Face::U, true, false));
        // U 面仍是全白
        for r in 0..3 {
            for col in 0..3 {
                assert_eq!(c.faces[Face::U as usize][r][col], Color::White);
            }
        }
    }

    #[test]
    fn test_4_times_back() {
        let mut c = Cube::solved();
        for _ in 0..4 {
            c.apply_move(&Move::new(Face::F, true, false));
        }
        assert_eq!(c, Cube::solved());
    }
}
