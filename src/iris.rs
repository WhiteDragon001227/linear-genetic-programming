pub mod iris_ops {
  use crate::collection::CollectionIndexPair;
  use crate::registers::RegisterValue;
  use crate::utils::AnyExecutable;

  fn add(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
    ordered_float::OrderedFloat(0.)
  }

  fn subtract(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
    ordered_float::OrderedFloat(0.)
  }

  fn divide(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
    ordered_float::OrderedFloat(0.)
  }

  pub const EXECUTABLES: &'static [AnyExecutable; 3] = &[self::add, self::subtract, self::divide];
}

pub mod iris_data {
  use core::fmt;

  use ordered_float::OrderedFloat;
  use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
  };
  use strum::EnumCount;

  use crate::{RegisterRepresentable, Registers};

  pub const IRIS_DATASET_LINK: &'static str =
    "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";

  #[derive(Debug, Clone, Copy, Eq, PartialEq, EnumCount)]
  pub enum IrisClass {
    Setosa = 0,
    Versicolour = 1,
    Virginica = 2,
  }

  #[derive(Deserialize, Debug, Clone, PartialEq)]
  pub struct IrisInput {
    sepal_length: f32,
    sepal_width: f32,
    petal_length: f32,
    petal_width: f32,
    #[serde(deserialize_with = "IrisInput::deserialize_iris_class")]
    pub class: IrisClass,
  }

  impl RegisterRepresentable for IrisInput {
    fn get_number_classes() -> usize {
      IrisClass::COUNT
    }

    fn get_number_features() -> usize {
      4
    }
  }

  impl Into<Registers> for IrisInput {
    fn into(self) -> Registers {
      return Registers(vec![
        OrderedFloat(self.sepal_length),
        OrderedFloat(self.sepal_width),
        OrderedFloat(self.petal_length),
        OrderedFloat(self.petal_width),
      ]);
    }
  }

  impl IrisInput {
    fn deserialize_iris_class<'de, D>(deserializer: D) -> Result<IrisClass, D::Error>
    where
      D: Deserializer<'de>,
    {
      const FIELDS: &'static [&'static str] = &["Iris-setosa", "Iris-versicolor", "Iris-virginica"];

      struct IrisClassVisitor;

      impl<'de> Visitor<'de> for IrisClassVisitor {
        type Value = IrisClass;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str(&FIELDS.join(" or "))
        }

        fn visit_str<E>(self, value: &str) -> Result<IrisClass, E>
        where
          E: de::Error,
        {
          match value {
            "Iris-setosa" => Ok(IrisClass::Setosa),
            "Iris-versicolor" => Ok(IrisClass::Versicolour),
            "Iris-virginica" => Ok(IrisClass::Virginica),
            _ => Err(de::Error::unknown_field(value, FIELDS)),
          }
        }
      }

      deserializer.deserialize_str(IrisClassVisitor)
    }
  }
}
