use crate::types::DataType;

pub trait DataTypeWrapper<T> {
    fn data_type(&self) -> &DataType;
}
