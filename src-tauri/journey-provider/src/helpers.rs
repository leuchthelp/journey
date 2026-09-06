pub fn check_exists<T: PartialEq>(current: &T, slice: &[T]) -> bool {
    slice.contains(current)
}
