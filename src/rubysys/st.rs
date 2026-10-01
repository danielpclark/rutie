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
    // st_table *
    // rb_st_init_existing_table_with_size(st_table *tab, const struct st_hash_type *type, st_index_t size)
    //
    // Ruby 3.3 only (3.4 removed it): initializes the `st_table` at `tab`
    // (memory the caller owns, such as one allocated with `ruby_xmalloc`
    // that `rb_st_free_table` may then free) as an empty table for about
    // `size` entries; returns `tab`.
    #[cfg(ruby_3_3)]
    pub fn rb_st_init_existing_table_with_size(
        table: *mut StTable,
        hash_type: *const StHashType,
        size: StIndex,
    ) -> *mut StTable;
    // st_table *
    // rb_st_replace(st_table *new_tab, st_table *old_tab)
    //
    // Ruby 3.3 only (3.4 removed it): makes the `st_table` at `new_tab` a
    // copy of `old_tab`, overwriting it without freeing what it held;
    // returns `new_tab`.
    #[cfg(ruby_3_3)]
    pub fn rb_st_replace(new_table: *mut StTable, old_table: *mut StTable) -> *mut StTable;
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

    // `st_table` is at most eight words (seven on 64-bit).
    #[cfg(ruby_3_3)]
    const TABLE_SIZE: usize = 8 * std::mem::size_of::<usize>();

    #[cfg(ruby_3_3)]
    extern "C" fn compare(left: StData, right: StData) -> c_int {
        (left != right) as c_int
    }

    #[cfg(ruby_3_3)]
    extern "C" fn hash(key: StData) -> StIndex {
        key
    }

    #[cfg(ruby_3_3)]
    #[test]
    fn test_st_existing_tables() {
        use crate::rubysys::gc::ruby_xmalloc;

        static TYPE: StHashType = StHashType { compare, hash };

        crate::on_ruby_thread(|| unsafe {
            let table = ruby_xmalloc(TABLE_SIZE) as *mut StTable;

            assert_eq!(rb_st_init_existing_table_with_size(table, &TYPE, 4), table);
            assert_eq!(rb_st_table_size(table), 0);
            assert_eq!(rb_st_insert(table, 7, 70), 0);
            assert_eq!(rb_st_insert(table, 7, 71), 1);

            let copy = ruby_xmalloc(TABLE_SIZE) as *mut StTable;

            assert_eq!(rb_st_replace(copy, table), copy);
            assert_eq!(rb_st_table_size(copy), 1);
            assert_eq!(rb_st_insert(copy, 8, 80), 0);
            assert_eq!((rb_st_table_size(table), rb_st_table_size(copy)), (1, 2));

            rb_st_free_table(table);
            rb_st_free_table(copy);
        });
    }
}
