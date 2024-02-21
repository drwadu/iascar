use std::collections::HashSet;

pub trait ToHashSet<T> {
    fn to_hashset(&self) -> HashSet<T>;
    fn union(&self, other: &[T]) -> Vec<T>;
    fn intersect(&self, other: &[T]) -> Vec<T>;
    fn intersect_with_hs(&self, other: &HashSet<T>) -> Vec<T>;
    fn to_intersection_with_hs(&self, other: &HashSet<T>) -> HashSet<T>;
}
impl<T> ToHashSet<T> for Vec<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    fn to_hashset(&self) -> HashSet<T> {
        self.iter().cloned().collect::<HashSet<_>>()
    }
    fn union(&self, other: &[T]) -> Vec<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.union(y).cloned().collect::<Vec<_>>()
    }
    fn intersect(&self, other: &[T]) -> Vec<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.intersection(y).cloned().collect::<Vec<_>>()
    }
    fn intersect_with_hs(&self, other: &HashSet<T>) -> Vec<T> {
        let x = self.to_hashset();

        x.intersection(other).cloned().collect::<Vec<_>>()
    }
    fn to_intersection_with_hs(&self, other: &HashSet<T>) -> HashSet<T> {
        let x = self.to_hashset();

        x.intersection(other).cloned().collect::<HashSet<_>>()
    }
}
impl<T> ToHashSet<T> for &[T]
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    fn to_hashset(&self) -> HashSet<T> {
        self.iter().cloned().collect::<HashSet<_>>()
    }
    fn union(&self, other: &[T]) -> Vec<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.union(y).cloned().collect::<Vec<_>>()
    }
    fn intersect(&self, other: &[T]) -> Vec<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.intersection(y).cloned().collect::<Vec<_>>()
    }
    fn intersect_with_hs(&self, other: &HashSet<T>) -> Vec<T> {
        let x = self.to_hashset();

        x.intersection(other).cloned().collect::<Vec<_>>()
    }
    fn to_intersection_with_hs<'a>(&self, other: &HashSet<T>) -> HashSet<T> {
        let x = self.to_hashset();

        x.intersection(other).cloned().collect::<HashSet<_>>()
    }
}
