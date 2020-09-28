use core::{
    iter::{IntoIter, IntoIterator}
}

pub struct Row<T> {
    pub row_id: usize,
    pub deleted: bool,
    pub data: T
}

pub struct Table<T> {
    pub storage: Vec<Row<T>>,
    pub available: Vec<usize>
}

impl<T> Table<T> {
    pub fn push(&mut self, value: T) -> usize {
        // inserts value to end of list if necessary, or finds an earlier slot if available.
        if let Some(avail) = self.available.pop() {
            let mut i = self.storage.get(avail).get_mut().unwrap();
            i.data = value;
            i.deleted = false;
            return avail
        } else {
            let new_idx = self.storage.len();
            let new_row = Row<T> {
                row_id: new_idx,
                deleted: false,
                data: value
            };
            self.storage.push(new_row);
            return new_idx
        }
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.available.clear();
    }

    pub fn remove(&mut self, idx: usize) -> bool {
        if let Some(res) = self.storage.get_mut(idx) {
            
        } else {
            false
        }
        
    }

    


}

impl IntoIterator<T> for Table<T> {
    type Item = Row<T>;
    type IntoIter = IntoIter<Row<T>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.storage.len(),
            inner: self.storage.into_iter()
        }
    }

}