mod table;

use std::marker::PhantomData;

use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};

// Helper that checks that the value witnessed in the given cell is within a given range.
// Depending on the range, it uses either range-check expression(small ranges) or a lookup(large ranges)
//
//      value   |   q_range_check   |  q_lookup  |   table_value  |
//        v     |         1         |     0      |       0        |
//        v'    |         0         |     1      |       1        |
//
//

#[derive(Debug, Clone)]
pub struct RangeCheckConfig<F: FieldExt, const RANGE: usize> {
    pub value: Column<Advice>,
    pub q_range_check: Selector,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const RANGE: usize> RangeCheckConfig<F, RANGE> {
    fn configure(meta: &mut ConstraintSystem<F>, value: Column<Advice>) -> Self {
        // Toggles the range check constraint
        let q_range_check = meta.selector();

        let config = Self {
            q_range_check,
            value,
            _marker: PhantomData::default(),
        };

        // Range-check gate
        // For a value v and range R, check that v < R
        //    v * (1 - v) * (2 - v) * ... * (R - 1 - v) = 0
        meta.create_gate("Range check", |meta| {
            let q_range_check = meta.query_selector(q_range_check);
            let value = meta.query_advice(value, Rotation::cur());

            let range_check = |range: usize, value: Expression<F>| {
                (0..range).fold(value.clone(), |expr, i| {
                    expr * (Expression::Constant(F::from(i as u64)) - value.clone())
                })
            };

            Constraints::with_selector(q_range_check, [("range check", range_check(RANGE, value))])
        });

        config
    }

    fn assign(&self, mut layouter: impl Layouter<F>, value: Value<F>) -> Result<(), Error> {
        layouter.assign_region(
            || "range-check",
            |mut region| {
                // Enable q_range_check
                self.q_range_check.enable(&mut region, 0)?;

                // Assign given value
                region.assign_advice(|| "value", self.value, 0, || value)?;

                Ok(())
            },
        )
    }
}

const RANGE: usize = 0x8;
#[derive(Debug, Default)]
pub struct TestCircuit<F: FieldExt> {
    pub value: Value<F>,
}

impl<F: FieldExt> Circuit<F> for TestCircuit<F> {
    type Config = RangeCheckConfig<F, RANGE>;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let value = meta.advice_column();
        RangeCheckConfig::configure(meta, value)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        RangeCheckConfig::assign(&config, layouter, self.value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::*, pasta::Fp};

    #[test]
    fn test_range_check() {
        let k = 4;

        // Successful cases
        for i in 0..RANGE {
            let circuit = TestCircuit {
                value: Value::known(Fp::from(i as u64)),
            };

            let prover = MockProver::run(k, &circuit, vec![]).unwrap();
            prover.assert_satisfied();
        }

        // Failed case
        let circuit = TestCircuit {
            value: Value::known(Fp::from(RANGE as u64)),
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert!(prover.verify().is_err());
    }
}
