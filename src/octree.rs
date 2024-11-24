#[derive(Default, Debug)]
pub struct Octree<const SZ: usize, T> {
    octants: Vec<Octant<T>>,
}

impl<const SZ: usize, T> Octree<SZ, T>
where
    T: Clone + Default,
{
    pub fn new(body: T) -> Self {
        Self {
            octants: vec![Octant {
                kind: OctantKind::Chunk(T::default()),
                position: OctantPos(0, 0, 0),
                size: SZ,
            }],
        }
    }

    pub fn get(&self, pos: OctantPos) -> &T {
        self.octants[0]
            .find_child_contents(pos, &self.octants)
            .expect("given position out of range of tree")
    }

    pub fn insert(&mut self, pos: OctantPos, value: T)
    where
        T: std::fmt::Debug,
    {
        let mut idx = self.octants[0]
            .find_containing_chunk_idx(0, pos, &self.octants)
            .expect("given position out of range of tree");

        while self.octants[idx].size > 1 {
            if let OctantKind::Node(subidxs) = &self.octants[idx].kind {
                idx = *subidxs
                    .iter()
                    .find(|&&subidx| self.octants[subidx].contains(pos))
                    .unwrap();
            } else {
                idx = *self
                    .split_chunk(idx)
                    .iter()
                    .find(|&&subidx| self.octants[subidx].contains(pos))
                    .unwrap();
            }
        }
        let OctantKind::Chunk(inner) = &mut self.octants[idx].kind else {
            unreachable!()
        };
        *inner = value;
    }

    fn split_chunk(&mut self, idx: usize) -> [usize; 8] {
        let (inner, &position, &size) = match &self.octants[idx] {
            Octant { size: 1, .. } => panic!("splitting octant of size 1"),
            Octant {
                kind: OctantKind::Node(_),
                ..
            } => panic!("splitting already split chunk"),
            Octant {
                kind: OctantKind::Chunk(inner),
                position,
                size,
            } => (inner.clone(), position, size),
        };

        let first_idx = self.octants.len();
        let indexes = [
            first_idx,
            first_idx + 1,
            first_idx + 2,
            first_idx + 3,
            first_idx + 4,
            first_idx + 5,
            first_idx + 6,
            first_idx + 7,
        ];
        for dx in 0..=1 {
            for dy in 0..=1 {
                for dz in 0..=1 {
                    self.octants.push(Octant {
                        kind: OctantKind::Chunk(inner.clone()),
                        position: position + OctantPos(dx, dy, dz),
                        size: size / 2,
                    });
                }
            }
        }
        self.octants[idx].kind = OctantKind::Node(indexes);
        indexes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OctantPos(usize, usize, usize);

impl std::ops::Add for OctantPos {
    type Output = OctantPos;
    fn add(self, rhs: Self) -> Self::Output {
        OctantPos(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

#[derive(Debug)]
struct Octant<T> {
    kind: OctantKind<T>,
    position: OctantPos,
    /// The side length of the cube
    size: usize,
}

#[derive(Debug)]
enum OctantKind<T> {
    /// A contiguous chunk of elements (may be a single element)
    Chunk(T),
    /// A split chunk of 8 different kinds
    Node([usize; 8]),
}

impl<T> Octant<T> {
    fn contains(&self, pos: OctantPos) -> bool {
        (self.position.0..(self.position.0 + self.size)).contains(&pos.0)
            && (self.position.1..(self.position.1 + self.size)).contains(&pos.1)
            && (self.position.2..(self.position.2 + self.size)).contains(&pos.2)
    }

    fn find_child_contents<'a>(&'a self, pos: OctantPos, nodes: &'a [Octant<T>]) -> Option<&'a T> {
        match &self.kind {
            OctantKind::Chunk(inner) if self.contains(pos) => Some(inner),
            OctantKind::Chunk(_) => None,
            OctantKind::Node(child_idxs) if self.contains(pos) => child_idxs
                .iter()
                .find_map(|&idx| nodes[idx].find_child_contents(pos, nodes)),
            OctantKind::Node(_) => None,
        }
    }

    fn find_containing_chunk_idx(
        &self,
        self_idx: usize,
        pos: OctantPos,
        nodes: &[Octant<T>],
    ) -> Option<usize> {
        match &self.kind {
            OctantKind::Chunk(_) if self.contains(pos) => Some(self_idx),
            OctantKind::Chunk(_) => None,
            OctantKind::Node(child_idxs) if self.contains(pos) => child_idxs
                .iter()
                .find_map(|&idx| nodes[idx].find_containing_chunk_idx(idx, pos, nodes)),
            OctantKind::Node(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn octree() {
        let mut tree: Octree<3, u8> = Octree::new(0);
        assert_eq!(*tree.get(OctantPos(0, 0, 0)), 0);
        tree.insert(OctantPos(1, 1, 1), 1);
        assert_eq!(*tree.get(OctantPos(0, 0, 0)), 0);
        assert_eq!(*tree.get(OctantPos(1, 1, 1)), 1);
        tree.insert(OctantPos(1, 1, 1), 2);
        assert_eq!(*tree.get(OctantPos(1, 1, 1)), 2);
    }
}
