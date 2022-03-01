pub trait SequentialId<T: Ord> {
    fn seq_id(&self) -> T;
}
