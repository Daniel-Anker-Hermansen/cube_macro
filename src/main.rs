use std::collections::{HashSet, HashMap};

fn main() {
    let mut cube = Cube::new();
    for _ in 0..1 {
        cube.apply_sequence("D' L' B D F' U' R F U2 F R2 U2 F2 D2 L2 F U2 R2 F' D R'");
    }
    let transpisitions = cube.extract_cycles().into_iter().flat_map(|s|s.into_transposition()).collect::<Vec<_>>();
    println!("{:?}", transpisitions);
    let tr = transpositions_to_operations(transpisitions);
    println!("{:?}", tr);
    let new = apply_transposition(SOLVED, tr);
    println!("{:b}", new);
}

const SOLVED: u128 = 0b000000000000000000000000_00000_00100_01000_01100_10000_10100_11000_11100_0000_00000_00010_00100_00110_01000_01010_01100_01110_10000_10010_10100_10110;

fn apply_transposition(cube: u128, trans: Tranposition) -> u128 {
    let mut new_cube = cube & trans.mask;
    for (i, j) in trans.rshifts {
        let mut mask = cube & i;
        mask >>= j;
        new_cube |= mask;
    }
    for (i, j) in trans.lshifts {
        let mut mask = cube & i;
        mask <<= j;
        new_cube |= mask;
    }
    new_cube
}

#[derive(Debug)]
struct Tranposition {
    mask: u128,
    rshifts: Vec<(u128, usize)>,
    lshifts: Vec<(u128, usize)>,
}

fn transpositions_to_operations(trans: Vec<(usize, isize)>) -> Tranposition {
    let mut mask = 0;
    let mut h = HashMap::new();
    for (i, j) in trans.iter() {
        match h.get_mut(j) {
            None => {
                h.insert(j, vec![*i]);
            }
            Some(v) => {
                v.push(*i);
            }
        }
        let k = 0b11111 << i;
        mask |= k;
    }
    let lshifts = h.iter().filter(|(i, _)| ***i > 0).map(|(i,j)|{
            let mut mask = 0;
            for z in j {
                let k = 0b11111 << z;
                mask |= k;
            }
            (mask, **i as usize)
        })
        .collect();
    let rshifts = h.iter().filter(|(i, _)| ***i < 0).map(|(i,j)|{
            let mut mask = 0;
            for z in j {
                let k = 0b11111 << z;
                mask |= k;
            }
            (mask, -**i as usize)
        })
        .collect();
    mask = !mask;
    Tranposition { mask, rshifts, lshifts }
}

#[derive(Debug, PartialEq, Eq)]
struct Cube {
    rep: [(u8, u8); 27]
}

impl Cube {
    fn new() -> Cube {
        let mut rep = [(0, 0); 27];
        for i in 0..27 {
            rep[i] = (i as u8, 0);
        }
        Cube { rep }
    }

    fn mut_iter(&mut self) -> impl Iterator<Item = (Position, &mut (u8, u8))> {
        self.rep.iter_mut().enumerate().map(|(idx, d)| {
            let pos = Position::new(idx / 9, (idx / 3) % 3, idx % 3);
            (pos, d)
        })
    } 

    fn get(&self, pos: &Position) -> (u8, u8) {
        self.rep[((pos.x + 1) * 9 + (pos.y + 1) * 3 + (pos.z + 1)) as usize]
    }

    fn rotate(&mut self, fixed: usize, real: usize, complex: usize, filter: isize) {
        let n = self.mut_iter()
            .map(|(mut pos, _)| {
                if *pos.n(fixed) == filter {
                    let mut new_pos = Position::new(0, 0, 0);
                    *new_pos.n(fixed) = *pos.n(fixed);
                    *new_pos.n(real) = *pos.n(complex) * - 1;
                    *new_pos.n(complex) = *pos.n(real);
                    new_pos
                }
                else {
                    pos
                }
            })
            .collect::<Vec<_>>();
        let mut new_rep = [(0, 0); 27];
        for i in 0..27 {
            new_rep[i] = self.get(&n[i]);
        }
        self.rep = new_rep;
        if fixed == 2 {
            self.mut_iter()
                .filter(|(pos, _)| *pos.clone().n(fixed) == filter && *pos.clone().n(real) * *pos.clone().n(complex) == 0)
                .for_each(|(_, (_, rot))| *rot = (*rot + 1) % 2);
        }
        if fixed != 1 {
            self.mut_iter()
                .filter(|(pos, _)| *pos.clone().n(fixed) == filter && *pos.clone().n(real) * *pos.clone().n(complex) != 0)
                .for_each(|(pos, (_, rot))| {
                    let tw = *pos.clone().n(real) * *pos.clone().n(complex) * (fixed as isize - 1) * filter;
                    *rot = (*rot + (tw + 3) as u8) % 3;
                });    
        }
    }

    fn do_move(&mut self, m: &str) {
        match m {
            "U" => self.rotate(1, 0, 2, 1),
            "D" => self.rotate(1, 2, 0, -1),
            "F" => self.rotate(2, 0, 1, -1),
            "B" => self.rotate(2, 1, 0, 1),
            "L" => self.rotate(0, 1, 2, -1),
            "R" => self.rotate(0, 2, 1, 1),
            _ => panic!("Illegal move you idiot! A cube only has six sides!"),
        }
    }

    fn apply_sequence(&mut self, alg: &str) {
        for m in alg.split(" ") {
            let s = &m[..1];
            self.do_move(s);
            if m.len() > 1 {
                self.do_move(s);
                if &m[1..2] == "'" {
                    self.do_move(s);
                }
            }
        }
    }

    fn extract_cycles(&self) -> Vec<Cycle> {
        let mut unvisited: HashSet<usize> =(0..27).collect();
        let mut acc = Vec::new();
        while let Some(v) = unvisited.iter().next() {
            let mut v = *v;
            let orig = v as u8;
            unvisited.remove(&v);
            let mut cycle = vec![v];
            while orig != self.rep[v].0 {
                v = self.rep[v].0 as _;
                unvisited.remove(&v);
                cycle.push(v);
            }
            acc.push(Cycle(cycle));
        }
        acc
    }
}

#[derive(Debug)]
struct Cycle(Vec<usize>);

impl Cycle {
    fn into_transposition(self) -> Vec<(usize, isize)> {
        if self.0.len() == 1 {
            vec![]
        }
        else {
            let first = self.0[0];
            self.0.iter().zip(self.0.iter().skip(1).chain(Some(&first)))
                .map(|(a, b)| {
                    let f = bit_map(*b);
                    let g = bit_map(*a);
                    (f, g as isize - f as isize)
                })
                .collect()
        }
    }
}

fn bit_map(piece: usize) -> usize {
    //each piece uses five bits. The given position is the index of the last bit. 
    //i.e. if you have a five bit value it has to be shifted this much left in a u128 to be in the right place.
    match piece {
        0 => 64 + 5 * 7, //DFL
        2 => 64 + 5 * 6, //DBL
        6 => 64 + 5 * 5, //UFL
        8 => 64 + 5 * 4, //UBL
        18 => 64 + 5 * 3, //DFR
        20 => 64 + 5 * 2, //DBR
        24 => 64 + 5 * 1, //UFR
        26 => 64 + 5 * 0, //UFL
        1 => 5 * 11, //DL
        3 => 5 * 10, //FL
        5 => 5 * 9, //BL
        7 => 5 * 8, //UL
        9 => 5 * 7, //DF
        11 => 5 * 6, //DB
        15 => 5 * 5, //UF
        17 => 5 * 4, //UB
        19 => 5 * 3, //DR
        21 => 5 * 2, //FR
        23 => 5 * 1, //BR
        25 => 5 * 0, //UR
        _ => unreachable!(),
    }
}

#[derive(Clone)]
struct Position  {
    x: isize,
    y: isize,
    z: isize,
}

impl Position {
    fn new(x: usize, y: usize, z: usize) -> Position {
        Position { x: x as isize - 1, y: y as isize - 1, z: z as isize - 1 }
    }

    fn n(&mut self, n: usize) -> &mut isize {
        match n {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("A cube only has 3 dimensions you idiot! I am surrounded by idiots!")
        }
    }
}