pub trait Metric {
    type ObservableType;
    type ResultType;

    fn observe(&mut self, value: Self::ObservableType) -> ();
    fn calculate(&self) -> Self::ResultType;
}
