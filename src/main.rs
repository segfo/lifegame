use std::num::Wrapping;

const OFFSET_BASIS32: usize = 2166136261;
const OFFSET_BASIS64: usize = 14695981039346656037;
const FNV_PRIME32: usize = 16777619;
const FNV_PRIME_64: usize = 1099511628211;
struct FNV1;
impl FNV1 {
    fn hash64(cells: &Vec<Vec<Cell>>) -> usize {
        let mut hash = OFFSET_BASIS64 as usize;
        for y in 1..cells.len() - 1 {
            for x in 1..cells[y].len() - 1 {
                hash = (Wrapping(FNV_PRIME_64 as usize) * Wrapping(hash)).0
                    ^ ((Wrapping(x + x) * Wrapping(y + y)).0
                        * if cells[y][x].is_live() { 1 } else { 0 });
            }
        }
        hash
    }
}

/**
 * リングバッファの実装です。
 * このプログラムでは主に、盤面状況（ハッシュ値）の記録をします。
 */
struct RingBuffer<T> {
    buf: Vec<T>,
    front: usize,
    rear: usize,
    buf_size: usize,
    n: usize,
}
impl<T> RingBuffer<T>
where
    T: Clone + PartialEq + Copy,
{
    fn new(n: usize, init_data: T) -> Self {
        RingBuffer {
            buf: vec![init_data; n],
            front: 0,
            rear: 0,
            buf_size: 0,
            n: n,
        }
    }
    fn enqueue(&mut self, data: T) {
        if self.buf_size < self.n {
            self.buf_size += 1;
        } else {
            self.rear %= self.n;
            self.rear += 1;
        }
        self.front %= self.n;
        self.buf[self.front] = data;
        self.front += 1;
    }
    fn dequeue(&mut self) -> Option<T> {
        if self.buf_size > 0 {
            self.buf_size -= 1;
        } else {
            return None;
        }
        self.rear %= self.n;
        let data = self.buf[self.rear];
        self.rear += 1;
        Some(data)
    }
    fn is_in_data(&self, data: T) -> bool {
        self.buf.contains(&data)
    }
}

/**
 * 盤面を表す構造体
 * 盤面全体の状態を管理します。
 */
struct Board {
    array: Vec<Vec<Cell>>,
    old_hash: RingBuffer<usize>,
    width: usize,
    height: usize,
}
impl Board {
    fn new(x: usize, y: usize, board_histories: usize) -> Self {
        Board {
            array: vec![vec![Cell::new(); x + 2]; y + 2],
            old_hash: RingBuffer::new(board_histories, 0),
            width: x,
            height: y,
        }
    }
    fn get_board_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    fn set_live(&mut self, points: Vec<(usize, usize)>) {
        for (x, y) in points {
            self.array[y + 1][x + 1].set_state(CellState::Live);
        }
    }
    fn reflesh_state(&mut self) {
        for y in 1..self.array.len() - 1 {
            for x in 1..self.array[y].len() - 1 {
                // セルが生きてたら、周りのセルに対して生存セルが1つ有ることを通知する
                if self.array[y][x].is_live() {
                    for y0 in 0..=2 {
                        for x0 in 0..=2 {
                            self.array[y + y0 - 1][x + x0 - 1].touch();
                        }
                    }
                    // 自分自身に対しての通知操作は取り消す。
                    self.array[y][x].untouch();
                }
            }
        }
    }
    fn commit_state(&mut self) {
        // コミット前の盤面のハッシュを取得しておく
        // もし、この状態と、is_doneメソッドが呼ばれた時に算出したハッシュが同一であれば終了と判定する。
        self.old_hash.enqueue(self.to_hash());
        self.array
            .iter_mut()
            .for_each(|row| row.into_iter().for_each(|cell| cell.commit_state()));
    }
    fn to_hash(&self) -> usize {
        FNV1::hash64(&self.array)
    }
    fn show_board(&self) {
        for row in &self.array {
            print!("[");
            for cell in row {
                print!("{}", if cell.is_live() { "*" } else { " " });
            }
            println!("]");
        }
        println!("======================================");
    }
    fn is_done(&self) -> bool {
        self.old_hash.is_in_data(self.to_hash())
    }
}
#[derive(Debug, Clone, PartialEq)]
enum CellState {
    Dead,
    Live,
}
#[derive(Debug, Clone)]
struct Cell {
    now_state: CellState,
    count: usize,
}
impl Cell {
    fn new() -> Self {
        Cell {
            now_state: CellState::Dead,
            count: 0,
        }
    }
    fn touch(&mut self) {
        self.count += 1
    }
    fn untouch(&mut self) {
        self.count -= 1
    }
    fn is_live(&self) -> bool {
        self.now_state == CellState::Live
    }
    fn commit_state(&mut self) {
        self.now_state = match self.count {
            3 => CellState::Live,
            2 => self.now_state.clone(),
            _ => CellState::Dead,
        };
        self.count = 0;
    }
    fn set_state(&mut self, state: CellState) {
        self.now_state = state;
    }
}

fn main() {
    let mut board = Board::new(25, 25, 100);
    let (mut x, mut y) = board.get_board_size();
    x /= 2;
    y /= 2;
    board.show_board();
    board.set_live(
        [
            (x + 1, y + 0),
            (x + 1, y + 1),
            (x + 0, y + 1),
            (x + 1, y + 2),
            (x + 6, y + 0),
            (x + 6, y + 1),
            (x + 7, y + 1),
            (x + 6, y + 2),
        ]
        .to_vec(),
    );

    loop {
        board.reflesh_state();
        board.commit_state();
        board.show_board();
        if board.is_done() {
            break;
        }
    }
}
