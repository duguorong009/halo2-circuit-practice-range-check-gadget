use std::marker::PhantomData;

use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};

fn main() {
    println!("Hello, world!");
}

// Helper that checks that the value witnessed in the given cell is within a given range.
//
//      value   |   q_range_check
//        v     |         1
//
//

#[derive(Debug)]
struct RangeCheckConfig<F: FieldExt, const RANGE: usize> {
    value: Column<Advice>,
    q_range_check: Selector,
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
}
