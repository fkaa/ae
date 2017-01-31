pub struct Task<'a, T: 'a + Copy + Clone> {
    func: fn(T),
    parent: Option<&'a mut Task<'a, T>>,
    unfinished: i32,
    data: [u8; 52],
}

impl<'a, T: Copy + Clone> Task<'a, T> {
    pub fn with_data(func: fn(T), data: T) -> Task<'a, T> {
        Task {
            func: func,
            parent: None,
            unfinished: 0,
            data: [0u8; 52]
        }
    }
}
