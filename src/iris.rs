pub mod iris_ops {
    use ordered_float::OrderedFloat;

    use crate::containers::CollectionIndexPair;
    use crate::registers::RegisterValue;
    use crate::utils::AnyExecutable;

    fn add(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
        registers.get_value() + data.get_value()
    }

    fn subtract(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
        registers.get_value() - data.get_value()
    }

    fn divide(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
        registers.get_value() / OrderedFloat(2f32)
    }

    fn multiply(registers: CollectionIndexPair, data: CollectionIndexPair) -> RegisterValue {
        registers.get_value() * data.get_value()
    }

    pub const EXECUTABLES: &'static [AnyExecutable; 4] =
        &[self::add, self::subtract, self::divide, self::multiply];
}

mod iris_impl {
    use ordered_float::OrderedFloat;
    use rand::{
        distributions::uniform::{UniformInt, UniformSampler},
        thread_rng,
    };
    use strum::EnumCount;

    use crate::{
        fitness::{Fitness, FitnessScore},
        inputs::Inputs,
        instruction::Instruction,
        metrics::{Accuracy, Metric},
        program::Program,
        registers::{RegisterRepresentable, Registers},
    };

    use super::iris_data::{IrisClass, IrisInput};

    impl<'a> Program<'a, IrisInput> {
        pub fn generate(inputs: &'a Inputs<IrisInput>, max_instructions: usize) -> Self {
            let register_len = <IrisInput as RegisterRepresentable>::get_number_classes();
            let registers = Registers::new(register_len);
            let input_len = <IrisInput as RegisterRepresentable>::get_number_features();

            let executables = super::iris_ops::EXECUTABLES;

            let n_instructions =
                UniformInt::<usize>::new(0, max_instructions).sample(&mut thread_rng());

            let instructions: Vec<Instruction> = (0..n_instructions)
                .map(|_| Instruction::generate(register_len, input_len, executables))
                .collect();

            Program {
                instructions,
                registers,
                inputs,
                fitness: None,
            }
        }
    }

    impl<'a> Fitness for Program<'a, IrisInput> {
        fn eval_fitness(&self) -> FitnessScore {
            let inputs = self.inputs;

            let mut fitness = Accuracy::new(0, 0);

            for input in inputs {
                let mut registers = self.registers.clone();

                for instruction in &self.instructions {
                    let [source_data, target_data] = instruction.get_data(&mut registers, input);

                    instruction.apply(source_data, target_data);
                }

                let correct_index = input.class as usize;
                let registers_argmax = registers.argmax(IrisClass::COUNT, correct_index);

                <Accuracy as Metric>::observe(
                    &mut fitness,
                    Some(correct_index) == registers_argmax,
                );
                registers.reset();
            }

            let fitness_score = fitness.calculate();

            fitness_score
        }
    }
}

pub mod iris_data {
    use core::fmt;

    use ordered_float::OrderedFloat;
    use serde::{
        de::{self, Visitor},
        Deserialize, Deserializer,
    };
    use strum::EnumCount;

    use crate::registers::{RegisterRepresentable, Registers};

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
            return Registers::from(vec![
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
            const FIELDS: &'static [&'static str] =
                &["Iris-setosa", "Iris-versicolor", "Iris-virginica"];

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
