use crate::rubysys::{
    libc::uintptr_t,
    types::{c_int, size_t},
};

// `struct st_table`, Ruby's hash table (`ruby/st.h`); only used through
// pointers.
#[repr(C)]
pub struct StTable {
    _private: [u8; 0],
}

// st_data_t: an `unsigned long`, or `unsigned long long` where that is the
// size of a pointer (64-bit Windows).
pub type StData = uintptr_t;

// st_index_t: the same size as `st_data_t`.
pub type StIndex = StData;

// `struct st_hash_type`: how a table compares and hashes its keys.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct StHashType {
    // Returns 0 when the keys are equal.
    pub compare: extern "C" fn(StData, StData) -> c_int,
    pub hash: extern "C" fn(StData) -> StIndex,
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // st_table *
    // rb_st_init_numtable(void)
    //
    // An empty table keyed by integers (or pointers); free it with
    // `rb_st_free_table`.
    pub fn rb_st_init_numtable() -> *mut StTable;
    // int
    // rb_st_insert(st_table *, st_data_t, st_data_t)
    //
    // Returns 1 when the key already existed (its value is replaced), else 0.
    pub fn rb_st_insert(table: *mut StTable, key: StData, value: StData) -> c_int;
    // void
    // rb_st_free_table(st_table *)
    pub fn rb_st_free_table(table: *mut StTable);
    // size_t
    // rb_st_table_size(const struct st_table *tbl)
    //
    // The number of entries in `tbl` (`st_table_size`).
    pub fn rb_st_table_size(table: *const StTable) -> size_t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_st_numtable() {
        crate::on_ruby_thread(|| unsafe {
            let table = rb_st_init_numtable();

            assert_eq!(rb_st_insert(table, 1, 10), 0);
            assert_eq!(rb_st_insert(table, 2, 20), 0);
            assert_eq!(rb_st_insert(table, 1, 11), 1);

            assert_eq!(rb_st_table_size(table), 2);

            rb_st_free_table(table);
        });
    }
}
