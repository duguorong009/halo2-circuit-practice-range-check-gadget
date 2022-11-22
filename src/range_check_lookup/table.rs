use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Layouter, Value},
    pasta::Fp,
    plonk::Error,
    plonk::{ConstraintSystem, TableColumn},
};

// A lookup table of values of NUM_BITS length
// e.g. NUM_BITS = 8, values = [0..255]

#[derive(Debug, Clone)]
pub(super) struct RangeCheckTable<F: FieldExt, const NUM_BITS: usize> {
    value: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const NUM_BITS: usize> RangeCheckTable<F, NUM_BITS> {
    pub(super) fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let value = meta.lookup_table_column();
        Self {
            value,
            _marker: PhantomData::default(),
        }
    }

    pub(super) fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "load range-check table",
            |mut table| {
                let mut offset = 0;
                for i in 0..(1 << NUM_BITS) {
                    table.assign_cell(
                        || "assign cell",
                        self.value,
                        offset,
                        || Value::known(F::from(i as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }
}
