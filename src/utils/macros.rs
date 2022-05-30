#[macro_use]
#[macro_export]
macro_rules! executable {
    ( $fn_name: ident,  $op: tt, $val: expr) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op $val
            }

            return registers;
        }
    };

    ( $fn_name: ident, $op: tt) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op data[index]
            }

            return registers;
        }
    };

}
