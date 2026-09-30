# Plan: complete Ruby 2 support in Rutie

**Status:** proposed, 2026-09-30. Written for the agent/maintainer who picks this up.
**Scope:** make everything Ruby 2 exposes through its C API that a Rust
integration reasonably needs available through Rutie's FFI layer, on Ruby
**2.5, 2.6 and 2.7**, dynamically linked, on Linux and macOS. Ruby 3 is
explicitly out of scope until this plan is complete; the design must not
make Ruby 3 harder later, but nothing here is allowed to block on it.

Rutie 0.10.0 (the revert of the `rb-sys` integration, PR #172) is the baseline.

---

## 0. Ground rules (learned the hard way; keep them)

1. **Hand-maintained FFI, no `rb-sys`.** `src/rubysys/` is the one place
   `extern "C"` declarations live. Every declaration carries the C prototype
   as a comment above it, exactly as the existing files do. `rubysys` is a
   private API (see CHANGELOG header) and may change in teeny releases.
2. **Layering is fixed:** `rubysys` (raw extern) → `binding` (thin unsafe
   wrappers taking/returning `Value`) → `class` (safe, typed public API) →
   `dsl` (macros). New surface must go through all three layers; nothing
   public calls `rubysys` directly.
3. **Every public item ships with a doctest** that runs (not `ignore`,
   `no_run` only when the example must not execute), and the doctest starts
   with the hidden `# VM::init();` line the existing ones use. Unit tests run
   their body through `crate::on_ruby_thread(|| { ... })` (see `src/lib.rs`):
   Ruby 2 is bound to the native thread that called `ruby_init` (the GC scans
   that thread's stack), while the test harness gives every test its own
   thread, so all unit tests share one long-lived Ruby thread, serialized by
   `LOCK_FOR_TEST`. Never call `VM::init` from a unit test directly.
4. **Verify on all three Rubies before pushing.** Locally: one Ruby per
   prefix (e.g. `/opt/rb/2.5.9`, `/opt/rb/2.6.10`, `/opt/rb/2.7.8`), put its
   `bin` first on `PATH`, `cargo clean`, `cargo test`. `build.rs` links
   against whatever `ruby` is on `PATH` (or `$RUBY`); a stale `target/` from
   another Ruby segfaults at test time, so always `cargo clean` when switching.
   Use release tarballs (or RVM) to build Rubies: git-tag checkouts of 2.5/2.6
   need patching of bison-generated `parse.c` under modern bison.
   Where `cache.ruby-lang.org` is unreachable (sandboxed agents): RVM's
   prebuilt, relocatable binaries at `https://rvm.io/binaries/` cover 2.5.9
   (`ubuntu/20.04/x86_64`) and 2.6.10 (`debian/11/x86_64`); 2.7.8 builds from
   the `v2_7_8` git tag after copying `config.guess`/`config.sub` into `tool/`,
   with `--enable-shared --with-baseruby=<any ruby> --without-openssl`
   (`make install` then stops at the default gems, after libruby, headers and
   stdlib are installed; that is enough). Linking needs `libgmp-dev`.
   Instead of `cargo clean`, a separate `CARGO_TARGET_DIR` per Ruby works too,
   as long as its path ends in `/target` (`build.rs` splits `OUT_DIR` on it),
   e.g. `CARGO_TARGET_DIR=$HOME/rt-2.7.8/target`.
5. **CI is the arbiter** (`.github/workflows/ci.yml`): 3 OS × {stable, beta}
   × {2.5.9, 2.6.10, 2.7.8} × {dynamic, static}. Dynamic Linux and macOS must
   be green. Static and Windows rows are best-effort (`continue-on-error`).
   macOS builds Ruby against a source-built OpenSSL 1.1.1 with
   `PKG_CONFIG_PATH` pointed at it; do not remove that or Ruby 2.5/2.6 stop
   building (their `ext/openssl` lets pkg-config's `openssl@3` outrank
   `--with-openssl-dir`).
6. **Version-specific ABI is real.** Struct layouts and flag bits differ
   between 2.5, 2.6 and 2.7 (see §3). Anything that reads a Ruby struct field
   directly must be gated per version or go through a C function instead.
   Prefer the function.
7. **CHANGELOG.md is updated in the same commit as the change**, Keep a
   Changelog format, with `thanks to @user` credit. Bump `Cargo.toml` per
   SemVer for the public API.
8. **Add safe methods beside existing ones; don't change existing ones.**
   Following the README's "Safety" section, the fast, raising methods (`send`,
   `unsafe_methods!`, …) keep their behaviour and cost. When a safe variant is
   needed, write a new method (usually `protect`-based and returning
   `Result<_, AnyException>`, like `protect_send` and `Enumerator`) instead
   of adding checks to the existing one. Fixing genuine memory-safety bugs in
   an existing method is the exception.

---

## 1. Where we are (inventory as of 0.10.0)

### Bound C functions (`src/rubysys/`)

| module | functions |
|---|---|
| array | `rb_ary_new`, `_new_capa`, `_new_from_values`, `_dup`, `_entry`, `_store`, `_push`, `_pop`, `_shift`, `_unshift`, `_concat`, `_join`, `_reverse`, `_sort`, `_sort_bang`, `_to_s` |
| class | `rb_define_class(_under)`, `_module(_under)`, `_method`, `_private_method`, `_singleton_method`, `_module_function`, `_attr`, `_const`, `rb_const_get`, `rb_class_new_instance`, `rb_class_superclass`, `rb_obj_class`, `rb_singleton_class`, `rb_mod_ancestors`, `rb_include_module`, `rb_prepend_module`, `rb_extend_object`, `rb_ivar_get/set`, `rb_respond_to`, `rb_equal`, `rb_eql`, `rb_obj_freeze`, `rb_obj_frozen_p`, `rb_scan_args` |
| encoding | 24 functions: enc index/lookup/associate, default external/internal, `rb_str_encode`, `rb_str_export_to_enc`, `rb_econv_prepare_opts`, `rb_enc_codepoint_len` |
| fixnum / float | `rb_int2inum`…`rb_num2ull` family (12), `rb_float_new`, `rb_num2dbl`, `rb_to_float` |
| gc | 13: enable/disable/start/count/stat, mark, mark_maybe, register/unregister address, register_mark_object, force_recycle, adjust_memory_usage |
| hash | `rb_hash_new`, `_aref`, `_aset`, `_delete`, `_clear`, `_dup`, `_size`, `_foreach` |
| rproc | `rb_proc_call_with_block`, `rb_binding_new`, `rb_obj_is_proc`, `rb_obj_is_method`, `check_arity` |
| string | 17: new/new_cstr/utf8 variants, value_cstr/ptr, strlen, cat, force_encoding, valid_encoding_p, asciionly_p, export_locale, check_string_type, locktmp/unlocktmp, new_frozen |
| symbol | `rb_intern`, `rb_intern2`, `rb_id2sym`, `rb_sym2id`, `rb_id2name` |
| thread | `rb_thread_call_with(out)_gvl(2)`, `rb_thread_create`, `rb_thread_wait_fd`, `rb_thread_interrupted` |
| typed_data | `rb_data_typed_object_wrap`, `rb_check_typeddata`, `rb_typeddata_inherited_p`, `rb_typeddata_is_kind_of` |
| vm | `ruby_init`, `ruby_init_loadpath`, `ruby_vm_at_exit`, `rb_eval_string(_protect)`, `rb_f_eval`, `rb_require`, `rb_protect`, `rb_funcallv(_public)`, `rb_block_call`, `rb_block_proc`, `rb_block_given_p`, `rb_yield`, `rb_yield_splat`, `rb_call_super`, `rb_raise`, `rb_exc_raise`, `rb_errinfo`, `rb_set_errinfo`, `rb_exit`, `rb_f_abort` |

### Public Rust surface (`src/class/`, `src/dsl.rs`)

`AnyObject`, `AnyException`, `Array`, `Binding`, `Boolean`, `Class`,
`Encoding`, `Enumerator`, `Fixnum`, `Float`, `GC`, `Hash`, `Integer`,
`Module`, `NilClass`, `Proc`, `RString`, `Symbol`, `Thread`, `VM`; traits
`Object`, `Exception`, `EncodingSupport`, `TryConvert`, `VerifiedObject`;
macros `class!`, `module!`, `methods!`, `unsafe_methods!`,
`wrappable_struct!`, `eval!`.

### Known debts carried into 0.10.0

- ~~`VM::at_exit` runs the closure **immediately**~~ — fixed by P0-2.
- README "Ruby 2 Notes" said Ruby 2 was supported "up through 0.8" — fixed in
  0.10.0 (now a support table plus the OpenSSL 1.1 recipe).
- Three doctests in `src/dsl.rs` (`wrappable_struct!`) are `ignore`d doc
  fragments, not tests. Make them runnable.
- ~~No `cfg` flags for the Ruby minor version exist~~ — added by P0-1.
- `examples/` are not built in CI.

---

## 2. Target: what "Ruby fully available on Rust" means here

The acceptance test for this plan is: **a Rust program embedding Ruby 2, or a
Ruby 2 extension written in Rust, can do everything the equivalent C
extension could do without dropping to raw `rubysys`**, for the API areas in
§4. Each area is "done" when:

1. the C functions in its table are declared in `rubysys` (prototype comment,
   correct types for 2.5–2.7),
2. a `binding` wrapper and a safe `class`-level API exist,
3. every public item has a running doctest and at least one unit test that
   exercises the Ruby side (round-trips a value through Ruby, not just Rust),
4. `cargo test` is green on 2.5.9, 2.6.10 and 2.7.8, stable and beta, on
   Linux and macOS in CI,
5. CHANGELOG has the entry.

---

## 3. Version-specific ABI notes (2.5 → 2.6 → 2.7)

Check these before touching anything that reads Ruby structs directly
(`src/rubysys/value.rs`, `string.rs`, `array.rs` helpers, `typed_data.rs`):

- `RBasic` flag bits (`RUBY_FL_USHIFT`, embed flags for `RString`/`RArray`,
  `RSTRING_EMBED_LEN_MAX`, `RARRAY_EMBED_LEN_MASK`) — the reason
  `wrappable_struct!` has `reserved_bytes` and why array/string lengths were
  once hand-computed. Prefer `rb_str_strlen`/`RSTRING_LEN` via function,
  `rb_ary_len` via `rb_ary_entry` loops or `RARRAY_LEN` via function.
- `rb_data_type_struct.function`: 2.5/2.6 have `dmark, dfree, dsize,
  reserved[2]`; 2.7 has `dmark, dfree, dsize, dcompact, reserved[1]`. The
  current `[null; 2]` layout is compatible with all three (2.7's `dcompact`
  lands on a null). Keep it that way; do **not** add a `dcompact` field
  without a 2.7-only cfg.
- `T_*` value types (`ValueType` enum): `T_IMEMO`/`T_ICLASS` positions are
  stable across 2.5–2.7 but confirm against `include/ruby/ruby.h` for each.
- `rb_scan_args` keyword handling: 2.7 introduced `rb_keyword_given_p` and
  the `:` spec; 2.5/2.6 treat a trailing hash as kwargs. Gate kwargs work.
- Taint/trust (`rb_obj_taint`, `rb_obj_untrust`) are no-ops in 2.7 and
  removed in 3.0 — **do not bind them**.
- `rb_gc_force_recycle` is deprecated from 2.7; keep it but mark deprecated.
- Plumbing (P0-1, done): `build.rs` emits `ruby_2_5` / `ruby_2_6` / `ruby_2_7`
  for the exact version and `ruby_gte_2_5` / `ruby_gte_2_6` / `ruby_gte_2_7`
  cumulatively, so bindings are gated with `#[cfg(ruby_gte_2_7)]` instead of
  runtime version sniffing. The cfgs also reach doctests.
- Found while doing P0 (verified with `nm -D` on each libruby):
  `rb_exec_end_proc` is exported by 2.5 and 2.6 but **not 2.7** — use
  `ruby_cleanup` to run end procs. `rb_frozen_error_raise`,
  `rb_keyword_given_p`, `rb_funcallv_kw` and `rb_scan_args_kw` are 2.7-only.
  `rb_get_kwargs` error messages differ (`missing keyword: z` on 2.5/2.6,
  `missing keyword: :z` on 2.7); don't assert on the exact text.
- `rb_scan_args` calls `rb_fatal` (process abort) on a malformed format and
  `rb_sys_fail` calls `rb_bug` when `errno` is 0; the safe wrappers validate
  the format and use `rb_syserr_fail` instead.

---

## 4. Work packages (in priority order)

Each package lists the C API to bind and the Rust surface to add. Tick items
as they land; keep this file current.

### P0 — plumbing and correctness (do first; everything else builds on it)

- [x] **P0-1 Version cfg flags** from `build.rs` (§3). The check that the
      flags match the linked Ruby is the unit test
      `current_ruby::cfg_flags_match_linked_ruby`, which runs in every CI row.
      Also exported downstream as `DEP_RUBY_VERSION_MAJOR`/`_MINOR`, and a
      `cargo:warning` is printed when the Ruby found is not 2.5–2.7.
- [x] **P0-2 Real `VM::at_exit`.** Bind `rb_set_end_proc(void (*)(VALUE),
      VALUE)`; box the closure (`Box<Box<dyn FnMut(VmPointer) + 'static>>`),
      leak it intentionally (it must outlive `main`), and call it from an
      `extern "C"` trampoline. Requires `F: 'static`; document the breaking
      change and keep the old immediate-call behaviour available under a
      clearly named method if anyone relied on it.
      Done: `F: FnOnce(VmPointer) + 'static`; `rb_set_end_proc` passes its
      data to `rb_gc_mark`, so the box pointer is tagged as a Fixnum. Old
      behaviour is `VM::call_protected`. `VM::cleanup` (`ruby_cleanup`, from
      P5) was added so embedders can run end procs and so it can be tested.
- [x] **P0-3 Exception control flow.** `rb_ensure`, `rb_rescue`, `rb_rescue2`,
      `rb_catch`, `rb_throw`, `rb_iter_break`, `rb_iter_break_value`,
      `rb_jump_tag`. Rust surface: `VM::ensure(body, ensure)`,
      `VM::rescue(body, handler)`, `VM::catch_throw`. Panics must never cross
      into Ruby: wrap every trampoline in `catch_unwind` and convert to a Ruby
      exception (`rb_raise` with a `RuntimeError`).
      Done: `VM::ensure`, `VM::rescue`, `VM::rescue_from(&[Class], ..)`,
      `VM::catch`/`VM::throw`, `VM::iter_break(_value)`, `VM::jump_tag`, plus
      `Object::send_with_block`/`protect_send_with_block` (`rb_block_call`)
      so Rust closures can be blocks. `rb_catch`/`rb_throw`/`rb_rescue` are
      bound in `rubysys` only (the `_obj`/`rescue2` forms cover them).
- [x] **P0-4 Argument checking helpers.** `rb_check_type`, `rb_check_frozen`,
      `rb_error_arity`, `rb_check_arity`, `rb_num_zerodiv`, `rb_notimplement`,
      `rb_sys_fail`, `rb_warn`, `rb_warning`. Surface: `VM::warn`,
      `Object::check_frozen`, arity errors from `methods!`.
      Done: `VM::warn`/`warning`, `VM::check_arity` (returns the error),
      `VM::raise_arity_error`, `VM::raise_zero_division`,
      `VM::not_implemented`, `VM::sys_fail`, `Object::check_frozen`,
      `Object::check_type`. Existing macros keep their behaviour: `methods!`
      passes `Err` for a missing argument (its documented contract) and
      `unsafe_methods!` stays check-free for speed (README "Safety"); callers
      that want arity errors use `VM::check_arity`/`VM::raise_arity_error`.
      `VM::raise` no longer passes the message to `rb_raise` as a printf
      format (it did: `%s` in a message read garbage); its signature is
      unchanged.
      **Rule followed here and for the rest of the plan:** never change the
      behaviour or cost of an existing public method to make it safe; add a
      new (usually `protect`-based, `Result`-returning) method beside it, as
      `Enumerator` and `protect_send` do.
- [x] **P0-5 `methods!`/`unsafe_methods!` completeness.** Variable arity
      (`argc = -1` with `rb_scan_args` specs including optional, splat, block
      and — gated — keyword args via `rb_get_kwargs`), `rb_define_method_id`,
      `rb_define_alias`, `rb_undef_method`, `rb_define_alloc_func` /
      `rb_undef_alloc_func`, `rb_define_attr` exposure, `rb_obj_call_init`.
      Done: `VM::scan_args(&args, "21*1:&") -> ScannedArgs` (real
      `rb_scan_args`, so keyword rules are the running Ruby's),
      `VM::get_kwargs -> KeywordArgs`, `VM::is_keyword_given` (2.7 only),
      `Class`/`Module::define_alias`/`undef_method`,
      `Class::define_alloc_func`/`undef_alloc_func`, `Object::call_init`.
      Variable-arity methods are plain `extern "C" fn(Argc, *const AnyObject,
      T)` functions using `VM::scan_args` (documented); the `methods!` macro
      grammar is unchanged. `rb_define_method_id` is bound in `rubysys` only;
      `rb_define_attr` was already exposed as `attr_reader`/`writer`/`accessor`.
- [x] **P0-6 Frozen semantics.** `rb_obj_freeze` exists; add `rb_str_freeze`,
      `rb_ary_freeze`, `rb_hash_freeze`, `rb_frozen_error_raise` (2.5+), and
      make mutating wrappers (`Array::push`, `RString::concat`, …) return
      `Result` or document that Ruby raises.
      Done: freeze functions bound in `rubysys`/`binding` (`Object::freeze`
      stays the public route); `rb_frozen_error_raise` is 2.7-only, so
      `rb_error_frozen_object` backs the raising path. The mutating `Array`,
      `Hash` and `RString` wrappers document that they raise `FrozenError`.

### P1 — core object model

- [ ] **Object:** `rb_obj_dup`, `rb_obj_clone`, `rb_obj_id`, `rb_inspect`,
      `rb_obj_as_string`, `rb_obj_is_kind_of`, `rb_obj_is_instance_of`,
      `rb_obj_method`, `rb_method_boundp`, `rb_check_funcall`,
      `rb_funcall_with_block`, `rb_funcallv_kw` (2.7, gated), `rb_obj_instance_variables`,
      `rb_ivar_defined`, `rb_obj_remove_instance_variable`, `rb_hash` (object hash).
- [ ] **Class/Module:** `rb_class_name`, `rb_class_path`, `rb_mod_name`,
      `rb_class_inherited_p`, `rb_mod_include_p`, `rb_mod_module_eval`,
      `rb_obj_instance_eval`, `rb_class_instance_methods`, `rb_cvar_get/set/defined`,
      `rb_const_defined(_at)`, `rb_const_set`, `rb_const_remove`, `rb_path2class`,
      `rb_define_global_const`, `rb_class_of` (immediates too), `rb_define_alias`.
- [ ] **Global variables:** `rb_gv_get`, `rb_gv_set`, `rb_define_variable`,
      `rb_define_readonly_variable`, `rb_define_virtual_variable`,
      `rb_define_hooked_variable`. Surface: `VM::global("$x")` get/set.
- [ ] **Symbols/IDs:** `rb_sym2str`, `rb_check_id`, `rb_to_id`, `rb_to_symbol`,
      `rb_is_const_id`, `rb_is_instance_id`, `rb_is_class_id`, `rb_intern_str`.
- [ ] **Kernel formatting:** `rb_sprintf`/`rb_str_format`, `rb_p`, `rb_String`,
      `rb_Array`, `rb_Integer`, `rb_Float`, `rb_Hash`.

### P2 — core types to parity with the C API

- [ ] **String:** `rb_str_dup`, `rb_str_substr`, `rb_str_split`, `rb_str_cmp`,
      `rb_str_equal`, `rb_str_hash`, `rb_str_inspect`, `rb_str_replace`,
      `rb_str_resize`, `rb_str_buf_new`, `rb_str_buf_append`, `rb_str_plus`,
      `rb_str_times`, `rb_str_to_inum`, `rb_str_to_dbl`, `rb_str_intern`,
      `rb_str_length`, `rb_str_capacity`, `rb_str_set_len`, `rb_str_modify`,
      `rb_str_conv_enc`, `rb_enc_str_coderange`, `rb_enc_mbclen`, `rb_enc_nth`,
      `rb_str_scrub`. Byte-slice (`&[u8]`) views must respect `rb_str_locktmp`.
- [ ] **Array:** `rb_ary_delete`, `rb_ary_delete_at`, `rb_ary_includes`,
      `rb_ary_clear`, `rb_ary_subseq`, `rb_ary_plus`, `rb_ary_cmp`, `rb_ary_replace`,
      `rb_ary_resize`, `rb_ary_rotate`, `rb_ary_assoc`, `rb_ary_rassoc`,
      `rb_ary_to_ary`, `rb_check_array_type`, `rb_ary_each` (via `rb_block_call`),
      `rb_ary_freeze`, `rb_ary_aref`. `Array` should implement `IntoIterator`
      (by `Value` copy) and `FromIterator<AnyObject>`.
- [ ] **Hash:** `rb_hash_lookup`, `rb_hash_lookup2`, `rb_hash_fetch`,
      `rb_hash_has_key`? (use `rb_hash_lookup2` with undef), `rb_hash_keys`,
      `rb_hash_values`, `rb_hash_update_by`, `rb_hash_set_ifnone`,
      `rb_hash_freeze`, `rb_check_hash_type`, `rb_hash_delete_if`?, `rb_hash_tbl`
      (avoid), `rb_env_clear`? (no). `Hash` gets `iter()` over `(AnyObject, AnyObject)`.
- [ ] **Numeric:** Bignum — `rb_big2str`, `rb_cstr_to_inum`, `rb_str2inum`,
      `rb_big_cmp`, `rb_big_plus/minus/mul/div/modulo/pow`, `rb_big2ll/ull/dbl`,
      `rb_dbl2big`, `rb_int_positive_pow`; `rb_num_coerce_bin/cmp/relop`,
      `rb_num2fix`, `rb_fix2str`, `rb_Integer`, `rb_Float`; **Rational/Complex** —
      `rb_rational_new`, `rb_rational_raw`, `rb_Rational`, `rb_rational_num/den`,
      `rb_complex_new`, `rb_complex_raw`, `rb_Complex`, `rb_complex_real/imag`
      (2.7 exposes `rb_complex_real`/`_imag`; gate older versions to
      `rb_funcall`). New types: `Bignum`? (fold into `Integer`), `Rational`,
      `Complex`. Implement `TryFrom<i128>/u128`, `From<f64>`.
- [ ] **Range:** `rb_range_new`, `rb_range_values`, `rb_range_beg_len`,
      `rb_arithmetic_sequence_extract` (2.6+). New type `Range`.
- [ ] **Regexp / MatchData:** `rb_reg_new_str`, `rb_reg_new`, `rb_reg_regcomp`,
      `rb_reg_match`, `rb_reg_match2`, `rb_reg_nth_match`, `rb_reg_last_match`,
      `rb_reg_backref_number`, `rb_backref_get/set`, `rb_reg_options`,
      `rb_reg_source`. New types `Regexp`, `MatchData`.
- [ ] **Time:** `rb_time_new`, `rb_time_nano_new`, `rb_time_timespec_new`,
      `rb_time_num_new`, `rb_time_interval`, `rb_time_timeval`, `rb_time_timespec`,
      `rb_time_utc_offset`. New type `Time` with `From<SystemTime>`/`Duration`.
- [ ] **Struct:** `rb_struct_define`, `rb_struct_define_under`, `rb_struct_new`,
      `rb_struct_alloc`, `rb_struct_aref`, `rb_struct_aset`, `rb_struct_getmember`,
      `rb_struct_members`, `rb_struct_size`. New type `Struct`.
- [ ] **Enumerator / Enumerable:** `rb_enumeratorize`, `rb_enumeratorize_with_size`
      (`RETURN_ENUMERATOR` equivalent for `methods!`), `rb_enum_values_pack`,
      `rb_cmpint`, `rb_cmperr`, `rb_obj_is_kind_of(Enumerable)`. Make
      `Enumerator` iterable from Rust (`next` via `rb_funcall` with
      `StopIteration` mapped to `None`).
- [ ] **Proc / Method / Binding:** `rb_proc_new`, `rb_proc_arity`,
      `rb_proc_lambda_p`, `rb_proc_call`, `rb_block_lambda`, `rb_method_call`,
      `rb_obj_method`, `rb_mod_method_arity`, `rb_obj_method_arity`,
      `rb_yield_values`, `rb_yield_values2`, `rb_need_block`, `rb_binding_new`
      (exists) + `Binding::local_variable_get/set` via `rb_funcall`.
- [ ] **Encoding:** `rb_enc_get`, `rb_enc_name`, `rb_enc_find`, `rb_ascii8bit_encoding`,
      `rb_utf8_encoding`, `rb_usascii_encoding`, `rb_locale_encoding`,
      `rb_filesystem_encoding`, `rb_enc_str_buf_cat`, `rb_enc_uint_chr`,
      `rb_enc_precise_mbclen`, `rb_enc_ascget`. Round out `Encoding`.

### P3 — exceptions, IO, and the standard objects an embedder hits

- [ ] **Exceptions:** `rb_exc_new_str`, `rb_exc_new_cstr`, `rb_exc_new`,
      `rb_class_new_instance` for exception classes, `rb_ensure`/`rb_rescue`
      (P0-3), `rb_exc_fatal`, `rb_interrupt`, `rb_bug` (bind but never call in
      library code), `rb_syserr_new`, `rb_mod_syserr_fail`. Expose the builtin
      exception class hierarchy as constants (`rb_eStandardError`,
      `rb_eArgError`, `rb_eTypeError`, `rb_eRuntimeError`, `rb_eNoMethodError`,
      `rb_eIOError`, `rb_eStopIteration`, `rb_eFrozenError` (2.5+),
      `rb_eZeroDivError`, `rb_eKeyError`, `rb_eRangeError`, `rb_eNotImpError`,
      `rb_eSystemExit`, `rb_eInterrupt`, `rb_eSignal`, `rb_eEncodingError`,
      `rb_eEncCompatError`, `rb_eLoadError`, `rb_eSecurityError`) as
      `extern static` VALUEs with a typed `Class` accessor each.
- [ ] **Builtin class/module globals:** `rb_cObject`, `rb_cBasicObject`,
      `rb_mKernel`, `rb_mComparable`, `rb_mEnumerable`, `rb_cString`, `rb_cArray`,
      `rb_cHash`, `rb_cInteger`, `rb_cFloat`, `rb_cRational`, `rb_cComplex`,
      `rb_cRange`, `rb_cRegexp`, `rb_cTime`, `rb_cSymbol`, `rb_cProc`, `rb_cMethod`,
      `rb_cThread`, `rb_cIO`, `rb_cFile`, `rb_cNilClass`, `rb_cTrueClass`,
      `rb_cFalseClass`, `rb_cEncoding`, `rb_cStruct`, `rb_cEnumerator`,
      `rb_cModule`, `rb_cClass`. Today many wrappers do `Class::from_existing("X")`
      string lookups; replace with the statics.
- [ ] **IO / File / Dir:** `rb_io_write`, `rb_io_puts`, `rb_io_print`, `rb_io_gets`,
      `rb_io_getbyte`, `rb_io_close`, `rb_io_flush`, `rb_io_eof`, `rb_io_binmode`,
      `rb_io_check_readable/writable/closed`, `rb_io_stdio_file`, `rb_stdin`,
      `rb_stdout`, `rb_stderr`, `rb_file_open`, `rb_file_open_str`,
      `rb_file_expand_path`, `rb_file_absolute_path`, `rb_file_dirname`,
      `rb_dir_getwd`, `rb_io_taint_check`? (no — taint). New types `IO`, `File`.
- [ ] **Marshal / ObjectSpace / GC extras:** `rb_marshal_dump`, `rb_marshal_load`,
      `rb_define_finalizer`, `rb_undefine_finalizer`, `rb_objspace_each_objects`
      (careful), `rb_memory_id`, `rb_gc_writebarrier`, `rb_gc_writebarrier_unprotect`,
      `rb_gc_latest_gc_info`, `rb_gc_register_mark_object` (exists). `GC::WeakMap`
      via `rb_funcall`.
- [ ] **Load / require / $LOAD_PATH:** `rb_load`, `rb_load_protect`, `rb_f_require`,
      `rb_provide`, `rb_provided`, `rb_feature_provided`, `ruby_incpush`,
      `rb_require_string` (2.7). Surface: `VM::load(path, wrap)`, `VM::provide`.

### P4 — concurrency

- [ ] **Threads:** `rb_thread_current`, `rb_thread_main`, `rb_thread_alone`,
      `rb_thread_schedule`, `rb_thread_sleep`, `rb_thread_sleep_forever`,
      `rb_thread_wait_for`, `rb_thread_wakeup`, `rb_thread_run`, `rb_thread_kill`,
      `rb_thread_local_aref/aset`, `rb_thread_check_ints`, `rb_thread_atfork`,
      `rb_thread_fd_writable`, `rb_thread_fd_select`? (avoid), `rb_thread_wait_fd`
      (exists). `Thread` gets `join`, `value`, `alive`, `kill`, `current`.
- [ ] **Mutex / Queue:** `rb_mutex_new`, `rb_mutex_lock`, `rb_mutex_unlock`,
      `rb_mutex_trylock`, `rb_mutex_locked_p`, `rb_mutex_synchronize`,
      `rb_mutex_sleep`. New type `Mutex` with an RAII guard that unlocks on drop
      (and on Ruby exceptions via `rb_ensure`).
- [ ] **Fiber:** `rb_fiber_new`, `rb_fiber_resume`, `rb_fiber_yield`,
      `rb_fiber_current`, `rb_fiber_alive_p`. New type `Fiber`.
- [ ] **GVL helpers:** `Thread::call_without_gvl` exists; add an unblocking
      function argument (`rb_thread_call_without_gvl` UBF, `RUBY_UBF_IO` /
      `RUBY_UBF_PROCESS` constants) and document Send/Sync expectations.

### P5 — VM lifecycle and embedding

- [ ] `ruby_setup`, `ruby_cleanup`, `ruby_finalize`, `ruby_options`,
      `ruby_run_node`, `ruby_exec_node`, `ruby_script`, `ruby_set_argv`,
      `ruby_prog_init`, `ruby_init_stack` (only where the platform needs it),
      `ruby_sysinit`, `ruby_native_thread_p`, `ruby_stack_check`,
      `ruby_stack_length`. Surface: `VM::init_with_args(&[&str])`,
      `VM::cleanup() -> i32`, `VM::run_file`, and make `VM::init` idempotent
      (it is not today: calling twice is UB).
- [ ] `rb_set_end_proc` proper `at_exit` (P0-2) and `ruby_vm_at_exit` semantics
      documented side by side. (`VM::cleanup` exists since P0-2.)
- [ ] Signals: `rb_f_trap`-equivalents are not public C API; document that
      `VM::trap` (exists via `Signal.trap`) is the supported route.

### P6 — DSL, docs, examples, release

- [ ] `wrappable_struct!`: make the three `ignore`d doctests runnable; add
      `dsize` support (`rb_data_typed_object_zalloc` + size fn) and a
      `#[derive]`-free way to declare the wrapped type `Send`-safe; document
      the 2.7 `dcompact` slot (§3).
- [ ] `methods!`: keyword args (gated), splat, optional args (P0-5); emit
      `rb_error_arity` on mismatch instead of panicking.
- [ ] Build all `examples/` in CI (they exercise `rutie_ruby_example`,
      `rutie_ruby_gvl_example`, `rutie_rust_example` end to end), on the same
      matrix as the crate.
- [x] README: "Ruby 2 Notes" rewritten for 0.10 (support table, OpenSSL 1.1
      recipe). Still to do: add the local multi-Ruby testing recipe from §0.4.
- [ ] `build.rs`: honour `$RUBY` consistently (it does for `rbconfig`; check
      `is_linked_ruby` test uses the same), emit the version cfgs (P0-1, done),
      and print a clear error when the linked Ruby's major version ≠ 2 (a
      `cargo:warning` is printed since P0-1; decide whether it should fail).
- [ ] Release cadence: 0.10.0 = this baseline; 0.11 = P0 + P1; 0.12 = P2;
      0.13 = P3 + P4; 0.14 = P5 + P6; then declare "Ruby 2 complete" (1.0 is
      a maintainer call) and only then branch for Ruby 3.

---

## 5. How to work a package (checklist for each item)

1. Read the C prototype in the oldest supported Ruby's headers
   (`include/ruby/intern.h`, `ruby.h`) **and** in 2.7's; note any signature
   or semantic drift and gate it with the version cfgs.
2. Add the `extern "C"` declaration to the right `src/rubysys/*.rs` with the
   prototype comment. Types come from `src/rubysys/types.rs`; never `u64` for
   `VALUE`.
3. Add the `binding` function (unsafe inside, `Value` in/out, no
   allocation policy decisions).
4. Add the `class`-level safe API. Anything that can raise in Ruby is either
   wrapped with `protect` (returns `Result<_, AnyException>`) or documented as
   raising.
5. Doctest + unit test. Tests that mutate global VM state (globals, constants,
   `$LOAD_PATH`) must clean up.
6. `cargo test` on 2.5.9, 2.6.10, 2.7.8 (stable; beta at least once per
   package). `cargo clippy` clean.
7. CHANGELOG entry under `[Unreleased]` with credit. Tick the box in this
   file. Open the PR against `master`; CI must be green on dynamic
   Linux/macOS rows.

---

## 6. Explicit non-goals for this plan

- Ruby 3.x (Ractors, `rb_ext_ractor_safe`, `RB_GC_GUARD` changes, taint
  removal fallout, keyword-argument separation) — after this plan.
- `rb-sys`/bindgen-based generation — rejected; see PR #172 revert rationale.
- Windows CI — best-effort only; no Ruby install step exists there.
- Static libruby — best-effort rows stay `continue-on-error`.
- Binding internal/`RUBY_INTERNAL` headers, `st_table` directly, or anything
  under `include/ruby/internal` — not public API even in Ruby 2.
