// Ruby's hash table, `st_table` (`ruby/st.h`): for C-level code that keeps
// tables of its own (Rust code is better served by `Hash` or Rust maps).
// `st_*` names in C are macros for these `rb_st_*` functions.
//
// Keys and values are `st_data_t`: integers, pointers or `VALUE`s. A table
// holding `VALUE`s is not marked by the GC on its own; mark it with
// `rb_mark_tbl`, `rb_mark_set` or `rb_mark_hash` (in `gc.rs`).

use crate::rubysys::{
    libc::uintptr_t,
    types::{c_char, c_int, c_long, c_void, size_t, Value},
};

// `struct st_table`; only used through pointers.
#[repr(C)]
pub struct StTable {
    _private: [u8; 0],
}

// st_data_t: an `unsigned long`, or `unsigned long long` where that is the
// size of a pointer (64-bit Windows).
pub type StData = uintptr_t;

// st_index_t: the same type as `st_data_t`.
pub type StIndex = StData;

// struct st_hash_type: how a table compares (0 for equal keys) and hashes
// its keys.
#[repr(C)]
pub struct StHashType {
    pub compare: Option<rutie_callback!(type fn(StData, StData) -> c_int)>,
    pub hash: Option<rutie_callback!(type fn(StData) -> StIndex)>,
}

// typedef int st_foreach_callback_func(st_data_t key, st_data_t value, st_data_t arg)
//
// Returns an `st_retval`: `ST_CONTINUE` (0), `ST_STOP` (1), `ST_DELETE` (2)
// or `ST_CHECK` (3).
pub type StForeachCallbackFunction =
    rutie_callback!(type fn(key: StData, value: StData, arg: StData) -> c_int);

// typedef int st_foreach_check_callback_func(st_data_t, st_data_t, st_data_t, int)
//
// Like `st_foreach_callback_func`; the last argument is non-zero when the
// table changed during the iteration.
pub type StForeachCheckCallbackFunction =
    rutie_callback!(type fn(key: StData, value: StData, arg: StData, error: c_int) -> c_int);

// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg,
//                                     int existing)
//
// May change `*key` and `*value` (`*value` is unset when `existing` is 0);
// returns `ST_CONTINUE` to store them or `ST_DELETE` to remove the entry.
pub type StUpdateCallbackFunction = rutie_callback!(type fn(
    key: *mut StData,
    value: *mut StData,
    arg: StData,
    existing: c_int,
) -> c_int);

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

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE)
    //
    // Inserts `argc` (even) keys and values into the `Hash`'s table.
    pub fn rb_hash_bulk_insert_into_st_table(argc: c_long, argv: *const Value, hash: Value);
    // void
    // rb_st_add_direct(st_table *, st_data_t, st_data_t)
    //
    // Inserts without checking whether the key is already there.
    pub fn rb_st_add_direct(table: *mut StTable, key: StData, value: StData);
    // void
    // rb_st_cleanup_safe(st_table *, st_data_t)
    pub fn rb_st_cleanup_safe(table: *mut StTable, never: StData);
    // void
    // rb_st_clear(st_table *)
    pub fn rb_st_clear(table: *mut StTable);
    // st_table *
    // rb_st_copy(st_table *)
    pub fn rb_st_copy(table: *mut StTable) -> *mut StTable;
    // int
    // rb_st_delete(st_table *, st_data_t *, st_data_t *)
    //
    // Removes `*key`; returns 1 and stores the entry's key and value (`value`
    // may be NULL) when it was there, else 0.
    pub fn rb_st_delete(table: *mut StTable, key: *mut StData, value: *mut StData) -> c_int;
    // int
    // rb_st_delete_safe(st_table *, st_data_t *, st_data_t *, st_data_t)
    pub fn rb_st_delete_safe(
        table: *mut StTable,
        key: *mut StData,
        value: *mut StData,
        never: StData,
    ) -> c_int;
    // int
    // rb_st_foreach(st_table *, st_foreach_callback_func *, st_data_t)
    //
    // Returns 0 (1 when the table changed in a way the iteration cannot
    // follow, with `ST_CHECK`).
    pub fn rb_st_foreach(
        table: *mut StTable,
        func: StForeachCallbackFunction,
        arg: StData,
    ) -> c_int;
    // int
    // rb_st_foreach_check(st_table *, st_foreach_check_callback_func *, st_data_t, st_data_t)
    pub fn rb_st_foreach_check(
        table: *mut StTable,
        func: StForeachCheckCallbackFunction,
        arg: StData,
        never: StData,
    ) -> c_int;
    // int
    // rb_st_foreach_with_replace(st_table *tab, st_foreach_check_callback_func *func,
    //                            st_update_callback_func *replace, st_data_t arg)
    //
    // `func` returning `ST_REPLACE` (4) calls `replace` for the entry.
    pub fn rb_st_foreach_with_replace(
        table: *mut StTable,
        func: StForeachCheckCallbackFunction,
        replace: StUpdateCallbackFunction,
        arg: StData,
    ) -> c_int;
    // int
    // rb_st_get_key(st_table *, st_data_t, st_data_t *)
    //
    // Like `rb_st_lookup`, storing the key as stored in the table.
    pub fn rb_st_get_key(table: *mut StTable, key: StData, result: *mut StData) -> c_int;
    // st_index_t
    // rb_st_hash(const void *ptr, size_t len, st_index_t h)
    //
    // Hashes `len` bytes, seeded with `h`.
    pub fn rb_st_hash(ptr: *const c_void, len: size_t, h: StIndex) -> StIndex;
    // st_index_t
    // rb_st_hash_end(st_index_t h)
    pub fn rb_st_hash_end(h: StIndex) -> StIndex;
    // st_index_t
    // rb_st_hash_start(st_index_t h)
    pub fn rb_st_hash_start(h: StIndex) -> StIndex;
    // st_index_t
    // rb_st_hash_uint(st_index_t h, st_index_t i)
    pub fn rb_st_hash_uint(h: StIndex, i: StIndex) -> StIndex;
    // st_index_t
    // rb_st_hash_uint32(st_index_t h, uint32_t i)
    pub fn rb_st_hash_uint32(h: StIndex, i: u32) -> StIndex;
    // st_table *
    // rb_st_init_numtable_with_size(st_index_t)
    pub fn rb_st_init_numtable_with_size(size: StIndex) -> *mut StTable;
    // st_table *
    // rb_st_init_strcasetable(void)
    //
    // Keys are C strings, compared ignoring ASCII case.
    pub fn rb_st_init_strcasetable() -> *mut StTable;
    // st_table *
    // rb_st_init_strcasetable_with_size(st_index_t)
    pub fn rb_st_init_strcasetable_with_size(size: StIndex) -> *mut StTable;
    // st_table *
    // rb_st_init_strtable(void)
    //
    // Keys are C strings (`const char *`), which the table does not copy.
    pub fn rb_st_init_strtable() -> *mut StTable;
    // st_table *
    // rb_st_init_strtable_with_size(st_index_t)
    pub fn rb_st_init_strtable_with_size(size: StIndex) -> *mut StTable;
    // st_table *
    // rb_st_init_table(const struct st_hash_type *)
    //
    // `type` must outlive the table.
    pub fn rb_st_init_table(hash_type: *const StHashType) -> *mut StTable;
    // st_table *
    // rb_st_init_table_with_size(const struct st_hash_type *, st_index_t)
    pub fn rb_st_init_table_with_size(hash_type: *const StHashType, size: StIndex) -> *mut StTable;
    // int
    // rb_st_insert2(st_table *, st_data_t, st_data_t, st_data_t (*)(st_data_t))
    //
    // Like `rb_st_insert`, storing `func(key)` as the key of a new entry
    // (with the hash of `key`, so `func` must keep keys equal).
    pub fn rb_st_insert2(
        table: *mut StTable,
        key: StData,
        value: StData,
        func: rutie_callback!(type fn(StData) -> StData),
    ) -> c_int;
    // st_index_t
    // rb_st_keys(st_table *table, st_data_t *keys, st_index_t size)
    //
    // Copies up to `size` keys into `keys`; returns how many.
    pub fn rb_st_keys(table: *mut StTable, keys: *mut StData, size: StIndex) -> StIndex;
    // st_index_t
    // rb_st_keys_check(st_table *table, st_data_t *keys, st_index_t size, st_data_t never)
    pub fn rb_st_keys_check(
        table: *mut StTable,
        keys: *mut StData,
        size: StIndex,
        never: StData,
    ) -> StIndex;
    // int
    // rb_st_locale_insensitive_strcasecmp(const char *s1, const char *s2)
    pub fn rb_st_locale_insensitive_strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    // int
    // rb_st_locale_insensitive_strncasecmp(const char *s1, const char *s2, size_t n)
    pub fn rb_st_locale_insensitive_strncasecmp(
        s1: *const c_char,
        s2: *const c_char,
        n: size_t,
    ) -> c_int;
    // int
    // rb_st_lookup(st_table *, st_data_t, st_data_t *)
    //
    // Returns 1 and stores the value in `*value` (which may be NULL) when
    // `key` is there, else 0.
    pub fn rb_st_lookup(table: *mut StTable, key: StData, value: *mut StData) -> c_int;
    // size_t
    // rb_st_memsize(const st_table *)
    pub fn rb_st_memsize(table: *const StTable) -> size_t;
    // int
    // rb_st_numcmp(st_data_t, st_data_t)
    pub fn rb_st_numcmp(x: StData, y: StData) -> c_int;
    // st_index_t
    // rb_st_numhash(st_data_t)
    pub fn rb_st_numhash(n: StData) -> StIndex;
    // int
    // rb_st_shift(st_table *, st_data_t *, st_data_t *)
    //
    // Removes the first entry, storing its key and value; 0 when empty.
    pub fn rb_st_shift(table: *mut StTable, key: *mut StData, value: *mut StData) -> c_int;
    // int
    // rb_st_update(st_table *table, st_data_t key, st_update_callback_func *func,
    //              st_data_t arg)
    //
    // Returns 1 when `key` existed.
    pub fn rb_st_update(
        table: *mut StTable,
        key: StData,
        func: StUpdateCallbackFunction,
        arg: StData,
    ) -> c_int;
    // st_index_t
    // rb_st_values(st_table *table, st_data_t *values, st_index_t size)
    pub fn rb_st_values(table: *mut StTable, values: *mut StData, size: StIndex) -> StIndex;
    // st_index_t
    // rb_st_values_check(st_table *table, st_data_t *values, st_index_t size, st_data_t never)
    pub fn rb_st_values_check(
        table: *mut StTable,
        values: *mut StData,
        size: StIndex,
        never: StData,
    ) -> StIndex;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rubysys::types::st_retval;
    use std::{ffi::CStr, ptr};

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

    rutie_callback! {
        fn sum_entries(key: StData, value: StData, arg: StData) -> c_int {
            unsafe { *(arg as *mut StData) += key * 100 + value };
            st_retval::Continue as c_int
        }
    }

    rutie_callback! {
        fn delete_odd_keys(key: StData, _value: StData, _arg: StData, _error: c_int) -> c_int {
            if key % 2 == 1 { st_retval::Delete as c_int } else { st_retval::Continue as c_int }
        }
    }

    rutie_callback! {
        fn replace_all(_key: StData, _value: StData, _arg: StData, _error: c_int) -> c_int {
            st_retval::Replace as c_int
        }
    }

    rutie_callback! {
        fn double_value(_key: *mut StData, value: *mut StData, arg: StData, existing: c_int) -> c_int {
            unsafe {
                *value = if existing != 0 { *value * 2 } else { arg };
            }
            st_retval::Continue as c_int
        }
    }

    rutie_callback! {
        fn add_one(key: StData) -> StData {
            key + 1
        }
    }

    rutie_callback! {
        fn compare_mod_10(x: StData, y: StData) -> c_int {
            (x % 10 != y % 10) as c_int
        }
    }

    rutie_callback! {
        fn hash_mod_10(x: StData) -> StIndex {
            x % 10
        }
    }

    static MOD_10: StHashType = StHashType {
        compare: Some(compare_mod_10),
        hash: Some(hash_mod_10),
    };

    #[test]
    fn test_st_tables() {
        crate::on_ruby_thread(|| unsafe {
            let table = rb_st_init_numtable_with_size(4);
            for key in 1..=4 {
                rb_st_add_direct(table, key, key * 10);
            }

            let mut value = 0;
            assert_eq!(rb_st_lookup(table, 3, &mut value), 1);
            assert_eq!(value, 30);
            assert_eq!(rb_st_lookup(table, 9, ptr::null_mut()), 0);
            let mut key = 0;
            assert_eq!(rb_st_get_key(table, 2, &mut key), 1);
            assert_eq!(key, 2);

            let mut sum: StData = 0;
            assert_eq!(
                rb_st_foreach(table, sum_entries, &mut sum as *mut StData as StData),
                0
            );
            assert_eq!(sum, 1000 + 100);

            let mut keys = [0; 8];
            assert_eq!(rb_st_keys(table, keys.as_mut_ptr(), 8), 4);
            assert_eq!(&keys[..4], &[1, 2, 3, 4]);
            let mut values = [0; 2];
            assert_eq!(rb_st_values(table, values.as_mut_ptr(), 2), 2);
            assert_eq!(values, [10, 20]);
            assert_eq!(rb_st_keys_check(table, keys.as_mut_ptr(), 8, 0), 4);
            assert_eq!(rb_st_values_check(table, values.as_mut_ptr(), 2, 0), 2);

            assert_eq!(rb_st_update(table, 4, double_value, 0), 1);
            assert_eq!(rb_st_update(table, 5, double_value, 55), 0);
            rb_st_lookup(table, 4, &mut value);
            assert_eq!(value, 80);
            rb_st_lookup(table, 5, &mut value);
            assert_eq!(value, 55);

            let copy = rb_st_copy(table);
            assert!(rb_st_memsize(copy) > 0);

            let mut key = 5;
            assert_eq!(rb_st_delete(table, &mut key, &mut value), 1);
            assert_eq!(value, 55);
            assert_eq!(rb_st_delete(table, &mut key, ptr::null_mut()), 0);
            let mut key = 4;
            assert_eq!(rb_st_delete_safe(table, &mut key, &mut value, 0), 1);
            rb_st_cleanup_safe(table, 0);

            let (mut first_key, mut first_value) = (0, 0);
            assert_eq!(rb_st_shift(table, &mut first_key, &mut first_value), 1);
            assert_eq!((first_key, first_value), (1, 10));

            assert_eq!(rb_st_foreach_check(copy, delete_odd_keys, 0, 0), 0);
            assert_eq!(rb_st_keys(copy, keys.as_mut_ptr(), 8), 2);
            assert_eq!(&keys[..2], &[2, 4]);
            assert_eq!(
                rb_st_foreach_with_replace(copy, replace_all, double_value, 0),
                0
            );
            rb_st_lookup(copy, 2, &mut value);
            assert_eq!(value, 40);

            // The key stored for a new entry is `add_one(7)` (meant for a
            // copy of the key, such as `strdup`, that compares equal).
            assert_eq!(rb_st_insert2(copy, 7, 70, add_one), 0);
            assert_eq!(rb_st_keys(copy, keys.as_mut_ptr(), 8), 3);
            assert_eq!(keys[2], 8);

            rb_st_clear(table);
            assert_eq!(rb_st_keys(table, keys.as_mut_ptr(), 8), 0);

            // Keys equal modulo 10.
            let custom = rb_st_init_table_with_size(&MOD_10, 2);
            rb_st_add_direct(custom, 13, 1);
            assert_eq!(rb_st_lookup(custom, 23, &mut value), 1);
            let custom = rb_st_init_table(&MOD_10);
            assert_eq!(rb_st_lookup(custom, 23, ptr::null_mut()), 0);

            // C string keys.
            let name = b"Name\0";
            let strings = rb_st_init_strtable();
            rb_st_add_direct(strings, name.as_ptr() as StData, 1);
            assert_eq!(
                rb_st_lookup(strings, b"Name\0".as_ptr() as StData, ptr::null_mut()),
                1
            );
            assert_eq!(
                rb_st_lookup(strings, b"name\0".as_ptr() as StData, ptr::null_mut()),
                0
            );
            let strings = rb_st_init_strtable_with_size(1);
            assert_eq!(
                rb_st_lookup(strings, name.as_ptr() as StData, ptr::null_mut()),
                0
            );
            let cases = rb_st_init_strcasetable();
            rb_st_add_direct(cases, name.as_ptr() as StData, 1);
            assert_eq!(
                rb_st_lookup(cases, b"NAME\0".as_ptr() as StData, ptr::null_mut()),
                1
            );
            let cases = rb_st_init_strcasetable_with_size(1);
            assert_eq!(
                rb_st_lookup(cases, name.as_ptr() as StData, ptr::null_mut()),
                0
            );

            let a = CStr::from_bytes_with_nul(b"Ruby\0").unwrap();
            let b = CStr::from_bytes_with_nul(b"rUBY!\0").unwrap();
            assert_ne!(
                rb_st_locale_insensitive_strcasecmp(a.as_ptr(), b.as_ptr()),
                0
            );
            assert_eq!(
                rb_st_locale_insensitive_strncasecmp(a.as_ptr(), b.as_ptr(), 4),
                0
            );

            assert_eq!(rb_st_numcmp(3, 3), 0);
            assert_ne!(rb_st_numcmp(3, 4), 0);
            assert_eq!(rb_st_numhash(5), rb_st_numhash(5));

            let bytes = b"rutie";
            let h = rb_st_hash(bytes.as_ptr() as *const c_void, bytes.len(), 0);
            assert_eq!(
                h,
                rb_st_hash(bytes.as_ptr() as *const c_void, bytes.len(), 0)
            );
            let h = rb_st_hash_start(h);
            let h = rb_st_hash_uint(h, 1);
            let h = rb_st_hash_uint32(h, 2);
            let _ = rb_st_hash_end(h);

            // A Hash's own table: one with more than 8 entries is an
            // `st_table` (smaller ones are arrays).
            let hash = crate::binding::vm::eval_string("(1..9).to_h { [_1, _1] }");
            let pairs = [
                crate::binding::fixnum::i32_to_num(100),
                crate::binding::fixnum::i32_to_num(200),
            ];
            rb_hash_bulk_insert_into_st_table(2, pairs.as_ptr(), hash);
            assert_eq!(
                crate::binding::hash::aref(hash, pairs[0]).value,
                pairs[1].value
            );

            // `rb_st_free_table` is not declared in this module; these few
            // tables are left to the OS.
        });
    }
}
