use crate::{executables, genes::registers::RegisterValue};
use ordered_float::OrderedFloat;

use crate::{executable, utils::common_traits::AnyExecutable};

executable!(add, +);
executable!(multiply, *);
executable!(subtract, -);
executable!(divide, /, OrderedFloat(2f32));

pub const IRIS_EXECUTABLES: &[AnyExecutable] =
    executables!(self::add, self::subtract, self::divide, self::multiply);
