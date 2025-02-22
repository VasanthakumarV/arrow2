use std::sync::Arc;

use crate::datatypes::DataType;

use super::Scalar;

#[derive(Debug, Clone)]
pub struct StructScalar {
    values: Vec<Arc<dyn Scalar>>,
    is_valid: bool,
    data_type: DataType,
}

impl PartialEq for StructScalar {
    fn eq(&self, other: &Self) -> bool {
        (self.data_type == other.data_type)
            && (self.is_valid == other.is_valid)
            && ((!self.is_valid) | (self.values == other.values))
    }
}

impl StructScalar {
    #[inline]
    pub fn new(data_type: DataType, values: Option<Vec<Arc<dyn Scalar>>>) -> Self {
        let is_valid = values.is_some();
        Self {
            values: values.unwrap_or_default(),
            is_valid,
            data_type,
        }
    }

    #[inline]
    pub fn values(&self) -> &[Arc<dyn Scalar>] {
        &self.values
    }
}

impl Scalar for StructScalar {
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.is_valid
    }

    #[inline]
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
}
