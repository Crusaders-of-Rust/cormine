use bevy::math::IVec3;
use either::Either;

#[derive(Debug, Clone)]
pub struct Octree<const SZ: usize, T> {
    octants: Vec<Octant<T>>,
}

impl<const SZ: usize, T: Default> Default for Octree<SZ, T> {
    fn default() -> Self {
        Self {
            octants: vec![Octant {
                kind: OctantKind::Chunk(T::default()),
                position: OctantPos(0, 0, 0),
                size: SZ,
                enabled: true,
            }],
        }
    }
}

impl<const SZ: usize, T> Octree<SZ, T>
where
    T: Clone + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Iterate over each value on the octree, in no specific order.
    pub fn iter(&self) -> impl Iterator<Item = (OctantPos, &T)> {
        // FIXME: Use find_map to avoid needing to process Nodes and needing Either
        self.iter_octants().flat_map(|octant| match &octant.kind {
            OctantKind::Chunk(inner) => {
                let start = octant.position;
                let end = start + OctantPos(octant.size, octant.size, octant.size);
                Either::Left((start.0..end.0).flat_map(move |x| {
                    (start.1..end.1).flat_map(move |y| {
                        (start.2..end.2).map(move |z| (OctantPos(x, y, z), inner))
                    })
                }))
            }
            OctantKind::Node(_) => Either::Right(std::iter::empty()),
        })
    }

    /// Iterate over each octant in the tree, in no specific order
    pub fn iter_octants(&self) -> impl Iterator<Item = &Octant<T>> {
        self.octants.iter().filter(|o| o.enabled)
    }

    pub fn get(&self, pos: OctantPos) -> &T {
        self.octants[0]
            .find_child_contents(pos, &self.octants)
            .expect("given position out of range of tree")
    }

    /// Get a mutable reference to the value at the given position
    /// Will split the octree if necessary
    pub fn get_mut(&mut self, pos: OctantPos) -> &mut T
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
                let indexes = self.split_chunk(idx);
                idx = *indexes
                    .iter()
                    .find(|&&subidx| self.octants[subidx].contains(pos))
                    .unwrap();
            }
        }
        let OctantKind::Chunk(inner) = &mut self.octants[idx].kind else {
            unreachable!()
        };
        inner
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
                ..
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
        let size = size / 2;
        for dx in 0..=1 {
            for dy in 0..=1 {
                for dz in 0..=1 {
                    self.octants.push(Octant {
                        kind: OctantKind::Chunk(inner.clone()),
                        position: position + OctantPos(dx * size, dy * size, dz * size),
                        size,
                        enabled: true,
                    });
                }
            }
        }
        self.octants[idx].kind = OctantKind::Node(indexes);
        indexes
    }

    /// Attempt to merge octants, returning `true` if any merges were possible
    pub fn merge(&mut self) -> bool
    where
        T: Eq,
    {
        let mut any = false;
        for idx in 0..self.octants.len() {
            if !self.octants[idx].enabled {
                continue;
            }
            let OctantKind::Node(subindexes) = self.octants[idx].kind else {
                continue;
            };
            let (first, rest) = subindexes.split_first().unwrap();
            let OctantKind::Chunk(first) = &self.octants[*first].kind else {
                continue;
            };
            if rest.iter().all(|&idx| {
                let OctantKind::Chunk(c) = &self.octants[idx].kind else {
                    return false;
                };
                first == c
            }) {
                any = true;
                bevy::log::trace!(
                    "merging node at {:?} (size {})",
                    self.octants[idx].position,
                    self.octants[idx].size
                );
                self.octants[idx].kind = OctantKind::Chunk(first.clone());
                for sub in subindexes {
                    self.octants[sub].enabled = false;
                }
            }
        }
        any
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OctantPos(usize, usize, usize);

impl OctantPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self(x, y, z)
    }

    pub fn new_u32(x: u32, y: u32, z: u32) -> Self {
        Self(x as _, y as _, z as _)
    }
}

impl std::ops::Add for OctantPos {
    type Output = OctantPos;
    fn add(self, rhs: Self) -> Self::Output {
        OctantPos(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl From<OctantPos> for (usize, usize, usize) {
    fn from(value: OctantPos) -> Self {
        (value.0, value.1, value.2)
    }
}

impl From<OctantPos> for IVec3 {
    fn from(value: OctantPos) -> Self {
        IVec3::new(value.0 as _, value.1 as _, value.2 as _)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Octant<T> {
    pub kind: OctantKind<T>,
    pub position: OctantPos,
    /// The side length of the cube
    pub size: usize,
    /// `false` when this entry is an orphan
    // FIXME: Have a way to reuse orphaned nodes
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OctantKind<T> {
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
        let mut tree: Octree<4, u8> = Octree::new();
        assert_eq!(*tree.get(OctantPos(0, 0, 0)), 0);
        *tree.get_mut(OctantPos(1, 1, 1)) = 1;
        assert_eq!(*tree.get(OctantPos(0, 0, 0)), 0);
        assert_eq!(*tree.get(OctantPos(1, 1, 1)), 1);
        *tree.get_mut(OctantPos(1, 1, 3)) = 2;
        assert_eq!(*tree.get(OctantPos(1, 1, 3)), 2);
        let mut elts = tree.iter().collect::<Vec<_>>();
        elts.sort_by_key(|(p, _)| *p);
        eprintln!("{elts:#?}");
        assert_eq!(elts.len(), 4 * 4 * 4);
    }
}
