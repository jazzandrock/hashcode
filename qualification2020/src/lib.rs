use std::io::Read;

use helpers::red::Red;

pub type LibraryId = usize;
pub type LibraryScore = u128;
pub type BookScore = u32;
pub type BookId = usize;

#[derive(Clone, Debug)]
pub struct Library {
    pub id: LibraryId,
    pub n_days: usize,
    pub n_ship_daily: usize,
    pub books: Vec<BookId>,
    pub is_zero_score: bool,
    pub used_books: Vec<BookId>,
}

impl Library {
    pub fn new(id: LibraryId, n_days: usize, n_ship_daily: usize, books: Vec<BookId>) -> Self {
        let is_zero_score = false;
        let used_books = Vec::new();
        Self {
            id,
            n_days,
            n_ship_daily,
            books,
            is_zero_score,
            used_books,
        }
    }

    pub fn mark_as_used(&mut self) {
        self.is_zero_score = true;
    }

    pub fn is_used(&self) -> bool {
        self.is_zero_score
    }

    pub fn n_books(&self) -> usize {
        self.books.len()
    }

    pub fn get_best_books(
        &self,
        curr_day: usize,
        n_days: usize,
        book_scores: &[BookScore],
    ) -> Vec<BookId> {
        use std::cmp::min;
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        if curr_day + self.n_days >= n_days {
            return Vec::new();
        }

        // curr_day + self.n_days is the day when we can start subscribing next / scanning
        // n_days = 4
        // self.n_days = 3
        // curr_day = 0
        // curr_day + self.n_days == 3 == n_days - 1
        // so we actually have 1 day
        let days_left = n_days - (curr_day + self.n_days);
        let n_books = days_left * self.n_ship_daily;
        let n_books = min(self.n_books(), n_books);

        let mut heap = BinaryHeap::with_capacity(n_books);
        for &book in &self.books {
            heap.push(Reverse((book_scores[book], book)));
            if heap.len() > n_books {
                heap.pop();
            }
            let mut heap_vec = heap.iter().collect::<Vec<_>>();
            heap_vec.sort();
        }

        let mut res = heap
            .iter()
            .map(|Reverse(tuple)| tuple.1)
            .filter(|b| book_scores[*b] > 0)
            .collect::<Vec<BookId>>();

        res.sort_by_key(|b| Reverse(book_scores[*b]));

        res
    }

    /// select top n_ship_daily books for today
    pub fn best_books_for_day(&self, book_scores: &[BookScore]) -> Vec<BookId> {
        let mut books = self.books.clone();
        books.sort_by_key(|&b| book_scores[b]);
        books.reverse();
        let ship_today = std::cmp::min(self.books.len(), self.n_ship_daily);
        books[0..ship_today].to_vec()
    }
}

pub fn read_problem(
    file_path: impl ToString,
) -> (usize, usize, usize, Vec<BookScore>, Vec<Library>) {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let n_books = red.read::<usize>();
    let n_libs = red.read::<usize>();
    let n_days = red.read::<usize>();

    let book_scores = red.read_vec::<BookScore>(n_books);

    let mut libs = Vec::with_capacity(n_libs);
    for id in 0..n_libs {
        let n_books = red.read::<usize>();
        let n_days = red.read::<usize>();
        let n_ship_daily = red.read::<usize>();
        let books = red.read_vec::<BookId>(n_books);
        libs.push(Library::new(id, n_days, n_ship_daily, books));
    }

    (n_books, n_libs, n_days, book_scores, libs)
}

pub struct SolutionLibrary {
    pub id: LibraryId,
    pub books: Vec<BookId>,
}

pub fn read_solution(file_path: impl ToString) -> Vec<SolutionLibrary> {
    let file = std::fs::File::open(file_path.to_string());
    let iter = std::io::BufReader::new(file.unwrap())
        .bytes()
        .map(Result::unwrap);
    let mut red = Red::new(iter);

    let n_libs = red.read::<usize>();
    let mut libs = Vec::with_capacity(n_libs);
    for _ in 0..n_libs {
        let id = red.read::<LibraryId>();
        let n_books = red.read::<usize>();
        let books = red.read_vec::<BookId>(n_books);
        libs.push(SolutionLibrary { id, books });
    }

    libs
}
