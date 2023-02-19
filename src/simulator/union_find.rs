pub trait UnionFind<T> {
    fn merge(&mut self, a: T, b: T) -> bool;
    fn find(&mut self, i: T) -> T;
    fn is_root(&mut self, i: T) -> bool;
}

impl UnionFind<i32> for Vec<i32> {
    fn find(&mut self, i: i32) -> i32 {
        if i < 0 {
            return -1;
        }

        if self[i as usize] == -1 {
            return i;
        }

        self[i as usize] = self.find(self[i as usize]) as i32;
        return self[i as usize];
    }

    fn merge(&mut self, a: i32, b: i32) -> bool {
        if a < 0 || b < 0 {
            return false;
        }

        let a_parent = self.find(a);
        let b_parent = self.find(b);

        if a_parent == b_parent {
            return false;
        }

        self[a_parent as usize] = b_parent as i32;

        return true;
    }

    fn is_root(&mut self, i: i32) -> bool {
        if i < 0 {
            return false;
        }

        self.find(i) == i
    }
}
