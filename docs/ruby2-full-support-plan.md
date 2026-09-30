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
   an existing method is the exception, and only when the method breaks no
   matter how carefully it is called (e.g. `GC::register` registering a dead
   stack address, `RString::encode` with options aborting Ruby). If a caller
   can protect themselves (checking bounds or arity, escaping `%`, passing
   only valid input), the existing method keeps its behaviour and cost, and
   the checked behaviour goes in a second, safer method (`VM::raise` /
   `VM::raise_message`, `unsafe_methods!` / `methods!`).
9. **SemVer as the README defines it.** Rutie won't reach 1.0, so MINOR
   versions may break the public API (`src/class/*`, `src/helpers/*`), and
   PATCH versions may only break the private API (`src/rubysys/*`,
   `src/binding/*`, `src/util.rs`). A change that breaks public callers
   (a changed signature or behaviour, a removed item) therefore waits for
   the next MINOR release; adding items and fixing bugs that break however
   a method is called (rule 8) are fine in a PATCH. Say in the CHANGELOG
   which kind each change is.

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
      `VM::raise` keeps passing its message to `rb_raise` as a printf format
      (callers can escape `%`, so per rule 8 it is unchanged and documented);
      `VM::raise_message` is the plain-text variant, and Rutie's own code
      uses it.
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

- [x] **Object:** `rb_obj_dup`, `rb_obj_clone`, `rb_obj_id`, `rb_inspect`,
      `rb_obj_as_string`, `rb_obj_is_kind_of`, `rb_obj_is_instance_of`,
      `rb_obj_method`, `rb_method_boundp`, `rb_check_funcall`,
      `rb_funcall_with_block`, `rb_funcallv_kw` (2.7, gated), `rb_obj_instance_variables`,
      `rb_ivar_defined`, `rb_obj_remove_instance_variable`, `rb_hash` (object hash).
      Done as `Object` trait methods. Names avoid clashing with other traits
      the types implement: `clone_object` (not `Clone::clone`),
      `inspect_object`/`as_string` (not `Exception::inspect`/`to_s`),
      `hash_value` (not `std::hash::Hash::hash`). `method`,
      `instance_eval` return `Result`; `remove_instance_variable` returns
      `Option`; `send_with_keywords` is 2.7-only.
- [x] **Class/Module:** `rb_class_name`, `rb_class_path`, `rb_mod_name`,
      `rb_class_inherited_p`, `rb_mod_include_p`, `rb_mod_module_eval`,
      `rb_obj_instance_eval`, `rb_class_instance_methods`, `rb_cvar_get/set/defined`,
      `rb_const_defined(_at)`, `rb_const_set`, `rb_const_remove`, `rb_path2class`,
      `rb_define_global_const`, `rb_class_of` (immediates too), `rb_define_alias`.
      Done on both `Class` and `Module` (`name`, `path`, `from_path`,
      `is_method_defined`, `inherits`, `includes_module`, `module_eval`,
      `instance_methods`, class variables, constants); `VM::define_global_const`.
      `rb_class_of` is a `static inline` in `ruby.h`, not an exported
      function; `Object::class`/`singleton_class` cover it. `rb_const_set` is
      bound; `const_set` keeps using `rb_define_const`.
- [x] **Global variables:** `rb_gv_get`, `rb_gv_set`, `rb_define_variable`,
      `rb_define_readonly_variable`, `rb_define_virtual_variable`,
      `rb_define_hooked_variable`. Surface: `VM::global("$x")` get/set.
      Done: `VM::global_get`/`global_set`/`protect_global_set`,
      `VM::define_variable`/`define_readonly_variable` -> `GlobalVariable`,
      `VM::define_virtual_variable` (closures, via a hooked variable whose
      data pointer starts with a `VALUE` because Ruby `rb_gc_mark_maybe`s it).
      Getter/setter ABI differs (2.7 drops the trailing `gvar` argument);
      callbacks take only the shared leading arguments.
- [x] **Symbols/IDs:** `rb_sym2str`, `rb_check_id`, `rb_to_id`, `rb_to_symbol`,
      `rb_is_const_id`, `rb_is_instance_id`, `rb_is_class_id`, `rb_intern_str`.
      Done: `Symbol::find` (never creates a symbol), `from_rstring`,
      `to_rstring`, `is_const_name`, `is_instance_variable_name`,
      `is_class_variable_name`. `rb_to_id`/`rb_intern_str` bound in `rubysys`.
- [x] **Kernel formatting:** `rb_sprintf`/`rb_str_format`, `rb_p`, `rb_String`,
      `rb_Array`, `rb_Integer`, `rb_Float`, `rb_Hash`.
      Done: `VM::format` (`rb_str_format`; `rb_sprintf` is C printf and is
      not exposed), `VM::p`, and `RString`/`Array`/`Integer`/`Float`/`Hash::convert`
      returning `Result`. Ruby 2's `Kernel#Hash` only uses `to_hash`.

### P2 — core types to parity with the C API

- [x] **String:** `rb_str_dup`, `rb_str_substr`, `rb_str_split`, `rb_str_cmp`,
      `rb_str_equal`, `rb_str_hash`, `rb_str_inspect`, `rb_str_replace`,
      `rb_str_resize`, `rb_str_buf_new`, `rb_str_buf_append`, `rb_str_plus`,
      `rb_str_times`, `rb_str_to_inum`, `rb_str_to_dbl`, `rb_str_intern`,
      `rb_str_length`, `rb_str_capacity`, `rb_str_set_len`, `rb_str_modify`,
      `rb_str_conv_enc`, `rb_enc_str_coderange`, `rb_enc_mbclen`, `rb_enc_nth`,
      `rb_str_scrub`. Byte-slice (`&[u8]`) views must respect `rb_str_locktmp`.
      Done: all bound in `rubysys`/`binding`; `RString` gained `with_capacity`,
      `capacity`, `compare`/`PartialOrd`, `ellipsize`, `plus`, `replace`,
      `truncate`, `scrub`, `split`, `byte_slice`, `substr`, `times`,
      `to_i`/`parse_integer`, `to_f`/`parse_float`, `coderange` (`CodeRange`)
      and `with_locked_bytes` (locktmp held via `rb_ensure`, panic-safe).
      Safety notes: `rb_str_subseq` is not bounds-checked (wrapped as
      `byte_slice` with a check); growing with `rb_str_resize` exposes
      uninitialized bytes (only `truncate` is public). `dup`, `inspect`,
      `hash`, `length`, `intern`, `equal`, `buf_append` are covered by the
      `Object` trait, `count_chars`, `Symbol::from_rstring` and existing
      methods. `set_len`, `modify`, `conv_enc`, `mbclen`, `nth` stay
      binding-level (raw pointers/encodings).
- [x] **Array:** `rb_ary_delete`, `rb_ary_delete_at`, `rb_ary_includes`,
      `rb_ary_clear`, `rb_ary_subseq`, `rb_ary_plus`, `rb_ary_cmp`, `rb_ary_replace`,
      `rb_ary_resize`, `rb_ary_rotate`, `rb_ary_assoc`, `rb_ary_rassoc`,
      `rb_ary_to_ary`, `rb_check_array_type`, `rb_ary_each` (via `rb_block_call`),
      `rb_ary_freeze`, `rb_ary_aref`. `Array` should implement `IntoIterator`
      (by `Value` copy) and `FromIterator<AnyObject>`.
      Done: `delete`, `delete_at`, `includes`, `clear`, `slice`, `plus`,
      `compare`, `replace`, `resize`, `rotate_bang`, `assoc`, `rassoc`,
      `TryConvert` (`rb_check_array_type`). `IntoIterator`/`FromIterator`
      already existed; freezing is `Object::freeze`. `rb_ary_aref`,
      `rb_ary_to_ary` bound in `rubysys`; `rb_ary_each` needs a Ruby block, so
      iteration uses the iterator or `send_with_block`.
- [x] **Hash:** `rb_hash_lookup`, `rb_hash_lookup2`, `rb_hash_fetch`,
      `rb_hash_has_key`? (use `rb_hash_lookup2` with undef), `rb_hash_keys`,
      `rb_hash_values`, `rb_hash_update_by`, `rb_hash_set_ifnone`,
      `rb_hash_freeze`, `rb_check_hash_type`, `rb_hash_delete_if`?, `rb_hash_tbl`
      (avoid), `rb_env_clear`? (no). `Hash` gets `iter()` over `(AnyObject, AnyObject)`.
      Done: `lookup`, `has_key` (`rb_hash_lookup2` with `Qundef`), `fetch`,
      `keys`, `values` (through `rb_hash_foreach`: `rb_hash_keys` is exported
      by 2.6/2.7 but in no public header, `rb_hash_values` by none),
      `update` (`rb_hash_update_by`), `set_default` (through `default=`, since
      `rb_hash_set_ifnone` skips the frozen check and keeps a default-proc
      flag), `iter` -> `HashIterator`, `TryConvert` (`rb_check_hash_type`).
- [x] **Numeric:** Bignum — `rb_big2str`, `rb_cstr_to_inum`, `rb_str2inum`,
      `rb_big_cmp`, `rb_big_plus/minus/mul/div/modulo/pow`, `rb_big2ll/ull/dbl`,
      `rb_dbl2big`, `rb_int_positive_pow`; `rb_num_coerce_bin/cmp/relop`,
      `rb_num2fix`, `rb_fix2str`, `rb_Integer`, `rb_Float`; **Rational/Complex** —
      `rb_rational_new`, `rb_rational_raw`, `rb_Rational`, `rb_rational_num/den`,
      `rb_complex_new`, `rb_complex_raw`, `rb_Complex`, `rb_complex_real/imag`
      (2.7 exposes `rb_complex_real`/`_imag`; gate older versions to
      `rb_funcall`). New types: `Bignum`? (fold into `Integer`), `Rational`,
      `Complex`. Implement `TryFrom<i128>/u128`, `From<f64>`.
      Done: `Integer::from_str_radix`, `to_s_radix`, `is_bignum`, exact
      `i128`/`u128` both ways (`rb_integer_pack`/`unpack`), `TryFrom<f64>`,
      `to_f64`, arithmetic (`div`/`modulo` return `Result`), `compare`.
      New `Rational` and `Complex` types; `Float::rationalize`. Notes: the
      `rb_big_*` functions take a Bignum receiver only (a Fixnum is UB), so
      arithmetic goes through the Integer methods; `rb_int_positive_pow` is
      in `internal.h` only (not bound); `rb_complex_real/imag/abs/arg` are
      2.6+ (2.5 falls back to method calls); `rb_cstr_to_inum` passes no
      length, which disables base-0 prefix detection, so parsing uses
      `rb_str_to_inum`. `rb_num_coerce_*` are bound (`binding::numeric`).
- [x] **Range:** `rb_range_new`, `rb_range_values`, `rb_range_beg_len`,
      `rb_arithmetic_sequence_extract` (2.6+). New type `Range`.
      Done. Ruby 2 stores ranges as `T_STRUCT`, so `Range`/`Struct` are
      verified with `kind_of` against `rb_cRange`/`rb_cStruct`, not `ty()`.
      `rb_range_beg_len` raises for non-integer bounds (wrapped in `Result`).
- [x] **Regexp / MatchData:** `rb_reg_new_str`, `rb_reg_new`, `rb_reg_regcomp`,
      `rb_reg_match`, `rb_reg_match2`, `rb_reg_nth_match`, `rb_reg_last_match`,
      `rb_reg_backref_number`, `rb_backref_get/set`, `rb_reg_options`,
      `rb_reg_source`. New types `Regexp`, `MatchData`.
      Done. `rb_reg_source` is not exported by any 2.x (uses `source`).
      Matching returns `Result` because strings with invalid bytes raise.
      With named groups, Onigmo does not capture unnamed groups.
- [x] **Time:** `rb_time_new`, `rb_time_nano_new`, `rb_time_timespec_new`,
      `rb_time_num_new`, `rb_time_interval`, `rb_time_timeval`, `rb_time_timespec`,
      `rb_time_utc_offset`. New type `Time` with `From<SystemTime>`/`Duration`.
      Done. `rb_time_num_new` does not validate its argument (must be an
      exact Integer/Rational), so `Time::at` calls `Time.at` instead.
- [x] **Struct:** `rb_struct_define`, `rb_struct_define_under`, `rb_struct_new`,
      `rb_struct_alloc`, `rb_struct_aref`, `rb_struct_aset`, `rb_struct_getmember`,
      `rb_struct_members`, `rb_struct_size`. New type `Struct`.
      Done. `rb_struct_define(_under)`/`rb_struct_new` are variadic with an
      unbounded member list, so definitions go through `Struct.new`; they are
      bound in `rubysys` for fixed-arity C-style use.
- [x] **Enumerator / Enumerable:** `rb_enumeratorize`, `rb_enumeratorize_with_size`
      (`RETURN_ENUMERATOR` equivalent for `methods!`), `rb_enum_values_pack`,
      `rb_cmpint`, `rb_cmperr`, `rb_obj_is_kind_of(Enumerable)`. Make
      `Enumerator` iterable from Rust (`next` via `rb_funcall` with
      `StopIteration` mapped to `None`).
      Done: `Enumerator::new`, `iter`/`IntoIterator` (`StopIteration` ends;
      other exceptions yielded once as `Err`), `Object::try_compare`
      (`rb_cmpint`). Found and fixed: a fiber may only be resumed under the
      `rb_protect` it was created under, so `protect_send`-based `next` broke
      when called from different stack depths; the enumerator methods now use
      `rb_rescue2`. `rb_enumeratorize_with_size`/`RETURN_ENUMERATOR` need
      the calling C frame (`rb_frame_this_func`), so `methods!` users call
      `Enumerator::new(&rtself, "method", &args)` instead.
- [x] **Proc / Method / Binding:** `rb_proc_new`, `rb_proc_arity`,
      `rb_proc_lambda_p`, `rb_proc_call`, `rb_block_lambda`, `rb_method_call`,
      `rb_obj_method`, `rb_mod_method_arity`, `rb_obj_method_arity`,
      `rb_yield_values`, `rb_yield_values2`, `rb_need_block`, `rb_binding_new`
      (exists) + `Binding::local_variable_get/set` via `rb_funcall`.
      Done: `Proc::new` (closure owned by a hidden typed-data object the
      proc's block marks, so it is freed with the proc), `arity`,
      `protect_call`; `Method` type; `method_arity`/`instance_method_arity`;
      `VM::yield_values`/`need_block`; `Binding` locals/receiver/eval.
      `rb_block_lambda`, `rb_method_call_with_block` bound in `rubysys`.
      Note: for non-lambda procs Ruby does not count optional arguments in
      `arity`.
- [x] **Encoding:** `rb_enc_get`, `rb_enc_name`, `rb_enc_find`, `rb_ascii8bit_encoding`,
      `rb_utf8_encoding`, `rb_usascii_encoding`, `rb_locale_encoding`,
      `rb_filesystem_encoding`, `rb_enc_str_buf_cat`, `rb_enc_uint_chr`,
      `rb_enc_precise_mbclen`, `rb_enc_ascget`. Round out `Encoding`.
      Done: `ascii_8bit`, `locale`, `filesystem`, `of`, `index`, `chr`,
      `is_ascii_compatible`, `is_dummy`, `RString::concat_bytes`; fixed
      `Encoding`'s `VerifiedObject` (instances are `T_DATA`). `rb_enc_name` is
      a macro (uses `name`); `precise_mbclen`/`ascget`/`codelen` stay in
      `rubysys`. An embedded VM knows only the built-in encodings until
      `VM::init_loadpath()` + `require "enc/encdb"`.

### P3 — exceptions, IO, and the standard objects an embedder hits

- [x] **Exceptions:** `rb_exc_new_str`, `rb_exc_new_cstr`, `rb_exc_new`,
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
      Done: all `rb_e*` statics (`rubysys::exception`) with `Class::*()`
      accessors generated with doctests (`src/class/builtins.rs`);
      `AnyException::from_class` (no name lookup), `from_errno`,
      `from_io_error`; `VM::raise_interrupt`. `rb_exc_fatal`, `rb_bug`,
      `rb_mod_syserr_fail`, `rb_exc_new(_cstr)` bound in `rubysys` only.
- [x] **Builtin class/module globals:** `rb_cObject`, `rb_cBasicObject`,
      `rb_mKernel`, `rb_mComparable`, `rb_mEnumerable`, `rb_cString`, `rb_cArray`,
      `rb_cHash`, `rb_cInteger`, `rb_cFloat`, `rb_cRational`, `rb_cComplex`,
      `rb_cRange`, `rb_cRegexp`, `rb_cTime`, `rb_cSymbol`, `rb_cProc`, `rb_cMethod`,
      `rb_cThread`, `rb_cIO`, `rb_cFile`, `rb_cNilClass`, `rb_cTrueClass`,
      `rb_cFalseClass`, `rb_cEncoding`, `rb_cStruct`, `rb_cEnumerator`,
      `rb_cModule`, `rb_cClass`. Today many wrappers do `Class::from_existing("X")`
      string lookups; replace with the statics.
      Done: `rubysys::builtins` + `Class::*()`/`Module::*()` accessors
      (`class_class`, `module_class`, `method_class`, `struct_class` avoid
      keyword/trait clashes). `rb_cFixnum`/`rb_cBignum`/`rb_cCont` are not
      exported by 2.5-2.7. Internal `VerifiedObject` checks (`Proc`,
      `Binding`, `Encoding`, `Enumerator`, `Exception`, `Thread`) and
      `VM::exit_bang` now use the statics; semantics are unchanged.
- [x] **IO / File / Dir:** `rb_io_write`, `rb_io_puts`, `rb_io_print`, `rb_io_gets`,
      `rb_io_getbyte`, `rb_io_close`, `rb_io_flush`, `rb_io_eof`, `rb_io_binmode`,
      `rb_io_check_readable/writable/closed`, `rb_io_stdio_file`, `rb_stdin`,
      `rb_stdout`, `rb_stderr`, `rb_file_open`, `rb_file_open_str`,
      `rb_file_expand_path`, `rb_file_absolute_path`, `rb_file_dirname`,
      `rb_dir_getwd`, `rb_io_taint_check`? (no — taint). New types `IO`, `File`.
      Done: `IO` (standard streams, write/puts/print/gets/getbyte/flush/close/
      eof/binmode, all `Result`), `File` (`open`, path helpers,
      `current_directory`; `Deref<Target = IO>`). `rb_io_check_*` and
      `rb_io_stdio_file` take the internal `rb_io_t *`, so they are not
      bound; `is_closed` uses `closed?`.
- [x] **Marshal / ObjectSpace / GC extras:** `rb_marshal_dump`, `rb_marshal_load`,
      `rb_define_finalizer`, `rb_undefine_finalizer`, `rb_objspace_each_objects`
      (careful), `rb_memory_id`, `rb_gc_writebarrier`, `rb_gc_writebarrier_unprotect`,
      `rb_gc_latest_gc_info`, `rb_gc_register_mark_object` (exists). `GC::WeakMap`
      via `rb_funcall`.
      Done: `Marshal::dump`/`load` (documented as unsafe for untrusted
      data), `GC::define_finalizer`/`undefine_finalizer`/`latest_info`/
      `write_barrier`/`write_barrier_unprotect`. `rb_memory_id` is 2.7-only
      (bound, gated). `rb_objspace_each_objects` hands out raw heap pages and
      is not bound; use `ObjectSpace.each_object` with `send_with_block`.
- [x] **Load / require / $LOAD_PATH:** `rb_load`, `rb_load_protect`, `rb_f_require`,
      `rb_provide`, `rb_provided`, `rb_feature_provided`, `ruby_incpush`,
      `rb_require_string` (2.7). Surface: `VM::load(path, wrap)`, `VM::provide`.
      Done: `VM::load` (`rb_load_protect`), `protect_require` (`rb_f_require`
      on every version; `rb_require_string` bound for 2.7), `provide`,
      `is_provided`, `add_load_path` (`ruby_incpush`), `find_file`. Features
      are recorded with their extension (`"x.so"`); `rb_provided("x")`
      without one only matches `.rb` features.

### P4 — concurrency

- [x] **Threads:** `rb_thread_current`, `rb_thread_main`, `rb_thread_alone`,
      `rb_thread_schedule`, `rb_thread_sleep`, `rb_thread_sleep_forever`,
      `rb_thread_wait_for`, `rb_thread_wakeup`, `rb_thread_run`, `rb_thread_kill`,
      `rb_thread_local_aref/aset`, `rb_thread_check_ints`, `rb_thread_atfork`,
      `rb_thread_fd_writable`, `rb_thread_fd_select`? (avoid), `rb_thread_wait_fd`
      (exists). `Thread` gets `join`, `value`, `alive`, `kill`, `current`.
      Done: `Thread::current`, `main`, `is_alone`, `pass`, `sleep`
      (`rb_thread_wait_for`), `check_interrupts`, `wait_fd_writable`, and
      instance `join`, `join_value` (not `value`, which would shadow
      `Object::value`), `is_alive`, `kill`, `wakeup` (`Result`: waking a dead
      thread raises), `local_get`/`local_set`. `rb_thread_sleep_forever`,
      `rb_thread_run`, `rb_thread_atfork` are bound in `rubysys` only;
      `rb_thread_fd_select` is not bound.
- [x] **Mutex / Queue:** `rb_mutex_new`, `rb_mutex_lock`, `rb_mutex_unlock`,
      `rb_mutex_trylock`, `rb_mutex_locked_p`, `rb_mutex_synchronize`,
      `rb_mutex_sleep`. New type `Mutex` with an RAII guard that unlocks on drop
      (and on Ruby exceptions via `rb_ensure`).
      Done: `Mutex` (`new`, `lock` → `MutexGuard`, `try_lock`, `is_locked`,
      `synchronize`) and `MutexGuard::sleep`; the guard is `!Send` and
      unlocks on drop. Ruby exceptions skip Rust destructors, so
      `synchronize` (`rb_mutex_synchronize`, which unlocks with `rb_ensure`,
      under `rb_protect`) is the documented choice when the locked code may
      raise. `Queue` has no C API in 2.x; use `Class::from_existing("Queue")`
      and `send`.
- [x] **Fiber:** `rb_fiber_new`, `rb_fiber_resume`, `rb_fiber_yield`,
      `rb_fiber_current`, `rb_fiber_alive_p`. New type `Fiber`.
      Done: `Fiber::new` (Rust closure, owned by the fiber's proc like
      `Proc::new`), `resume`, `yield_values`, `current`, `is_alive`. Ruby
      creates and switches fibers only while the thread has an active tag,
      and resumes a fiber only under the `rb_protect` tag it was created
      under, so these use `rb_rescue2` (like the enumerator fix in P2d); a
      fiber made inside `VM::eval`/`VM::protect` cannot be resumed from Rust,
      and `eval` refuses to run directly on top of a fiber.
- [x] **GVL helpers:** `Thread::call_without_gvl` exists; add an unblocking
      function argument (`rb_thread_call_without_gvl` UBF, `RUBY_UBF_IO` /
      `RUBY_UBF_PROCESS` constants) and document Send/Sync expectations.
      Done: `call_without_gvl` already takes a Rust unblocking closure;
      `Thread::call_without_gvl_io` passes `RUBY_UBF_IO` and documents that
      the closure must not touch Ruby and must share only `Send`/`Sync`
      data. `RUBY_UBF_PROCESS` is the same function in 2.x.

### P5 — VM lifecycle and embedding

- [x] `ruby_setup`, `ruby_cleanup`, `ruby_finalize`, `ruby_options`,
      `ruby_run_node`, `ruby_exec_node`, `ruby_script`, `ruby_set_argv`,
      `ruby_prog_init`, `ruby_init_stack` (only where the platform needs it),
      `ruby_sysinit`, `ruby_native_thread_p`, `ruby_stack_check`,
      `ruby_stack_length`. Surface: `VM::init_with_args(&[&str])`,
      `VM::cleanup() -> i32`, `VM::run_file`, and make `VM::init` idempotent
      (it is not today: calling twice is UB).
      Done: all of these are bound in `rubysys` (with `rb_argv0`). Surface:
      `VM::try_init` (`ruby_setup`, error instead of `exit`),
      `VM::init_with_args`, `VM::set_argv`, `VM::set_script_name`,
      `VM::run_file`, `VM::is_initialized` (`rb_cObject` is set),
      `VM::is_ruby_thread`, `VM::is_stack_near_limit`, `VM::stack_length`;
      `VM::cleanup` since P0-2. `VM::init` was already idempotent
      (`ruby_setup` returns early when the VM exists; re-init after
      `ruby_cleanup` is still unsupported), now documented and tested.
      `ruby_options` works once per process (2.7 reads past its builtin
      table when the prelude loads again, and the `ruby` command has already
      called it in an extension), so `run_file` checks `rb_argv0` and
      returns an error instead; it leaks its argv on purpose, as Ruby keeps
      it for `$0=`. `ruby_finalize`, `ruby_run_node`, `ruby_sysinit`,
      `ruby_init_stack`, `ruby_prog_init` stay `rubysys`-only.
- [x] `rb_set_end_proc` proper `at_exit` (P0-2) and `ruby_vm_at_exit` semantics
      documented side by side. (`VM::cleanup` exists since P0-2.)
      Done: `VM::at_vm_exit` (`ruby_vm_at_exit`, plain `extern "C" fn`,
      runs when the VM is freed) cross-referenced from `VM::at_exit`.
- [x] Signals: `rb_f_trap`-equivalents are not public C API; document that
      `VM::trap` (exists via `Signal.trap`) is the supported route.
      Done: documented on `VM::trap`, with a running doctest.

### P6 — DSL, docs, examples, release

- [x] `wrappable_struct!`: make the three `ignore`d doctests runnable; add
      `dsize` support (`rb_data_typed_object_zalloc` + size fn) and a
      `#[derive]`-free way to declare the wrapped type `Send`-safe; document
      the 2.7 `dcompact` slot (§3).
      Done: the three examples run and assert (0 ignored doctests). New
      optional `size(data) { .. }` clause (`dsize`, reported by
      `ObjectSpace.memsize_of`), accepted before or after `mark`. The wrapper
      stays `Sync` for any `T`, since it only holds the type descriptor (no
      derive needed). `dcompact` (`reserved[0]` on 2.7) is left empty, so
      objects marked with `GC::mark` are pinned by `GC.compact`; documented.
      Internal macro calls go through `$crate::` so `rutie::wrappable_struct!`
      works without importing the macro.
- [x] `methods!`: keyword args (gated), splat, optional args (P0-5); emit
      `rb_error_arity` on mismatch instead of panicking.
      Done: splat, the README's "Variadic Functions / Splat Operator" goal
      (done here instead of a separate P9): a trailing `*name` parameter
      takes the remaining arguments as an `Array`, with no unsafe code, and
      the README now shows it. Each method's parameter list goes to an
      internal `@method` rule, so large `methods!` blocks don't recurse
      deeply. Optional, keyword and block parameters use
      `VM::scan_args` in a plain `extern fn` (P0-5, documented from
      `methods!`); keeping the rest of the grammar unchanged keeps existing
      code compiling. `methods!` never panicked on missing arguments (each
      one is a `Result`). `unsafe_methods!` stays unchecked: its contract is
      that the caller guarantees the arguments, so per rule 8 the checked
      version is `methods!`, not a change to `unsafe_methods!`.
- [x] Build all `examples/` in CI (they exercise `rutie_ruby_example`,
      `rutie_ruby_gvl_example`, `rutie_rust_example` end to end), on the same
      matrix as the crate.
      Done: a CI step on every dynamic Linux/macOS row runs `examples/eval.rs`,
      `rutie_rust_example`'s tests, and both Ruby extension examples'
      minitest suites (`rutie` gem plus minitest `~> 5.15.0`; the reporter gem
      is now optional in their `test_helper.rb`). Verified locally on 2.5.9,
      2.6.10 and 2.7.8. The failure step now prints only the end of the RVM
      make log, which used to push the test output out of reach.
- [x] README: "Ruby 2 Notes" rewritten for 0.10 (support table, OpenSSL 1.1
      recipe). Still to do: add the local multi-Ruby testing recipe from §0.4.
      Done: "Testing against several Rubies" under Contributing (ground rule
      4's recipe).
- [x] `build.rs`: honour `$RUBY` consistently (it does for `rbconfig`; check
      `is_linked_ruby` test uses the same), emit the version cfgs (P0-1, done),
      and print a clear error when the linked Ruby's major version ≠ 2 (a
      `cargo:warning` is printed since P0-1; decide whether it should fail).
      Done: `is_linked_ruby` now runs `$RUBY` like `build.rs`. Decision: keep
      the warning rather than failing, so the later Ruby 3 work can build
      the crate while it's being ported; the warning says Ruby 2 is the
      supported target.
- [ ] Release cadence: superseded. Each Rutie minor now maps to three Rubies:
      0.10 = 2.5/2.6/2.7 (this plan, P0–P8; 0.10.0 is not released yet, so
      all of it, including the breaking `VM::at_exit` change, ships in
      0.10.0), 0.11 = 3.0/3.1/3.2, 0.12 = 3.1/3.2/3.3, 0.13 = 3.2/3.3/3.4
      (P9).

### P7 — unit tests for every public API

Every package above ships with tests for what it adds, written as a
`#[cfg(test)] mod tests` at the bottom of the file that defines the API and run
through `crate::on_ruby_thread` (§0.3). P7 backfills the API that existed
before this plan, so that **every public item has at least one unit test that
round-trips through Ruby**, not only a doctest.

- [x] Inventory: list every public item (`src/class/**`, `src/helpers/**`,
      `src/dsl.rs`, `src/util.rs` public fns, `typed_data`) and the unit tests
      covering it; keep the table in this section current.
      Done: public functions per file that a unit test calls (counted by a
      script over the `mod tests` blocks; trait impls such as `From` and
      `VerifiedObject` are exercised by the same tests):

      | File | Covered |
      |---|---|
      | `class/any_exception.rs` | 3/3 |
      | `class/array.rs` | 31/31 |
      | `class/binding.rs` | 7/7 |
      | `class/boolean.rs` | 2/2 |
      | `class/class.rs` | 37/37 |
      | `class/complex.rs` | 8/8 |
      | `class/encoding.rs` | 15/15 |
      | `class/enumerator.rs` | 8/8 |
      | `class/fiber.rs` | 5/5 |
      | `class/fixnum.rs` | 5/5 |
      | `class/float.rs` | 5/5 |
      | `class/gc.rs` | 19/19 |
      | `class/global_variable.rs` | 2/2 |
      | `class/hash.rs` | 16/16 |
      | `class/integer.rs` | 19/19 |
      | `class/io.rs` | 18/18 |
      | `class/marshal.rs` | 2/2 |
      | `class/method.rs` | 4/4 |
      | `class/module.rs` | 34/34 |
      | `class/mutex.rs` | 6/6 |
      | `class/nil_class.rs` | 1/1 |
      | `class/range.rs` | 6/6 |
      | `class/rational.rs` | 6/6 |
      | `class/regexp.rs` | 12/12 |
      | `class/rproc.rs` | 5/5 |
      | `class/rstruct.rs` | 9/9 |
      | `class/string.rs` | 34/34 |
      | `class/symbol.rs` | 10/10 |
      | `class/thread.rs` | 20/20 |
      | `class/time.rs` | 7/7 |
      | `class/traits/encoding_support.rs` | 6/6 |
      | `class/traits/exception.rs` | 9/9 |
      | `class/traits/object.rs` | 52/52 |
      | `class/traits/try_convert.rs` | 1/1 |
      | `class/traits/verified_object.rs` | 2/2 |
      | `class/vm.rs` | 66/70 |
      | `helpers/codepoint_iterator.rs` | 1/1 |
      | `helpers/scan_args.rs` | 2/2 |
      | `util.rs` | 17/18 |

      The five not counted are doctest-only on purpose: `VM::cleanup`,
      `VM::run_file`, `VM::at_vm_exit` and `VM::exit_bang` end the process or
      its VM, and `util::ptr_to_data` is called through a turbofish the
      script does not match. The `dsl` macros are tested in `dsl.rs`.
- [x] Backfill a bottom-of-file test module for each file that lacks one
      (today: `any_exception`, `any_object`, `binding`, `boolean`, `encoding`,
      `enumerator`, `fixnum`, `float`, `gc`, `module`, `nil_class`, `rproc`,
      `thread`, `traits/*`, `helpers/codepoint_iterator`, `typed_data`,
      `dsl` macros), covering success paths, error paths (`Err`/raised
      exceptions via `VM::protect`), frozen receivers and GC survival
      (`GC::start` between creating and using objects) where relevant.
      Done: every file with public items has one. Two bugs found and fixed
      under rule 8 (they broke however they were called): `GC::register`
      registered a dead stack address, and `RString::encode` with options
      aborted Ruby (`rb_econv_prepare_opts`'s output was ignored).
      `on_ruby_thread` now repeats a failing test's panic message on the
      test's own thread (the Ruby thread's output went to whichever test
      started it).
- [x] Version-specific behaviour gets version-specific tests under
      `#[cfg(ruby_2_5)]`/`#[cfg(ruby_gte_2_6)]`/`#[cfg(ruby_gte_2_7)]`.
      Done: the gated APIs (`Range::arithmetic_sequence`,
      `Object::send_with_keywords`, `VM::is_keyword_given`) have gated tests,
      and `cfg_flags_match_linked_ruby` checks the flags against the running
      Ruby.
- [x] `cargo test --lib` green on 2.5.9, 2.6.10 and 2.7.8, stable and beta.

### P8 — doctest audit

- [x] Every public item has a doctest that **runs**: remove `ignore`
      (the three `wrappable_struct!` fragments, P6), and keep `no_run`/`text`
      only where running is impossible (process exit, signals), each with a
      comment saying why.
      Done: no `ignore` or `no_run` blocks are left. The `text` blocks that
      remain are Ruby call sequences or show how `protect_send` is built,
      and each sits next to a running example. `VM::exit` runs under
      `VM::protect` (it raises `SystemExit` there); `VM::exit_bang` really
      exits, with a comment saying why nothing follows it.
- [x] Doctests assert results (`assert!`/`assert_eq!`), not just call the API;
      examples that only print are given assertions.
      Done: an audit script (every public fn, trait method and macro in
      `src/class`, `src/helpers`, `src/dsl.rs`) finds no example without an
      assertion. `VM::p` captures `$stdout`; the `GC::mark*`/`is_marked`
      examples mark from a `wrappable_struct!` mark function (their only
      correct use) and check the objects survive. Trait method declarations
      (`EncodingSupport`, `TryConvert`, `VerifiedObject`) got their own
      examples.
- [x] Doctests must not depend on version-specific messages (see §3); gate
      version-specific examples with `# #[cfg(ruby_gte_2_7)]`.
- [ ] `cargo test --doc` green on 2.5.9, 2.6.10 and 2.7.8, stable and beta, and
      in CI on Linux and macOS.
      Local: green on all three, stable and beta (627/628/630 doctests).
      CI: Linux green; macOS 2.5/2.6 unit tests crash before the doctests run
      (under investigation, see the PR).

### P9 — Ruby 3 upgrade plan

- [x] Plan the path from 0.10 (Ruby 2) to 0.11 (Ruby 3.0–3.2), 0.12
      (3.1–3.3) and 0.13 (3.2–3.4): `docs/ruby3-upgrade-plan.md`. It lists the
      verified ABI differences (special constants, `RString`/`RArray`
      layouts), removed and deprecated C APIs Rutie binds, behaviour changes
      that affect tests, and the work packages for each release. The README
      has the version roadmap table.

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
5. Doctest + unit test, written as you go: the unit test lives in the
   `#[cfg(test)] mod tests` at the bottom of the same file and runs through
   `crate::on_ruby_thread`. Tests that mutate global VM state (globals,
   constants, `$LOAD_PATH`) must clean up or use names unique to the test.
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
