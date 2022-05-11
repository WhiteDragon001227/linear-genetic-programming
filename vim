diff --git a/src/main.rs b/src/main.rs
index 14f0556..b951e98 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -62,71 +62,88 @@ struct TestLGP;
 const IRIS_DATASET_LINK: &'static str =
     "https://archive.ics.uci.edu/ml/machine-learning-databases/iris/bezdekIris.data";
 
-#[derive(Debug, Clone)]
-struct Collection<ItemType>(Vec<ItemType>);
+type Collection<ItemType> = Vec<ItemType>;
 
 #[derive(Debug, Clone)]
 struct Registers(Collection<f32>);
 
-#[derive(Debug, Clone)]
-struct Inputs<InputType: VectorConvertable>(Collection<InputType>);
+impl Registers {
+    fn init(n_registers: usize) -> Registers {
+        Registers(vec![0.; n_registers])
+    }
 
-#[derive(Debug, Clone)]
-struct Fitness(f32);
+    fn reset(&mut self) -> () {
+        let registers = &mut self.0;
 
-trait Verifiable: PartialEq + Eq + Debug {}
-trait Auditable {
-    fn eval_fitness(&self) -> Fitness;
+        for index in 1..registers.len() {
+            registers[index - 1] = 0.;
+        }
+    }
 }
-trait VectorConvertable: Clone + fmt::Debug + Into<Registers>
-where
-    Self::TrueType: Verifiable,
-{
-    type TrueType;
 
-    fn output_is_correct(&self, output_value: Self::TrueType) -> bool;
+#[derive(Debug, Clone)]
+struct Inputs<InputType: VectorConvertable>(Collection<InputType>);
+
+trait Auditable: Eq + fmt::Debug {
+    fn eval_fitness(&self) -> f32;
 }
 
-trait Operable: fmt::Debug
+trait VectorConvertable: Clone + fmt::Debug + Into<Registers> {}
+
+trait Executable: fmt::Debug
 where
     Self::InputType: VectorConvertable,
 {
     type InputType;
 
-    fn exec(&self) -> ();
-
-    // Accessors.
-    fn get_source_index(&self) -> i8;
-    fn get_target_index(&self) -> i8;
-    fn get_registers(&self) -> Registers;
-    fn get_data(&self) -> Exemplars<Self::InputType>;
+    fn exec(
+        &self,
+        registers: &mut Registers,
+        data: Exemplars<Self::InputType>,
+        source_index: i8,
+        target_index: i8,
+    ) -> ();
 
-    fn dyn_clone(&self) -> Box<dyn Operable<InputType = Self::InputType>>;
+    fn dyn_clone(&self) -> AnyExecutable<Self::InputType>;
 }
 
-trait Executable: fmt::Debug + Auditable
+type AnyExecutable<T> = Box<dyn Executable<InputType = T>>;
+
+trait Programmable: fmt::Debug + Auditable
 where
     Self::InputType: VectorConvertable,
 {
     type InputType;
 
-    fn get_input(&self) -> Option<Self::InputType>;
-    fn get_instructions(&self) -> Vec<Box<dyn Operable<InputType = Self::InputType>>>;
-    fn dyn_clone(&self) -> Box<dyn Executable<InputType = Self::InputType>>;
+    fn get_inputs(&self) -> Rc<Inputs<Self::InputType>>;
+    fn get_instructions(&self) -> Collection<AnyExecutable<Self::InputType>>;
+
+    fn dyn_clone(&self) -> AnyProgrammable<Self::InputType>;
 }
 
+impl<T> Clone for AnyProgrammable<T>
+where
+    T: VectorConvertable,
+{
+    fn clone(&self) -> Self {
+        &self.dyn_clone()
+    }
+}
+
+type AnyProgrammable<T> = Box<dyn Programmable<InputType = T>>;
+
 trait Runnable
 where
     Self::InputType: VectorConvertable,
-    Self::ExecutableType: Executable,
+    Self::ProgramType: Programmable,
 {
     type InputType;
-    type ExecutableType;
+    type ProgramType;
 
-    fn load_inputs(&self, file_path: &Path) -> Inputs<Self::InputType>;
-    fn generate_individual(&self) -> Self::ExecutableType;
-    fn init_population(&self, size: usize) -> Population<Self::ExecutableType>;
-    fn compete(&self, retention_percent: f32) -> Population<Self::ExecutableType>;
+    fn load_inputs(&self, file_path: &Path) -> Rc<Inputs<Self::InputType>>;
+    fn generate_individual(&self) -> Self::ProgramType;
+    fn init_population(&self, size: usize) -> Population<Self::ProgramType>;
+    fn compete(&self, retention_percent: f32) -> Population<Self::ProgramType>;
 }
 
 #[derive(Debug, Clone)]
@@ -138,7 +155,10 @@ where
     Input(&'a InputType),
 }
 
-impl<T: VectorConvertable> Clone for Box<dyn Operable<InputType = T>> {
+impl<InputType> Clone for AnyExecutable<InputType>
+where
+    InputType: VectorConvertable,
+{
     fn clone(&self) -> Self {
         self.dyn_clone()
     }
@@ -160,39 +180,30 @@ struct Program<InputType>
 where
     InputType: VectorConvertable,
 {
-    instructions: Vec<Box<dyn Operable<InputType = InputType>>>,
+    instructions: Collection<AnyExecutable<InputType>>,
     inputs: Rc<Inputs<InputType>>,
+    registers: Registers,
 }
 
 impl Auditable for Program<IrisInput> {
-    fn eval_fitness(&self) -> Fitness {
-        /*
-         *for Inputs(input in &self.inputs {
-         *    let mut registers = Collection(Vec::new());
-         *    for instruction in &self.instructions {
-         *        instruction.apply(input, Registers(registers), 0, 1);
-         *    }
-         *}
-         */
-
-        return Fitness(0.);
-    }
-}
+    fn eval_fitness(&self) -> f32 {
+        let inputs = &self.inputs.0;
+        let registers = &self.registers;
 
-impl Executable for Program<IrisInput> {
-    type InputType = IrisInput;
+        for input in inputs {
+            for instruction in &self.instructions {}
+            registers.reset();
 
-    fn get_input(&self) -> Option<Self::InputType> {
-        todo!()
-    }
+            // reset
+            // count - metrics
+        }
 
-    fn get_instructions(&self) -> Vec<Box<dyn Operable<InputType = Self::InputType>>> {
-        todo!()
+        0.
     }
+}
 
-    fn dyn_clone(&self) -> Box<dyn Executable<InputType = Self::InputType>> {
-        todo!()
-    }
+impl Programmable for Program<IrisInput> {
+    type InputType = IrisInput;
 }
 
 #[derive(Debug, Clone)]
@@ -203,7 +214,7 @@ struct HyperParameters {
     input_file_path: PathBuf,
 }
 
-impl<InputType> Clone for Box<dyn Executable<InputType = InputType>>
+impl<InputType> Clone for AnyProgrammable<InputType>
 where
     InputType: VectorConvertable,
 {
@@ -213,28 +224,28 @@ where
 }
 
 #[derive(Debug, Clone)]
-struct Population<ExecutableType: Executable>(Collection<ExecutableType>);
+struct Population<ProgramType: Programmable>(Collection<ProgramType>);
 
 #[derive(Debug, Clone)]
 struct LinearGeneticProgramming<InputType, ExecutableType>
 where
     InputType: VectorConvertable,
-    ExecutableType: Executable,
+    ExecutableType: Programmable,
 {
     hyper_parameters: HyperParameters,
     population: Population<ExecutableType>,
-    inputs: Inputs<InputType>,
+    inputs: Rc<Inputs<InputType>>,
 }
 
-impl<InputType, ExecutableType> LinearGeneticProgramming<InputType, ExecutableType>
+impl<InputType, ProgramType> LinearGeneticProgramming<InputType, ProgramType>
 where
     InputType: VectorConvertable,
-    ExecutableType: Executable,
+    ProgramType: Programmable,
 {
     fn new<T>(
         lgp: T,
         hyper_parameters: HyperParameters,
-    ) -> LinearGeneticProgramming<T::InputType, T::ExecutableType>
+    ) -> LinearGeneticProgramming<T::InputType, T::ProgramType>
     where
         T: Runnable,
         T::InputType: VectorConvertable,
@@ -249,14 +260,19 @@ where
         };
     }
 
-    fn run(&self, lgp: impl Runnable) {}
+    fn run(&self, lgp: impl Runnable) {
+        //
+        // until generation limit is met:
+        //    for every program,
+        //      population = lgp.compete
+    }
 }
 
 impl Runnable for TestLGP {
     type InputType = IrisInput;
-    type ExecutableType = Program<Self::InputType>;
+    type ProgramType = Program<Self::InputType>;
 
-    fn load_inputs(&self, file_path: &Path) -> Inputs<Self::InputType> {
+    fn load_inputs(&self, file_path: &Path) -> Rc<Inputs<Self::InputType>> {
         let mut csv_reader = ReaderBuilder::new()
             .has_headers(false)
             .from_path(file_path)
@@ -267,20 +283,18 @@ impl Runnable for TestLGP {
             .map(|input| -> Self::InputType { input.unwrap() })
             .collect();
 
-        let inputs = Collection(raw_inputs);
-
-        return Inputs(inputs);
+        return Rc::new(Inputs(raw_inputs));
     }
 
-    fn init_population(&self, size: usize) -> Population<Self::ExecutableType> {
+    fn init_population(&self, size: usize) -> Population<Self::ProgramType> {
         todo!()
     }
 
-    fn compete(&self, retention_percent: f32) -> Population<Self::ExecutableType> {
+    fn compete(&self, retention_percent: f32) -> Population<Self::ProgramType> {
         todo!()
     }
 
-    fn generate_individual(&self) -> Self::ExecutableType {
+    fn generate_individual(&self) -> Self::ProgramType {
         todo!()
     }
 }
@@ -289,11 +303,9 @@ impl Runnable for TestLGP {
 enum IrisClass {
     Setosa,
     Versicolour,
-    Virginica = 2,
+    Virginica,
 }
 
-impl Verifiable for IrisClass {}
-
 #[derive(Deserialize, Debug, Clone)]
 struct IrisInput {
     sepal_length: f32,
@@ -304,22 +316,14 @@ struct IrisInput {
     class: IrisClass,
 }
 
-impl VectorConvertable for IrisInput {
-    type TrueType = IrisClass;
-
-    fn output_is_correct(&self, output_value: Self::TrueType) -> bool {
-        output_value == self.class
-    }
-}
-
 impl Into<Registers> for IrisInput {
     fn into(self) -> Registers {
-        return Registers(Collection(vec![
+        return Registers(vec![
             self.sepal_length,
             self.sepal_width,
             self.petal_length,
             self.petal_width,
-        ]));
+        ]);
     }
 }
 
@@ -358,6 +362,12 @@ impl IrisInput {
 }
 
 fn main() {
+    // TODO:
+    // 1. Load Data
+    // 2. Generate Population
+    // 3. Run Programs in Population
+    // 4. Evaluate Programs
+    // 5. Repeat From 3 until N Generations Have Been Created
     println!("Hello, world!");
 }
 
@@ -408,7 +418,7 @@ mod tests {
         let content = get_iris_content().await?;
         writeln!(&tmpfile, "{}", &content)?;
         let test_lgp = TestLGP;
-        let Inputs(Collection(inputs)) = Runnable::load_inputs(&test_lgp, tmpfile.path());
+        let inputs = &Runnable::load_inputs(&test_lgp, tmpfile.path()).0;
         assert_ne!(inputs.len(), 0);
         Ok(())
     }
