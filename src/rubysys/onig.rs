/*
* The MIT License (MIT)
* 
* Copyright (c) 2015 Will Speak <will@willspeak.me>, Ivan Ivashchenko
* <defuz@me.com>, and contributors.
* 
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
* 
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
* 
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/

// Source code excerpt from
// https://github.com/rust-onig/rust-onig
// with minor changes to match Ruby.

use rubysys::types::{c_int, c_char, c_void, EncodingIndex};
use std::os::raw::{c_uchar, c_uint};

pub type OnigCodePoint = c_uint;
pub type OnigUChar = c_uchar;
pub type OnigCtype = c_uint;
pub type OnigCaseFoldType = c_uint;

pub type OnigEncoding = *const OnigEncodingType;

pub type OnigApplyAllCaseFoldFunc =
    extern "C" fn(from: OnigCodePoint, to: *const OnigCodePoint, to_len: c_int, arg: *const c_void)
        -> c_int;

#[repr(C)]
pub struct OnigEncodingType {
    // int    (*precise_mbc_enc_len)(const OnigUChar* p,const OnigUChar* e, const struct OnigEncodingTypeST* enc);
    pub precise_mbc_enc_len: extern "C" fn(p: *const OnigUChar) -> c_int,
    // const char*   name;
    pub name: *const c_char,
    // int           max_enc_len;
    pub max_enc_len: c_int,
    // int           min_enc_len;
    pub min_enc_len: c_int,
    // int    (*is_mbc_newline)(const OnigUChar* p, const OnigUChar* end, const struct OnigEncodingTypeST* enc);
    pub is_mbc_newline: extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> c_int,
    // OnigCodePoint (*mbc_to_code)(const OnigUChar* p, const OnigUChar* end, const struct OnigEncodingTypeST* enc);
    pub mbc_to_code: extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> OnigCodePoint,
    // int    (*code_to_mbclen)(OnigCodePoint code, const struct OnigEncodingTypeST* enc);
    pub code_to_mbclen: extern "C" fn(code: OnigCodePoint) -> c_int,
    // int    (*code_to_mbc)(OnigCodePoint code, OnigUChar *buf, const struct OnigEncodingTypeST* enc);
    pub code_to_mbc: extern "C" fn(code: OnigCodePoint, buf: *mut OnigUChar) -> c_int,
    // int    (*mbc_case_fold)(OnigCaseFoldType flag, const OnigUChar** pp, const OnigUChar* end, OnigUChar* to, const struct OnigEncodingTypeST* enc);
    pub mbc_case_fold: extern "C" fn(
        flag: OnigCaseFoldType,
        pp: *const *const OnigUChar,
        end: *const OnigUChar,
        to: *const OnigUChar,
    ) -> c_int,
    // int    (*apply_all_case_fold)(OnigCaseFoldType flag, OnigApplyAllCaseFoldFunc f, void* arg, const struct OnigEncodingTypeST* enc);
    pub apply_all_case_fold:
        extern "C" fn(flag: OnigCaseFoldType, f: OnigApplyAllCaseFoldFunc, arg: *const c_void)
            -> c_int,
    // int    (*get_case_fold_codes_by_str)(OnigCaseFoldType flag, const OnigUChar* p, const OnigUChar* end, OnigCaseFoldCodeItem acs[], const struct OnigEncodingTypeST* enc);
    pub get_case_fold_codes_by_str: extern "C" fn(
        flag: OnigCaseFoldType,
        p: *const OnigUChar,
        end: *const OnigUChar, /* ... */
    ) -> c_int,
    // int    (*property_name_to_ctype)(const struct OnigEncodingTypeST* enc, const OnigUChar* p, const OnigUChar* end);
    pub property_name_to_ctype:
        extern "C" fn(enc: OnigEncoding, p: *const OnigUChar, end: *const OnigUChar) -> c_int,
    // int    (*is_code_ctype)(OnigCodePoint code, OnigCtype ctype, const struct OnigEncodingTypeST* enc);
    pub is_code_ctype: extern "C" fn(code: OnigCodePoint, ctype: OnigCtype) -> c_int,
    // int    (*get_ctype_code_range)(OnigCtype ctype, OnigCodePoint* sb_out, const OnigCodePoint* ranges[], const struct OnigEncodingTypeST* enc);
    pub get_ctype_code_range:
        extern "C" fn(ctype: OnigCtype, sb_out: *const OnigCodePoint /* ... */) -> c_int,
    // OnigUChar* (*left_adjust_char_head)(const OnigUChar* start, const OnigUChar* p, const OnigUChar* end, const struct OnigEncodingTypeST* enc);
    pub left_adjust_char_head:
        extern "C" fn(start: *const OnigUChar, p: *const OnigUChar) -> *const OnigUChar,
    // int    (*is_allowed_reverse_match)(const OnigUChar* p, const OnigUChar* end, const struct OnigEncodingTypeST* enc);
    pub is_allowed_reverse_match:
        extern "C" fn(p: *const OnigUChar, end: *const OnigUChar) -> c_int,
    // int    (*case_map)(OnigCaseFoldType* flagP, const OnigUChar** pp, const OnigUChar* end, OnigUChar* to, OnigUChar* to_end, const struct OnigEncodingTypeST* enc);
    pub case_map: extern "C" fn(
        flag_p: *const OnigCaseFoldType,
        pp: *const *const OnigUChar,
        end: *const OnigUChar,
        to: *const OnigUChar,
        to_end: *const OnigUChar,
        /* ... */
    ) -> c_int,
    // int ruby_encoding_index;
    pub ruby_encoding_index: EncodingIndex,
    // unsigned int  flags;
    pub flags: c_uint,
}
